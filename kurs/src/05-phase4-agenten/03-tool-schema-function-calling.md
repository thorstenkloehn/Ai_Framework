# Lektion 3: Tool-Schema und Function Calling

## Problem

Ein Sprachmodell kennt nur Text. Damit ein Agent einen Taschenrechner, eine Websuche
oder eine Datenbankabfrage benutzen kann, muss das Modell drei Dinge erfahren, **bevor**
es antwortet: Welche Werkzeuge (**Tools**) gibt es überhaupt, was tun sie (in Worten),
und mit welchen Parametern werden sie aufgerufen (in einer Struktur, die sich
maschinell auslesen lässt)? Diese Fähigkeit, einem Modell strukturierte Werkzeuge
anzubieten und seine Antwort als "ich möchte Werkzeug X mit diesen Argumenten aufrufen"
statt als freien Text zu lesen, heißt **Function Calling** oder **Tool Use**.

## Code (Zielbild)

```rust
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaschenrechnerArgumente {
    pub a: f64,
    pub b: f64,
    pub operation: String,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn beschreibung(&self) -> &str;
    fn parameter_schema(&self) -> serde_json::Value;
    async fn ausfuehren(&self, argumente: serde_json::Value) -> Result<String, ToolFehler>;
}
```

## Dekonstruktion

### `trait Tool` — jedes Werkzeug ist austauschbar

Der `Tool`-Trait beschreibt, was **jedes** Werkzeug können muss, unabhängig davon, ob es
ein Taschenrechner, eine Websuche oder (optional, [Lektion 7](07-mcp-client.md)) ein
externer MCP-Server ist. Das ist dasselbe Prinzip wie `LlmProvider` aus
[Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md): ein Trait als
**Port**, mehrere austauschbare Implementierungen dahinter. Der Agent Loop
([Lektion 4](04-agent-loop.md)) hält eine Liste `Vec<Box<dyn Tool>>` — er kennt die
konkreten Werkzeuge nicht, nur den Vertrag.

### Warum `#[async_trait]`, obwohl `async fn` in Traits inzwischen erlaubt ist?

Seit Rust 1.75 darfst du `async fn` direkt in einem Trait schreiben, ganz ohne
Zusatz-Crate — **solange** du den Trait nicht als Trait-Objekt (`dyn Tool`) benutzt.
Genau das tun wir aber: `Vec<Box<dyn Tool>>` braucht ein **Trait-Objekt**, weil in einer
Liste unterschiedliche konkrete Typen (Taschenrechner, Websuche, ...) nebeneinander
stehen sollen ([Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md)
vertieft `dyn Trait`). Das Problem: Jede `async fn` erzeugt intern einen eigenen,
unterschiedlich großen Future-Typ — der Compiler müsste bei einem Trait-Objekt aber
vorab wissen, wie groß der Rückgabewert von `ausfuehren(...)` ist, und "je nach
konkretem Typ unterschiedlich groß" ist bei `dyn Trait` grundsätzlich nicht erlaubt. Das
Crate `async_trait` löst das mit einem Makro: Es verpackt den Rückgabewert automatisch in
ein `Pin<Box<dyn Future<Output = ...> + Send>>` — eine Box hat immer dieselbe Größe (einen
Zeiger), egal was tatsächlich drinsteht. Der Preis ist eine zusätzliche Speicher-Allokation
pro Aufruf; für Werkzeugaufrufe, die ohnehin Netzwerk- oder Festplattenzugriffe machen,
fällt das nicht ins Gewicht.

### `Send + Sync` als Supertraits

```rust
pub trait Tool: Send + Sync { ... }
```

`Tool: Send + Sync` heißt: Jeder Typ, der `Tool` implementiert, muss zusätzlich `Send`
(darf sicher an einen anderen Thread übergeben werden) und `Sync` (darf sicher von
mehreren Threads gleichzeitig *gelesen* werden) sein. Tokios Runtime verteilt Tasks
möglicherweise auf mehrere Worker-Threads ([Lektion 1](01-async-und-tokio.md)) — ein
`Box<dyn Tool>`, der in einem Agent Loop über `.await`-Punkte hinweg lebt, muss deshalb
zwischen Threads wandern dürfen. Wir vertiefen `Send` in
[Lektion 5](05-state-und-memory.md) an einem Fall, in dem genau das **nicht**
automatisch klappt.

