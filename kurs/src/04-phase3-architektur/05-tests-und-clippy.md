# Lektion 5: Integrationstests und clippy

## Problem

Alle Tests, die wir bisher geschrieben haben — seit
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) — liegen in `#[cfg(test)]
mod tests`-Blöcken **innerhalb** derselben Datei wie der getestete Code. Das nennt man
**Unit-Tests**: Sie laufen als Teil derselben Crate, dürfen deshalb auch private Details
sehen (wie `FakeVerhalten` in [Lektion 4](04-fake-provider.md), das nicht `pub` ist). Das
ist wertvoll, um einzelne Bausteine isoliert zu prüfen — aber es beweist nicht, dass
`mein_core` auch für einen **fremden** Aufrufer (wie `mein_cli`, oder später `mein_agent`
aus [Phase 4](../05-phase4-agenten/README.md)) tatsächlich benutzbar ist, wenn dieser nur
die öffentliche `pub`-Oberfläche sieht.

Gleichzeitig haben wir seit [Phase 1s Definition of Done](../02-phase1-fundament/07-release-1.md)
informell `cargo clippy` laufen lassen, ohne genauer zu verstehen, was es eigentlich prüft.
Jetzt, wo unser Code eine echte Architektur mit mehreren Modulen hat, lohnt sich der genaue
Blick: Clippy ist Rusts Standard-Linter — ein Werkzeug, das über reine Compilerfehler hinaus
nach **Stil- und Qualitätsproblemen** sucht, die zwar kompilieren, aber vermeidbar oder
riskant sind.

## Code (Zielbild)

```rust
// mein_core/tests/llm_provider_grenzfaelle.rs — ein Integrationstest
use mein_core::adapter::fake::FakeProvider;
use mein_core::port::{ChatAnfrage, LlmProvider};
use mein_core::error::ProviderFehler;

#[test]
fn timeout_wird_von_aussen_sichtbar() {
    let provider = FakeProvider::simuliert_timeout();
    let anfrage = ChatAnfrage {
        nachrichten: vec![],
        modell: "irgendein-modell".into(),
    };

    let ergebnis = provider.chat(anfrage);

    assert!(matches!(ergebnis, Err(ProviderFehler::Timeout)));
}
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

## Dekonstruktion

### `tests/` als eigenes Verzeichnis — eine ganz andere Art Test

Jede Datei in `mein_core/tests/` wird von Cargo als **eigene, separate Crate** kompiliert,
die `mein_core` als Abhängigkeit einbindet — genau so, wie es `mein_cli` auch tut. Das ist
der fundamentale Unterschied zu einem `#[cfg(test)] mod tests`-Block: Integrationstests
sehen `mein_core` **von außen**, exakt wie jede andere Anwendung, die die Crate benutzt. Sie
können nur auf `pub`-Elemente zugreifen — kein Zugriff auf private Felder oder Module.

### Der Haken: `#[cfg(test)]` gilt nicht für Integrationstests

Versuch testweise, den Code aus dem Zielbild genau so in `mein_core/tests/` abzulegen und
auszuführen:

```
error[E0433]: failed to resolve: could not find `fake` in `adapter`
 --> tests/llm_provider_grenzfaelle.rs:1:29
  |
1 | use mein_core::adapter::fake::FakeProvider;
  |                             ^^^^ could not find `fake` in `adapter`
```

Das ist zunächst überraschend, wenn man `#[cfg(test)]` aus [Lektion 4](04-fake-provider.md)
im Kopf hat — `cargo test` läuft doch gerade! Der Punkt ist: `#[cfg(test)]` gilt nur, wenn
**die Crate selbst** (`mein_core`) für ihre eigenen internen Tests kompiliert wird. Eine
Datei in `tests/` gehört zu einer **separaten** Crate, die `mein_core` nur als fertige,
normal gebaute Abhängigkeit einbindet — für diesen Build-Schritt ist `mein_core` ein ganz
gewöhnlicher Release-Build, und `fake` existiert darin nicht, exakt wie schon am Ende von
Lektion 4 mit `mein_cli` demonstriert.

### Die Lösung: ein `test-utils`-Feature-Flag

