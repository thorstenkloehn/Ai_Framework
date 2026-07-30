# Rust-Spickzettel

Kompakte Syntax-Referenz zum schnellen Nachschlagen — bewusst ohne lange Erklärungen. Die
Hintergründe stehen in den jeweiligen Lektionen, siehe [Glossar](01-glossar.md) für
Verweise.

## Variablen & Typen

```rust
let x = 5;              // unveränderlich (Standard)
let mut y = 5;           // veränderlich
y = 6;

let z: i32 = 5;           // explizite Typangabe
let name: String = String::from("Hallo");
let text: &str = "Hallo";  // Textreferenz, nicht besessen

const MAX: u32 = 100_000;  // Konstante, Typ zwingend
```

Häufige Grundtypen:

| Typ | Bedeutung |
|---|---|
| `i32`, `i64`, `u32`, `u64`, `usize` | Ganzzahlen (vorzeichenbehaftet `i`, vorzeichenlos `u`) |
| `f32`, `f64` | Gleitkommazahlen |
| `bool` | `true` / `false` |
| `char` | ein einzelnes Unicode-Zeichen |
| `String` | besessener, wachsender Text |
| `&str` | ausgeliehene Textreferenz |
| `()` | "unit", kein Wert |

## Funktionen & Methoden

```rust
fn addieren(a: i32, b: i32) -> i32 {
    a + b   // kein Semikolon = Rückgabewert
}

impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht { rolle, inhalt: inhalt.into() }
    }

    pub fn ist_leer(&self) -> bool {         // &self: lesend
        self.inhalt.is_empty()
    }

    pub fn kuerzen(&mut self, n: usize) {     // &mut self: verändernd
        self.inhalt.truncate(n);
    }
}
```

- Assoziierte Funktion (kein `self`): `Typ::funktion(...)`
- Methode (mit `self`/`&self`/`&mut self`): `wert.methode(...)`

## Kontrollfluss

```rust
if bedingung {
    // ...
} else if andere_bedingung {
    // ...
} else {
    // ...
}

match rolle {
    Rolle::System => println!("System"),
    Rolle::Benutzer => println!("Benutzer"),
    Rolle::Assistent => println!("Assistent"),
    // match muss ALLE Fälle abdecken
}

loop {
    break;              // Endlosschleife, manuell verlassen
}

while bedingung {
    // ...
}

for element in liste {
    // ...
}

for i in 0..5 {          // 0,1,2,3,4 (exklusiv)
    // ...
}
```

## Structs & Enums

```rust
#[derive(Debug, Clone)]
struct Nachricht {
    rolle: Rolle,
    inhalt: String,
}

let n = Nachricht { rolle: Rolle::Benutzer, inhalt: String::from("Hi") };
println!("{}", n.inhalt);

#[derive(Debug, Clone, PartialEq)]
enum Rolle {
    System,
    Benutzer,
    Assistent,
}

#[derive(Debug, Clone, PartialEq)]
enum NachrichtFehler {
    LeererInhalt,
    ZuLang(usize),        // Enum-Variante mit Daten
}
```

## Option & Result

```rust
let vielleicht: Option<i32> = Some(5);
let nichts: Option<i32> = None;

let ergebnis: Result<i32, String> = Ok(5);
let fehler: Result<i32, String> = Err("kaputt".to_string());
```

| Ausdruck | Bedeutung |
|---|---|
| `wert?` | bei `Err`/`None`: sofort aus Funktion zurückgeben; sonst entpacken |
| `wert.unwrap()` | entpacken oder `panic!` — nur in Tests/Prototypen |
| `wert.expect("nachricht")` | wie `unwrap()`, aber mit eigener Panic-Nachricht |
| `wert.unwrap_or(standard)` | entpacken oder Standardwert nehmen |
| `wert.is_ok()` / `wert.is_err()` | Erfolg/Fehler prüfen, ohne zu entpacken |
| `wert.is_some()` / `wert.is_none()` | vorhanden/leer prüfen, ohne zu entpacken |

```rust
if let Some(x) = vielleicht {
    println!("{x}");
}

if let Err(fehler) = ergebnis {
    eprintln!("Fehler: {fehler}");
}

match ergebnis {
    Ok(wert) => println!("{wert}"),
    Err(fehler) => eprintln!("{fehler}"),
}
```

## Collections

