# Lektion 4: Konversation mit Vec\<Nachricht\>

## Problem

Eine einzelne `Nachricht` reicht nicht — ein Gespräch mit einem LLM besteht aus einer
**Folge** von Nachrichten, in Reihenfolge, oft beginnend mit einer optionalen
Systemnachricht. Wir brauchen einen Typ, der diesen Verlauf hält, sicher erweitert und
ausgibt, ohne dass Aufrufer*innen (wie `mein_cli`) sich um die interne Speicherung
kümmern müssen.

## Code (Zielbild)

```rust
#[derive(Debug, Clone, Default)]
pub struct Konversation {
    verlauf: Vec<Nachricht>,
}

impl Konversation {
    pub fn neu() -> Self {
        Konversation::default()
    }

    pub fn mit_systemnachricht(inhalt: impl Into<String>) -> Result<Self, NachrichtFehler> {
        let mut konversation = Konversation::neu();
        konversation.hinzufuegen(Rolle::System, inhalt)?;
        Ok(konversation)
    }

    pub fn hinzufuegen(
        &mut self,
        rolle: Rolle,
        inhalt: impl Into<String>,
    ) -> Result<(), NachrichtFehler> {
        let nachricht = Nachricht::neu(rolle, inhalt)?;
        self.verlauf.push(nachricht);
        Ok(())
    }

    pub fn verlauf(&self) -> &[Nachricht] {
        &self.verlauf
    }
}
```

## Dekonstruktion

### Warum `verlauf: Vec<Nachricht>` **privat** ist (kein `pub`)

Anders als bei `Nachricht` in [Lektion 2](02-rolle-und-nachricht.md) ist `verlauf` hier
**nicht** `pub`. Das ist eine bewusste Designentscheidung, nicht Vergesslichkeit: Wäre
`verlauf` öffentlich, könnte jeder Aufrufer per `konversation.verlauf.push(irgendwas)`
eine `Nachricht` einfügen, **ohne** durch `Nachricht::neu()` zu gehen — und damit ohne die
Invariante aus [Lektion 3](03-invarianten.md) (kein leerer Inhalt). Ein privates Feld plus
eine öffentliche Methode (`hinzufuegen`) ist das Standardmuster, um Invarianten über die
gesamte Lebensdauer eines Werts zu garantieren, nicht nur bei der Konstruktion. Das ist der
Kern von **Kapselung** (*encapsulation*) — ein Konzept aus der objektorientierten
Programmierung, das in Rust genauso gilt, auch wenn Rust selbst keine Klassen im
klassischen Sinn hat.

### `#[derive(Default)]` und `Konversation::default()`

`Default` ist ein weiteres ableitbares Trait: Es erzeugt einen "Standardwert" eines Typs.
Für `Vec<Nachricht>` ist der Standardwert eine leere Liste. Damit können wir
`Konversation::neu()` denkbar einfach schreiben, statt das leere `Vec` von Hand
anzulegen (`Vec::new()`). Das ist ein kleines, aber typisches Rust-Idiom: Wo möglich,
Standardverhalten ableiten statt neu schreiben.

### `&mut self` vs. `&self` — wer darf verändern?

```rust
pub fn hinzufuegen(&mut self, ...) -> Result<(), NachrichtFehler> { ... }
pub fn verlauf(&self) -> &[Nachricht] { ... }
```

`self` bezeichnet den Wert, auf dem die Methode aufgerufen wird (vergleichbar mit `this`
in anderen Sprachen). `&mut self` heißt: "Diese Methode braucht **veränderbaren** Zugriff
auf die `Konversation`" — folgerichtig, weil `hinzufuegen` das interne `verlauf` per
`push` verändert. `&self` heißt: "Diese Methode braucht nur **lesenden** Zugriff" —
`verlauf()` verändert nichts, sie gibt nur eine Sicht auf die Daten zurück.