Ein verbreitetes, reales Muster in der Rust-Welt: Statt `FakeProvider` fest an `cfg(test)`
zu binden, koppeln wir ihn zusätzlich an ein eigenes **Cargo-Feature** (Feature-Flags
vertiefen wir formal in
[Phase 7, Lektion 2](../08-phase7-release/02-feature-flags.md), hier nutzen wir nur das
Minimum). In `mein_core/Cargo.toml`:

```toml
[features]
test-utils = []
```

Und in `mein_core/src/adapter.rs`:

```rust
pub mod openai_kompatibel;

#[cfg(any(test, feature = "test-utils"))]
pub mod fake;
```

`cfg(any(test, feature = "test-utils"))` heißt: "kompiliere dieses Modul, wenn *entweder* im
Testmodus der Crate selbst gebaut wird, *oder* das Feature `test-utils` explizit aktiviert
ist." Ein Integrationstest aktiviert dieses Feature, indem er `mein_core` mit dem Feature
anfordert — dafür trägt `mein_core/Cargo.toml` sich testweise selbst als
Dev-Dependency mit dem Feature ein:

```toml
[dev-dependencies]
mein_core = { path = ".", features = ["test-utils"] }
```

> **💡 Tipp**
>
> Eine Crate, die sich selbst als `dev-dependency` einträgt, wirkt beim ersten Lesen
> ungewöhnlich — sie ist aber ein etabliertes Muster genau für diesen Zweck: Der reguläre
> Build (für `mein_cli` oder später Nutzer*innen von `mein_core`) bekommt `test-utils`
> **nicht** automatisch mit, nur Cargos eigene Test- und Integrationstest-Builds tun das.
> Damit bleibt `FakeProvider` weiterhin unsichtbar für alle, die `mein_core` normal
> benutzen — nur unsere eigenen Integrationstests bekommen ihn zu sehen.

Mit diesem Setup kompiliert der Integrationstest aus dem Zielbild — er sieht `FakeProvider`
jetzt als Teil der öffentlichen (aber feature-gated) API, genau wie `mein_cli` es später für
echte Provider-Auswahl über eine CLI-Flag tun könnte.

### clippy: mehr als der Compiler

`cargo check`/`cargo build` prüfen, ob dein Code **gültiges** Rust ist. `cargo clippy` prüft
zusätzlich, ob er **idiomatisches, wartbares** Rust ist — Regeln, die kein Compilerfehler
sind, aber fast immer auf einen besseren Weg hindeuten. Clippy-Lints sind in Kategorien
eingeteilt (u. a. `correctness` — wahrscheinliche Bugs, `style` — unidiomatische, aber
funktionierende Muster, `complexity` — unnötig kompliziert geschriebener Code,
`perf` — vermeidbare Laufzeitkosten). Ein Beispiel, das in dieser Phase leicht passiert —
eine Prüfung, ob ein `ProviderFehler` ein Timeout ist, per Hand als `match` geschrieben:

```rust
fn ist_timeout(fehler: &ProviderFehler) -> bool {
    match fehler {
        ProviderFehler::Timeout => true,
        _ => false,
    }
}
```

```bash
cargo clippy -p mein_core
```

```
warning: match expression looks like `matches!` macro
  --> src/error.rs:42:5
   |
42 | /     match fehler {
43 | |         ProviderFehler::Timeout => true,
44 | |         _ => false,
45 | |     }
   | |_____^ help: try: `matches!(fehler, ProviderFehler::Timeout)`
   |
   = note: `#[warn(clippy::match_like_matches_macro)]` on by default
```

Der Code ist nicht *falsch* — er kompiliert, er tut das Richtige. Clippy weist trotzdem
darauf hin, dass `matches!` (das wir bereits in [Lektion 4](04-fake-provider.md) benutzt
haben) genau diesen Fall kürzer und ohne Boolean-Umweg ausdrückt. `#[warn(...)]` am Ende
zeigt: Diese Regel ist standardmäßig eine **Warnung**, kein Fehler — dein Code baut trotzdem.

### `-D warnings` — aus Warnung wird Fehler

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

`--workspace` prüft alle Crates (`mein_core` **und** `mein_cli`), nicht nur die im aktuellen
Verzeichnis. `--all-targets` bezieht auch Tests, Beispiele und Integrationstests mit ein, nicht
nur den regulären Build. `-D warnings` (die Flags nach dem `--` gehen an den Compiler
durch, nicht an Cargo selbst) steht für *deny warnings*: **Jede** Clippy-Warnung wird zu
einem harten Fehler, der Build schlägt fehl. Das ist der Unterschied zwischen "eine
Warnung, die man ignorieren kann" (wie in
[Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md) beschrieben) und
einem echten **Quality Gate**: Ein Team (oder eine CI-Pipeline) kann so erzwingen, dass
niemand — auch nicht aus Versehen — eine neue Warnung einführt, ohne sie zu beheben.

