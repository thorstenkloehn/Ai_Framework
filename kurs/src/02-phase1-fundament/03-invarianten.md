# Lektion 3: Invarianten schützen

## Problem

`Nachricht::neu(Rolle::Benutzer, "")` kompiliert und läuft anstandslos — eine Nachricht mit
leerem Inhalt entsteht. Ist das ein Problem? Für ein KI-Framework: ja. Eine leere
Nutzernachricht an ein LLM zu schicken kostet API-Aufrufe ohne Sinn, und ein leerer
Assistant-Turn im Verlauf verwirrt spätere Auswertung. Wir wollen: **Eine `Nachricht` mit
leerem Inhalt soll gar nicht erst entstehen können.** Eine Regel, die für jeden Wert eines
Typs immer gelten muss, heißt **Invariante**.

## Code (Zielbild)

```rust
impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Result<Self, NachrichtFehler> {
        let inhalt = inhalt.into();
        if inhalt.trim().is_empty() {
            return Err(NachrichtFehler::LeererInhalt);
        }
        Ok(Nachricht { rolle, inhalt })
    }
}
```

## Dekonstruktion

Wir haben zwei grundsätzliche Strategien, eine Invariante durchzusetzen:

1. **Zur Laufzeit prüfen und einen Fehler zurückgeben** — der Weg, den wir hier gehen.
2. **Den ungültigen Zustand gar nicht erst darstellbar machen** — das haben wir bei
   `Rolle` in [Lektion 2](02-rolle-und-nachricht.md) schon gesehen (`enum` statt
   `String`). Für "Text ist nicht leer" gibt es in Rusts Standardbibliothek keinen
   eingebauten Typ — deshalb greifen wir zu Strategie 1.

### Warum `Result<Self, NachrichtFehler>` statt `panic!`?

Rust kennt zwei grundsätzlich verschiedene Wege, mit "das hätte nicht passieren dürfen"
umzugehen:

- **`panic!`** — das Programm bricht sofort ab. Sinnvoll für Programmierfehler, die
  *nie* im Normalbetrieb auftreten sollten (z. B. ein Index außerhalb eines Arrays).
- **`Result<T, E>`** ([Kapitel 0](../01-grundlagen/04-daten-buendeln.md) hat es
  eingeführt) — der Fehlerfall ist ein normaler, erwartbarer Teil des Programmablaufs, den
  Aufrufer*innen behandeln können und **müssen**.

Ein leerer Nachrichteninhalt ist kein Bug im Sinne von "das darf logisch nie passieren" —
er kann leicht durch Nutzereingabe entstehen (jemand drückt Enter auf ein leeres
Eingabefeld). Das ist ein erwartbarer Fehlerfall, also `Result`, nicht `panic!`.

### Ein eigener Fehlertyp: `NachrichtFehler`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NachrichtFehler {
    LeererInhalt,
}
```

Auch hier wieder `enum`, aus demselben Grund wie bei `Rolle`: Es gibt (aktuell) genau
einen Fehlerfall, aber die Struktur ist bereit, weitere hinzuzufügen (z. B. später
`ZuLang(usize)`), ohne die Aufrufer-API zu brechen. Ein `String` als Fehlertyp
(`Err("leerer Inhalt".to_string())`) wäre die naive Alternative — aber dann müsste
aufrufender Code den Fehlertext parsen, um zu wissen, welcher Fehler vorliegt. Mit `enum`
kann er stattdessen `match` benutzen:

```rust
match Nachricht::neu(Rolle::Benutzer, eingabe) {
    Ok(nachricht) => { /* weiter */ }
    Err(NachrichtFehler::LeererInhalt) => println!("Bitte gib etwas ein."),
}
```

Wir bauen `NachrichtFehler` hier noch von Hand als einfaches `enum`. In
[Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md) lernen wir
`thiserror`, eine Bibliothek, die genau solche Fehlertypen mit weniger Schreibarbeit und
besseren Fehlermeldungen erzeugt — wir heben uns das bewusst für später auf, um jetzt erst
das Grundprinzip ohne zusätzliche Abhängigkeit zu verstehen.

### `inhalt.trim().is_empty()` statt `inhalt.is_empty()`

`trim()` entfernt Leerzeichen/Zeilenumbrüche an Anfang und Ende. Ohne `trim()` würde
`Nachricht::neu(Rolle::Benutzer, "   ")` (nur Leerzeichen) fälschlich als "nicht leer"
durchgehen. Diese Art von Detail — der Unterschied zwischen "technisch nicht leer" und
"inhaltlich sinnvoll" — ist typisch für Invarianten: Die erste Version deckt den
offensichtlichen Fall ab, das genaue Hinsehen deckt die Randfälle auf.

## Schritt-Reveal

**Schritt 1 — Fehlertyp anlegen.** Füge in `mein_core/src/lib.rs` oberhalb von
`Nachricht` hinzu:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NachrichtFehler {
    LeererInhalt,
}
```

`cargo check -p mein_core` — kompiliert, `NachrichtFehler` wird nur noch nicht benutzt
(daher evtl. eine `warning: ... is never used`, das lösen wir im nächsten Schritt).

