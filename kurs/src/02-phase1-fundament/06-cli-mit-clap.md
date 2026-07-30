# Lektion 6: CLI mit clap

## Problem

`mein_cli` ruft bislang eine fest eincodierte Nachricht auf. Ein echtes
Kommandozeilenprogramm braucht **Subcommands** (z. B. `mein_cli chat`, `mein_cli
verlauf`) und **Flags** (z. B. `--system "Du bist hilfreich."`). Wir könnten
`std::env::args()` von Hand parsen — mühsam, fehleranfällig, und ohne automatische
`--help`-Ausgabe. Wir nutzen stattdessen **clap**, das De-facto-Standardcrate für
Kommandozeilen-Parsing in Rust.

## Code (Zielbild)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mein_cli", about = "CLI für das Ai_Framework")]
struct Cli {
    #[command(subcommand)]
    befehl: Befehl,
}

#[derive(Subcommand)]
enum Befehl {
    /// Startet eine neue Konversation mit einer Nutzernachricht
    Chat {
        nachricht: String,
        #[arg(long)]
        system: Option<String>,
    },
}
```

```bash
mein_cli chat "Hallo!" --system "Du bist hilfreich."
mein_cli chat --help
```

## Dekonstruktion

### `#[derive(Parser)]` — ein Struct wird zur CLI-Definition

Genauso wie `#[derive(Deserialize)]` in [Lektion 5](05-serde-konfiguration.md) aus einem
`struct` Code zum Lesen von JSON generiert, generiert `#[derive(Parser)]` (aus `clap`)
Code zum Parsen von Kommandozeilenargumenten. Die **Struktur des Typs beschreibt die
Struktur der CLI** — ein Muster, das dir jetzt schon vertraut vorkommen sollte.

### `#[derive(Subcommand)]` — ein `enum` wird zu Unterbefehlen

```rust
#[derive(Subcommand)]
enum Befehl {
    Chat { nachricht: String, #[arg(long)] system: Option<String> },
}
```

Wieder `enum` als "genau eine von mehreren Möglichkeiten" — hier: welcher Unterbefehl
wurde aufgerufen. `Option<String>` für `system` bedeutet: Das Flag `--system` ist
**optional**. Wird es nicht angegeben, ist `system` zur Laufzeit `None`
([Option kennst du aus Kapitel 0](../01-grundlagen/04-daten-buendeln.md)) — kein
Sonderwert wie ein leerer String nötig, um "nicht angegeben" auszudrücken.

Der Doc-Kommentar `/// Startet eine neue Konversation mit einer Nutzernachricht` direkt
über `Chat { ... }` ist kein gewöhnlicher Kommentar — `clap` liest ihn aus und zeigt ihn
in der automatisch generierten `--help`-Ausgabe an. Das ist ein wiederkehrendes
Rust-Muster: Dokumentation im Code wird von Werkzeugen (hier `clap`, in
[Phase 7](../08-phase7-release/03-rustdoc-beispiele.md) `rustdoc`) direkt weiterverwendet,
statt nur für menschliche Leser*innen im Editor zu existieren.

### `#[arg(long)]` — wie aus einem Feld ein Flag wird

`#[arg(long)]` vor `system: Option<String>` sagt: Dieses Feld wird über `--system <wert>`
gesetzt (die lange Form, `long`, im Gegensatz zu einer Kurzform wie `-s`, die du mit
`#[arg(short)]` zusätzlich aktivieren könntest). Felder ohne `#[arg(...)]`-Attribut, wie
`nachricht`, werden **positional** — also einfach als nächstes Wort auf der Kommandozeile
erwartet, ohne Flag-Namen davor (`mein_cli chat "Hallo!"`, nicht `mein_cli chat
--nachricht "Hallo!"`).

### Vom `Befehl` zur `Konversation` — die Verbindung zu Phase 1

Der entscheidende Architekturpunkt dieser Lektion: `mein_cli` übersetzt den geparsten
`Befehl` in Aufrufe auf unsere `Konversation`-API aus
[Lektion 4](04-konversation.md) — und **nichts sonst**. `mein_cli` weiß nichts über
`Vec<Nachricht>`, nichts über die interne Struktur. Es kennt nur die öffentliche API:
`Konversation::neu()`, `Konversation::mit_systemnachricht(...)`, `hinzufuegen(...)`,
`verlauf()`.

## Schritt-Reveal

**Schritt 1** — Abhängigkeit ergänzen, `mein_cli/Cargo.toml`:

```toml
[dependencies]
mein_core = { path = "../mein_core" }
clap = { version = "...", features = ["derive"] }
```