## Schritt-Reveal

**Schritt 1** — Provoziere den E0433-Fehler bewusst: Lege `mein_core/tests/`-Verzeichnis mit
dem Zielbild-Code an, ohne das `test-utils`-Feature. `cargo test -p mein_core` schlägt fehl.

**Schritt 2** — Ergänze `[features] test-utils = []` und die Dev-Dependency in
`mein_core/Cargo.toml`, ändere `#[cfg(test)]` zu `#[cfg(any(test, feature =
"test-utils"))]` in `adapter.rs`.

**Schritt 3** — `cargo test -p mein_core` (Cargo zieht Dev-Dependencies und deren Features
automatisch für `cargo test` heran):

```
running 1 test
test timeout_wird_von_aussen_sichtbar ... ok

running 2 tests
test adapter::fake::tests::erfolgreiche_antwort_wird_durchgereicht ... ok
test adapter::fake::tests::timeout_wird_ohne_echte_api_erkannt ... ok
```

Beachte die zwei separaten `running N tests`-Blöcke: der erste ist der Integrationstest aus
`tests/`, der zweite die Unit-Tests aus [Lektion 4](04-fake-provider.md) — `cargo test`
führt beide Arten aus, meldet sie aber als getrennte Testläufe.

**Schritt 4** — Schreibe testweise den `ist_timeout`-Helfer mit dem `match`-Stil von oben in
`mein_core/src/error.rs`, führe `cargo clippy -p mein_core` aus, lies die
`match_like_matches_macro`-Warnung, korrigiere zu `matches!(fehler,
ProviderFehler::Timeout)`.

**Schritt 5** — `cargo clippy --workspace --all-targets -- -D warnings`. Läuft er sauber
durch (`Finished`, kein `error`), ist das Quality Gate erfüllt.

## Ausführung

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Alle drei sollten fehlerfrei durchlaufen — das ist ab jetzt unser fester Dreiklang vor jedem
Commit, formalisiert in der [Definition of Done in Lektion 8](08-release-3.md).

## Zusammenfassung

- Unit-Tests (`#[cfg(test)] mod tests`) sehen private Details, laufen als Teil derselben
  Crate. Integrationstests (`tests/*.rs`) sehen nur die öffentliche API, wie ein externer
  Aufrufer.
- `#[cfg(test)]` gilt **nicht** für Integrationstests — sie kompilieren `mein_core` als
  normale, externe Abhängigkeit. Ein `test-utils`-Feature-Flag (kombiniert mit `cfg(any(test,
  feature = "..."))`) ist ein etabliertes Muster, um Test-Doubles trotzdem gezielt
  freizugeben.
- Clippy prüft Stil, Komplexität und Performance jenseits reiner Compilerfehler.
- `-D warnings` macht Clippy-Warnungen zu Build-Fehlern — aus gutem Vorsatz wird ein
  erzwungenes Quality Gate.
- Der Timeout-Test aus [Lektion 4](04-fake-provider.md) ist jetzt zweifach abgesichert: als
  Unit-Test *und* als Integrationstest, der beweist, dass auch ein externer Aufrufer den
  Fehlerfall korrekt sieht.

## Übung

Aktiviere probeweise eine strengere Clippy-Lint-Gruppe: `cargo clippy --workspace
--all-targets -- -D warnings -W clippy::pedantic`. Diese Gruppe ist bewusst *nicht*
standardmäßig aktiv (viele Projekte finden sie zu streng für den Alltag) — lies dir drei
Warnungen an, die dabei in deinem bisherigen Phase-1-bis-3-Code auftauchen, und entscheide
für jede: Würdest du sie beheben, oder ist sie für dieses Projekt eher Geschmackssache?
Begründe deine Entscheidung in einem Kommentar. Es gibt hier keine "richtige" Antwort — die
Übung trainiert, Lint-Vorschläge kritisch zu bewerten statt sie blind zu übernehmen.

[Weiter: Lektion 6 — Golden Set und LLM-as-Judge](06-golden-set-llm-judge.md)
