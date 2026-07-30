# Lektion 1: Benchmarks mit criterion

## Problem

"Fühlt sich schnell genug an" ist keine Messung. In
[Phase 5](../06-phase5-rag-betrieb/02-chunking.md) haben wir eine Chunking-Funktion
gebaut, die lange Dokumente in kleinere Stücke zerlegt, bevor sie eingebettet werden. Wenn
wir jetzt die Chunking-Größe ändern, eine zweite Strategie einbauen oder einfach nur
wissen wollen, ob unser Code für ein 500-Seiten-Dokument noch praktikabel ist, brauchen
wir eine ehrliche Antwort — keine Stoppuhr am Handgelenk. Ein einzelner `cargo run
--release` mit `Instant::now()` drumherum reicht nicht: Ein Lauf ist verrauscht (der
Rechner macht nebenbei anderes), und ein einzelner Wert sagt nichts über Streuung oder
Ausreißer.

## Code (Zielbild)

```rust
// mein_rag/benches/chunking_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};
use mein_rag::chunking::chunke_dokument;
use mein_rag::loader::Document;
use std::collections::HashMap;

fn chunking_benchmark(c: &mut Criterion) {
    let dokument = Document {
        id: "bench".into(),
        content: "Lorem ipsum ".repeat(5_000),
        metadata: HashMap::new(),
    };
    c.bench_function("chunke_5000_woerter", |b| {
        b.iter(|| chunke_dokument(&dokument, 200, 40))
    });
}

criterion_group!(benches, chunking_benchmark);
criterion_main!(benches);
```

```
chunke_5000_woerter    time:   [812.34 µs 815.02 µs 818.11 µs]
```

## Dekonstruktion

### Was macht `criterion` anders als Handmessung?

**criterion** ist ein Rust-Crate für *statistisch fundierte* Benchmarks. Statt einmal zu
messen, führt es die Funktion sehr oft aus (typischerweise hunderte Male), verwirft
Aufwärm-Effekte, berechnet Konfidenzintervalle und warnt sogar automatisch, wenn ein
Benchmark im Vergleich zum letzten Lauf **signifikant** langsamer geworden ist — nicht
nur zufällig verrauscht. Das macht Benchmarks zu einem Werkzeug, dem man tatsächlich
trauen kann, statt zu einer Zahl, die man einmal notiert und nie wieder anschaut.

### Warum ein eigenes `benches/`-Verzeichnis?

Cargo kennt neben normalen Unit-Tests (`#[test]` in `src/`) und Integrationstests
(`tests/`) noch eine dritte Kategorie: **Benchmarks** in `benches/`. Der entscheidende
Unterschied: Benchmarks laufen im `--release`-Modus (mit vollen Compiler-Optimierungen —
sonst würden wir gar nicht die tatsächliche Produktionsgeschwindigkeit messen), und sie
brauchen einen eigenen Einstiegspunkt (`criterion_main!`), weil `criterion` sein eigenes
Mess-Framework mitbringt statt Cargos eingebautem Test-Harness.

### `criterion_group!` und `criterion_main!`

Zwei Makros ([Kapitel 0](../01-grundlagen/02-funktionen.md) hat Funktionen eingeführt —
ein Makro ist verwandt, erzeugt aber zur Kompilierzeit Code statt zur Laufzeit einen Wert
zurückzugeben): `criterion_group!` bündelt eine oder mehrere Benchmark-Funktionen zu einer
Gruppe, `criterion_main!` erzeugt daraus eine eigene `main`-Funktion für die
Benchmark-Binary. Deshalb braucht `benches/chunking_bench.rs` keine eigene `fn main()` —
`criterion_main!` generiert sie.

### `b.iter(|| ...)` und Compiler-Optimierungen

`c.bench_function` bekommt eine Closure ([Kapitel 0](../01-grundlagen/02-funktionen.md)),
die intern `b.iter(...)` aufruft — das ist der Codeblock, der wiederholt gemessen wird.
Ein wichtiges Detail, das criterion für uns löst: Ein optimierender Compiler könnte
theoretisch feststellen, dass das Ergebnis von `chunke(...)` nirgends benutzt wird, und
den ganzen Aufruf wegoptimieren ("dead code elimination") — dann würden wir eine leere
Funktion messen. criterion verhindert das intern (über `black_box`), sodass wir uns beim
einfachen Fall wie oben nicht selbst darum kümmern müssen.

## Schritt-Reveal

**Schritt 1 — Abhängigkeit als Dev-Dependency ergänzen.** Ein Benchmark-Werkzeug gehört
nicht in den Produktionscode, den `mein_rag` an Nutzer*innen ausliefert — nur in die
Entwicklungsumgebung. In `mein_rag/Cargo.toml`:

```toml
[dev-dependencies]
criterion = "..."

[[bench]]
name = "chunking_bench"
harness = false
```

