# Lektion 4: Der Agent Loop

## Problem

Wir haben jetzt alle Bausteine einzeln: Der `LlmProvider` aus
[Phase 3](../04-phase3-architektur/01-llmprovider-port.md) liefert Antworten, `Tool`
([Lektion 3](03-tool-schema-function-calling.md)) beschreibt Werkzeuge und führt sie
aus. Was fehlt, ist der Klebstoff: die Schleife, die die Modellantwort **liest**,
entscheidet, ob sie ein Werkzeugaufruf oder eine finale Antwort ist, im ersten Fall das
Werkzeug ausführt, das Ergebnis dem Modell zurückmeldet — und von vorn beginnt. Diese
Schleife, **Plan → Tool-Aufruf → Beobachtung → nächste Aktion**, ist der Kern dessen,
was einen Agenten von einem einfachen Chat unterscheidet. Sie heißt in diesem Kurs
`AgentLoop`.

## Code (Zielbild)

```rust
pub struct AgentLoop {
    provider: Box<dyn mein_core::port::LlmProvider>,
    werkzeuge: Vec<Box<dyn crate::agent::Tool>>,
    max_schritte: usize,
}

impl AgentLoop {
    pub async fn ausfuehren(&self, aufgabe: impl Into<String>) -> Result<String, AgentFehler> {
        // Plan (Modell fragen) → Tool-Aufruf → Beobachtung → nächste Aktion, bis
        // entweder eine finale Antwort kommt oder das Schrittlimit erreicht ist.
        // ...
    }
}
```

## Dekonstruktion

### Warum die Datei `loop.rs` heißt — und warum das ein Problem ist

`loop` ist in Rust ein **Schlüsselwort** (für Endlosschleifen, siehe
[Kontrollfluss](../01-grundlagen/03-kontrollfluss.md)) — genau wie `fn`, `struct` oder
`match`. CANON und Roadmap dieses Kurses nennen die Datei bewusst `agent/loop.rs`, weil
das fachlich der treffendste Name ist. Das Problem: Ein Modulname `loop` kollidiert mit
dem Schlüsselwort. Wir provozieren diesen Fehler gleich bewusst und lösen ihn mit einem
Werkzeug, das dir noch neu ist: dem **rohen Bezeichner** (*raw identifier*, `r#loop`).

### Der grobe Ablauf, bevor wir ihn in Code gießen

1. **Plan** — Wir schicken den bisherigen Verlauf (inklusive einer System-Nachricht, die
   die verfügbaren Werkzeuge beschreibt, [Lektion 3](03-tool-schema-function-calling.md))
   an den `LlmProvider`.
2. **Tool-Aufruf oder finale Antwort?** — Wir prüfen mit
   `agent::tool::als_werkzeugaufruf(...)`, ob die Antwort ein Werkzeugaufruf ist.
3. **Beobachtung** — Ist es ein Aufruf: Werkzeug in `werkzeuge` suchen, ausführen, das
   Ergebnis als neue Nachricht anhängen — dann zurück zu Schritt 1.
4. **Ende** — Ist es **kein** Aufruf (reiner Text): Das ist die finale Antwort, die
   Schleife endet erfolgreich.

Und über allem: ein **Abbruchkriterium**. Ohne eines würde ein Agent, der sich in einer
Schleife aus Tool-Aufrufen verfängt (z. B. weil das Modell wiederholt dasselbe Werkzeug
mit denselben Argumenten aufruft), niemals von selbst aufhören. Wir zählen deshalb
Schritte und brechen spätestens bei `max_schritte` ab — [Lektion 6](06-abbruchbedingungen-limits.md)
vertieft das; hier legen wir schon die Grundlage dafür.

### Woher `provider: Box<dyn LlmProvider>` kommt — und warum sich die Signatur ändert