Ersetze `"..."` mit der aktuellen stabilen Version, z. B. per `cargo add clap --features
derive` im Ordner `mein_cli` ausgeführt — das trägt Versionsnummer und Features
automatisch korrekt ein.

**Schritt 2** — CLI-Struktur definieren (siehe Zielbild oben) in `mein_cli/src/main.rs`,
oberhalb von `fn main()`.

**Schritt 3** — `main` anpassen:

```rust
use clap::Parser;
use mein_core::{Konversation, Rolle};

fn main() {
    let cli = Cli::parse();

    match cli.befehl {
        Befehl::Chat { nachricht, system } => {
            let mut konversation = match system {
                Some(text) => match Konversation::mit_systemnachricht(text) {
                    Ok(k) => k,
                    Err(fehler) => {
                        eprintln!("Ungültige Systemnachricht: {:?}", fehler);
                        return;
                    }
                },
                None => Konversation::neu(),
            };

            if let Err(fehler) = konversation.hinzufuegen(Rolle::Benutzer, nachricht) {
                eprintln!("Ungültige Nachricht: {:?}", fehler);
                return;
            }

            for eintrag in konversation.verlauf() {
                println!("{:?}: {}", eintrag.rolle, eintrag.inhalt);
            }
        }
    }
}
```

`Cli::parse()` liest `std::env::args()` selbst aus, validiert sie gegen unsere
`#[derive(Parser)]`-Definition und beendet das Programm mit einer hilfreichen
Fehlermeldung (inklusive Vorschlägen bei Tippfehlern!), falls die Eingabe nicht passt —
das alles, ohne dass wir es selbst schreiben mussten.

> **⚠️ Warnung**
>
> Verschachtelte `match`-Ausdrücke wie oben (`match system { Some(...) => match ... }`)
> werden schnell unübersichtlich. Das ist bewusst so belassen, um `Option`/`Result`
> gemeinsam mit `match` zu üben — in echtem, gewachsenem Code würdest du hier eher `?`
> und Hilfsfunktionen einsetzen. Die Übung am Ende dieser Lektion greift genau das auf.

## Ausführung

```bash
cargo run -p mein_cli -- chat "Hallo, wer bist du?"
```

```
Benutzer: Hallo, wer bist du?
```

```bash
cargo run -p mein_cli -- chat "Hallo!" --system "Du bist ein hilfreicher Assistent."
```

```
System: Du bist ein hilfreicher Assistent.
Benutzer: Hallo!
```

```bash
cargo run -p mein_cli -- chat --help
```

`clap` gibt automatisch eine formatierte Hilfe aus — inklusive des Doc-Kommentars von
oben. Probiere auch bewusst einen Tippfehler, z. B. `mein_cli chta "Hallo"` — `clap`
schlägt in der Fehlermeldung den korrekten Befehlsnamen vor.

> **💡 Tipp**
>
> Das doppelte `--` in `cargo run -p mein_cli -- chat "Hallo!"` trennt Cargo-eigene
> Argumente von den Argumenten, die an dein Programm weitergereicht werden. Ohne `--`
> würde Cargo versuchen, `chat` als eigenes Cargo-Flag zu interpretieren.

## Zusammenfassung

- `#[derive(Parser)]` + `#[derive(Subcommand)]` übersetzen eine Typdefinition direkt in
  eine vollständige CLI mit Validierung, Fehlermeldungen und `--help` — ganz ohne
  manuelles Parsen.
- `Option<T>` modelliert optionale Flags natürlich, ohne Sonderwerte wie leere Strings.
- `mein_cli` bleibt eine dünne Übersetzungsschicht zwischen Kommandozeile und der
  `Konversation`-API aus `mein_core` — die Trennung aus [Lektion 1](01-workspace-lesen.md)
  trägt bis hierher.

## Übung

Füge einen zweiten Subcommand `Verlauf` hinzu, der (fiktiv, da wir noch keine Persistenz
haben, siehe [Phase 2, Lektion 7](../03-phase2-llm-anbindung/07-persistenz-sqlx.md)) für
jetzt einfach eine Konversation mit drei fest eincodierten Nachrichten anlegt und ausgibt.
Räume anschließend den verschachtelten `match`-Block aus Schritt 3 auf: Extrahiere eine
private Hilfsfunktion `fn konversation_starten(system: Option<String>) ->
Result<Konversation, NachrichtFehler>`, die die Fallunterscheidung kapselt, und nutze `?`
in `main`, wo es die Struktur erlaubt.

[Weiter: Lektion 7 · Release 1 — conversation-in-memory](07-release-1.md)