Das ist Rusts **Borrowing**-System in Aktion (kurz in
[Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md) erwähnt): Zu
jedem Zeitpunkt darf entweder **eine** verändernde Referenz (`&mut`) oder **beliebig
viele** lesende Referenzen (`&`) auf einen Wert existieren — nie beides gleichzeitig. Der
Compiler erzwingt das beim Kompilieren. Das verhindert eine ganze Klasse von Bugs (Daten
werden gelesen, während sie gleichzeitig woanders verändert werden), die in anderen
Sprachen erst zur Laufzeit oder gar nicht auffallen.

Ruft man `hinzufuegen` auf, muss die Variable selbst `mut` sein — das siehst du gleich im
Schritt-Reveal.

### `Result<(), NachrichtFehler>` — Erfolg ohne Wert

`()` ("unit", schon in
[Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md) erwähnt)
bedeutet "kein interessanter Rückgabewert". `hinzufuegen` muss trotzdem `Result`
zurückgeben (nicht einfach `()`), weil es fehlschlagen kann (leerer Inhalt) — der
Erfolgsfall trägt aber nichts Weiteres außer "hat geklappt".

### `?` — der Fehler-Weiterreicher

```rust
pub fn hinzufuegen(&mut self, rolle: Rolle, inhalt: impl Into<String>) -> Result<(), NachrichtFehler> {
    let nachricht = Nachricht::neu(rolle, inhalt)?;
    self.verlauf.push(nachricht);
    Ok(())
}
```

`Nachricht::neu(rolle, inhalt)` gibt ein `Result<Nachricht, NachrichtFehler>` zurück. Das
`?` direkt danach heißt: "Wenn das `Ok(wert)` ist, entpacke `wert` und mach weiter. Wenn es
`Err(fehler)` ist, **gib sofort** `Err(fehler)` aus der aktuellen Funktion zurück." Ohne
`?` müsstest du das von Hand mit `match` schreiben:

```rust
let nachricht = match Nachricht::neu(rolle, inhalt) {
    Ok(n) => n,
    Err(fehler) => return Err(fehler),
};
```

`?` ist reine Abkürzung für genau dieses Muster — funktioniert aber nur, wenn die
umgebende Funktion selbst ein passendes `Result` zurückgibt (deshalb `-> Result<(),
NachrichtFehler>` bei `hinzufuegen`). Du wirst `?` ab jetzt in fast jeder fehlbaren
Funktion sehen; es ist eines der am häufigsten genutzten Sprachmittel in echtem
Rust-Code.

### `&[Nachricht]` statt `&Vec<Nachricht>` als Rückgabetyp

`verlauf()` gibt `&[Nachricht]` zurück, einen **Slice** — eine Sicht auf zusammenhängende
Elemente, ohne Aussage darüber, ob sie aus einem `Vec`, einem Array oder woanders
stammen. Das ist ein verbreitetes Rust-Idiom: Nach außen so wenig wie möglich über die
interne Speicherung verraten. Sollten wir `verlauf` intern später gegen eine andere
Datenstruktur austauschen (z. B. eine `VecDeque` für effizientes Einfügen am Anfang), bliebe
die öffentliche Signatur `verlauf() -> &[Nachricht]` unverändert — kein Breaking Change für
Aufrufer*innen.

## Schritt-Reveal

**Schritt 1** — Füge `Konversation` wie im Zielbild zu `mein_core/src/lib.rs` hinzu.
`cargo check -p mein_core`.

**Schritt 2** — Provoziere den Borrow-Checker bewusst. Schreibe testweise in
`mein_cli/src/main.rs`:

```rust
let konversation = Konversation::neu(); // ohne `mut`!
konversation.hinzufuegen(Rolle::Benutzer, "Hallo")?;
```

Der Compiler meldet:

```
error[E0596]: cannot borrow `konversation` as mutable, as it is not declared as mutable
```

Das ist exakt das Borrowing-Prinzip von oben in Aktion. Korrigiere zu `let mut
konversation = ...`.

**Schritt 3** — Vollständiges `main`:

```rust
use mein_core::{Konversation, Rolle};

fn main() {
    let mut konversation = Konversation::neu();

    if let Err(fehler) = konversation.hinzufuegen(Rolle::Benutzer, "Hallo, wer bist du?") {
        eprintln!("Fehler: {:?}", fehler);
        return;
    }

    for nachricht in konversation.verlauf() {
        println!("{:?}: {}", nachricht.rolle, nachricht.inhalt);
    }
}
```

`if let Err(fehler) = ...` ist eine Kurzform von `match`, wenn dich nur **ein** Fall
interessiert (hier: der Fehlerfall) und du den anderen ignorieren willst.

## Ausführung

```bash
cargo run -p mein_cli
```

```
Benutzer: Hallo, wer bist du?
```

```bash
cargo test -p mein_core
```

Ergänze in `mod tests` einen weiteren Test:

```rust
#[test]
fn konversation_sammelt_verlauf_in_reihenfolge() {
    let mut k = Konversation::neu();
    k.hinzufuegen(Rolle::System, "Du bist hilfreich.").unwrap();
    k.hinzufuegen(Rolle::Benutzer, "Hallo!").unwrap();

    assert_eq!(k.verlauf().len(), 2);
    assert_eq!(k.verlauf()[0].rolle, Rolle::System);
    assert_eq!(k.verlauf()[1].rolle, Rolle::Benutzer);
}
```

`.unwrap()` entpackt ein `Result` und **bricht das Programm mit `panic!` ab**, falls es
`Err` war. In Tests ist das üblich und akzeptabel (ein fehlschlagender Test soll laut
abbrechen) — in Produktionscode aus [Phase 2](../03-phase2-llm-anbindung/README.md) an
werden wir `.unwrap()` bewusst vermeiden und stattdessen `?` oder explizites
Fehlerhandling nutzen.

```
running 3 tests
test tests::gueltiger_inhalt_wird_akzeptiert ... ok
test tests::leerer_inhalt_wird_abgelehnt ... ok
test tests::konversation_sammelt_verlauf_in_reihenfolge ... ok
```

## Zusammenfassung

- Ein privates Feld (`verlauf`) plus eine öffentliche, prüfende Methode
  (`hinzufuegen`) erzwingt Invarianten über die gesamte Lebensdauer eines Werts, nicht nur
  bei der Konstruktion.
- `&self` für Lesezugriff, `&mut self` für Schreibzugriff — Rusts Borrow-Checker erzwingt
  diese Unterscheidung beim Kompilieren.
- `?` reicht einen `Err`-Fall automatisch an die Aufrufer*innen weiter, spart
  `match`-Boilerplate.
- `&[T]` statt `&Vec<T>` als Rückgabetyp verrät nach außen keine internen
  Speicherdetails.

## Übung — Transferaufgabe der Phase

Erweitere `Konversation::mit_systemnachricht` (bereits im Zielbild oben vorgesehen) so,
dass `mein_cli` beim Start **optional** eine Systemnachricht setzen kann, **ohne** dass
`mein_cli` weiß, dass `Konversation` intern ein `Vec<Nachricht>` ist. Baue dafür in
`main.rs` eine kleine Fallunterscheidung: Wenn ein Kommandozeilenargument vorhanden ist
(nutze vorerst `std::env::args()`, roh, ohne `clap` — das kommt in
[Lektion 6](06-cli-mit-clap.md)), erzeuge die Konversation mit `mit_systemnachricht(...)`,
sonst mit `Konversation::neu()`. Prüfe: Ändert sich an `Konversation` selbst etwas, wenn du
später (Lektion 6) `clap` einführst? Wenn deine Antwort "nein" ist, hast du die Aufgabe
richtig gelöst — das ist genau die Trennung von Domäne (`mein_core`) und Interface
(`mein_cli`), die wir seit [Lektion 1](01-workspace-lesen.md) verfolgen.

[Weiter: Lektion 5 — Konfiguration mit serde](05-serde-konfiguration.md)
