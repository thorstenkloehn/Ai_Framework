# Lektion 5: Konfiguration mit serde

## Problem

Ab [Phase 2](../03-phase2-llm-anbindung/README.md) sprechen wir mit einem echten
LLM-Anbieter — das braucht einen API-Key, einen Modellnamen, vielleicht eine
Basis-URL. Diese Werte gehören **niemals** hartkodiert in den Quellcode (schon gar nicht
ein API-Key, der damit versehentlich in Git landen könnte). Wir brauchen eine
`Konfiguration`, die aus einer Datei gelesen wird — und einen Mechanismus, der Textdaten
(z. B. JSON oder TOML) automatisch in einen typisierten Rust-Wert verwandelt.

## Code (Zielbild)

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Konfiguration {
    pub modell: String,
    #[serde(default = "standard_temperatur")]
    pub temperatur: f64,
}

fn standard_temperatur() -> f64 {
    0.7
}
```

```rust
let json = r#"{ "modell": "irgendein-modell" }"#;
let konfiguration: Konfiguration = serde_json::from_str(json)?;
// konfiguration.temperatur == 0.7 (Standardwert, da im JSON nicht angegeben)
```

## Dekonstruktion

### Was ist `serde`?

**serde** (*serialize/deserialize*) ist Rusts Standardbibliothek für die Umwandlung
zwischen Rust-Werten und Datenformaten wie JSON, TOML oder YAML. Wichtig zu verstehen:
`serde` selbst kennt kein konkretes Format — es definiert nur die Traits `Serialize`
(Rust-Wert → Format) und `Deserialize` (Format → Rust-Wert). Das konkrete Format kommt aus
einem separaten Crate, z. B. `serde_json` für JSON oder `toml` für TOML-Dateien. Diese
Trennung ist Absicht: Derselbe `#[derive(Deserialize)]`-Typ funktioniert für JSON **und**
TOML **und** YAML, ohne dass du pro Format eigenen Code schreibst.

Erinnerst du dich an `mein_core/Cargo.toml` aus [Lektion 1](01-workspace-lesen.md)? Dort
stand schon `serde = { version = "1.0", features = ["derive"] }` — das `derive`-Feature
ist genau das, was uns `#[derive(Deserialize)]` erlaubt.

### `#[derive(Deserialize)]` — automatisch aus Text lesen

Genau wie `Debug`, `Clone`, `PartialEq` in [Lektion 2](02-rolle-und-nachricht.md) ist
`Deserialize` ein ableitbares Trait. Der Compiler generiert Code, der weiß, wie man aus
einer JSON-Struktur wie `{"modell": "...", "temperatur": 0.5}` einen `Konfiguration`-Wert
baut — Feldname im JSON muss (standardmäßig) zum Feldnamen in Rust passen.

### `#[serde(default = "standard_temperatur")]` — fehlende Felder abfedern

Nicht jede Konfigurationsdatei muss jedes Feld angeben. `#[serde(default = "...")]` sagt:
"Fehlt `temperatur` im JSON, rufe die Funktion `standard_temperatur()` auf und nimm deren
Rückgabewert." Ohne dieses Attribut würde `serde_json::from_str` bei fehlendem Feld einen
`Err` zurückgeben ("missing field `temperatur`") — manchmal gewünscht (Pflichtfeld wie
`modell`), manchmal nicht (optionales Feld mit sinnvollem Standard wie `temperatur`). Wir
entscheiden das bewusst pro Feld.

> **💡 Tipp**
>
> Für einfache Standardwerte reicht auch `#[serde(default)]` ohne Funktionsangabe — dann
> wird `Default::default()` genutzt (`0.0` für `f64`). Wir nutzen hier bewusst eine
> eigene Funktion, weil `0.0` als Temperatur-Standard fachlich falsch wäre (ein LLM mit
> Temperatur 0 antwortet praktisch deterministisch, das ist selten der gewünschte
> Standard).

### `Rolle` und `Nachricht` serialisierbar machen

