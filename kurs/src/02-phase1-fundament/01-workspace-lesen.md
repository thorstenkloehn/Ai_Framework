# Lektion 1: Den Workspace lesen

## Problem

Bevor wir eine einzige Zeile neuen Code schreiben, müssen wir verstehen, was schon da ist.
Ein fremdes (oder das eigene, aber schon Wochen alte) Projekt zu öffnen und sich in fünf
Minuten zu orientieren, ist eine Fähigkeit für sich — und wichtiger als die meisten
Anfänger*innen denken. Konkrete Frage dieser Lektion: **Welche Bausteine hat
Ai_Framework bereits, wer ist wofür zuständig, und wo genau setzt unsere nächste Lektion
an?**

## Code (Zielbild)

Am Ende dieser Lektion kannst du diese Fragen aus dem Stand beantworten:

```bash
cargo metadata --no-deps --format-version 1 | jq '.packages[].name'
# "mein_core"
# "mein_cli"
```

— und weißt, warum es nur diese zwei Pakete sind, obwohl die Roadmap von `core`, `cli`
und später `server` spricht.

## Dekonstruktion

Öffne den geklonten Ordner `Ai_Framework` in VS Code
([Werkzeuge einrichten](../00-einleitung/03-werkzeuge-einrichten.md)) und sieh dir die
Struktur an:

```
Ai_Framework/
├── Cargo.toml          ← Workspace-Definition
├── mein_core/
│   ├── Cargo.toml
│   └── src/lib.rs
├── mein_cli/
│   ├── Cargo.toml
│   └── src/main.rs
├── AGENTS.md
├── roadmap.md
└── README.md
```

**Ein Cargo-Workspace** ist mehrere Crates (Pakete), die eine gemeinsame `Cargo.toml` im
Wurzelverzeichnis teilen und dieselbe `target/`-Ausgabe und `Cargo.lock` (feste
Versionen aller Abhängigkeiten) nutzen. Statt eines riesigen Pakets mit allem drin,
trennen wir Verantwortlichkeiten in eigene Crates. Das ist eine Architekturentscheidung,
die wir ab Tag 1 treffen, nicht erst, wenn das Projekt "zu groß" geworden ist:

- **`mein_core`** — eine **Bibliothek** (*library*, erkennbar an `src/lib.rs`). Enthält die
  Domänenlogik: Was ist eine Nachricht, was ist eine Rolle, später: was ist ein Provider,
  ein Agent. `mein_core` weiß nichts von Kommandozeile, Terminal-Ausgabe oder
  Nutzerinteraktion.
- **`mein_cli`** — ein **Binary** (ausführbares Programm, erkennbar an `src/main.rs`).
  Nutzt `mein_core`, kümmert sich um alles, was mit "ein Mensch tippt etwas ins
  Terminal" zu tun hat.

Diese Trennung — Domänenlogik hier, Anwendung/Interface dort — ist der erste, kleinste
Vorgeschmack auf die **Hexagonal Architecture**, die wir in
[Phase 3](../04-phase3-architektur/02-hexagonal-architecture.md) explizit benennen und
vertiefen. Für jetzt reicht die Faustregel: *"Würde ich dieselbe Logik auch in einer
zukünftigen Web-API (`mein_server`, Phase 5) brauchen? Dann gehört sie in `mein_core`,
nicht in `mein_cli`."*

### Der Workspace-Root: `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "mein_core",
    "mein_cli",
]
```

- `members` listet die Crates des Workspace auf.
- `resolver = "2"` legt fest, nach welchem Algorithmus Cargo Abhängigkeitsversionen
  auflöst, wenn verschiedene Crates unterschiedliche Anforderungen an dieselbe
  Abhängigkeit haben. Resolver 2 (seit Rust 2021 Standard bei neuen Projekten) trifft
  dabei sauberer zwischen normalen Abhängigkeiten und Test-/Dev-Abhängigkeiten. Für uns
  als Anfänger*innen reicht: Immer `resolver = "2"` verwenden, es ist der moderne Standard.

### `mein_core/Cargo.toml`

```toml
[package]
name = "mein_core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

`edition = "2024"` legt fest, welche Sprachversion von Rust dieses Crate nutzt — Editionen
sind Rusts Mechanismus, Sprachänderungen einzuführen, ohne alten Code zu brechen (jedes
Crate wählt seine Edition unabhängig). `serde` ist bereits als Abhängigkeit eingetragen,
obwohl wir es erst in [Lektion 5](05-serde-konfiguration.md) benutzen — ein Vorgriff, den
wir dort einlösen.

### `mein_cli/Cargo.toml`

```toml
[package]
name = "mein_cli"
version = "0.1.0"
edition = "2024"

[dependencies]
mein_core = { path = "../mein_core" }
```

Die Zeile `mein_core = { path = "../mein_core" }` ist eine **Pfad-Abhängigkeit**:
`mein_cli` hängt von `mein_core` ab, aber nicht über crates.io (das öffentliche
Rust-Package-Registry, Thema in [Phase 7](../08-phase7-release/05-crates-io-checkliste.md)),
sondern über den lokalen Ordner. So können wir beide Crates gleichzeitig entwickeln, ohne
`mein_core` erst veröffentlichen zu müssen.

