# Lektion 7: Tracing, Kosten-Tracking und Secrets

## Problem

Drei Betriebsprobleme, die alle damit zu tun haben, dass unser Framework jetzt als
Dienst läuft ([Lektion 5](05-rest-axum-oder-tui.md)) statt als kurzlebiger CLI-Aufruf:

1. **Nachvollziehbarkeit:** Wenn eine Anfrage an `mein_server` fehlschlägt oder lange
   dauert, reicht ein `println!` nicht mehr — wir brauchen strukturierte, durchsuchbare
   Aufzeichnungen darüber, was während einer Anfrage passiert ist (**Tracing**, zu
   Deutsch etwa "Verfolgen des Ablaufs" — nicht zu verwechseln mit dem deutschen Wort
   "Nachverfolgung" im Sinne von Logistik; gemeint ist hier das englische
   Software-Konzept, das auch das gleichnamige Rust-Crate `tracing` prägt).
2. **Kosten:** Jeder LLM-Aufruf kostet Geld, abhängig von der Anzahl verarbeiteter
   Tokens. Ohne Tracking weiß niemand, wie teuer eine einzelne Anfrage oder ein Tag im
   Betrieb war.
3. **Secrets:** Ein API-Key, der im Klartext in einem Log oder einer Debug-Ausgabe
   landet, ist ein Sicherheitsvorfall. Wir brauchen einen Typ, der einen Schlüssel hält,
   ihn aber nirgendwo versehentlich preisgibt — und ihn beim Verwerfen aktiv aus dem
   Speicher löscht.

## Code (Zielbild)

```rust
#[instrument(skip(anfrage_text))]
async fn anfrage_an_provider(modell: &str, anfrage_text: &str) -> Result<String, String> {
    info!(zeichen = anfrage_text.len(), "sende Anfrage an Provider");
    // ...
}
```

```rust
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ApiSchluessel(String);
// Debug-Ausgabe zeigt "ApiSchluessel(***)", niemals den echten Wert.
```

## Dekonstruktion

### `mein_core::telemetry` — strukturiertes statt zeilenweises Logging

`println!`/`eprintln!` (aus [Phase 1](../02-phase1-fundament/03-invarianten.md)) geben
reinen Text aus — praktisch für ein CLI-Programm, unpraktisch für einen Server, dessen
Logs später maschinell durchsucht oder gefiltert werden sollen ("zeig mir alle
Ereignisse zu Anfrage-ID X"). Das Crate `tracing` führt dafür zwei Konzepte ein:

- **Events** (`info!`, `warn!`, `error!`) — einzelne, strukturierte Aufzeichnungen mit
  benannten Feldern statt einem einzigen zusammengebauten String.
- **Spans** (`#[instrument]`) — ein Zeitraum mit Anfang und Ende, der mehrere Events
  gruppiert und automatisch mit Kontext versieht (z. B. "alles, was innerhalb dieses
  Funktionsaufrufs passiert ist").

`#[instrument(skip(anfrage_text))]` erzeugt automatisch einen Span um die gesamte
Funktion, inklusive ihrer Parameter als Kontext — `skip(anfrage_text)` nimmt den langen
Anfragetext bewusst davon aus (sonst würde jeder komplette Prompt in jedem Log-Eintrag
auftauchen, was sowohl unübersichtlich als auch potenziell sensibel ist).

### `mein_core::secrets` — API-Keys, die sich nicht versehentlich zeigen

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ApiSchluessel(String);
```

`Zeroize` (ein Trait aus dem gleichnamigen Crate) überschreibt den Speicher eines Werts
beim Verwerfen aktiv mit Nullen, statt ihn einfach freizugeben. Der Unterschied ist
wichtig: Normales Rust-`Drop` gibt Speicher nur zur Wiederverwendung frei — der
*Inhalt* (der API-Key) kann rein technisch noch eine Weile im Arbeitsspeicher stehen
bleiben, bis er überschrieben wird, und wäre theoretisch z. B. über einen Speicherauszug
(*memory dump*) auslesbar. `ZeroizeOnDrop` sorgt dafür, dass genau das beim automatischen
`Drop` (wenn ein `ApiSchluessel` seinen Gültigkeitsbereich verlässt) passiert.

```rust
impl std::fmt::Debug for ApiSchluessel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiSchluessel(***)")
    }
}
```

Wir implementieren `Debug` **von Hand** statt es abzuleiten (`#[derive(Debug)]`), damit
`{:?}`-Ausgaben (z. B. in einem versehentlichen `println!("{:?}", config)` oder einem
`tracing`-Event) niemals den echten Schlüssel zeigen — nur die Maskierung `***`.

