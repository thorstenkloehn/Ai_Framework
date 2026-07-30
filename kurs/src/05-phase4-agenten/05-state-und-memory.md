# Lektion 5: State und Memory

## Problem

`AgentLoop::ausfuehren` aus [Lektion 4](04-agent-loop.md) hält seinen Zustand — die
`Konversation` und den Schrittzähler — bislang in zwei losen lokalen Variablen
(`konversation`, `schritt`). Das funktioniert für eine einzelne, in sich geschlossene
Ausführung. Aber schon zwei naheliegende Erweiterungen sprengen das: Wollen wir den
**Fortschritt eines laufenden Agenten** von außen beobachten (z. B. für ein Live-Log,
während der Agent noch läuft), oder wollen wir den Zustand **pausieren und später
fortsetzen**, brauchen wir einen Zustand, der ein eigenständiger, benannter Typ ist —
und der sich sicher **teilen** lässt, auch über mehrere nebenläufige Tokio-Tasks hinweg.

## Code (Zielbild)

```rust
pub struct AgentState {
    konversation: mein_core::Konversation,
    schritt_zaehler: usize,
}

impl AgentState {
    pub fn neu(werkzeuge: &[Box<dyn crate::agent::Tool>], aufgabe: String) -> Result<Self, crate::agent::AgentFehler> {
        // ...
    }

    pub fn beobachtung(&mut self, werkzeug_name: &str, aufruf_roh: &str, ergebnis: &str) -> Result<(), crate::agent::AgentFehler> {
        // ...
    }
}
```

## Dekonstruktion

### Warum ein eigener Typ statt zwei loser Variablen?

Dasselbe Argument wie bei `Konversation` selbst in
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md): Ein privates Feld plus
kontrollierende Methoden verhindert, dass irgendwo im Code der Schrittzähler erhöht
wird, ohne dass eine passende Beobachtung dazu existiert (oder umgekehrt). `AgentState`
kapselt genau dieses Zusammenspiel — "Konversation" und "Schrittzähler" sind fachlich
untrennbar: Jeder abgeschlossene Werkzeugaufruf verändert **beide** gleichzeitig.

### `Konversation` vs. `AgentState` — zwei verschiedene Gedächtnisse

Es lohnt sich, hier präzise zu sein, weil beide Typen ähnlich aussehen, aber
unterschiedliche Aufgaben haben:

- **`Konversation`** ([Phase 1](../02-phase1-fundament/04-konversation.md)) ist der
  **fachliche** Gesprächsverlauf — das, was wir dem Modell als Kontext schicken.
- **`AgentState`** ist der **Ausführungszustand** des Agenten selbst: wie viele Schritte
  schon gelaufen sind, und (in der Übung dieser Lektion) optional weitere Buchführung,
  die nie an das Modell geschickt wird, z. B. Zeitstempel oder interne Notizen.

`AgentState` **enthält** eine `Konversation`, ist aber nicht dasselbe wie sie — genau wie
ein Auto ein Lenkrad enthält, aber kein Lenkrad *ist*.

### `AgentState::neu` gibt `Result` zurück, nicht `Self`

```rust
pub fn neu(werkzeuge: &[Box<dyn Tool>], aufgabe: String) -> Result<Self, AgentFehler> {
    let katalog = tool::katalog_als_text(werkzeuge);
    let konversation = mein_core::Konversation::mit_systemnachricht(katalog)
        .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
    let mut zustand = AgentState { konversation, schritt_zaehler: 0 };
    zustand
        .konversation
        .hinzufuegen(mein_core::Rolle::Benutzer, aufgabe)
        .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
    Ok(zustand)
}
```

`Konversation::mit_systemnachricht` und `hinzufuegen` können theoretisch fehlschlagen
(leerer Inhalt, siehe [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md)).
Auch wenn wir als Autor\*innen von `mein_agent` sicherstellen, dass der Werkzeugkatalog
und die Aufgabe hier nie leer sind: Produktionscode aus Phase 2 an vermeidet `.unwrap()`
bewusst (siehe [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)) — wir
reichen den Fehler stattdessen sauber über `?` weiter, genau wie im Agent Loop selbst.

## Schritt-Reveal

**Schritt 1 — `AgentState` schreiben.** Lege `mein_agent/src/agent/state.rs` an:

```rust
use crate::agent::{tool, AgentFehler, Tool};
use mein_core::{Konversation, Rolle};

pub struct AgentState {
    konversation: Konversation,
    schritt_zaehler: usize,
}

impl AgentState {
    pub fn neu(werkzeuge: &[Box<dyn Tool>], aufgabe: String) -> Result<Self, AgentFehler> {
        let katalog = tool::katalog_als_text(werkzeuge);
        let konversation = Konversation::mit_systemnachricht(katalog)
            .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
        let mut zustand = AgentState { konversation, schritt_zaehler: 0 };
        zustand
            .konversation
            .hinzufuegen(Rolle::Benutzer, aufgabe)
            .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
        Ok(zustand)
    }

    pub fn konversation(&self) -> &Konversation {
        &self.konversation
    }

    pub fn schritt_zaehler(&self) -> usize {
        self.schritt_zaehler
    }

    pub fn beobachtung(
        &mut self,
        werkzeug_name: &str,
        aufruf_roh: &str,
        ergebnis: &str,
    ) -> Result<(), AgentFehler> {
        self.konversation
            .hinzufuegen(Rolle::Assistent, aufruf_roh.to_string())
            .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
        self.konversation
            .hinzufuegen(Rolle::System, format!("Beobachtung von {werkzeug_name}: {ergebnis}"))
            .map_err(|e| AgentFehler::Aufbau(format!("{e:?}")))?;
        self.schritt_zaehler += 1;
        Ok(())
    }
}
```

Trage `state` in `mein_agent/src/agent/mod.rs` ein:

```rust
pub mod state;
pub mod tool;
pub mod r#loop;

pub use r#loop::{AgentFehler, AgentLoop};
pub use state::AgentState;
pub use tool::{Tool, ToolFehler, Werkzeugaufruf};
```

**Schritt 2 — `AgentLoop::ausfuehren` auf `AgentState` umstellen.** Ersetze die beiden
losen Variablen aus [Lektion 4](04-agent-loop.md) durch den neuen Typ:

```rust
pub async fn ausfuehren(&self, aufgabe: impl Into<String>) -> Result<String, AgentFehler> {
    let mut zustand = AgentState::neu(&self.werkzeuge, aufgabe.into())?;

    loop {
        if zustand.schritt_zaehler() >= self.max_schritte {
            return Err(AgentFehler::SchrittLimitErreicht { max: self.max_schritte });
        }

        let antwort = self
            .provider
            .antworten(zustand.konversation())
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
                zustand.beobachtung(werkzeug.name(), &antwort.inhalt, &ergebnis)?;
            }
            None => return Ok(antwort.inhalt),
        }
    }
}
```

`cargo check -p mein_agent` — spürbar kürzer und lesbarer als vorher: `AgentLoop` weiß
jetzt nichts mehr über die interne Struktur des Zustands, nur noch über die drei
Methoden `neu`, `konversation`, `beobachtung`, `schritt_zaehler` — dasselbe
Kapselungsprinzip wie bei `Konversation` selbst.

**Schritt 3 — Provoziere den `Send`-Fehler bewusst.** Stell dir vor, wir wollen den
Fortschritt eines laufenden Agenten **nebenläufig** protokollieren: ein zweiter
Tokio-Task, der parallel zum Agenten läuft und mitliest. Naiv geteilt über `Rc`/`RefCell`
(aus [Kapitel 0](../01-grundlagen/04-daten-buendeln.md) kennst du `Rc` eventuell noch
nicht — es ist ein Zeigertyp für "mehrere Besitzer\*innen im selben Thread", ohne den
Overhead von Thread-Sicherheit):

```rust
use std::cell::RefCell;
use std::rc::Rc;

async fn protokolliere_nebenlaeufig(zustand: Rc<RefCell<AgentState>>) {
    tokio::spawn(async move {
        let schritte = zustand.borrow().schritt_zaehler();
        println!("Bisher {schritte} Schritte.");
    });
}
```

`cargo check -p mein_agent` meldet sinngemäß:

```
error[E0277]: `Rc<RefCell<AgentState>>` cannot be sent between threads safely
   --> mein_agent/src/agent/state.rs:52:18
    |
52  |     tokio::spawn(async move {
    |                  ^^^^^^^^^^^ `Rc<RefCell<AgentState>>` cannot be sent between threads safely
    |
    = help: the trait `Send` is not implemented for `Rc<RefCell<AgentState>>`
    = note: required because it's used within this `async` block
note: required by a bound in `tokio::spawn`
```

### Warum genau dieser Fehler, und warum jetzt?

`tokio::spawn(...)` übergibt den Future an Tokios Scheduler, der ihn — bei der
Multi-Thread-Runtime aus [Lektion 1](01-async-und-tokio.md) — auf **jedem beliebigen**
Worker-Thread laufen lassen (und zwischen Threads verschieben) darf. Deshalb verlangt
`tokio::spawn` von seinem Future: `Send`, also "darf sicher zwischen Threads wandern".
`Rc<RefCell<T>>` ist absichtlich **nicht** `Send`: Sein interner Referenzzähler ist nicht
gegen gleichzeitigen Zugriff aus mehreren Threads abgesichert — zwei Threads, die
gleichzeitig einen `Rc`-Zähler erhöhen, könnten sich gegenseitig überschreiben, ein
klassisches *Data Race*. Der Compiler verhindert das nicht zur Laufzeit (wie in Sprachen
mit Garbage Collector), sondern schon beim Kompilieren, indem er `Rc<RefCell<T>>` schlicht
kein `Send` gibt — der Fehler, den du gerade siehst.