### `parameter_schema()` und `schemars` — das Vokabular kennst du schon

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaschenrechnerArgumente {
    pub a: f64,
    pub b: f64,
    pub operation: String,
}
```

`#[derive(JsonSchema)]` kennst du aus
[Phase 2, Lektion 6](../03-phase2-llm-anbindung/06-structured-output.md) — dort hast du
es genutzt, um vom Modell strukturierte **Antworten** zu erzwingen. Hier verwenden wir
dieselbe Idee umgekehrt: Wir generieren aus einem Rust-`struct` ein JSON-Schema, das
**beschreibt, wie ein Aufruf aussehen muss** (welche Felder, welche Typen, welche sind
Pflicht). Dieses Schema schicken wir als Teil unserer Anfrage mit, und das Modell soll
seine Tool-Aufrufe daran ausrichten:

```rust
fn taschenrechner_schema() -> serde_json::Value {
    let schema = schemars::schema_for!(TaschenrechnerArgumente);
    serde_json::to_value(schema).expect("Schema ist immer gültiges JSON")
}
```

`schema_for!` ist ein Makro, das zur Kompilierzeit ein `schemars::schema::RootSchema`
erzeugt — dieselbe Typinformation, die `#[derive(JsonSchema)]` bereitstellt, nur diesmal
für die Eingabe statt die Ausgabe.

> **⚠️ Warnung**
>
> `.expect(...)` benutzen wir hier bewusst, nicht `?`: Die Umwandlung eines von
> `schemars` erzeugten `RootSchema` in `serde_json::Value` kann bei uns praktisch nicht
> fehlschlagen — es gibt keinen Nutzereingabe-Pfad, der hier eine `Err`-Variante
> auslösen könnte. Das ist ein bewusster Unterschied zu Fehlern, die aus echten
> Nutzereingaben oder Netzwerkantworten stammen (siehe die Warnung dazu in
> [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)) — dort bleibt `?`
> oder explizites Fehlerhandling Pflicht.

### Wie das Modell einen Aufruf "ankündigt"

Echte LLM-APIs wie die von OpenAI bieten dafür oft ein dediziertes Antwortfeld
(`tool_calls`), getrennt vom eigentlichen Text. Ob dein `LlmProvider`-Adapter aus
[Phase 2](../03-phase2-llm-anbindung/README.md)/[Phase 3](../04-phase3-architektur/README.md)
dieses Feld schon typisiert zurückgibt, hängt vom jeweiligen Anbieter ab. Wir bauen hier
einen Weg, der mit **jedem** Modell funktioniert, auch ohne natives Function-Calling: Wir
vereinbaren per System-Prompt ein festes Format, und parsen die Modellantwort dagegen.

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Werkzeugaufruf {
    pub werkzeug: String,
    #[serde(default)]
    pub argumente: serde_json::Value,
}

/// Versucht, eine Modellantwort als Werkzeugaufruf zu lesen.
/// `None` bedeutet: normaler Text, kein Aufruf.
pub fn als_werkzeugaufruf(text: &str) -> Option<Werkzeugaufruf> {
    serde_json::from_str(text).ok()
}
```

Antwortet das Modell mit `{"werkzeug": "taschenrechner", "argumente": {"a": 2, "b": 3,
"operation": "plus"}}`, liefert `als_werkzeugaufruf` ein `Some(...)`. Antwortet es mit
normalem Fließtext ("Die Hauptstadt von Frankreich ist Paris."), scheitert das Parsen als
JSON, `serde_json::from_str(...).ok()` wird zu `None`, und wir wissen: Das war die
finale Antwort, kein Aufruf. Genau diese Fallunterscheidung ist der Kern des
[Agent Loop](04-agent-loop.md) in der nächsten Lektion.

> **💡 Tipp**
>
> Wenn dein `LlmProvider`-Adapter das native `tool_calls`-Feld einer API bereits
> typisiert liefert, kannst du `als_werkzeugaufruf` überspringen und direkt dieses Feld
> auswerten — das Prinzip (strukturiertes Signal statt Freitext-Rätselraten) bleibt
> identisch. Wir zeigen hier bewusst den anbieter-unabhängigen Weg, der ohne
> Sonderfeature auskommt.

## Schritt-Reveal

**Schritt 0 — Modulstruktur anlegen.** `mein_agent` bekommt ab jetzt die Struktur, die
wir für den Rest der Phase beibehalten: ein Modul `agent` mit je einer Datei pro
Verantwortlichkeit. Lege an:

```
mein_agent/src/
├── lib.rs
└── agent/
    ├── mod.rs
    ├── tool.rs
    ├── state.rs      (ab Lektion 5)
    └── loop.rs        (ab Lektion 4)
