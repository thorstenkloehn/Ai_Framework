# Canon-Dokument — NICHT Teil des veröffentlichten Buchs

Dies ist die interne Konsistenz-Referenz für alle, die an diesem mdBook-Kurs schreiben
(Mensch oder Agent). Sie legt fest, wie sich der Code von Phase zu Phase entwickelt, wie
Dinge heißen, und wie eine Lektion aufgebaut ist. Jede Lektion muss dazu passen, damit der
Code über alle Phasen hinweg wie AUS EINEM GUSS wirkt.

Quellen: `../KURS.md`, `../roadmap.md`, das echte Repo
`https://github.com/thorstenkloehn/Ai_Framework` (Stand: geklont nach `../ref_repo`).

## 0. Fakten aus dem echten Repo (Stand jetzt)

Workspace-Root `Cargo.toml`: `resolver = "2"`, members `mein_core`, `mein_cli`.

`mein_core/Cargo.toml`: `edition = "2024"`, dependency `serde = { version = "1.0", features
= ["derive"] }`.

`mein_core/src/lib.rs` (echter, bereits existierender Code — das ist unser Ausgangspunkt,
NICHT das cargo-Standardtemplate mehr):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Rolle {
    System,
    Benutzer,
    Assistent,
}
#[derive(Debug, Clone)]
pub struct Nachricht {
    pub rolle: Rolle,
    pub inhalt: String,
}
impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht {
            rolle,
            inhalt: inhalt.into(),
        }
    }
}
```

`mein_cli/Cargo.toml`: `edition = "2024"`, dependency `mein_core = { path = "../mein_core" }`.

`mein_cli/src/main.rs` (echt, aktueller Stand — bewusst noch ungeformatiert, das ist unser
allererstes Lernmaterial für rustfmt/clippy):

```rust
use::mein_core::{Nachricht,Rolle};
fn main() {
    let nachricht = Nachricht::neu(Rolle::Benutzer,"Hallo wie gehts");
    println!("{:?}",nachricht);
}
```

Wichtig für Lektion 1+2: Der echte Stand ist NICHT das leere `cargo new --lib`-Template
(das beschreibt nur die ältere `KURS.md`-Zusammenfassung). Er zeigt bereits `Rolle` und
`Nachricht`. Phase 1, Lektion 1 liest diesen Ist-Zustand und leitet daraus die nächsten
Schritte ab (Lektion 2 rekonstruiert, WARUM `Rolle`/`Nachricht` so aussehen, nicht WIE man
sie neu erfindet — wir erklären vorhandenen Code). Ab Lektion 3 bauen wir aktiv weiter aus.

`mein_core` enthält ein verschachteltes `.git` (kein Submodul) — in Lektion 1 als
Kuriosität erwähnen: vor `git add` entfernen (`rm -rf mein_core/.git`).

## 1. Namenskonvention (verbindlich für alle Phasen)

Domain-Begriffe (Chat/Konversation) bleiben **deutsch**, weil der echte Repo-Code das so
vormacht:

| Deutsch (Code)     | Bedeutung                                   |
|---------------------|----------------------------------------------|
| `Rolle`             | Enum: `System`, `Benutzer`, `Assistent`      |
| `Nachricht`          | Struct: `rolle: Rolle`, `inhalt: String`     |
| `Konversation`       | Struct: `Vec<Nachricht>` + Methoden          |
| `Konfiguration` | serde-Struct für API-Key, Modellname, Parameter (Phase 1 Lektion 5) — bewusst deutsch benannt, konsistent mit `Konversation`/`Kostenschaetzung` (Faustregel: Geschäfts-/Domänenbegriff, den auch Nicht-Programmierer verstehen sollen) |
| `anlegen`, `hinzufuegen`, `neu`, `mit_systemnachricht` | Methodennamen auf `Konversation`/`Nachricht` (`mit_systemnachricht` statt `system_nachricht` — Builder-artiger Name, der zeigt, dass eine Systemnachricht optional beim Erzeugen mitgesetzt wird) |

Architektur-/Pattern-Begriffe bleiben **englisch**, weil `roadmap.md` sie explizit so
nennt (Zitat: "LlmProvider-Trait", "Runnable-Trait (LangChain-Prinzip)", "Retriever"):

| Englisch (Code)     | Phase | Bedeutung |
|----------------------|-------|-----------|
| `LlmProvider`         | 3     | Trait, Port für Anbieter |
| `Runnable`            | 3     | Trait, Chain-Pattern |
| `Retriever`           | 5     | Trait, RAG-Abfrage |
| `VectorStore`         | 5     | Trait, Port für Embeddings-Speicher |
| `DocumentLoader`      | 5     | Trait |
| `Agent`, `Tool`, `AgentLoop` | 4 | zentrale Agenten-Begriffe |

Faustregel, wenn ein Begriff in keiner Tabelle steht: Ist es ein Chat-/Domänenbegriff, den
auch Nicht-Programmierer verstehen sollen → deutsch. Ist es ein Architektur-/
Rust-Fachbegriff, den man 1:1 in jeder Rust-Doku wiederfindet → englisch. Nie mischen
innerhalb eines Typs (kein `LlmAnbieter`, kein `RolleProvider`). Ausnahme, etabliert ab
Phase 3: `ProviderFehler` als Fehlertyp des `LlmProvider`-Ports — "Provider" gilt hier als
im Deutschen etabliertes Lehnwort für den Architekturbegriff, nicht als Übersetzung; alle
anderen domänenspezifischen Fehlertypen (`NachrichtFehler` etc.) bleiben rein deutsch.

## 2. Crate-/Modul-Layout über die Phasen

- Phase 1 Ende: Workspace `mein_core` (lib), `mein_cli` (bin). `mein_core::domain`
  (`rolle.rs`, `nachricht.rs`, `konversation.rs`), `mein_core::config`. `mein_cli` bekommt
  `main.rs` mit `clap`.
- Phase 2 Ende: `mein_core::provider` (Request/Response-Typen, `reqwest`-Client),
  `mein_core::error` (thiserror), `mein_core::prompt` (Templating), `mein_core::persistence`
  (sqlx-Skizze). `mein_cli` nutzt `anyhow` für Fehlerkontext.
- Phase 3 Ende: `mein_core::port::LlmProvider` (Trait), `mein_core::adapter::openai_kompatibel`
  (oder generischer Adapter), Ordnerstruktur nach Hexagonal: `domain/`, `port/`, `adapter/`.
  Test-Adapter `mein_core::adapter::fake` nur unter `#[cfg(test)]`. Neues Crate optional:
  `mein_eval` für Golden-Set/Judge — falls eingeführt, als eigenes Workspace-Mitglied.
