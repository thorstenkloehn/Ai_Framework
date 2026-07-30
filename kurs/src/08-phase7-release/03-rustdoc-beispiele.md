# Lektion 3: Rustdoc und Beispiele

## Problem

Seit [Phase 1](../02-phase1-fundament/README.md) schreiben wir `//`-Kommentare, die *uns*
— den Autor*innen im Kurs — erklären, wie ein Codeabschnitt funktioniert. Für jemanden,
der `mein_core` künftig als fremdes Crate von [crates.io](https://crates.io) einbindet,
sind interne Kommentare wertlos: Er oder sie sieht nur kompilierten Code und liest
höchstens die generierte API-Dokumentation. Ohne Dokumentation bleibt einer außenstehenden
Person nur der Quellcode selbst als Referenz — für ein öffentliches Crate inakzeptabel.
Wir brauchen Dokumentation, die aus dem Code selbst entsteht, immer aktuell bleibt (weil
sie direkt neben dem Code steht) und — im Idealfall — sich sogar selbst auf Richtigkeit
prüft.

## Code (Zielbild)

```rust
/// Erstellt eine neue [`Nachricht`] mit der gegebenen [`Rolle`].
///
/// # Fehler
///
/// Gibt [`NachrichtFehler::LeererInhalt`] zurück, wenn `inhalt` nach dem
/// Entfernen von Leerzeichen leer ist.
///
/// # Beispiel
///
/// ```
/// use mein_core::{Nachricht, Rolle};
///
/// let nachricht = Nachricht::neu(Rolle::Benutzer, "Hallo")?;
/// assert_eq!(nachricht.inhalt, "Hallo");
/// # Ok::<(), mein_core::NachrichtFehler>(())
/// ```
pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Result<Self, NachrichtFehler> {
    // ...
}
```

## Dekonstruktion

### `///` statt `//` — Doc Comments

Ein Kommentar mit drei Schrägstrichen (`///`) statt zwei ist kein gewöhnlicher Kommentar
mehr, sondern ein **Doc Comment**: Rust behandelt ihn als strukturierten Bestandteil der
öffentlichen API. `cargo doc` liest jeden `///`-Kommentar über einem `pub`-Element und
erzeugt daraus eine durchsuchbare HTML-Dokumentation — dieselbe Art von Seiten, die du
vermutlich schon für Crates wie `serde` oder `tokio` auf
[docs.rs](https://docs.rs) gesehen hast. Ein Doc Comment über `mein_core::Nachricht::neu`
wird für jede Person sichtbar, die dieses Crate einbindet, ganz ohne unseren Quellcode
gesehen zu haben.

### Intra-Doc-Links: `[`Nachricht`]`

Die eckigen Klammern mit Backticks, `[`Nachricht`]`, sind kein Markdown-Link im
klassischen Sinn, sondern ein **Intra-Doc-Link** — rustdoc verknüpft ihn automatisch mit
der Dokumentationsseite des Typs `Nachricht`, sofern er im selben Crate (oder einer
Dependency) sichtbar ist. Klickst du in der generierten HTML-Doku darauf, springst du
direkt zur `Nachricht`-Seite. Das funktioniert nur, weil rustdoc den Code tatsächlich
versteht — nicht nur den Text liest wie ein gewöhnliches Markdown-Tool.

### Der Codeblock in einem Doc Comment ist ein Test

Das ist der eigentliche Clou dieser Lektion: Ein mit ` ```rust ` (oder einfach ` ``` `,
Rust ist rustdocs Standardsprache) eingeleiteter Codeblock in einem Doc Comment wird von
`cargo test` **automatisch mitkompiliert und ausgeführt** — als sogenannter **Doctest**.
Ändert sich `Nachricht::neu`s Signatur künftig und das Beispiel im Doc Comment passt nicht
mehr dazu, schlägt `cargo test` fehl — nicht erst, wenn eine echte Nutzerin sich
beschwert, sondern schon bei uns, lokal, vor jedem Release. Dokumentation, die lügt,
kompiliert schlicht nicht mehr.

### Die `# `-Zeile — versteckte Hilfszeilen

Die Zeile `# Ok::<(), mein_core::NachrichtFehler>(())` beginnt mit `#` gefolgt von einem
Leerzeichen — eine rustdoc-Konvention, die diese Zeile beim `cargo test`-Lauf mitausführt,
aber in der gerenderten HTML-Dokumentation **ausblendet**. Wir brauchen sie hier, weil
unser Beispiel mit `?` arbeitet (das `NachrichtFehler` propagiert), Doctests aber
standardmäßig eine Funktion ohne Rückgabewert erwarten — die versteckte Zeile liefert den
passenden `Ok(())`-Abschluss, ohne dass Leser*innen der Dokumentation diesen technischen
Kniff überhaupt sehen müssen.

## Schritt-Reveal

**Schritt 1 — Doc Comment für `Nachricht::neu` ergänzen** (siehe Zielbild oben), direkt
über der bestehenden Funktion aus [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md).

**Schritt 2 — `cargo doc` ausführen und ansehen.**

```bash
cargo doc -p mein_core --no-deps --open
```

Das öffnet die generierte HTML-Dokumentation im Browser — deine `///`-Kommentare als
formatierte, verlinkte Seite. `--no-deps` beschleunigt den Bau, indem nur `mein_core`
selbst dokumentiert wird, nicht jede Abhängigkeit mit.

**Schritt 3 — Provoziere einen fehlschlagenden Doctest bewusst.** Ändere im Beispiel-Code
oben testweise `assert_eq!(nachricht.inhalt, "Hallo")` zu `assert_eq!(nachricht.inhalt,
"Falsch")`:

```bash
cargo test -p mein_core --doc
```

```
running 1 test
test src/lib.rs - Nachricht::neu (line 15) ... FAILED

failures:

---- src/lib.rs - Nachricht::neu (line 15) stdout ----
thread 'main' panicked at 'assertion `left == right` failed
  left: "Hallo"
 right: "Falsch"
```

Der Testname `src/lib.rs - Nachricht::neu (line 15)` zeigt genau, welcher Doc Comment an
welcher Zeile fehlgeschlagen ist. Setze das Beispiel zurück, bevor du weitermachst.

**Schritt 4 — `#![warn(missing_docs)]` an den Crate-Kopf setzen.** In
`mein_core/src/lib.rs`, ganz oben:

```rust
#![warn(missing_docs)]
```

```bash
cargo check -p mein_core
```

```
warning: missing documentation for a struct
 --> mein_core/src/lib.rs:20:1
   |
20 | pub struct Konfiguration {
   | ^^^^^^^^^^^^^^^^^^^^^^^^
```

Dieses Lint macht fehlende Dokumentation für jedes öffentliche Element sichtbar — ein
guter Wächter, bevor ein Crate veröffentlicht wird. Ergänze fehlende `///`-Kommentare,
bis die Warnung verschwindet.

## Ausführung

```bash
cargo test -p mein_core --doc
```

```
running 3 tests
test src/lib.rs - Konfiguration (line 8) ... ok
test src/lib.rs - Nachricht::neu (line 15) ... ok
test src/routing.rs - RoutingProvider (line 4) ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

> **💡 Tipp**
>
> Ein Doctest, der einen echten API-Aufruf zeigen würde (z. B. `LlmProvider::chat` gegen
> einen echten Anbieter), würde bei jedem `cargo test` Netzwerkkosten verursachen und bei
> fehlendem API-Key fehlschlagen. Nutze für Doctests, die den Provider-Port zeigen sollen,
> immer den `adapter::fake`-Testadapter aus
> [Phase 3, Lektion 4](../04-phase3-architektur/04-fake-provider.md) — genau wie in
> normalen Tests.

## Zusammenfassung

- `///`-Doc-Comments erzeugen über `cargo doc` eine echte, durchsuchbare API-Dokumentation
  für Menschen, die unseren Quellcode nie sehen.
- Codeblöcke in Doc Comments sind Doctests — `cargo test` kompiliert und führt sie aus,
  sodass veraltete Beispiele automatisch als fehlgeschlagener Test auffallen.
- Intra-Doc-Links (`[`Typ`]`) verknüpfen Dokumentationsseiten automatisch miteinander.
- `#![warn(missing_docs)]` macht fehlende Dokumentation an öffentlichen Elementen
  sichtbar, bevor ein Crate veröffentlicht wird.

## Übung

Dokumentiere `Konversation` und den `LlmProvider`-Trait selbst mit `///`-Kommentaren
inklusive je einem lauffähigen Doctest. Nutze für den `LlmProvider`-Doctest den
Fake-Provider aus Phase 3, damit `cargo test --doc` ohne echten API-Key und ohne
Netzwerkzugriff durchläuft. Prüfe danach mit `cargo doc -p mein_core --no-deps --open`, ob
die Intra-Doc-Links zwischen `Konversation`, `Nachricht` und `LlmProvider` tatsächlich
klickbar zueinander führen.

[Weiter: Lektion 4 — SemVer und Breaking Changes](04-semver-breaking-changes.md)