**Schritt 4 — Korrektur mit `Arc<tokio::sync::Mutex<...>>`.**

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn protokolliere_nebenlaeufig(zustand: Arc<Mutex<AgentState>>) {
    tokio::spawn(async move {
        let schritte = zustand.lock().await.schritt_zaehler();
        println!("Bisher {schritte} Schritte.");
    });
}
```

`Arc<T>` ("*atomic* reference counted") ist das Thread-sichere Gegenstück zu `Rc<T>`: Sein
Zähler wird über atomare CPU-Operationen verändert, sicher auch bei gleichzeitigem
Zugriff aus mehreren Threads — deshalb implementiert `Arc<T>` `Send` (wenn `T: Send +
Sync` ist). `tokio::sync::Mutex` (nicht `std::sync::Mutex`!) ist Tokios eigene, **async-
fähige** Variante: `.lock()` selbst ist eine `async fn`, die den aktuellen Task pausiert,
statt den ganzen Thread zu blockieren, während sie auf freien Zugriff wartet — ein
blockierender `std::sync::Mutex::lock()` innerhalb eines `.await`-Punkts wäre ein
subtiler Performance-Fehler, den Tokios eigener Mutex vermeidet.

> **⚠️ Warnung**
>
> `Arc<Mutex<T>>` löst das `Send`-Problem, ist aber kein Freibrief, Zustand überall
> gemeinsam zu teilen. Jeder `.lock().await` ist ein potenzieller Wartepunkt für alle
> anderen, die denselben Zustand gerade brauchen. Für den `AgentLoop` selbst
> (Lektionen 4-6) brauchen wir das **nicht** — er läuft sequenziell in einem einzigen
> Task. Teile Zustand erst dann, wenn du wirklich mehrere Tasks hast, die gleichzeitig
> darauf zugreifen müssen (ein Vorgeschmack auf
> [Phase 6, Multi-Agent-Orchestrierung](../07-phase6-performance/04-multi-agent-orchestrierung.md)).

## Ausführung

```bash
cargo test -p mein_agent
```

Ergänze einen Test direkt für `AgentState`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neuer_zustand_hat_null_schritte() {
        let zustand = AgentState::neu(&[], "Testaufgabe".to_string()).unwrap();
        assert_eq!(zustand.schritt_zaehler(), 0);
        assert_eq!(zustand.konversation().verlauf().len(), 2); // System + Benutzer
    }

    #[test]
    fn beobachtung_erhoeht_schrittzaehler() {
        let mut zustand = AgentState::neu(&[], "Testaufgabe".to_string()).unwrap();
        zustand.beobachtung("taschenrechner", "{}", "5").unwrap();
        assert_eq!(zustand.schritt_zaehler(), 1);
        assert_eq!(zustand.konversation().verlauf().len(), 4); // + Assistent + System
    }
}
```

```
running 2 tests
test agent::state::tests::neuer_zustand_hat_null_schritte ... ok
test agent::state::tests::beobachtung_erhoeht_schrittzaehler ... ok
```

## Zusammenfassung

- `AgentState` kapselt `Konversation` (fachlicher Verlauf) und Schrittzähler
  (Ausführungszustand) hinter kontrollierenden Methoden — dieselbe Kapselung wie bei
  `Konversation` selbst.
- `AgentLoop` kennt nur noch die öffentliche Schnittstelle von `AgentState`, nicht seine
  interne Struktur.
- `Rc<RefCell<T>>` ist für einen einzelnen Thread gedacht und deshalb nicht `Send` —
  `tokio::spawn` verlangt `Send`, weil die Multi-Thread-Runtime Tasks zwischen Threads
  verschieben darf.
- `Arc<tokio::sync::Mutex<T>>` ist die Thread- **und** Async-sichere Alternative:
  atomarer Referenzzähler, non-blocking `.lock().await`.
- Zustand zu teilen ist kein Standardfall für den `AgentLoop` selbst — er läuft
  sequenziell; geteilter Zustand lohnt sich erst, wenn echte Nebenläufigkeit dazukommt.

## Übung

Erweitere `AgentState` um ein Feld `notizen: Vec<String>` (das eigene "Memory" des
Agenten, das **nicht** Teil der `Konversation` ist und deshalb auch nie an das Modell
geschickt wird) und eine Methode `merke(&mut self, notiz: impl Into<String>)`. Rufe sie
testweise nach jeder Beobachtung mit einer kurzen Zusammenfassung auf (z. B. `"Werkzeug
{name} ergab: {ergebnis}"`). Schreibe einen Test, der prüft, dass `notizen.len()` nach
zwei Beobachtungen `2` ist. Überlege dir als Leitfrage: Warum sollten `notizen`
**bewusst nicht** automatisch in die `Konversation` einfließen — was würde passieren,
wenn jede interne Notiz zusätzlich Tokens beim nächsten Modellaufruf kosten würde?

[Weiter: Lektion 6 — Abbruchbedingungen und Limits](06-abbruchbedingungen-limits.md)