> **⚠️ Warnung**
>
> `#[derive(Debug)]` **und** ein manuelles `impl Debug` gleichzeitig ergeben einen
> Compilerfehler (siehe Schritt-Reveal) — der Compiler kennt sonst zwei widersprüchliche
> Implementierungen für denselben Trait auf demselben Typ. Du musst dich für eine
> Variante entscheiden.

### `Kostenschätzung` — bewusst deutsch benannt

```rust
#[derive(Debug, Clone, Default)]
pub struct Kostenschaetzung {
    pub eingabe_tokens: u32,
    pub ausgabe_tokens: u32,
    pub kosten_usd: f64,
}
```

Anders als `DocumentLoader` oder `Retriever` ist Kosten-Tracking ein Geschäftsbegriff,
den auch Nicht-Programmierer:innen verstehen sollen (Faustregel aus dem
Namenskonventions-Abschnitt, den du seit
[Phase 1](../02-phase1-fundament/README.md) kennst) — deshalb bleibt der Typname
deutsch, konsistent mit `Konfiguration` und `Konversation`.

## Schritt-Reveal

**Schritt 1 — Abhängigkeiten ergänzen** in `mein_core/Cargo.toml`:

```bash
cargo add tracing
cargo add tracing-subscriber --features env-filter
cargo add zeroize --features derive
```

**Schritt 2 — `ApiSchluessel` bewusst fehlerhaft anlegen** (Debug doppelt):

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct ApiSchluessel(String);

impl std::fmt::Debug for ApiSchluessel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiSchluessel(***)")
    }
}
```

`cargo check -p mein_core`:

```
error[E0119]: conflicting implementations of trait `Debug` for type `ApiSchluessel`
 --> src/secrets.rs:3:10
  |
3 | #[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
  |          ^^^^^ conflicting implementation for `ApiSchluessel`