- Phase 4 Ende: neues Crate `mein_agent` (oder Modul `mein_core::agent`, hier: eigenes
  Crate, da es `tokio` und Agent-spezifische Deps isoliert). Enthält `agent/loop.rs`,
  `agent/tool.rs`, `agent/state.rs`.
- Phase 5 Ende: neues Crate `mein_rag` (`loader.rs`, `chunking.rs`, `embedding.rs`,
  `retriever.rs`). Neues Binary/Crate `mein_server` (Axum) als Alternative/Ergänzung zu
  `mein_cli`. Querschnitt: `mein_core::telemetry` (tracing), `mein_core::secrets` (zeroize).
- Phase 6: keine neuen Crates, sondern `benches/` (criterion) und `tests/proptest_*.rs` in
  bestehenden Crates, plus `mein_core::routing`.
- Phase 7: `mein_core` wird zum öffentlichen Crate mit stabiler `pub`-Oberfläche,
  Builder (`ConversationBuilder`, `ProviderBuilder` o.ä.), Feature-Flags in `Cargo.toml`
  (`[features] rag = [...]`, `agent = [...]`).

Agents, die eine spätere Phase schreiben: Baut auf dem oben beschriebenen Zielbild der
VORHERIGEN Phase auf (lest den entsprechenden Abschnitt), auch wenn der tatsächliche Code
in `../ref_repo` diese spätere Phase noch nicht enthält — ihr schreibt vorausschauend, aber
konsistent zu diesem Dokument, nicht zum aktuellen (noch früheren) Repo-Stand.

## 3. Kapitelformat (jede Lektion, ausnahmslos)

Aus `roadmap.md`/`KURS.md` verbindlich vorgegeben:

1. **Problem** — konkrete Nutzer- oder Architektur-Anforderung, 1 Absatz, Alltagsmetapher
   erlaubt/erwünscht.
2. **Code (Zielbild)** — kurzer Blick auf das fertige Ergebnis dieser Lektion (Signatur
   oder kurzer Ausschnitt, NICHT die ganze Lösung).
3. **Dekonstruktion** — Typen, Module, Abhängigkeiten in Prosa einordnen: warum diese
   Struktur, welche Alternativen verworfen wurden.
4. **Schritt-Reveal** — die Lösung in kleinen, je für sich kompilierbaren Schritten. Jeder
   Schritt: kurzer Codeblock + 2-4 Sätze Erklärung + oft ein bewusst provozierter
   Compilerfehler samt Erklärung, was er bedeutet und wie man ihn liest.
