# Lektion 2: Eigenschaften und Fuzzing mit proptest

## Problem

In [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) haben wir eine
Invariante durchgesetzt: `Nachricht::neu` lehnt leeren Inhalt ab. Wir haben das mit zwei
Unit-Tests geprüft — einem Beispiel mit nur Leerzeichen, einem mit "Hallo". Zwei
Beispiele. Aber die Behauptung "eine `Nachricht` hat *nie* leeren Inhalt" ist eine
Aussage über *alle möglichen* Eingaben — unendlich viele Zeichenketten, nicht nur die
zwei, die uns beim Schreiben des Tests eingefallen sind. Was ist mit einem String voller
Tabs und Zeilenumbrüche? Mit Emoji? Mit einer Zeichenkette, die nur aus
Unicode-Leerzeichen besteht, die `trim()` vielleicht anders behandelt, als wir denken? Wir
können nicht jeden Fall von Hand aufschreiben — aber wir können den Computer tausende
zufällige Fälle ausprobieren lassen.

## Code (Zielbild)

```rust
// mein_core/tests/proptest_nachricht.rs
use mein_core::{Nachricht, NachrichtFehler, Rolle};
use proptest::prelude::*;

proptest! {
    #[test]
    fn nachricht_hat_nie_leeren_inhalt(zufallstext in ".*") {
        match Nachricht::neu(Rolle::Benutzer, zufallstext) {
            Ok(nachricht) => prop_assert!(!nachricht.inhalt.trim().is_empty()),
            Err(NachrichtFehler::LeererInhalt) => {} // korrekt abgelehnt
        }
    }
}
```

## Dekonstruktion

### Property-Based-Testing statt Beispiel-Testing

Ein klassischer Unit-Test prüft: "Für *dieses eine* Beispiel gilt *dieses* Ergebnis."
**Property-Based-Testing** dreht die Frage um: "Für *jede* Eingabe, die einer bestimmten
Form entspricht, muss *diese Eigenschaft* (Property) gelten." Wir formulieren also nicht
mehr einzelne Testfälle, sondern eine allgemeine Regel — und überlassen es dem
Test-Framework, sich Eingaben auszudenken, die diese Regel auf die Probe stellen.
**proptest** ist das etablierte Rust-Crate dafür.

### Was heißt hier eigentlich "Eigenschaft"?

Bei `Nachricht::neu` lautet die Eigenschaft: "Für jede Zeichenkette gilt entweder (a) die
Konstruktion schlägt mit `LeererInhalt` fehl, oder (b) sie gelingt, und dann ist
`nachricht.inhalt` nach dem Trimmen garantiert nicht leer." Das ist exakt unsere
Invariante aus Phase 1 — nur jetzt als Regel formuliert, die proptest an hunderten
generierten Beispielen prüft, statt an den zwei, die wir uns ausgedacht haben.

### `".*"` — eine Strategy

Der Ausdruck `zufallstext in ".*"` sieht aus wie ein regulärer Ausdruck, ist aber eine
**Strategy** — proptests Begriff für "eine Regel, nach der Zufallswerte eines bestimmten
Typs erzeugt werden". `".*"` heißt hier: "generiere beliebige Strings, die diesem
Muster entsprechen" (praktisch: beliebiger Text beliebiger Länge, inklusive leerem
String, inklusive Sonderzeichen). proptest bringt Strategien für die meisten
Standardtypen mit (`any::<i32>()`, `any::<String>()`, Bereichsangaben wie `0..100`) — wir
könnten hier auch `any::<String>()` schreiben; `".*"` erlaubt uns zusätzlich, das Muster
gezielt einzuschränken, falls wir später z. B. nur druckbare ASCII-Zeichen testen wollen.

### Shrinking — der eigentliche Clou

Findet proptest einen fehlschlagenden Fall, gibt es sich nicht mit dem ersten
Zufallstreffer zufrieden. Es versucht systematisch, die Eingabe zu **verkleinern**
("shrinking") — kürzere Strings, einfachere Zeichen — solange der Test noch fehlschlägt.
Am Ende bekommst du nicht "ein zufälliger String aus 47 Zeichen mit drei Emoji hat den
Test kaputtgemacht", sondern den *minimalen* Fall, der das Problem reproduziert — oft ein
einzelnes Zeichen. Das macht das Debuggen erheblich einfacher als bei klassischem,
unstrukturiertem Fuzzing.

## Schritt-Reveal

**Schritt 1 — Dev-Dependency ergänzen.** In `mein_core/Cargo.toml`:

```toml
[dev-dependencies]
proptest = "..."
```

Ersetze `"..."` mit der aktuellen stabilen Version, z. B. per `cargo add proptest --dev`
im Ordner `mein_core` ausgeführt.

**Schritt 2 — Testdatei anlegen.** `mein_core/tests/proptest_nachricht.rs` wie im Zielbild
oben. Erinnerung an [Phase 3, Lektion 5](../04-phase3-architektur/05-tests-und-clippy.md):
Dateien in `tests/` sind Integrationstests — sie sehen `mein_core` von außen, genau wie
`mein_cli` es tut, und benutzen deshalb `use mein_core::{...}` statt `use super::*;`.