In [Phase 3](../04-phase3-architektur/01-llmprovider-port.md) hattest du `LlmProvider`
noch mit `fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler>`
festgelegt — eine einzelne Anfrage rein, eine einzelne Antwort raus. Ein `AgentLoop` denkt
aber nicht in einzelnen Anfragen: Er schickt bei jedem Schritt den **gesamten bisherigen
Verlauf** (System-Nachricht mit Werkzeugbeschreibung, alle bisherigen Nachrichten und
Tool-Ergebnisse) neu an das Modell, damit es im Kontext der ganzen Konversation
entscheidet, was als Nächstes zu tun ist. Deshalb entwickeln wir den Port in dieser Phase
zusätzlich zum `async` aus Release 3 weiter: Er nimmt jetzt direkt eine `&Konversation`
entgegen und gibt eine `Nachricht` zurück, statt der schmaleren `ChatAnfrage`/`ChatAntwort`
-Typen aus Phase 3. Der Fehlertyp wird hier bewusst zu einem generischen
`Box<dyn std::error::Error + Send + Sync>` vereinfacht, damit die Lektion sich auf den
Agent Loop konzentrieren kann — in deinem eigenen Code ist es genauso gültig, stattdessen
weiter mit deinem konkreten `ProviderFehler` aus Phase 3 zu arbeiten:

```rust
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn antworten(
        &self,
        verlauf: &mein_core::Konversation,
    ) -> Result<mein_core::Nachricht, Box<dyn std::error::Error + Send + Sync>>;
}
```

`AgentLoop` besitzt den Provider als `Box<dyn LlmProvider>` — genau das Muster, das du in
[Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md) für
Trait-Objekte an der Grenze gelernt hast. `AgentLoop` weiß nicht, ob dahinter ein echter
API-Aufruf oder (in Tests) ein Fake-Provider steckt — dieselbe Entkopplung, die schon
`mein_cli` von der konkreten `Konversation`-Speicherung trennt.

### `Vec<Box<dyn Tool>>` — der Werkzeugkasten

Der Agent kennt seine Werkzeuge nur über den `Tool`-Trait aus
[Lektion 3](03-tool-schema-function-calling.md). Neue Werkzeuge hinzuzufügen heißt: eine
neue `Tool`-Implementierung schreiben und der `Vec` beim Erzeugen des `AgentLoop`
übergeben — `AgentLoop` selbst muss dafür nicht verändert werden. Das ist dasselbe
Open/Closed-Prinzip, das schon `LlmProvider` als Port ermöglicht: offen für neue
Implementierungen, geschlossen für Änderungen am Kern.

### `AgentFehler` — ein Fehlertyp für die ganze Schleife

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentFehler {
    #[error("Schrittlimit erreicht ({max} Schritte)")]
    SchrittLimitErreicht { max: usize },
    #[error("unbekanntes Werkzeug angefragt: {0}")]
    UnbekanntesWerkzeug(String),
    #[error("Werkzeugfehler: {0}")]
    Werkzeug(#[from] crate::agent::ToolFehler),
    #[error("Provider-Fehler: {0}")]
    Provider(String),
    #[error("interner Aufbaufehler: {0}")]
    Aufbau(String),
}
```

`#[from]` auf `Werkzeug(#[from] ToolFehler)` kennst du im Prinzip schon aus
[Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md): Es erzeugt
automatisch eine `From<ToolFehler>`-Implementierung, sodass `?` einen `ToolFehler`
direkt in einen `AgentFehler::Werkzeug(...)` umwandelt, ohne dass wir `.map_err(...)` von
Hand schreiben müssen. `UnbekanntesWerkzeug` bekommt **keinen** `#[from]` — dieser Fehler
entsteht nicht durch Weiterreichen, sondern durch eine bewusste Prüfung, die wir selbst
schreiben (siehe unten).

## Schritt-Reveal

**Schritt 1 — Provoziere den Fehler bewusst.** Lege
`mein_agent/src/agent/loop.rs` an, und trage sie in `mein_agent/src/agent/mod.rs` ein:

```rust
pub mod tool;
pub mod loop;
```

`cargo check -p mein_agent` meldet:

```
error: expected identifier, found keyword `loop`
 --> mein_agent/src/agent/mod.rs:2:9
  |
2 | pub mod loop;
  |         ^^^^ expected identifier, found keyword
  |
help: escape `loop` to use it as an identifier
  |
2 | pub mod r#loop;
  |         ++
```

Der Compiler erwartet nach `mod` einen **Bezeichner** (einen Namen), findet aber das
Schlüsselwort `loop` — dieselbe Kategorie Fehler wie in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md), wo klein
geschriebene Traitnamen nicht erkannt wurden, nur diesmal geht es nicht um
Groß-/Kleinschreibung, sondern um ein reserviertes Wort. Praktisch: rustc schlägt die
Lösung selbst vor.

