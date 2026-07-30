# Lektion 2: Hexagonal Architecture

## Problem

`mein_core/src/lib.rs` ist inzwischen ziemlich voll: `Rolle`, `Nachricht`,
`Konversation`, `Konfiguration`, seit [Lektion 1](01-llmprovider-port.md) das
`LlmProvider`-Trait, dazu der konkrete `OpenAiKompatiblerClient`. Bisher lag das alles
mehr oder weniger nebeneinander in derselben Datei oder in flachen Modulen wie `provider.rs`
und `error.rs`. Das funktioniert bei einem kleinen Projekt — aber schon jetzt lässt sich der
Code nicht mehr auf einen Blick erfassen, und in [Phase 4](../05-phase4-agenten/README.md)
und [Phase 5](../06-phase5-rag-betrieb/README.md) kommen weitere Ports (`Retriever`,
`VectorStore`) und weitere Adapter dazu. Wir brauchen eine Ordnerstruktur, die die
Architektur **sichtbar macht**, statt sie nur im Kopf zu behalten.

Die Antwort aus der Softwarearchitektur heißt **Hexagonal Architecture** (auch *Ports and
Adapters Architecture*, 2005 von Alistair Cockburn beschrieben). Die Grundidee: Zeichne dein
System als Sechseck. In der Mitte sitzt die **Domäne** — die fachliche Logik, die nichts von
der Außenwelt weiß. An den Kanten sitzen **Ports** — Verträge, die beschreiben, was die
Domäne von außen braucht oder anbietet. Außerhalb der Kanten sitzen **Adapter** — konkrete
Technik (HTTP, Datenbanken, Dateisystem), die diese Verträge erfüllt. Die Domäne kennt keine
Adapter, nur Ports. Ob "außen" ein echter API-Aufruf oder ein Test-Double steckt, ist der
Domäne egal.

## Code (Zielbild)

```text
mein_core/src/
├── lib.rs               // Modul-Deklarationen + Re-Exports für Abwärtskompatibilität
├── domain.rs
├── domain/
│   ├── rolle.rs
│   ├── nachricht.rs
│   └── konversation.rs
├── port.rs               // LlmProvider, ChatAnfrage, ChatAntwort (aus Lektion 1)
├── adapter.rs
├── adapter/
│   └── openai_kompatibel.rs
├── config.rs
├── error.rs
├── prompt.rs
└── persistence.rs
```

## Dekonstruktion

### Warum drei Ordner statt einer flachen Datei?

- **`domain/`** — `Rolle`, `Nachricht`, `Konversation`: reine Fachlogik. Kein `reqwest`,
  kein `sqlx`, keine Zeile, die weiß, dass es das Internet gibt. Wenn du dir unsicher bist,
  ob etwas in die Domäne gehört, frag: "Würde dieser Code auch existieren, wenn wir nie mit
  einem LLM sprechen würden, sondern nur Nachrichten im Speicher verwalten?" Bei `Rolle`,
  `Nachricht`, `Konversation` lautet die Antwort ja.
- **`port.rs`** — `LlmProvider`: der Vertrag. Er beschreibt, *was* die Domäne von einem
  Anbieter braucht ("kann chatten"), nicht *wie* das technisch passiert. `ChatAnfrage` und
  `ChatAntwort` gehören ebenfalls hierher, nicht in die Domäne und nicht in den Adapter —
  sie sind die *Sprache des Vertrags*: Jeder Adapter muss sie verstehen, aber sie legen sich
  auf keine bestimmte Technik fest (kein `reqwest`-Typ taucht in ihrer Definition auf).
- **`adapter/`** — `openai_kompatibel.rs`: die konkrete Technik. Hier und nur hier taucht
  `reqwest` auf. Ändert sich die HTTP-Bibliothek, oder kommt ein zweiter Anbieter dazu,
  ändert sich nur dieser Ordner — `domain/` und `port.rs` bleiben unberührt.

Die Faustregel für die Abhängigkeitsrichtung: **`adapter` hängt von `port` ab, `port` hängt
höchstens von `domain` ab, `domain` hängt von nichts ab.** Nie umgekehrt. Diese Regel ist in
Rust nicht durch den Compiler erzwungen (anders als z. B. Sichtbarkeit) — sie ist eine
Design-Disziplin, die du beim Schreiben von `use`-Anweisungen im Kopf behältst. Ein `use
reqwest::...;` in `domain/nachricht.rs` wäre ein Alarmsignal.