**Schritt 2 — `neu` anpassen.** Ändere die Signatur und den Körper wie im Zielbild oben.
`cargo check -p mein_core` — jetzt meldet der Compiler einen Fehler in `mein_cli/src/main.rs`:

```
error[E0308]: mismatched types
 --> mein_cli/src/main.rs:3:20
  |
3 |     println!("{:?}", nachricht);
  |                       ^^^^^^^^^ expected `Nachricht`, found `Result<Nachricht, NachrichtFehler>`
```

Das ist erwartet und richtig so: `Nachricht::neu(...)` gibt jetzt keine `Nachricht` mehr
direkt zurück, sondern ein `Result`, das behandelt werden muss.

**Schritt 3 — Aufrufer anpassen.** In `mein_cli/src/main.rs`:

```rust
use mein_core::{Nachricht, Rolle};

fn main() {
    match Nachricht::neu(Rolle::Benutzer, "Hallo wie gehts") {
        Ok(nachricht) => println!("{:?}", nachricht),
        Err(fehler) => eprintln!("Ungültige Nachricht: {:?}", fehler),
    }
}
```

`eprintln!` gibt (anders als `println!`) auf dem Fehlerkanal (*stderr*) statt dem
Standardkanal (*stdout*) aus — die übliche Konvention für Fehlermeldungen, damit sie sich
z. B. beim Umleiten der Programmausgabe in eine Datei (`programm > ausgabe.txt`) weiterhin
im Terminal zeigen.

## Ausführung

```bash
cargo run -p mein_cli
```

```
Nachricht { rolle: Benutzer, inhalt: "Hallo wie gehts" }
```

Provoziere jetzt den Fehlerfall bewusst — ändere den Text in `main.rs` testweise zu `""`:

```bash
cargo run -p mein_cli
```

```
Ungültige Nachricht: LeererInhalt
```

Setze den Text zurück, bevor du weitermachst.

```bash
cargo test -p mein_core
```

Noch kein Test vorhanden — das holen wir jetzt nach. Füge in `mein_core/src/lib.rs` am
Ende hinzu:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leerer_inhalt_wird_abgelehnt() {
        let ergebnis = Nachricht::neu(Rolle::Benutzer, "   ");
        assert_eq!(ergebnis, Err(NachrichtFehler::LeererInhalt));
    }

    #[test]
    fn gueltiger_inhalt_wird_akzeptiert() {
        let ergebnis = Nachricht::neu(Rolle::Benutzer, "Hallo");
        assert!(ergebnis.is_ok());
    }
}
```

`#[cfg(test)]` sorgt dafür, dass dieses Modul nur beim Testen mitkompiliert wird, nicht im
regulären Build. `use super::*;` holt alles aus dem umgebenden Modul (`Nachricht`, `Rolle`,
`NachrichtFehler`) in den Testmodul-Scope. `assert_eq!`/`assert!` sind Makros, die den Test
fehlschlagen lassen, wenn die Bedingung nicht zutrifft.

```bash
cargo test -p mein_core
```

```
running 2 tests
test tests::gueltiger_inhalt_wird_akzeptiert ... ok
test tests::leerer_inhalt_wird_abgelehnt ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

> **⚠️ Warnung**
>
> `assert_eq!(ergebnis, Err(NachrichtFehler::LeererInhalt))` braucht, dass `Result` und
> unser `NachrichtFehler` beide `PartialEq` (für den Vergleich) und `Debug` (damit
> `assert_eq!` bei einem Fehlschlag beide Seiten ausgeben kann) implementieren. Wir haben
> beides schon oben abgeleitet — falls du diese `derive`-Angaben vergisst, meldet der
> Compiler exakt, welches Trait fehlt.

## Zusammenfassung

- Eine Invariante ist eine Regel, die für jeden Wert eines Typs immer gelten muss —
  hier: "Inhalt ist nie leer."
- Zwei Strategien, Invarianten zu erzwingen: den ungültigen Zustand undarstellbar machen
  (nutzten wir bei `Rolle`), oder zur Konstruktionszeit prüfen und `Result` zurückgeben
  (nutzen wir hier bei "leerer Inhalt").
- `panic!` für Programmierfehler, die nie passieren sollten; `Result` für erwartbare,
  behandelbare Fehlerfälle.
- Ein eigenes `enum` als Fehlertyp erlaubt Aufrufer*innen, mit `match` gezielt zu
  reagieren, statt Fehlertexte zu parsen.
- Tests gehören ab jetzt zu jeder neuen Invariante — sowohl der Erfolgsfall als auch der
  bewusst provozierte Fehlerfall.

## Übung

Erweitere `NachrichtFehler` um einen zweiten Fall, `ZuLang(usize)` (der `usize`-Wert soll
die tatsächliche Länge tragen), und lehne in `Nachricht::neu` Inhalte länger als z. B.
4000 Zeichen ab (`inhalt.chars().count()`, nicht `inhalt.len()` — überlege dir anhand
von [Variablen und Typen](../01-grundlagen/01-variablen-und-typen.md) und der
Unicode-Natur von `char`, warum das für Nicht-ASCII-Text wie Umlaute oder Emoji einen
Unterschied macht). Schreibe dafür einen eigenen Test.

[Weiter: Lektion 4 — Konversation mit Vec\<Nachricht\>](04-konversation.md)