**Schritt 2 — Korrektur mit rohem Bezeichner.**

```rust
pub mod tool;
pub mod r#loop;

pub use r#loop::{AgentFehler, AgentLoop};
pub use tool::{Tool, ToolFehler, Werkzeugaufruf};
```

`r#loop` ("raw identifier") sagt dem Compiler: "Behandle `loop` hier ausnahmsweise als
normalen Namen, nicht als Schlüsselwort." Das Präfix `r#` funktioniert für **jedes**
Schlüsselwort, das du als Bezeichner brauchst — nützlich vor allem, wenn sich die
Sprache weiterentwickelt und ein früher freier Name (`async`, `try`) zu einem
Schlüsselwort wird. Der Clou: Dank `pub use r#loop::{...}` in `mod.rs` muss **niemand
sonst** `r#loop` tippen — von außen heißt es einfach `mein_agent::agent::AgentLoop`.

> **💡 Tipp**
>
> Datei- und Modulname bleiben trotzdem `loop.rs` — nur die `mod`-Deklaration selbst
> braucht `r#`. Das ist eine kleine, aber lehrreiche Ausnahme von der sonst so
> konsistenten Rust-Syntax.

**Schritt 3 — Werkzeugkatalog als Text.** Bevor wir `AgentLoop` schreiben, brauchen wir
noch eine kleine Funktion, die die Werkzeugliste in eine System-Nachricht verwandelt.
Ergänze in `mein_agent/src/agent/tool.rs`:

```rust
pub fn katalog_als_text(werkzeuge: &[Box<dyn Tool>]) -> String {
    let mut text = String::from(
        "Du kannst folgende Werkzeuge benutzen. Antworte NUR mit normalem Text, wenn du \
         fertig bist, oder NUR mit einem JSON-Objekt der Form \
         {\"werkzeug\": \"<name>\", \"argumente\": {...}}, wenn du eines aufrufen willst:\n",
    );
    for werkzeug in werkzeuge {
        text.push_str(&format!(
            "- {}: {} (Parameter: {})\n",
            werkzeug.name(),
            werkzeug.beschreibung(),
            werkzeug.parameter_schema()
        ));
    }
    text
}
```

**Schritt 4 — `AgentLoop` implementieren.** In `mein_agent/src/agent/loop.rs`:

```rust
use crate::agent::tool::{self, als_werkzeugaufruf, Tool};
use mein_core::{Konversation, Rolle};

pub struct AgentLoop {
    provider: Box<dyn mein_core::port::LlmProvider>,
    werkzeuge: Vec<Box<dyn Tool>>,
    max_schritte: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentFehler {
    #[error("Schrittlimit erreicht ({max} Schritte)")]
    SchrittLimitErreicht { max: usize },
    #[error("unbekanntes Werkzeug angefragt: {0}")]
    UnbekanntesWerkzeug(String),
    #[error("Werkzeugfehler: {0}")]
    Werkzeug(#[from] super::tool::ToolFehler),
    #[error("Provider-Fehler: {0}")]
    Provider(String),
    #[error("interner Aufbaufehler: {0}")]
    Aufbau(String),
}

impl AgentLoop {
    pub fn neu(
        provider: Box<dyn mein_core::port::LlmProvider>,
        werkzeuge: Vec<Box<dyn Tool>>,
        max_schritte: usize,
    ) -> Self {
        AgentLoop { provider, werkzeuge, max_schritte }
    }

    pub async fn ausfuehren(&self, aufgabe: impl Into<String>) -> Result<String, AgentFehler> {
        let katalog = tool::katalog_als_text(&self.werkzeuge);
        let mut konversation = Konversation::mit_systemnachricht(katalog)
            .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
        konversation
            .hinzufuegen(Rolle::Benutzer, aufgabe.into())
            .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;

        let mut schritt = 0;

        loop {
            if schritt >= self.max_schritte {
                return Err(AgentFehler::SchrittLimitErreicht { max: self.max_schritte });
            }

            let antwort = self
                .provider
                .antworten(&konversation)
                .await
                .map_err(|e| AgentFehler::Provider(e.to_string()))?;

            match als_werkzeugaufruf(&antwort.inhalt) {
                Some(aufruf) => {
                    let werkzeug = self
                        .werkzeuge
                        .iter()
                        .find(|w| w.name() == aufruf.werkzeug)
                        .ok_or_else(|| AgentFehler::UnbekanntesWerkzeug(aufruf.werkzeug.clone()))?;

                    let ergebnis = werkzeug.ausfuehren(aufruf.argumente).await?;

                    konversation
                        .hinzufuegen(Rolle::Assistent, antwort.inhalt.clone())
                        .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
                    konversation
                        .hinzufuegen(
                            Rolle::System,
                            format!("Beobachtung von {}: {ergebnis}", werkzeug.name()),
                        )
                        .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;

                    schritt += 1;
                }
                None => return Ok(antwort.inhalt),
            }
        }
    }
}
```

Beachte: Wir schreiben hier ein echtes `loop { ... }` — das Schlüsselwort selbst, jetzt
als Kontrollfluss verwendet — **innerhalb** einer Datei, die wegen desselben Wortes einen
rohen Bezeichner brauchte. Kein Widerspruch: `mod loop` (ein Name) und `loop { }` (eine
Anweisung) sind zwei völlig verschiedene Grammatikstellen; nur die erste kollidiert mit
dem Schlüsselwort.

`ok_or_else(...)` kennst du im Prinzip von `Option` — hier nutzen wir es, um "Werkzeug
nicht gefunden" **sofort und sicher** in einen `Err(AgentFehler::UnbekanntesWerkzeug(...))`
zu verwandeln, den `?` (implizit über den Rückgabetyp der Funktion) weiterreicht. Das ist
exakt der "sichere Abbruch bei unbekanntem Tool" aus der Transferaufgabe dieser Phase —
kein `panic!`, kein Absturz, sondern ein sprechender, behandelbarer Fehler.

Warum hängen wir sowohl `antwort.inhalt` (der rohe Werkzeugaufruf, als
`Rolle::Assistent`) **als auch** die Beobachtung (`Rolle::System`) an die Konversation
an? Damit das Modell beim nächsten Durchlauf seinen eigenen letzten Schritt und dessen
Ergebnis sieht — ohne Gedächtnis würde es denselben Aufruf möglicherweise wiederholen.
Warum `Rolle::System` für die Beobachtung, nicht `Rolle::Benutzer`? Die Beobachtung kommt
weder vom Menschen noch vom Modell, sondern von unserem eigenen Code — inhaltlich am
nächsten an "System teilt einen Fakt mit". Wir erweitern `Rolle` dafür bewusst **nicht**
um einen vierten Wert (etwa `Werkzeug`) — das würde die Namenskonvention aus
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md) brechen, die für
den ganzen Kurs gilt. `Konversation` erlaubt mehrere Systemnachrichten, auch nicht als
allererste — genau das nutzen wir hier aus.