> **💡 Tipp**
>
> Der Name "Hexagonal" (sechseckig) kommt daher, dass Cockburns ursprüngliche Zeichnung ein
> Sechseck mit mehreren Kanten für mehrere Ports zeigte — die Zahl sechs selbst hat keine
> tiefere Bedeutung, ein System kann zwei oder zehn Ports haben. Gebräuchlicher als
> "hexagonal" ist im Alltag oft einfach **Ports and Adapters**, derselbe Ansatz.

### Modul-Dateien ohne `mod.rs`

Seit der Rust-2018-Edition (unser Workspace nutzt bereits Edition 2024, siehe
[Lektion 1 von Phase 1](../02-phase1-fundament/01-workspace-lesen.md)) muss ein Ordner mit
Untermodulen **nicht** zwingend eine `mod.rs`-Datei enthalten. Es reicht eine Datei mit dem
Namen des Ordners direkt daneben — `domain.rs` neben `domain/` — die die Untermodule
deklariert:

```rust
// mein_core/src/domain.rs
pub mod konversation;
pub mod nachricht;
pub mod rolle;

pub use konversation::Konversation;
pub use nachricht::{Nachricht, NachrichtFehler};
pub use rolle::Rolle;
```

Die drei `pub use`-Zeilen am Ende sind **Re-Exports**: Sie machen `Nachricht` zusätzlich
unter dem kürzeren Pfad `mein_core::domain::Nachricht` *und* — kombiniert mit einem weiteren
Re-Export in `lib.rs` gleich — sogar weiterhin unter `mein_core::Nachricht` verfügbar, genau
wie in Phase 1 und 2. Ohne diesen Kniff müsste jede Stelle im Code, die bisher `use
mein_core::{Nachricht, Rolle};` schreibt, zu `use mein_core::domain::{Nachricht, Rolle};`
geändert werden — eine Umbenennung, die niemandem etwas nützt und nur Fleißarbeit in
`mein_cli` erzeugt.

### `lib.rs` als Übersicht

```rust
pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod persistence;
pub mod port;
pub mod prompt;

pub use domain::{Konversation, Nachricht, NachrichtFehler, Rolle};
pub use port::{ChatAnfrage, ChatAntwort, LlmProvider};
```

`lib.rs` wird dadurch zu dem, was es in einer guten Crate sein sollte: eine Landkarte, kein
Ort für Implementierung. Wer die Datei öffnet, sieht sofort, aus welchen Bereichen
`mein_core` besteht, ohne eine einzige Codezeile Fachlogik lesen zu müssen.

## Schritt-Reveal

**Schritt 1** — Lege `mein_core/src/domain.rs` mit den drei `pub mod`- und drei `pub
use`-Zeilen von oben an. Verschiebe den Inhalt der bisherigen `Rolle`/`Nachricht`/
`Konversation`-Definitionen aus `lib.rs` in `domain/rolle.rs`, `domain/nachricht.rs`,
`domain/konversation.rs` (je eine Datei pro Typ — orientiere dich an den Typdefinitionen aus
[Phase 1](../02-phase1-fundament/README.md)).

**Schritt 2** — Entferne die alten Typdefinitionen aus `lib.rs`, ersetze sie durch `pub mod
domain;` und `pub use domain::{Konversation, Nachricht, NachrichtFehler, Rolle};`.

`cargo check -p mein_core`:

```
error[E0412]: cannot find type `Nachricht` in this scope
  --> src/port.rs:2:29
   |
 2 | use crate::Nachricht;
   |             ^^^^^^^^ not found in `crate`
```

Das ist der Compiler, der uns exakt zeigt, wo unser Refactoring noch unvollständig ist:
`port.rs` verweist über `crate::Nachricht` auf den alten, jetzt nicht mehr existierenden
Pfad. Diese Art Fehler ist bei Umbauten üblich und harmlos — der Compiler findet **jede**
Stelle für dich, du musst sie nur der Reihe nach abarbeiten.

**Schritt 3** — Korrigiere den Import in `port.rs` auf `use crate::domain::Nachricht;` (oder
nutze das Re-Export: `use crate::Nachricht;` funktioniert nach Schritt 2 auch wieder, sobald
`pub use domain::{...}` in `lib.rs` steht — beide Schreibweisen sind ab jetzt gültig, das
ist gerade der Sinn des Re-Exports).

