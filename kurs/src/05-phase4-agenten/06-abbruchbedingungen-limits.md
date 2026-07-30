# Lektion 6: Abbruchbedingungen, Limits und Fehlerpfade

## Problem

Ein Agent, der nicht von selbst aufhören kann, ist gefährlich — nicht theoretisch,
sondern konkret: Jeder Schritt kostet Zeit, Geld (API-Aufrufe) und im schlimmsten Fall
reale Nebenwirkungen (ein Werkzeug, das eine E-Mail verschickt oder eine Datei löscht).
[Lektion 4](04-agent-loop.md) hat schon ein Schrittlimit eingebaut. Diese Lektion geht
einen Schritt weiter: Wir unterscheiden systematisch, **welche** Abbruchgründe es gibt,
stellen sicher, dass **jeder** davon zu einem sauberen `Err` führt statt zu einem Absturz
oder einer Endlosschleife — und ergänzen ein Zeitlimit pro Modellaufruf, damit ein
hängender Netzwerkaufruf den Agenten nicht für immer blockiert.

## Code (Zielbild)

```rust
pub struct AgentLoop {
    // ...
    max_schritte: usize,
    zeitlimit_pro_aufruf: std::time::Duration,
}

// Drei Abbruchgründe, jeder mit eigenem AgentFehler-Fall:
// 1. Schrittlimit erreicht      -> AgentFehler::SchrittLimitErreicht
// 2. Unbekanntes Werkzeug       -> AgentFehler::UnbekanntesWerkzeug
// 3. Modellaufruf zu langsam    -> AgentFehler::Zeitueberschreitung
```

## Dekonstruktion

### Drei Arten, wie ein Agent aufhören muss (nicht: darf)

- **Erfolgreicher Abschluss** — das Modell antwortet mit Text statt einem Werkzeugaufruf.
  Kein Fehlerfall, der Normalweg.
- **Erschöpfte Ressource** — das Schrittlimit ist erreicht, ohne dass eine finale Antwort
  kam. Kein Programmierfehler, sondern ein *erwartbarer* Grenzfall (vergleiche
  `Result` vs. `panic!` in
  [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md)) — deshalb `Err`, kein
  Absturz.
- **Ungültige Anfrage** — das Modell fordert ein Werkzeug an, das es nicht gibt (Tippfehler
  im Modell-Output, ein Werkzeug wurde entfernt, ein Prompt-Injection-Versuch von außen).
  Auch das ist erwartbar — kein Grund, das Programm abstürzen zu lassen.
- **Zeitüberschreitung** — ein Modellaufruf antwortet nicht innerhalb einer angemessenen
  Zeit. Netzwerke sind unzuverlässig; ohne Zeitlimit würde der Agent unbegrenzt lange auf
  eine Antwort warten, die vielleicht nie kommt.

Die gemeinsame Regel: **Kein einziger dieser Fälle darf zu einem Panic führen.** Jeder
bekommt eine eigene, benannte `AgentFehler`-Variante, die Aufrufer\*innen gezielt
behandeln können — genau wie `NachrichtFehler::LeererInhalt` in Phase 1 mit `match`
behandelbar war, statt einen kryptischen Programmabsturz zu verursachen.

### `tokio::time::timeout` — ein Future mit eingebauter Geduldsgrenze

```rust
use tokio::time::timeout;

let ergebnis = timeout(self.zeitlimit_pro_aufruf, self.provider.antworten(konversation)).await;
```

`timeout(dauer, future)` gibt selbst ein neues `Future` zurück, das entweder das
Ergebnis des inneren `Future`s liefert (wenn es rechtzeitig fertig wird) oder nach
`dauer` abbricht. Das Ergebnistyp ist entscheidend: `timeout(...)` liefert
`Result<T, Elapsed>`, wobei `T` **selbst** das ist, was dein inneres Future liefert. Ruft
also `self.provider.antworten(...)` bereits ein `Result<Nachricht, Fehler>`, bekommst du
nach `.await` ein **doppeltes** `Result`:
`Result<Result<Nachricht, Fehler>, Elapsed>`. Das ist eine der häufigsten
Überraschungen beim ersten Einsatz von `timeout` — wir provozieren sie gleich bewusst.

## Schritt-Reveal

**Schritt 1 — `AgentFehler` um Zeitüberschreitung erweitern.** In
`mein_agent/src/agent/loop.rs`:

```rust
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
    #[error("Zeitüberschreitung nach {0:?}")]
    Zeitueberschreitung(std::time::Duration),
    #[error("interner Aufbaufehler: {0}")]
    Aufbau(String),
}
```

**Schritt 2 — Provoziere den Fehler bewusst.** Baue `timeout` naiv mit `?` ein:

```rust
use tokio::time::{timeout, Duration};

async fn frage_mit_zeitlimit(&self, konversation: &Konversation) -> Result<mein_core::Nachricht, AgentFehler> {
    let antwort = timeout(self.zeitlimit_pro_aufruf, self.provider.antworten(konversation)).await?;
    Ok(antwort)
}
```