```

`mein_agent/src/lib.rs`:

```rust
pub mod agent;
```

`mein_agent/src/agent/mod.rs`:

```rust
pub mod tool;

pub use tool::{Tool, ToolFehler, Werkzeugaufruf};
```

Der ganze Code dieser Lektion (der `Tool`-Trait, `TaschenrechnerArgumente`,
`Werkzeugaufruf`, `als_werkzeugaufruf`, `ToolFehler`, `Taschenrechner`) wandert in
`mein_agent/src/agent/tool.rs`. `pub use` in `mod.rs` reicht die wichtigsten Namen nach
außen weiter — Aufrufer schreiben später `mein_agent::agent::Tool` statt des längeren
`mein_agent::agent::tool::Tool`.

**Schritt 1 — Abhängigkeiten ergänzen.** In `mein_agent/Cargo.toml`:

```toml
[dependencies]
mein_core = { path = "../mein_core" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
futures-util = "0.3"
async-trait = "0.1"
schemars = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2"
```

`thiserror` kennst du bereits aus
[Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md) — wir nutzen es
auch hier für `ToolFehler`, statt wie in
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) einen Fehlertyp von Hand
zu schreiben. `mein_agent` ist Produktionscode, kein erster Lernschritt mehr — die
Abkürzung, die uns `thiserror` gibt (`#[error("...")]` erzeugt `Display` automatisch),
lohnt sich hier von Anfang an.

**Schritt 2 — Provoziere den Fehler bewusst.** Lass `JsonSchema` beim Argumente-Struct
versehentlich weg:

```rust
#[derive(Debug, serde::Deserialize)]
pub struct TaschenrechnerArgumente {
    pub a: f64,
    pub b: f64,
    pub operation: String,
}

fn schema() -> serde_json::Value {
    let s = schemars::schema_for!(TaschenrechnerArgumente);
    serde_json::to_value(s).unwrap()
}
```

`cargo check -p mein_agent` meldet sinngemäß:

```
error[E0277]: the trait bound `TaschenrechnerArgumente: JsonSchema` is not satisfied
  --> mein_agent/src/lib.rs:9:35
   |
 9 |     let s = schemars::schema_for!(TaschenrechnerArgumente);
   |                                   ^^^^^^^^^^^^^^^^^^^^^^^ the trait `JsonSchema` is not implemented for `TaschenrechnerArgumente`
   |
   = help: the following other types implement trait `JsonSchema`: ...
```

`schema_for!` ist nur ein Makro — es verlangt hinter den Kulissen, dass der übergebene
Typ `JsonSchema` implementiert, genau wie `serde_json::from_str::<T>(...)` verlangt, dass
`T: Deserialize` implementiert. Ohne `#[derive(JsonSchema)]` fehlt dieser Baustein, und
der Compiler benennt exakt, welches Trait fehlt.

**Schritt 3 — Korrektur.** `#[derive(JsonSchema)]` zur `derive`-Liste ergänzen (siehe
Zielbild oben). `cargo check -p mein_agent` — sauber.

**Schritt 4 — `ToolFehler` und ein erstes Werkzeug.**

```rust
#[derive(Debug, thiserror::Error)]
pub enum ToolFehler {
    #[error("ungültige Argumente: {0}")]
    UngueltigeArgumente(String),
    #[error("Ausführung fehlgeschlagen: {0}")]
    Ausfuehrung(String),
}

pub struct Taschenrechner;

#[async_trait]
impl Tool for Taschenrechner {
    fn name(&self) -> &str {
        "taschenrechner"
    }

    fn beschreibung(&self) -> &str {
        "Addiert oder subtrahiert zwei Zahlen a und b. operation ist \"plus\" oder \"minus\"."
    }

    fn parameter_schema(&self) -> serde_json::Value {
        let schema = schemars::schema_for!(TaschenrechnerArgumente);
        serde_json::to_value(schema).expect("Schema ist immer gültiges JSON")
    }

    async fn ausfuehren(&self, argumente: serde_json::Value) -> Result<String, ToolFehler> {
        let args: TaschenrechnerArgumente = serde_json::from_value(argumente)
            .map_err(|e| ToolFehler::UngueltigeArgumente(e.to_string()))?;

        let ergebnis = match args.operation.as_str() {
            "plus" => args.a + args.b,
            "minus" => args.a - args.b,
            sonst => return Err(ToolFehler::Ausfuehrung(format!("unbekannte Operation: {sonst}"))),
        };

        Ok(ergebnis.to_string())
    }
}
```

Beachte: `ausfuehren` ist `async`, obwohl der Taschenrechner selbst gar nicht wartet — das
ist in Ordnung, der Trait verlangt es einheitlich für **alle** Werkzeuge, weil andere
(eine Websuche, ein MCP-Werkzeug in [Lektion 7](07-mcp-client.md)) echte I/O machen.

## Ausführung

```bash
cargo test -p mein_agent
```

Ergänze dafür einen Test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn taschenrechner_addiert() {
        let werkzeug = Taschenrechner;
        let argumente = serde_json::json!({ "a": 2.0, "b": 3.0, "operation": "plus" });
        let ergebnis = werkzeug.ausfuehren(argumente).await.unwrap();
        assert_eq!(ergebnis, "5");
    }

    #[test]
    fn text_ist_kein_werkzeugaufruf() {
        assert!(als_werkzeugaufruf("Die Hauptstadt von Frankreich ist Paris.").is_none());
    }
}
```

`#[tokio::test]` ist das Async-Gegenstück zu `#[test]` — es startet für die Dauer des
Tests eine eigene, kleine Tokio-Runtime, damit `.await` innerhalb des Tests erlaubt ist
(dieselbe Regel wie in [Lektion 1](01-async-und-tokio.md), `E0728`).