> **💡 Tipp**
>
> `AgentLoop` **ruft** `Runnable` und `LlmProvider` aus
> [Phase 3](../04-phase3-architektur/README.md) auf, **implementiert** sie aber nicht
> selbst — noch nicht. Ein Agent, der selbst `Runnable` implementiert, ließe sich wie
> jede andere Chain-Komponente verketten (ein Agent als ein Glied in einer größeren
> Kette). Das vertiefen wir bewusst erst in
> [Phase 6, Lektion 4](../07-phase6-performance/04-multi-agent-orchestrierung.md) —
> hier reicht die einfache, direkt aufrufbare Form.

## Ausführung

```bash
cargo test -p mein_agent
```

Für einen Test brauchen wir einen Fake-Provider, der eine feste Folge von Antworten
liefert — dasselbe Prinzip wie der Fake-Provider aus
[Phase 3, Lektion 4](../04-phase3-architektur/04-fake-provider.md), hier lokal für den
Agent Loop gebaut, weil er eine ganze **Sequenz** von Antworten simulieren muss, nicht
nur eine einzelne:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mein_core::{Nachricht, Rolle};
    use std::sync::Mutex;

    struct FakeProvider {
        antworten: Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl mein_core::port::LlmProvider for FakeProvider {
        async fn antworten(
            &self,
            _verlauf: &Konversation,
        ) -> Result<Nachricht, Box<dyn std::error::Error + Send + Sync>> {
            let mut rest = self.antworten.lock().unwrap();
            let text = rest.remove(0);
            Ok(Nachricht::neu(Rolle::Assistent, text).expect("Testtext ist nicht leer"))
        }
    }

    #[tokio::test]
    async fn agent_beendet_sich_mit_finaler_antwort() {
        let provider = FakeProvider {
            antworten: Mutex::new(vec!["Die Antwort ist 5.".to_string()]),
        };
        let agent = AgentLoop::neu(Box::new(provider), vec![], 5);

        let ergebnis = agent.ausfuehren("Was ist 2 + 3?").await.unwrap();
        assert_eq!(ergebnis, "Die Antwort ist 5.");
    }
}
```

```
running 1 test
test agent::r#loop::tests::agent_beendet_sich_mit_finaler_antwort ... ok
```

## Zusammenfassung

- Der Agent Loop ist Plan → Tool-Aufruf → Beobachtung → nächste Aktion als `loop`, mit
  einem harten Abbruchkriterium (Schrittlimit).
- `loop` als Modulname kollidiert mit dem Schlüsselwort `loop` — gelöst über den rohen
  Bezeichner `r#loop`, nach außen unsichtbar dank `pub use`.
- `AgentLoop` hält `Box<dyn LlmProvider>` und `Vec<Box<dyn Tool>>` — beides Ports, keine
  konkreten Typen, genau wie in Phase 3 gelernt.
- Ein unbekanntes Werkzeug führt über `ok_or_else` + `?` zu einem sauberen
  `Err(AgentFehler::UnbekanntesWerkzeug(...))`, nie zu einem Panic.
- Sowohl der Werkzeugaufruf als auch seine Beobachtung werden der Konversation
  hinzugefügt (`Rolle::Assistent` bzw. `Rolle::System`) — ohne diese Erinnerung würde
  das Modell seinen letzten Schritt vergessen.

## Übung

Erweitere den Test `agent_beendet_sich_mit_finaler_antwort` um einen zweiten
`FakeProvider`, dessen `antworten`-Liste **zwei** Einträge hat: zuerst ein
Werkzeugaufruf-JSON (`{"werkzeug": "taschenrechner", "argumente": {...}}`), danach eine
finale Textantwort. Übergib diesmal einen echten `Taschenrechner` (aus
[Lektion 3](03-tool-schema-function-calling.md)) in der `werkzeuge`-Liste und prüfe, dass
`ausfuehren(...)` am Ende die finale Textantwort liefert — **und** dass der
`FakeProvider` tatsächlich zweimal aufgerufen wurde (z. B. über einen zusätzlichen
Zähler im Fake). Damit testest du zum ersten Mal den vollständigen Kreislauf der
Schleife, nicht nur ihr sofortiges Ende.

[Weiter: Lektion 5 — State und Memory](05-state-und-memory.md)
