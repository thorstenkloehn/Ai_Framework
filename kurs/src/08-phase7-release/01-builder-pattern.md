# Lektion 1: Builder Pattern und Defaults

## Problem

Erinnerst du dich an das allererste, unformatierte `main.rs` aus
[Phase 1, Lektion 1](../02-phase1-fundament/01-workspace-lesen.md)? Seitdem ist unsere
`Konfiguration` ([Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md))
gewachsen: API-Key, Modellname, Basis-URL, Temperatur — und seit
[Phase 6](../07-phase6-performance/03-model-routing-fallback.md) potenziell noch eine
Routing-Schwelle und ein zweites Modell für den Fallback. Ein Konstruktor mit sieben
positionellen Parametern (`Client::neu(api_key, modell, basis_url, temperatur, timeout,
guenstig_modell, schwelle)`) ist für niemanden lesbar — und gefährlich: Vertauschst du
zwei `String`-Parameter derselben Art, merkt der Compiler nichts, das Programm kompiliert
anstandslos und verhält sich falsch. Wir brauchen eine Konstruktion, die auch mit vielen
optionalen Werten lesbar bleibt und Vertauschungsfehler unmöglich macht.

## Code (Zielbild)

```rust
let client = ClientBuilder::neu()
    .modell("irgendein-modell")
    .temperatur(0.3)
    .api_key_aus_umgebung("MEIN_API_KEY")
    .bauen()?;
```

## Dekonstruktion

### Das Builder Pattern

Das **Builder Pattern** trennt "einen Wert Schritt für Schritt zusammenstellen" von "den
fertigen Wert benutzen". Statt eines einzigen Konstruktors mit allen möglichen Parametern
auf einmal, bekommen wir einen eigenen `ClientBuilder`-Typ mit einer Methode pro
Konfigurationsoption. Jede Methode hat einen sprechenden Namen (`.modell(...)`,
`.temperatur(...)`) statt einer anonymen Position in einer Parameterliste — die
Vertauschungsgefahr von oben verschwindet, weil jeder Wert an seinem eigenen, benannten
Aufruf hängt. Nicht gesetzte Werte bekommen sinnvolle **Defaults** (z. B. Temperatur 0.7,
wie schon in Phase 1 mit `#[serde(default = "...")]` vorbereitet), Pflichtfelder wie der
API-Key werden erst beim finalen `.bauen()` geprüft.

### Warum `self` (konsumierend) statt `&mut self`?

Es gibt zwei gängige Builder-Stile in Rust:

```rust
// Stil A: konsumierend — jede Methode nimmt und gibt Self zurück
pub fn modell(mut self, modell: impl Into<String>) -> Self {
    self.modell = Some(modell.into());
    self
}

// Stil B: referenzierend — jede Methode arbeitet über &mut self
pub fn modell(&mut self, modell: impl Into<String>) -> &mut Self {
    self.modell = Some(modell.into());
    self
}
```

Wir wählen Stil A. Der Vorteil: Die Aufrufkette `ClientBuilder::neu().modell(...).temperatur(...).bauen()`
liest sich als ein einziger, flüssiger Ausdruck — genau wie im Zielbild oben. Der
Nachteil, den wir bewusst in Kauf nehmen: Ein einmal in eine Kette eingebundener Builder
kann nicht in einer Variablen "zwischengespeichert" und in zwei verschiedene Richtungen
weitergebaut werden (jeder Aufruf verbraucht — *moved* — den vorherigen Wert, ein Konzept,
das [Kapitel 0](../01-grundlagen/04-daten-buendeln.md) und Phase 1 bereits eingeführt
haben). Für unseren Anwendungsfall — einmalig einen Client konfigurieren — ist das kein
Nachteil, den wir spüren.

### Warum `Result` aus `.bauen()`, nicht `Self` direkt?

Genau wie bei der Invariante aus
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) gilt: Ein fehlender
API-Key ist kein Programmierfehler, sondern ein erwartbarer Konfigurationsfehler. Deshalb
gibt `.bauen()` `Result<Client, BuilderFehler>` zurück statt mit `panic!` abzubrechen —
dieselbe Entscheidung, dieselbe Begründung, jetzt an einer neuen Stelle angewendet.

## Schritt-Reveal

**Schritt 1 — `ClientBuilder` als Struct mit `Option`-Feldern anlegen.** Jedes noch nicht
gesetzte Feld ist `None`:

```rust
#[derive(Default)]
pub struct ClientBuilder {
    modell: Option<String>,
    temperatur: Option<f64>,
    api_key: Option<String>,
}

impl ClientBuilder {
    pub fn neu() -> Self {
        ClientBuilder::default()
    }
}
```