**Schritt 3 — Provoziere einen Fehlschlag bewusst.** Schwäche testweise die Invariante in
`mein_core/src/lib.rs`: Ersetze `inhalt.trim().is_empty()` durch `inhalt.is_empty()` (ohne
`trim()`) — genau der Fehler, den wir in
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) bewusst vermieden haben.
Führe aus:

```bash
cargo test -p mein_core --test proptest_nachricht
```

```
thread 'main' panicked at 'Test failed: assertion failed: !nachricht.inhalt.trim().is_empty();
    minimal failing input: zufallstext = " "
    ...
Diff < left / right > :
< true
---
> false
```

`minimal failing input: zufallstext = " "` — proptest hat aus tausenden Zufallsstrings
genau den einen kürzesten übrig gelassen, der den Fehler zeigt: ein einzelnes Leerzeichen.
Ohne `trim()` gilt `" ".is_empty() == false`, die Nachricht wird also fälschlich
akzeptiert, obwohl sie inhaltlich leer ist — proptest hat den Randfall gefunden, den wir in
Phase 1 per Hand bedacht, aber nie automatisiert geprüft hatten. Mache die Änderung
rückgängig (`trim()` wieder einsetzen), bevor du weitermachst.

**Schritt 4 — proptest-regressions.** Nach einem Fehlschlag legt proptest automatisch eine
Datei `mein_core/proptest-regressions/proptest_nachricht.txt` an, die den minimalen
fehlschlagenden Fall speichert. Beim nächsten Testlauf prüft proptest diesen Fall zuerst,
bevor es neue Zufallsfälle generiert — ein einmal gefundener Bug bleibt also dauerhaft
abgedeckt, statt sich beim nächsten Lauf im Zufall zu verstecken.

> **💡 Tipp**
>
> Commite `proptest-regressions/*.txt` mit ins Repository. Es ist reiner Text, winzig,
> und macht gefundene Randfälle Teil der dauerhaften Testsuite — für dein Team genauso
> wertvoll wie ein von Hand geschriebener Regressionstest.

## Ausführung

Mit korrigiertem `trim()`:

```bash
cargo test -p mein_core --test proptest_nachricht
```

```
running 1 test
test nachricht_hat_nie_leeren_inhalt ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Im Hintergrund hat proptest dafür standardmäßig 256 zufällige Fälle durchprobiert (nicht
nur einen) — sichtbar, wenn du `PROPTEST_VERBOSE=1 cargo test -p mein_core --test
proptest_nachricht -- --nocapture` ausführst.

> **⚠️ Warnung**
>
> proptest-Läufe sind standardmäßig zufällig (mit einem zeitbasierten Seed), aber
> reproduzierbar bei Bedarf: Ein Fehlschlag druckt den verwendeten Seed mit aus. Verwechsle
> proptest nicht mit reinem "Fuzzing" im Sicherheitssinn (unstrukturierte Bytes gegen einen
> Parser werfen, z. B. mit `cargo-fuzz`) — proptest generiert *typisierte, strukturierte*
> Zufallswerte gegen von dir formulierte Eigenschaften. Für unser Framework reicht das:
> Wir wollen Invarianten in unserer eigenen Domäne prüfen, keinen rohen Byte-Parser härten.

## Zusammenfassung

- Property-Based-Testing prüft eine allgemein formulierte Eigenschaft gegen viele
  generierte Eingaben, statt einzelne Beispiele von Hand zu wählen.
- proptest generiert Werte über **Strategies** (`any::<T>()`, Muster wie `".*"`,
  Bereiche) und **shrinkt** fehlschlagende Fälle auf den minimalen Reproduzierer.
- Ein Fehlschlag wird automatisch in `proptest-regressions/` gespeichert und bleibt damit
  dauerhaft Teil der Testsuite.
- proptest ist kein Ersatz für Unit-Tests, sondern eine Ergänzung: Unit-Tests dokumentieren
  konkrete, benannte Fälle ("leerer String wird abgelehnt"); proptest sucht systematisch
  nach Fällen, an die wir nicht gedacht haben.

## Übung

Formuliere eine Eigenschaft für das Chunking aus
[Phase 5, Lektion 2](../06-phase5-rag-betrieb/02-chunking.md): Für jedes zufällig
generierte Dokument (beliebige Länge, beliebiger Text) und jede sinnvolle Chunk-Größe
zwischen z. B. 10 und 1000 muss gelten, dass kein einzelner Chunk die angegebene
Maximalgröße überschreitet, *und* dass beim Aneinanderfügen aller Chunks kein Textinhalt
verloren geht. Zwei Hinweise: Für "Chunk-Größe im Bereich" kannst du proptests
`10..1000usize`-Syntax als Strategy verwenden; für "kein Textverlust" reicht zunächst ein
einfacherer Vergleich als exakte Gleichheit — überlege, welche Eigenschaft du wirklich
garantieren willst, bevor du sie codierst.

[Weiter: Lektion 3 — Model Routing und Fallback](03-model-routing-fallback.md)