**Schritt 4** — Wandle `adapter.rs` + `adapter/openai_kompatibel.rs` analog um: Verschiebe
`OpenAiKompatiblerClient` und `impl LlmProvider for OpenAiKompatiblerClient` aus
[Lektion 1](01-llmprovider-port.md) dorthin.

```rust
// mein_core/src/adapter.rs
pub mod openai_kompatibel;
```

```rust
// mein_core/src/adapter/openai_kompatibel.rs
use crate::error::ProviderFehler;
use crate::port::{ChatAnfrage, ChatAntwort, LlmProvider};

pub struct OpenAiKompatiblerClient {
    client: reqwest::blocking::Client,
    api_key: String,
    basis_url: String,
}

impl OpenAiKompatiblerClient {
    pub fn anfrage_senden(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        // ... unverändert aus Phase 2 ...
        # unimplemented!()
    }
}

impl LlmProvider for OpenAiKompatiblerClient {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        self.anfrage_senden(anfrage)
    }
}
```

Ergänze `pub mod adapter;` in `lib.rs`.

**Schritt 5** — `cargo check -p mein_core` und `cargo check -p mein_cli`, bis beide sauber
durchlaufen. Wenn `mein_cli` weiterhin `use mein_core::{Konversation, Rolle};` schreibt
(wie in [Phase 1, Lektion 6](../02-phase1-fundament/06-cli-mit-clap.md)) und trotzdem
kompiliert, hast du die Re-Exports richtig gesetzt.

> **⚠️ Warnung**
>
> Ein leicht zu übersehender Fallstrick beim Verschieben von Code zwischen Dateien:
> `#[cfg(test)] mod tests { ... }`-Blöcke wandern mit dem Typ, den sie testen, mit. Ein Test
> für `Nachricht::neu`, der bisher unten in `lib.rs` stand, gehört jetzt ans Ende von
> `domain/nachricht.rs` — sonst verlierst du beim Aufräumen unbemerkt Testabdeckung.

## Ausführung

```bash
cargo check --workspace
cargo test --workspace
```

Alle bisherigen Tests aus Phase 1 und 2 sollten unverändert grün bleiben — Hexagonal
Architecture ist ein reines Umbau-Refactoring dieser Lektion, keine Verhaltensänderung.

```bash
cargo fmt --check
```

## Zusammenfassung

- Hexagonal Architecture trennt **Domäne** (fachliche Logik, keine Außenwelt-Kenntnis),
  **Port** (Vertrag, was die Domäne von außen braucht) und **Adapter** (konkrete Technik,
  die den Vertrag erfüllt).
- Abhängigkeitsrichtung: Adapter → Port → Domäne, nie umgekehrt.
- Ordner ohne `mod.rs`: eine Datei `name.rs` neben einem Ordner `name/` deklariert dessen
  Untermodule (Edition 2018+, auch in unserer Edition 2024).
- `pub use` am Modul- und Crate-Root hält bestehende, kürzere Importpfade
  (`mein_core::Nachricht`) am Leben, obwohl die Datei sich verschoben hat.
- Ein Refactoring wie dieses lässt sich Schritt für Schritt vom Compiler führen: Jeder
  E0412-Fehler zeigt eine Stelle, die noch angepasst werden muss.

## Übung

[Phase 5](../06-phase5-rag-betrieb/README.md) wird einen `Retriever`-Port und einen
`VectorStore`-Port einführen (siehe die Namenskonvention: Architektur-Begriffe bleiben
englisch). Skizziere — nur als Kommentar, ohne echten Code — wo in dieser Ordnerstruktur die
Dateien für einen zukünftigen `Retriever`-Port und einen `InMemoryVectorStore`-Adapter
liegen würden. Überlege dir auch: Müsste die `domain/`-Schicht dafür angepasst werden, oder
reicht es, `port/` und `adapter/` zu erweitern? Diese Übung hat keine Musterlösung im Code —
sie trainiert, die Architektur-Entscheidung selbst zu treffen, bevor du sie in Phase 5
wirklich brauchst.

[Weiter: Lektion 3 — dyn Trait und Ownership an der Grenze](03-dyn-trait-ownership.md)