Damit wir Konversationen später speichern oder als JSON an eine API schicken können
(Phase 2), erweitern wir auch unsere Phase-1-Typen:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Rolle {
    System,
    Benutzer,
    Assistent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nachricht {
    pub rolle: Rolle,
    pub inhalt: String,
}
```

`Serialize` (Rust → JSON) ergänzt `Deserialize` (JSON → Rust) — meistens leitet man beide
zusammen ab, wenn ein Typ in beide Richtungen durch eine Grenze wandern soll.

## Schritt-Reveal

**Schritt 1** — Abhängigkeit ergänzen. In `mein_core/Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Schritt 2** — `Rolle`/`Nachricht` um `Serialize`/`Deserialize` ergänzen (siehe oben),
Import ergänzen: `use serde::{Deserialize, Serialize};` am Dateianfang von
`mein_core/src/lib.rs`.

`cargo check -p mein_core` — sollte weiterhin sauber durchlaufen.

**Schritt 3** — `Konfiguration` als neuen Typ hinzufügen (siehe Zielbild oben).

**Schritt 4** — Provoziere einen Deserialisierungsfehler bewusst. Schreibe einen Test:

```rust
#[test]
fn fehlendes_pflichtfeld_wird_abgelehnt() {
    let json = r#"{ "temperatur": 0.9 }"#; // "modell" fehlt!
    let ergebnis: Result<Konfiguration, _> = serde_json::from_str(json);
    assert!(ergebnis.is_err());
}

#[test]
fn fehlende_temperatur_bekommt_standardwert() {
    let json = r#"{ "modell": "irgendein-modell" }"#;
    let konfiguration: Konfiguration = serde_json::from_str(json).unwrap();
    assert_eq!(konfiguration.temperatur, 0.7);
}
```

`cargo test -p mein_core` — beide Tests sollten grün sein. Lies dir bei Bedarf einmal die
`Err`-Meldung von `serde_json` aus, indem du testweise `.unwrap()` auf den ersten,
fehlerhaften Fall anwendest — sie benennt exakt das fehlende Feld.

## Ausführung

```bash
cargo test -p mein_core
```

```
running 5 tests
test tests::gueltiger_inhalt_wird_akzeptiert ... ok
test tests::leerer_inhalt_wird_abgelehnt ... ok
test tests::konversation_sammelt_verlauf_in_reihenfolge ... ok
test tests::fehlendes_pflichtfeld_wird_abgelehnt ... ok
test tests::fehlende_temperatur_bekommt_standardwert ... ok
```

## Zusammenfassung

- `serde` definiert die Traits `Serialize`/`Deserialize`, ein Format-Crate (`serde_json`,
  `toml`, ...) definiert das konkrete Textformat.
- `#[derive(Deserialize)]` erzeugt automatisch Code, der Text in einen typisierten
  Rust-Wert umwandelt — inklusive Fehlermeldung bei fehlenden Pflichtfeldern.
- `#[serde(default = "...")]` erlaubt fehlertolerante, aber bewusst gewählte
  Standardwerte pro Feld.
- Domain-Typen wie `Rolle`/`Nachricht` werden schon jetzt serialisierbar gemacht, weil
  Phase 2 sie direkt für API-Requests braucht.

## Übung

Lies die `Konfiguration` statt aus einem Literal-String aus einer echten Datei
`config.json` im Projektverzeichnis (`std::fs::read_to_string("config.json")`, dann
`serde_json::from_str`). Verkette den Dateilesefehler und den Deserialisierungsfehler
sinnvoll — beide sind unterschiedliche Fehlertypen (`std::io::Error` vs.
`serde_json::Error`). Für jetzt genügt ein `match` mit zwei Zweigen und einer sprechenden
`eprintln!`-Meldung pro Fall; in
[Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md) lernst du mit
`thiserror`/`anyhow` einen saubereren Weg, unterschiedliche Fehlertypen zu vereinheitlichen.

[Weiter: Lektion 6 — CLI mit clap](06-cli-mit-clap.md)