```rust
let mut zahlen: Vec<i32> = Vec::new();
zahlen.push(1);
let zahlen = vec![1, 2, 3];       // Kurzform
zahlen.len();
zahlen.iter();                     // Iterator über Referenzen

use std::collections::HashMap;
let mut karte: HashMap<String, i32> = HashMap::new();
karte.insert("eins".to_string(), 1);
karte.get("eins");                 // Option<&i32>
```

> `HashMap` wird im Kurs nur am Rand gestreift — Nachschlagen bei Bedarf in der
> [std-Doku](04-ressourcen.md).

## Traits & Generics

```rust
trait LlmProvider {
    fn anfragen(&self, prompt: &str) -> Result<String, Fehler>;
}

struct OpenAiAdapter;

impl LlmProvider for OpenAiAdapter {
    fn anfragen(&self, prompt: &str) -> Result<String, Fehler> {
        // ...
        Ok(String::new())
    }
}

fn nutze_provider(p: &impl LlmProvider) { /* ... */ }   // impl Trait, statisch
fn nutze_provider2(p: &dyn LlmProvider) { /* ... */ }    // dyn Trait, dynamisch

struct Wrapper<T> {   // Generics
    wert: T,
}
```

| Form | Wann |
|---|---|
| `impl Trait` (Parameter) / Generics `<T: Trait>` | Typ steht zur Kompilierzeit fest, schneller |
| `dyn Trait` (meist in `Box<dyn Trait>`) | mehrere unterschiedliche Typen zur Laufzeit, z. B. austauschbare Adapter |

## Fehlerbehandlung: thiserror & anyhow

```rust
// Bibliothekscode (mein_core): eigener, präziser Fehlertyp
use thiserror::Error;

#[derive(Debug, Error)]
enum ProviderFehler {
    #[error("Netzwerkfehler: {0}")]
    Netzwerk(#[from] reqwest::Error),
    #[error("ungültige Antwort")]
    UngueltigeAntwort,
}

// Anwendungscode (mein_cli): pragmatischer Sammel-Fehlertyp
use anyhow::{Context, Result};

fn lade_konfiguration() -> Result<Konfiguration> {
    let text = std::fs::read_to_string("config.json")
        .context("Konfigurationsdatei konnte nicht gelesen werden")?;
    let konfig = serde_json::from_str(&text)?;
    Ok(konfig)
}
```

## Häufige Cargo-Befehle

| Befehl | Zweck |
|---|---|
| `cargo new <name>` | neues Crate anlegen |
| `cargo build` | kompilieren |
| `cargo run -p <crate>` | bauen und ausführen |
| `cargo check` | nur auf Kompilierbarkeit prüfen (schnell) |
| `cargo test` | Tests ausführen |
| `cargo fmt` | Code automatisch formatieren |
| `cargo clippy --workspace` | Linter, findet typische Rust-Fallstricke |
| `cargo doc --no-deps --open` | Dokumentation generieren und öffnen |
| `cargo add <crate>` | Abhängigkeit hinzufügen |
| `cargo publish` | Crate auf crates.io veröffentlichen |
| `rustc --explain E0308` | ausführliche Erklärung zu einem Fehlercode |

## Häufige Attribute

| Attribut | Zweck |
|---|---|
| `#[derive(Debug)]` | erlaubt `{:?}`-Ausgabe |
| `#[derive(Clone)]` | erlaubt `.clone()` |
| `#[derive(PartialEq)]` | erlaubt `==`-Vergleich |
| `#[derive(Default)]` | erlaubt `Typ::default()` |
| `#[derive(Serialize, Deserialize)]` | serde: Rust ↔ JSON/TOML |
| `#[derive(Parser)]` / `#[derive(Subcommand)]` | clap: CLI aus Typdefinition |
| `#[derive(Error)]` | thiserror: eigener Fehlertyp |
| `#[serde(default)]` | fehlendes Feld → `Default::default()` |
| `#[serde(default = "fn_name")]` | fehlendes Feld → eigener Standardwert |
| `#[cfg(test)]` | Modul/Funktion nur beim Testen kompilieren |
| `#[test]` | Funktion als Testfall markieren |
| `#[tokio::main]` | `async fn main()` lauffähig machen |
| `#[arg(long)]` | clap: Feld wird zu `--flag` |
| `#[command(subcommand)]` | clap: Feld enthält Subcommand-Enum |
