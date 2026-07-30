# Das erste Programm

Jetzt wenden wir alles aus diesem Kapitel an einem eigenständigen Mini-Projekt an — bewusst
**getrennt** vom Ai_Framework-Repository, damit du ohne Sorge experimentieren kannst, bevor
wir ab [Phase 1](../02-phase1-fundament/README.md) am echten Projekt arbeiten.

## Schritt 1: Ein neues Cargo-Projekt anlegen

An einem beliebigen Ort auf deiner Festplatte, außerhalb des `Ai_Framework`-Ordners:

```bash
cargo new erste_schritte
cd erste_schritte
```

`cargo new` legt einen Ordner mit dieser Struktur an:

```
erste_schritte/
├── Cargo.toml
└── src/
    └── main.rs
```

`Cargo.toml` beschreibt dein Projekt (Name, Version, Abhängigkeiten) — vergleichbar mit
`package.json` in JavaScript-Projekten oder `pyproject.toml` in Python, falls dir das
etwas sagt. `src/main.rs` ist der Startpunkt deines Programms. Cargo hat dort schon etwas
vorbereitet:

```rust
fn main() {
    println!("Hello, world!");
}
```

`fn main()` ist die Funktion, die Rust beim Programmstart automatisch aufruft — jedes
ausführbare Rust-Programm braucht genau eine. `println!(...)` gibt Text auf der
Konsole aus (das `!` markiert es als **Makro**, dazu später mehr — für jetzt: `println!`
verhält sich wie eine Funktion für Textausgabe).

Führe es aus:

```bash
cargo run
```

Erwartete Ausgabe:

```
   Compiling erste_schritte v0.1.0 (/pfad/zu/erste_schritte)
    Finished dev [unoptimized + debuginfo] target(s) in 0.42s
     Running `target/debug/erste_schritte`
Hello, world!
```

## Schritt 2: Eigenen Code schreiben

Ersetze den Inhalt von `src/main.rs` durch (**selbst tippen**, siehe
[Wie dieser Kurs funktioniert](../00-einleitung/02-wie-dieser-kurs-funktioniert.md)):

```rust
enum Tageszeit {
    Morgen,
    Mittag,
    Abend,
}

fn begruessung(zeit: &Tageszeit) -> &'static str {
    match zeit {
        Tageszeit::Morgen => "Guten Morgen!",
        Tageszeit::Mittag => "Guten Tag!",
        Tageszeit::Abend => "Guten Abend!",
    }
}

fn main() {
    let zeiten = vec![Tageszeit::Morgen, Tageszeit::Mittag, Tageszeit::Abend];

    for zeit in &zeiten {
        println!("{}", begruessung(zeit));
    }
}
```

Das kombiniert alles aus diesem Kapitel: ein `enum` ([Daten
bündeln](04-daten-buendeln.md)), eine Funktion mit `match`
([Funktionen](02-funktionen.md), [Kontrollfluss](03-kontrollfluss.md)), ein `Vec` und
eine `for`-Schleife ([Daten bündeln](04-daten-buendeln.md)).

```bash
cargo run
```

Erwartete Ausgabe:

```
Guten Morgen!
Guten Tag!
Guten Abend!
```

## Schritt 3: Einen Fehler bewusst provozieren

Entferne testweise einen `match`-Fall — lösche die Zeile mit `Tageszeit::Abend`. Führe
`cargo check` aus. Der Compiler verweigert die Kompilierung, weil `match` nicht mehr alle
`enum`-Fälle abdeckt (siehe [Der Compiler als Lehrer](05-der-compiler-als-lehrer.md)).
Lies die Fehlermeldung selbst, bevor du die Zeile zurückschreibst — genau dieses Muster
("Fehler zulassen, Meldung selbst lesen, dann korrigieren") ist der rote Faden durch den
ganzen restlichen Kurs.

## Zusammenfassung

- Ein Cargo-Projekt entsteht mit `cargo new`, läuft mit `cargo run`, wird geprüft mit
  `cargo check`.
- `fn main()` ist der Startpunkt jedes ausführbaren Rust-Programms.
- `enum` + `match` + `Vec` + `for`-Schleife reichen schon für ein kleines, aber
  vollständiges Programm.
- Ein absichtlich unvollständiger `match` ist ein Compilerfehler, kein Laufzeitfehler —
  genau das Verhalten, das wir uns von Rust wünschen.

## Übung

Erweitere `erste_schritte` um eine vierte `Tageszeit`, `Nacht`, mit passender Begrüßung.
Füge außerdem eine Funktion `ist_bettzeit(zeit: &Tageszeit) -> bool` hinzu, die `true`
zurückgibt für `Nacht`, sonst `false`. Rufe sie in `main` für jede Tageszeit auf und gib
das Ergebnis mit aus.

Wenn das läuft und kompiliert, bist du bereit für den echten Code:
[Phase 1 — Fundament](../02-phase1-fundament/README.md).