`cargo check -p mein_agent` meldet:

```
error[E0277]: `?` couldn't convert the error to `AgentFehler`
   --> mein_agent/src/agent/loop.rs:41:85
    |
41  |     let antwort = timeout(self.zeitlimit_pro_aufruf, self.provider.antworten(konversation)).await?;
    |                                                                                             ^ the trait `From<Elapsed>` is not implemented for `AgentFehler`
    |
    = help: the trait `From<Elapsed>` is implemented for a bunch of other types (not this one)
note: the following error was returned but ... doesn't have a `From` implementation for it
   --> mein_agent/src/agent/loop.rs:41:85
```

Zwei Probleme stecken in dieser einen Zeile, auch wenn der Compiler zuerst nur das erste
zeigt: Erstens fehlt `AgentFehler: From<Elapsed>` (`Elapsed` ist der Fehlertyp von
`timeout`, wenn die Zeit abläuft). Zweitens — selbst wenn wir das beheben — bekommt
`antwort` danach den Typ `Result<mein_core::Nachricht, ProviderFehler>` statt direkt
`mein_core::Nachricht`, weil `?` hier nur die **äußere** `Elapsed`-Schicht entfernt, nicht
die innere vom `LlmProvider` selbst. Der Rückgabetyp der Funktion (`Ok(antwort)` mit
`antwort: mein_core::Nachricht`) passt dann nicht mehr.

**Schritt 3 — Korrektur: beide Schichten explizit behandeln.**

```rust
async fn frage_mit_zeitlimit(
    &self,
    konversation: &Konversation,
) -> Result<mein_core::Nachricht, AgentFehler> {
    let ergebnis = timeout(
        self.zeitlimit_pro_aufruf,
        self.provider.antworten(konversation),
    )
    .await
    .map_err(|_elapsed| AgentFehler::Zeitueberschreitung(self.zeitlimit_pro_aufruf))?;

    ergebnis.map_err(|e| AgentFehler::Provider(e.to_string()))
}
```

Die erste `.map_err(...)` behandelt die äußere Schicht (`Elapsed` → unser eigener
`AgentFehler::Zeitueberschreitung`), das erste `?` entpackt sie. Danach ist `ergebnis` vom
Typ `Result<mein_core::Nachricht, ProviderFehler>` — die innere Schicht, die wir mit
einem zweiten `.map_err(...)` (ohne `?`, als letzter Ausdruck der Funktion) ebenfalls in
unseren eigenen Fehlertyp übersetzen.

> **💡 Tipp**
>
> Diese "Zwiebelschale" — ein `Result` innerhalb eines `Result` — taucht in
> asynchronem Rust häufiger auf, überall dort, wo ein zeitbegrenztes (`timeout`) oder
> abbrechbares (`select!`) Future ein anderes, selbst fehlbares Future umschließt. Die
> Faustregel: Schau dir bei jedem `.await` **genau** den Rückgabetyp an (`cargo check`
> oder dein Editor zeigen ihn dir), statt ihn zu erraten.

**Schritt 4 — In `AgentLoop::ausfuehren` einsetzen.** Ersetze den direkten Aufruf von
`self.provider.antworten(...)` durch `self.frage_mit_zeitlimit(zustand.konversation())`.
Ergänze das Feld im Struct, einen sinnvollen Standardwert in `neu(...)`, und eine
optionale Builder-Methode, mit der Tests (und später Aufrufer\*innen) ihn gezielt
überschreiben können:

```rust
pub struct AgentLoop {
    provider: Box<dyn mein_core::port::LlmProvider>,
    werkzeuge: Vec<Box<dyn Tool>>,
    max_schritte: usize,
    zeitlimit_pro_aufruf: Duration,
}

impl AgentLoop {
    pub fn neu(
        provider: Box<dyn mein_core::port::LlmProvider>,
        werkzeuge: Vec<Box<dyn Tool>>,
        max_schritte: usize,
    ) -> Self {
        AgentLoop {
            provider,
            werkzeuge,
            max_schritte,
            zeitlimit_pro_aufruf: Duration::from_secs(30),
        }
    }

    pub fn mit_zeitlimit(mut self, dauer: Duration) -> Self {
        self.zeitlimit_pro_aufruf = dauer;
        self
    }

    // ausfuehren(...) und frage_mit_zeitlimit(...) wie oben
}
```

`mit_zeitlimit` folgt dem **Builder-Muster** (*builder pattern*): Es nimmt `self` per
Wert entgegen, verändert ein Feld und gibt `self` zurück — dadurch lässt sich `AgentLoop::neu(...).mit_zeitlimit(...)`
verkettet schreiben. Wir vertiefen dieses Muster systematisch erst in
[Phase 7, Lektion 1](../08-phase7-release/01-builder-pattern.md); hier reicht die kleine,
konkrete Anwendung.

**Schritt 5 — Tokio-Feature `time` sicherstellen.** Falls noch nicht geschehen (wir haben
es schon in [Lektion 1](01-async-und-tokio.md) mit aufgenommen):