```
running 2 tests
test tests::text_ist_kein_werkzeugaufruf ... ok
test tests::taschenrechner_addiert ... ok
```

## Zusammenfassung

- `trait Tool` ist ein Port wie `LlmProvider`: austauschbare Werkzeuge hinter einem
  gemeinsamen Vertrag, verwaltet als `Vec<Box<dyn Tool>>`.
- `#[async_trait]` macht `async fn` in Traits **objektsicher** (dyn-tauglich) — nötig,
  weil native `async fn`-in-Traits das für Trait-Objekte nicht unterstützen.
- `Send + Sync` als Supertraits garantieren, dass Werkzeuge sicher über Tokio-Tasks und
  -Threads wandern können.
- `#[derive(JsonSchema)]` (aus Phase 2 bekannt) generiert das Schema, das wir dem Modell
  als Beschreibung eines Werkzeugs mitgeben — dieselbe Idee wie Structured Output, nur
  für Eingaben statt Ausgaben.
- Ein vereinbartes JSON-Format (`{"werkzeug": ..., "argumente": ...}`) macht
  Function Calling anbieter-unabhängig; native `tool_calls`-Felder sind eine
  Optimierung desselben Prinzips.

## Übung

Baue ein zweites Werkzeug, `UhrzeitWerkzeug`, das **keine** Argumente braucht (leerer
`TaschenrechnerArgumente`-artiger Struct, oder direkt `serde_json::Value::Null` als
Eingabetyp) und als Ergebnis die aktuelle Systemzeit als Text zurückgibt (`std::time`
oder eine Platzhalter-Zeichenkette, falls du Zeitfunktionen noch nicht kennst). Schreibe
einen Test, der prüft, dass `parameter_schema()` für dieses Werkzeug ein gültiges,
nicht-leeres JSON-Objekt liefert. Überlege: Wie müsste `als_werkzeugaufruf` erweitert
werden, wenn ein Modell versehentlich `{"werkzeug": "uhrzeit"}` **ohne** das Feld
`argumente` schickt — schau dir dazu noch einmal `#[serde(default)]` aus
[Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md) an.

[Weiter: Lektion 4 — Der Agent Loop](04-agent-loop.md)