Ersetze `"..."` mit der aktuellen stabilen Version, z. B. per `cargo add criterion --dev`
im Ordner `mein_rag` ausgeführt.

`harness = false` sagt Cargo: "Für diese Datei übernimmt criterion den Testeinstiegspunkt,
nicht dein eingebautes Test-Harness." Ohne diese Zeile versucht Cargo, `chunking_bench.rs`
mit dem Standard-Test-Harness zu bauen — das kollidiert mit criterions eigener `main`.

**Schritt 2 — Provoziere den Fehler bewusst.** Lösche testweise die Zeile `harness =
false` und führe aus:

```bash
cargo bench -p mein_rag
```

```
error[E0428]: the name `main` is defined multiple times
  --> benches/chunking_bench.rs
   |
   = note: `main` must be defined only once in the crate namespace
```

Zwei `main`-Funktionen wollen denselben Platz belegen: die von Cargos Standard-Harness und
die, die `criterion_main!` generiert. Die Fehlermeldung liest sich zunächst kryptisch,
sagt aber genau das: zwei Einstiegspunkte, ein Namensraum. Setze `harness = false` zurück,
bevor du weitermachst.

**Schritt 3 — Die Benchmark-Datei anlegen.** `mein_rag/benches/chunking_bench.rs` wie im
Zielbild oben. Tippe das selbst ab, bevor du weiterliest — `chunke_dokument` und
`Document` sind dieselben Typen aus
[Phase 5, Lektion 1](../06-phase5-rag-betrieb/01-document-loader.md) und
[Lektion 2](../06-phase5-rag-betrieb/02-chunking.md).

**Schritt 4 — `cargo bench -p mein_rag` ausführen** (siehe Ausführung unten).

## Ausführung

```bash
cargo bench -p mein_rag
```

```
Compiling mein_rag v0.1.0
    Finished bench [optimized] target(s) in 4.12s
     Running benches/chunking_bench.rs

chunke_5000_woerter     time:   [812.34 µs 815.02 µs 818.11 µs]
                        change: [-2.1034% +0.4521% +3.0122%] (p = 0.71 > 0.05)
                        No change in performance detected.
```

Die drei Werte in `time: [...]` sind das untere Ende, der Schätzwert und das obere Ende
des 95%-Konfidenzintervalls — nicht drei Einzelmessungen. Beim zweiten Lauf vergleicht
criterion automatisch gegen den letzten gespeicherten Lauf (`target/criterion/`) und sagt
dir, ob eine Änderung statistisch signifikant war oder im Rauschen liegt. Ändere probeweise
die Chunk-Größe von `200` auf `50` und führe den Benchmark erneut aus — beobachte, wie sich
`time:` und `change:` verändern.

> **💡 Tipp**
>
> `cargo bench` erzeugt zusätzlich einen HTML-Report unter
> `target/criterion/chunke_5000_woerter/report/index.html` (benötigt `gnuplot`, sonst
> begnügt sich criterion mit dem Textreport oben — beides ist völlig ausreichend, um mit
> dieser Lektion zu arbeiten).

## Zusammenfassung

- criterion misst statistisch fundiert statt einmalig — mit Konfidenzintervallen und
  automatischem Vergleich gegen den letzten Lauf.
- Benchmarks leben in `benches/`, sind eine eigene Cargo-Testkategorie, und laufen im
  `--release`-Modus.
- `harness = false` in `[[bench]]` übergibt den Einstiegspunkt an criterion — vergisst man
  das, kollidieren zwei `main`-Funktionen.
- Ein Dev-Dependency wie `criterion` beeinflusst nicht die Abhängigkeiten, die Nutzer*innen
  unseres Crates mitkompilieren müssen (relevant für [Lektion 2 in Phase
  7](../08-phase7-release/02-feature-flags.md), wenn wir Compile-Zeit-Abhängigkeiten
  bewusst steuern).

## Übung

Baue einen zweiten Benchmark, der zwei Chunking-Konfigurationen direkt gegenüberstellt
(z. B. Chunk-Größe 100 vs. 500) innerhalb derselben `criterion_group!`, sodass beide
Ergebnisse im selben Report erscheinen. Nutze dafür zwei Aufrufe von `c.bench_function` mit
unterschiedlichen Namen (z. B. `"chunke_klein"`, `"chunke_gross"`) in derselben
Benchmark-Funktion. Überlege: Welche Trade-offs zwischen Chunk-Größe und
Retrieval-Qualität aus [Phase 5, Lektion 2](../06-phase5-rag-betrieb/02-chunking.md) lassen
sich mit reiner Geschwindigkeitsmessung *nicht* beantworten?

[Weiter: Lektion 2 — Eigenschaften und Fuzzing mit proptest](02-fuzzing-proptest.md)