```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

> **⚠️ Warnung**
>
> Ein zu kurzes `zeitlimit_pro_aufruf` bricht legitime, nur etwas langsame Anfragen ab
> — ein zu langes lässt den Agenten (und wartende Nutzer\*innen) unnötig lange hängen.
> Es gibt keinen universell richtigen Wert; miss die tatsächliche Antwortzeit deines
> Providers und wähle großzügig darüber, nicht knapp.

## Ausführung

```bash
cargo test -p mein_agent
```

Ein Test für das Zeitlimit braucht einen `FakeProvider`, der künstlich verzögert
(`tokio::time::sleep` vor der Antwort):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct LangsamerFakeProvider;

    #[async_trait::async_trait]
    impl mein_core::port::LlmProvider for LangsamerFakeProvider {
        async fn antworten(
            &self,
            _verlauf: &Konversation,
        ) -> Result<mein_core::Nachricht, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok(mein_core::Nachricht::neu(mein_core::Rolle::Assistent, "zu spät").unwrap())
        }
    }

    #[tokio::test]
    async fn zeitlimit_bricht_zu_langsamen_aufruf_ab() {
        let agent = AgentLoop::neu(Box::new(LangsamerFakeProvider), vec![], 5)
            .mit_zeitlimit(Duration::from_millis(20));

        let ergebnis = agent.ausfuehren("Testaufgabe").await;
        assert!(matches!(ergebnis, Err(AgentFehler::Zeitueberschreitung(_))));
    }
}
```

```
running 1 test
test agent::r#loop::tests::zeitlimit_bricht_zu_langsamen_aufruf_ab ... ok
```

`matches!(...)` prüft, ob ein Wert zu einem bestimmten Pattern passt, ohne dass du den
Fehler auf Gleichheit prüfen musst (praktisch, weil `Duration` in unserem Fehler steckt
und wir uns nicht für den exakten Wert interessieren, nur für die Variante).

## Zusammenfassung

- Vier saubere Abbruchwege statt eines: Erfolg, Schrittlimit, unbekanntes Werkzeug,
  Zeitüberschreitung — jeder mit eigener `AgentFehler`-Variante, keiner mit `panic!`.
- `tokio::time::timeout(...)` liefert ein doppeltes `Result`, wenn das umschlossene
  Future selbst fehlbar ist — beide Schichten brauchen eigene Fehlerbehandlung.
- Ein Zeitlimit schützt vor hängenden Netzwerkaufrufen, ein Schrittlimit vor
  Endlos-Schleifen, eine Prüfung auf bekannte Werkzeugnamen vor unerwarteten Anfragen —
  drei unabhängige Schutzmechanismen für drei unabhängige Risiken.

## Übung — Transferaufgabe der Phase

**Der Agent darf höchstens fünf Schritte ausführen und muss bei einem unbekannten Tool
sicher abbrechen.**

Konfiguriere einen `AgentLoop` mit `max_schritte = 5` und schreibe zwei Tests, die genau
diese Anforderung beweisen:

1. Ein `FakeProvider`, der **immer** einen Werkzeugaufruf zurückgibt (z. B. dasselbe
   Taschenrechner-JSON wieder und wieder), kombiniert mit einem echten `Taschenrechner`
   als einzigem Werkzeug. Prüfe, dass `ausfuehren(...)` nach genau fünf Schritten mit
   `Err(AgentFehler::SchrittLimitErreicht { max: 5 })` zurückkommt — nicht früher, nicht
   später, und ganz sicher nicht mit einem Panic oder einer hängenden Testausführung.
2. Ein `FakeProvider`, der einen Aufruf für ein Werkzeug namens `"nicht_vorhanden"`
   zurückgibt, das **nicht** in der `werkzeuge`-Liste steht. Prüfe, dass
   `ausfuehren(...)` sofort (nach dem ersten Schritt, nicht erst nach fünf) mit
   `Err(AgentFehler::UnbekanntesWerkzeug("nicht_vorhanden".to_string()))` zurückkommt.

Zwei Leitfragen, falls du nicht weiterkommst: Wie zählst du in deinem `FakeProvider`
mit, wie oft er schon aufgerufen wurde, um beim Test 1 sicherzustellen, dass es
tatsächlich genau fünf Durchläufe waren (nicht nur "irgendwann kam ein Fehler")? Und:
Prüfst du beim Schrittlimit **vor** oder **nach** dem Modellaufruf — was bedeutet das für
die Anzahl der tatsächlich ausgeführten Werkzeugaufrufe bei `max_schritte = 5`? Schau dir
dazu die Reihenfolge in `AgentLoop::ausfuehren` aus [Lektion 5](05-state-und-memory.md)
noch einmal genau an. Du prüfst deine Lösung in
[Lektion 8](08-release-4.md) gegen die Definition of Done.

[Weiter: Lektion 7 — Optionaler MCP-Client](07-mcp-client.md)