`#[derive(Default)]` funktioniert hier, weil `Option<T>` selbst `Default` implementiert
(`None`) — der Compiler leitet daraus automatisch ab, dass auch der ganze Struct einen
sinnvollen Default hat.

**Schritt 2 — Setter-Methoden ergänzen** (siehe Dekonstruktion, Stil A) für `modell` und
`temperatur`.

**Schritt 3 — `api_key_aus_umgebung` ergänzen**, ein Setter, der bewusst nicht den
API-Key direkt als Parameter nimmt, sondern nur den Namen der Umgebungsvariable — so
landet der geheime Wert nie sichtbar im Aufrufcode selbst:

```rust
pub fn api_key_aus_umgebung(mut self, variable: &str) -> Self {
    self.api_key = std::env::var(variable).ok();
    self
}
```

**Schritt 4 — `.bauen()` mit Pflichtfeld-Prüfung.**

```rust
pub fn bauen(self) -> Result<Client, BuilderFehler> {
    let modell = self.modell.ok_or(BuilderFehler::FehltModell)?;
    let api_key = self.api_key.ok_or(BuilderFehler::FehltApiKey)?;
    let temperatur = self.temperatur.unwrap_or(0.7);
    Ok(Client { modell, api_key, temperatur })
}
```

**Schritt 5 — Provoziere den Fehler bewusst.** Rufe `.bauen()` ohne `.modell(...)` auf:

```rust
let ergebnis = ClientBuilder::neu().bauen();
```

`cargo check -p mein_core` kompiliert einwandfrei — der Fehler zeigt sich hier nicht zur
Kompilierzeit, sondern erst zur Laufzeit als `Err(BuilderFehler::FehltModell)`. Das ist
eine bewusste Grenze des Builder Patterns in dieser Form: Es macht Konstruktion lesbar,
aber Pflichtfelder werden erst beim `.bauen()`-Aufruf geprüft, nicht schon vom Compiler
erzwungen (anders als bei `Nachricht::neu`, wo alle Parameter von Anfang an Pflicht
sind). Schreibe einen Test, der genau das dokumentiert:

```rust
#[test]
fn fehlendes_modell_wird_beim_bauen_erkannt() {
    let ergebnis = ClientBuilder::neu().bauen();
    assert_eq!(ergebnis.unwrap_err(), BuilderFehler::FehltModell);
}
```

> **💡 Tipp**
>
> Für Builder mit *wirklich* zur Kompilierzeit erzwungenen Pflichtfeldern gibt es
> fortgeschrittenere Muster (typisierte Builder-Zustände, ein `Builder<HatModell,
> HatApiKey>`-Typparameter, der sich bei jedem Setter ändert). Das ist mächtig, aber für
> unseren Kurs bewusst zu viel Komplexität für den Nutzen — wir bleiben beim einfacheren,
> zur Laufzeit geprüften Builder.

## Ausführung

```bash
cargo test -p mein_core builder
```

```
running 2 tests
test builder::tests::fehlendes_modell_wird_beim_bauen_erkannt ... ok
test builder::tests::vollstaendige_konfiguration_wird_gebaut ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Zusammenfassung

- Das Builder Pattern ersetzt einen langen, positionellen Konstruktor durch benannte,
  verkettbare Setter-Methoden — lesbarer und weniger fehleranfällig bei vielen
  optionalen Werten.
- Konsumierende Builder (`self -> Self`) erlauben flüssige Aufrufketten, verzichten dafür
  auf Wiederverwendbarkeit eines Zwischenzustands.
- Pflichtfelder werden erst beim finalen `.bauen()` geprüft und liefern — wie schon bei
  Invarianten in Phase 1 — ein `Result`, keinen `panic!`.
- `#[derive(Default)]` auf einem Struct aus lauter `Option<T>`-Feldern spart Schreibarbeit
  für den Ausgangszustand des Builders.

## Übung

Baue einen `KonversationBuilder`, der die Transferaufgabe aus
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) (optionale
Systemnachricht) in Builder-Form anbietet, z. B.
`KonversationBuilder::neu().mit_systemnachricht("Du bist hilfreich").bauen()`. Vergleiche
anschließend: Für welchen Anwendungsfall ist die ursprüngliche Methode
`mit_systemnachricht` auf `Konversation` selbst weiterhin die bessere Wahl, und wann lohnt
sich stattdessen ein eigener Builder? Es gibt hier keine einzig richtige Antwort — die
Übung testet dein Gespür für API-Ergonomie.

[Weiter: Lektion 2 — Feature Flags](02-feature-flags.md)