...
6 | impl std::fmt::Debug for ApiSchluessel {
  | -------------------------------------- first implementation here
```

`#[derive(Debug)]` erzeugt bereits eine `impl Debug for ApiSchluessel` — unsere eigene,
maskierende Implementierung direkt darunter ist eine **zweite** Implementierung
desselben Traits für denselben Typ, und das erlaubt Rust nicht (jeder Trait darf pro Typ
nur einmal implementiert sein, sonst wüsste der Compiler bei `{:?}` nicht, welche der
beiden gemeint ist).

**Schritt 3 — `Debug` aus der `derive`-Liste entfernen:**

```rust
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ApiSchluessel(String);

impl ApiSchluessel {
    pub fn neu(wert: impl Into<String>) -> Self {
        ApiSchluessel(wert.into())
    }
}

impl std::fmt::Debug for ApiSchluessel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiSchluessel(***)")
    }
}
```

`cargo check -p mein_core` — kompiliert.

**Schritt 4 — Tracing in einen Provider-Aufruf einbauen** (in
`mein_core::provider`, ergänzend zu `mit_backoff` aus [Lektion 6](06-retry-rate-limit-backoff.md)):

```rust
use tracing::{info, instrument, warn};

#[instrument(skip(anfrage_text))]
async fn anfrage_an_provider(modell: &str, anfrage_text: &str) -> Result<String, String> {
    info!(zeichen = anfrage_text.len(), "sende Anfrage an Provider");
    // ... eigentlicher reqwest-Aufruf ...
    if modell == "kaputtes-modell" {
        warn!("Provider antwortet mit Fehler");
        return Err("503".to_string());
    }
    Ok("Antwort vom Modell".to_string())
}
```

**Schritt 5 — Einen Subscriber in `main` einrichten**, damit Events überhaupt irgendwo
ausgegeben werden (ohne Subscriber sammelt `tracing` Events, zeigt sie aber nirgends an):

```rust
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ergebnis = anfrage_an_provider("gpt-irgendwas", "Wie beantrage ich Urlaub?").await;
    info!(?ergebnis, "Anfrage abgeschlossen");
}
```

## Ausführung

```bash
RUST_LOG=info cargo run -p mein_cli
```

```
2026-07-30T11:09:26Z  INFO anfrage_an_provider{modell="gpt-irgendwas"}: mein_core: sende Anfrage an Provider zeichen=25
2026-07-30T11:09:26Z  INFO mein_core: Anfrage abgeschlossen ergebnis=Ok("Antwort vom Modell")
```

`RUST_LOG=info` steuert, welche Detailstufe der Subscriber ausgibt (`trace` < `debug` <
`info` < `warn` < `error`) — ohne diese Umgebungsvariable bleibt die Konsole meist still,
weil der Standard-Filterlevel höher liegt.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_zeigt_schluessel_nicht_im_klartext() {
        let schluessel = ApiSchluessel::neu("sk-geheim-123");
        let ausgabe = format!("{:?}", schluessel);
        assert!(!ausgabe.contains("sk-geheim-123"));
        assert_eq!(ausgabe, "ApiSchluessel(***)");
    }
}
```

```bash
cargo test -p mein_core
```

```
running 1 test
test secrets::tests::debug_zeigt_schluessel_nicht_im_klartext ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Provoziere den Fehlerpfad bewusst: Rufe `anfrage_an_provider("kaputtes-modell", "...")`
auf und beobachte, dass zusätzlich zum `Err`-Rückgabewert eine `WARN`-Zeile im Log
auftaucht — Tracing und Fehlerbehandlung ergänzen sich, sie ersetzen sich nicht.

## Zusammenfassung

- `tracing` liefert strukturierte Events und Spans statt reiner Textzeilen —
  durchsuchbar, mit automatischem Funktionskontext über `#[instrument]`.
- `Kostenschaetzung` bleibt bewusst deutsch benannt, weil Kosten-Tracking ein
  Geschäftsbegriff ist, kein Architektur-Fachbegriff.
- `zeroize`/`ZeroizeOnDrop` löschen sensible Daten aktiv aus dem Speicher, statt sich auf
  normales `Drop` zu verlassen.
- `#[derive(Debug)]` und ein manuelles `impl Debug` für denselben Typ sind ein
  Compilerfehler (`E0119`) — für maskierte Debug-Ausgaben brauchst du **entweder** die
  Ableitung **oder** die eigene Implementierung, nie beides.
- Secrets gehören strukturell getrennt von normalen Konfigurationswerten, mit einem Typ,
  der versehentliches Preisgeben so schwer wie möglich macht.

## Übung

Baue eine `Kostenschaetzung` so in `mit_backoff` aus
[Lektion 6](06-retry-rate-limit-backoff.md) ein, dass jeder (auch fehlgeschlagene)
Versuch als eigener `tracing`-Span sichtbar wird, mit der jeweiligen Versuchsnummer als
Feld. Nutze `#[instrument]` oder einen manuell erzeugten Span
(`tracing::info_span!("versuch", nummer = versuch)`) und überlege, warum es für die
Fehlersuche wertvoll ist, in den Logs zu sehen, dass eine Anfrage erst beim dritten
Versuch erfolgreich war, statt nur das Endergebnis zu sehen.

[Weiter: Lektion 8 — Prompt-Injection-Schutz, Docker und CI](08-security-docker-ci.md)