> **💡 Tipp**
>
> `AGENTS.md` im Repo-Root nennt schon jetzt alle Befehle, die wir laufend brauchen
> werden: `cargo build`, `cargo run -p mein_cli`, `cargo test`, `cargo check`, `cargo
> clippy --workspace`, `cargo fmt`. Das `-p mein_cli` bei `cargo run` heißt "package
> mein_cli" — nötig, weil der Workspace mehr als ein ausführbares Crate haben könnte.

> **⚠️ Warnung**
>
> `mein_core` enthält aktuell einen verschachtelten `.git`-Ordner (kein richtiges Git-
> Submodul, sondern ein Überbleibsel). Das führt dazu, dass `git status` im Repo-Root
> `mein_core` als unversioniertes "Modul" anzeigt statt als normale Dateien. Bevor du
> etwas committest: `rm -rf mein_core/.git` einmalig ausführen, dann verhält sich Git wie
> erwartet.

## Schritt-Reveal

Kein neuer Code in dieser Lektion — stattdessen drei Kommandos, die du selbst im Terminal
im Repo-Root ausführst, um dir den Ist-Zustand zu erschließen:

**Schritt 1 — Welche Pakete gibt es?**

```bash
cargo metadata --no-deps --format-version 1 | grep '"name"'
```

**Schritt 2 — Was steht wirklich in `mein_core/src/lib.rs`?**

Öffne die Datei in VS Code. Du solltest sehen:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Rolle {
    System,
    Benutzer,
    Assistent,
}
#[derive(Debug, Clone)]
pub struct Nachricht {
    pub rolle: Rolle,
    pub inhalt: String,
}
impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht {
            rolle,
            inhalt: inhalt.into(),
        }
    }
}
```

Das ist **kein** leeres Standardtemplate mehr — jemand hat hier schon `Rolle` und
`Nachricht` angelegt. Genau diesen Code entschlüsseln wir komplett in
[Lektion 2](02-rolle-und-nachricht.md).

**Schritt 3 — Was macht `mein_cli` gerade damit?**

Öffne `mein_cli/src/main.rs`:

```rust
use::mein_core::{Nachricht,Rolle};
fn main() {
    let nachricht = Nachricht::neu(Rolle::Benutzer,"Hallo wie gehts");
    println!("{:?}",nachricht);
}
```

Beachte die fehlende Formatierung (Leerzeichen nach Kommas fehlen, `use::` statt `use
::` bzw. üblicher `use mein_core::...`) — das korrigieren wir gleich mit `cargo fmt`.

## Ausführung

```bash
cargo build
```

Erwartete Ausgabe (sinngemäß):

```
   Compiling mein_core v0.1.0 (.../mein_core)
   Compiling mein_cli v0.1.0 (.../mein_cli)
    Finished dev [unoptimized + debuginfo] target(s) in 0.8s
```

```bash
cargo run -p mein_cli
```

Erwartete Ausgabe:

```
Nachricht { rolle: Benutzer, inhalt: "Hallo wie gehts" }
```

Das `{:?}` in `println!("{:?}", nachricht)` nutzt das `Debug`-Trait (das `#[derive(Debug,
...)]` über `Nachricht` und `Rolle`), um eine für Entwickler*innen lesbare Darstellung zu
erzeugen — mehr dazu in [Lektion 2](02-rolle-und-nachricht.md).

Räume jetzt schon die Formatierung auf:

```bash
cargo fmt
```

Sieh dir per `git diff` an, was sich geändert hat — vor allem in `mein_cli/src/main.rs`.

## Zusammenfassung

- Ein Cargo-Workspace bündelt mehrere Crates unter einer gemeinsamen `Cargo.toml`.
- `mein_core` (Bibliothek, Domänenlogik) und `mein_cli` (Binary, Kommandozeilen-Interface)
  sind bewusst getrennt — die erste, kleine Vorstufe zur Hexagonal Architecture.
- Der reale Ist-Zustand ist weiter als "leeres Template": `Rolle` und `Nachricht`
  existieren bereits und sind unser Ausgangspunkt für Lektion 2.
- `cargo fmt` bringt Code auf einheitliche Formatierung — wir nutzen es ab jetzt nach
  jedem Schritt.

## Übung

Führe `cargo doc --no-deps --open` im Repo-Root aus. Cargo generiert daraus eine
HTML-Dokumentation aus deinem Code (aktuell noch ohne Doc-Kommentare, aber die Struktur ist
schon sichtbar). Finde darin die Seite für `mein_core::Nachricht` und notiere dir: Welche
Felder und Methoden zeigt sie an, und was fehlt (noch) an Erklärung? Das bereitet
[Phase 7, Lektion 3](../08-phase7-release/03-rustdoc-beispiele.md) vor, wo wir genau diese
Dokumentation vervollständigen.

[Weiter: Lektion 2 — Rolle und Nachricht als Domain Types](02-rolle-und-nachricht.md)