5. **Ausführung** — konkrete Shell-Befehle (`cargo check`, `cargo test`, `cargo run -p
   mein_cli -- ...`) mit erwarteter Ausgabe.
6. **Zusammenfassung** — Stichpunkte: getroffene Entscheidungen, Trade-offs.
7. **Übung (Transferaufgabe)** — eigenständige Aufgabe, die das gelernte Konzept auf eine
   NEUE Anforderung überträgt (nicht nur Wiederholung). Keine Lösung angeben, aber 1-2
   Hinweise/Leitfragen.

Zusätzlich, wo passend: eine `> **💡 Tipp:**`-Box und/oder `> **⚠️ Warnung:**`-Box als
mdbook-Blockquote (kein Plugin verfügbar, daher reines Markdown, siehe Vorlage unten).

### Markdown-Vorlage für Boxen

```markdown
> **💡 Tipp**
>
> Text des Tipps.

> **⚠️ Warnung**
>
> Text der Warnung — typischer Rust- oder Architekturfehler an dieser Stelle.
```

## 4. Ton & Sprache

- Wir-Form durchgehend ("Wir modellieren...", "Wir kompilieren...").
- Englische Rust-/Architekturfachbegriffe werden beim ersten Vorkommen in einem Kapitel
  kurz auf Deutsch erklärt (z. B. "Ownership — wer im Programm gerade Besitzer eines
  Werts ist und ihn deshalb freigeben darf/muss").
- Zielgruppe: **absolute Programmier-Anfänger** (siehe Kapitel 0). Das heißt: In Phase 1
  keine unerklärten Begriffe wie "Heap", "Stack", "Trait", "Macro" ohne Kurz-Erklärung
  oder Verweis auf Kapitel 0/Glossar. Lieber einmal zu viel erklärt als vorausgesetzt.
- **Wichtigste Leitplanke (aus `AGENTS.md`/`roadmap.md`):** Der Kurs beschreibt Code zum
  SELBST-TIPPEN in VS Code, nicht zum Copy-Paste in eine echte Codebase. Die Lektionen
  dürfen vollständigen Code zeigen (das ist ja ein Buch, kein Agent, der Dateien
  schreibt) — aber der Fließtext soll die Leser*innen explizit auffordern, selbst zu
  tippen, zu kompilieren, Fehler auszuprobieren, bevor sie weiterlesen. Beispiel-Formulierung:
  "Tippe das jetzt selbst in `mein_core/src/domain/nachricht.rs`, bevor du weiterliest."
- rustfmt-konform formatierte Codebeispiele (Ausnahme: bewusst gezeigter unformatierter
  Ist-Zustand, z. B. das echte `main.rs` in Lektion 1).
- Bewusste Compilerfehler sind ein Kernstilmittel: Zeige den Fehler, zeige die
  Fehlermeldung wörtlich, erkläre sie Zeile für Zeile, zeige dann die Korrektur.

## 5. Datei-/Link-Konventionen

- Jede Phase hat ein `README.md` als Phasen-Übersichtsseite: Ziel der Phase (Zitat aus
  `roadmap.md`), Liste der Lektionen mit 1-Satz-Teaser, Verweis auf die Transferaufgabe
  der Phase, Link zum vorherigen Release-Tag.
- Querverweise zwischen Lektionen als relative mdbook-Links, z. B.
  `[Kapitel 0](../01-grundlagen/README.md)`.
- Jede Release-Lektion (letzte Lektion einer Phase) enthält: `git tag`-Vorschlag (Format
  aus KURS.md, z. B. `conversation-in-memory`), eine Checkliste "Definition of Done"
  (Zitat-Kriterium aus `roadmap.md`: kompiliert, mind. 1 Test, Fehlerpfad besprochen,
  Transferaufgabe hat eigene Lösungsidee), und einen kurzen Ausblick auf die nächste Phase.

## 6. Was NICHT tun

- Keine fertigen `.rs`-Dateien außerhalb der Buch-Seiten anlegen/committen — der Kurs
  beschreibt Code, er erzeugt kein Repo.
- Keine Abhängigkeit von einem mdbook-Plugin (kein `mdbook-admonish` etc.) — nur Standard-
  Markdown, damit `mdbook build` ohne weitere Installation läuft.
- Keine Versionsnummern für Crates erfinden, die nicht im echten Repo stehen oder aus dem
  Kontext ableitbar sind — bei Unsicherheit "aktuelle stabile Version, z. B. via `cargo add
  <crate>`" schreiben statt eine Zahl zu raten, die schnell veraltet.
