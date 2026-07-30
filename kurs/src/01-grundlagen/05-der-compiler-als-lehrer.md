# Der Compiler als Lehrer: Rusts Besonderheit

## Warum Rust "streng" wirkt

Wenn du schon in einer anderen Sprache programmiert hast, wirst du in den ersten Tagen mit
Rust häufiger als gewohnt gegen den Compiler laufen. Das ist normal — und in diesem Kurs
sogar gewollt. Die meisten Sprachen lassen dich Code ausführen, der Fehler enthält, und
der Fehler zeigt sich erst zur Laufzeit (manchmal Wochen später, beim Kunden, um 3 Uhr
nachts). Rust verschiebt so viele dieser Fehler wie möglich **auf den Zeitpunkt des
Kompilierens** — den einzigen Zeitpunkt, an dem sie garantiert billig sind, zu beheben.

Drei Bereiche prüft der Rust-Compiler besonders streng, die andere Sprachen dir selbst
überlassen:

1. **Typen** — siehe [Variablen und Typen](01-variablen-und-typen.md). Eine Funktion, die
   `String` erwartet, akzeptiert keine Zahl.
2. **Vollständigkeit** — siehe [Kontrollfluss](03-kontrollfluss.md). Ein `match` über ein
   `enum` muss alle Möglichkeiten abdecken.
3. **Ownership** — wer im Programm gerade "Besitzer" eines Werts im Speicher ist, und ob
   ein anderer Teil des Programms diesen Wert gerade lesen oder verändern darf. Dazu mehr,
   sobald wir es in [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)
   am echten `Nachricht`-Typ brauchen — hier reicht: Es ist ein Mechanismus, der
   verhindert, dass zwei Teile deines Programms gleichzeitig denselben Speicher auf eine
   Weise benutzen, die zu Abstürzen oder Dateninkonsistenzen führen würde. Andere Sprachen
   lösen dasselbe Problem zur Laufzeit mit einem Garbage Collector (der regelmäßig prüft,
   welcher Speicher noch gebraucht wird) — Rust löst es beim Kompilieren, ganz ohne
   Laufzeitkosten.

## Eine Fehlermeldung lesen

Nimm diesen (absichtlich fehlerhaften) Code:

```rust
fn verdoppeln(zahl: i32) -> i32 {
    zahl * 2;
}
```

Der Compiler meldet sinngemäß:

```
error[E0308]: mismatched types
 --> src/main.rs:2:5
  |
1 | fn verdoppeln(zahl: i32) -> i32 {
  |                             --- expected `i32` because of return type
2 |     zahl * 2;
  |     ^^^^^^^^- help: remove this semicolon to return this value
  |     |
  |     expected `i32`, found `()`
```

So liest du das, Zeile für Zeile:

- `error[E0308]` — Fehlercode. Bei Unsicherheit hilft `rustc --explain E0308` im Terminal
  für eine ausführlichere Erklärung mit Beispielen.
- `--> src/main.rs:2:5` — genauer Ort: Datei, Zeile 2, Spalte 5.
- Der Codeausschnitt mit `^^^` markiert **exakt**, welcher Teil gemeint ist.
- `expected `i32`, found `()`` — der Kern: Die Funktion sollte `i32` zurückgeben (das
  steht in `-> i32`), tut es aber nicht — `()` ("unit", sprich: "nichts") kommt heraus,
  weil das Semikolon aus `zahl * 2` eine Anweisung macht statt eines Rückgabewerts (siehe
  [Funktionen](02-funktionen.md)).
- `help: remove this semicolon` — der Compiler schlägt sogar die Lösung vor.

Das ist kein Zufall, sondern Designphilosophie: Rusts Fehlermeldungen sind bewusst so
geschrieben, dass sie *erklären*, nicht nur *melden*. In diesem Kurs zeigen wir dir in
fast jeder Lektion mindestens einen solchen Fehler bewusst, bevor wir die Korrektur
zeigen — Fehler lesen zu können ist eine Fähigkeit, die du übst wie jede andere.

> **💡 Tipp**
>
> Wenn eine Fehlermeldung überwältigend lang wirkt: Lies zuerst nur die **erste**
> gemeldete Zeile mit `error[...]` und den `-->`-Ort. Rust meldet oft mehrere Fehler auf
> einmal, aber Folgefehler sind häufig nur Konsequenzen des ersten. Behebe den ersten,
> kompiliere neu, sieh dann weiter.

> **⚠️ Warnung**
>
> Ein **Warning** (`warning: ...`, gelb) ist kein Fehler — dein Programm kompiliert trotzdem
> und läuft. Warnungen weisen auf mögliche Probleme hin (z. B. eine nie genutzte Variable).
> In diesem Kurs behandeln wir Warnungen trotzdem ernst: Ein sauberer `cargo build` ohne
> Warnungen ist Teil unserer "Definition of Done" ab [Phase 1](../02-phase1-fundament/README.md).

## `cargo check` vs. `cargo build` vs. `cargo run`

Drei Befehle, die du ständig brauchen wirst:

- `cargo check` — prüft, ob der Code kompilieren **würde**, ohne eine ausführbare Datei zu
  erzeugen. Am schnellsten, unser Standardwerkzeug während des Tippens.
- `cargo build` — kompiliert wirklich, erzeugt eine ausführbare Datei in `target/debug/`.
- `cargo run` — baut (falls nötig) und führt danach direkt aus.

In diesem Kurs rufen wir nach fast jedem Schritt `cargo check` auf — nicht, weil es
Pflicht ist, sondern weil ein kompilierender Zwischenstand dir sofort zeigt, ob du dem
Zielbild noch folgst.

[Weiter: Das erste Programm](06-erstes-programm.md)
