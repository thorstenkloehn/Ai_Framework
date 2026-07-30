# Lektion 6 · Abschluss-Release: ai-framework-0.1.0

## Problem

Sieben Phasen, ein Workspace, viele einzelne Entscheidungen. Bevor wir den finalen Tag
setzen, fehlt noch ein Blick auf das große Ganze: Fügt sich alles, was wir über den ganzen
Kurs hinweg gebaut haben, wirklich zu **einem** kohärenten, veröffentlichbaren System
zusammen — oder haben wir sieben gute Einzelteile, die nie als Ganzes geprüft wurden? Das
ist die letzte technische Aufgabe dieses Kurses: der Integrationscheck über den gesamten
Workspace, bevor wir `ai-framework-0.1.0` taggen.

## Code (Zielbild)

Erinnerst du dich an das allererste, unformatierte `main.rs` aus
[Phase 1, Lektion 1](../02-phase1-fundament/01-workspace-lesen.md)?

```rust
use::mein_core::{Nachricht,Rolle};
fn main() {
    let nachricht = Nachricht::neu(Rolle::Benutzer,"Hallo wie gehts");
    println!("{:?}",nachricht);
}
```

Das hier ist, was aus derselben Grundidee nach sieben Phasen geworden ist — Code, den
jemand, der unser Crate nie hat entstehen sehen, einfach nur benutzt:

```rust
use mein_core::ClientBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ClientBuilder::neu()
        .modell("irgendein-modell")
        .api_key_aus_umgebung("MEIN_API_KEY")
        .bauen()?;

    let antwort = client.chat_text("Was ist Ownership in Rust?").await?;
    println!("{}", antwort);
    Ok(())
}
```

## Dekonstruktion

Sechs Zeilen produktiver Code. Dahinter liegen, unsichtbar für die Person, die diese sechs
Zeilen schreibt:

- Ein Domain-Modell (`Rolle`, `Nachricht`, `Konversation`), das ungültige Zustände so weit
  wie möglich unmöglich macht.
- Eine typisierte HTTP-Grenze mit sauberem Fehlerhandling statt gerateter JSON-Strukturen.
- Ein austauschbarer `LlmProvider`-Port hinter einer Hexagonal Architecture — der
  Anbieter dahinter könnte heute Anbieter A und morgen Anbieter B sein, ohne dass diese
  sechs Zeilen sich ändern.
- Ein Agent Loop mit Tools, Zustand und Abbruchbedingungen, falls `client` intern einen
  Agenten statt eines einfachen Chat-Aufrufs nutzt.
- RAG-Fähigkeiten, Retry-Logik, Tracing und Secret-Handling für den Betrieb.
- Vermessenes, orchestrierbares Verhalten mit Routing und Fallback-Strategien.
- Eine stabile, dokumentierte, versionierte öffentliche Schnittstelle, die genau diese
  sechs Zeilen so einfach wie möglich macht.

Das ist der eigentliche Sinn von Phase 7: Komplexität wird nicht entfernt, sie wird
**hinter einer guten API verborgen**. Die Person, die `ClientBuilder::neu()...` schreibt,
muss nichts von alldem wissen — genau wie wir es bei jedem guten Crate erwarten, das wir
selbst einbinden.

## Schritt-Reveal — Die Reise in sieben Stationen

**[Phase 1 — Fundament](../02-phase1-fundament/README.md).** Aus dem leeren
Cargo-Standardtemplate wurde ein echtes Domain-Modell: `Rolle`, `Nachricht`,
`Konversation`, eine erste Invariante ("nie leerer Inhalt"), `serde`-Konfiguration, eine
`clap`-CLI. Release: `conversation-in-memory`.

**[Phase 2 — Core & LLM-Anbindung](../03-phase2-llm-anbindung/README.md).** Aus der
Konversation im Speicher wurde ein echter API-Client: `reqwest`, typisierte
Request-/Response-Grenzen, `thiserror`/`anyhow`, Prompt-Templating, Structured Output,
eine Persistenzskizze mit `sqlx`. Release: `typed-provider-boundary`.

**[Phase 3 — Architektur & Qualitätssicherung](../04-phase3-architektur/README.md).**
`LlmProvider` wurde zum Port, Hexagonal Architecture trennte Domäne, Ports und Adapter,
ein Fake-Provider machte Tests unabhängig von echten API-Kosten, `clippy` wurde
Gewohnheit, ein Golden Set mit LLM-as-Judge machte Qualität messbar. Release:
`provider-agnostic-core`.

**[Phase 4 — Agenten & Concurrency](../05-phase4-agenten/README.md).** Mit `tokio` und
async/await lernte unser System, gleichzeitig auf Dinge zu warten, SSE-Streaming zu lesen,
Tools per Function Calling aufzurufen und in einem Agent Loop zu denken, zu handeln und zu
beobachten — mit klaren Abbruchbedingungen. Release: `tool-using-agent`.

**[Phase 5 — RAG, Deployment & Betrieb](../06-phase5-rag-betrieb/README.md).** Eigene
Dokumente wurden ladbar, zerlegbar, durchsuchbar und mit Quellenangaben in Prompts
einbindbar. `mein_server` brachte eine REST-Schicht, Retry und Backoff machten das System
robust gegen instabile Netzwerke, Tracing und Kosten-Tracking machten Betrieb
beobachtbar, `zeroize` schützte Geheimnisse, Prompt-Injection-Schutz behandelte
Retrieval-Inhalte als das, was sie sind: nicht vertrauenswürdige Daten. Release:
`operable-rag-service`.

**[Phase 6 — Performance & Orchestrierung](../07-phase6-performance/README.md).**
`criterion` und `proptest` ersetzten Bauchgefühl durch Messung und systematische
Randfall-Suche. Model-Routing wählte günstige Modelle für einfache Aufgaben mit
kontrolliertem Fallback, Multi-Agent-Orchestrierung teilte große Aufgaben in Agenten mit
klaren Verantwortungsgrenzen. Kein eigener Tag — reine Vermessung des Bestehenden.

**Phase 7 — Öffentliches Release (diese Phase).** Ein Builder Pattern machte
Konstruktion ergonomisch, Feature Flags machten Compile-Kosten wählbar, rustdoc-Doctests
machten Dokumentation selbstprüfend, SemVer gab uns eine Sprache für zukünftige
Änderungen, die crates.io-Checkliste bereitete die tatsächliche Veröffentlichung vor.

## Ausführung

Der finale Check über den gesamten Workspace, mit allen Features:

```bash
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
cargo publish -p mein_core --dry-run
```

```
Finished dev [unoptimized + debuginfo] target(s) in 12.4s
running 61 tests
test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
Finished dev [unoptimized + debuginfo] target(s) in 9.1s
Uploading mein_core v0.1.0 (dry run)
```

## Definition of Done — für den gesamten Kurs

- [ ] `cargo build --workspace --all-features` läuft ohne Fehler und ohne Warnungen.
- [ ] `cargo test --workspace --all-features` — alle Tests aus allen sieben Phasen sind
      grün, inklusive Doctests (`cargo test --doc`) und Property-Tests aus
      [Phase 6, Lektion 2](../07-phase6-performance/02-fuzzing-proptest.md).
- [ ] `cargo clippy --workspace --all-features` läuft ohne Beanstandungen.
- [ ] `cargo fmt --check` zeigt keine Abweichungen.
- [ ] `cargo publish --dry-run` läuft für `mein_core` (und jedes weitere zur
      Veröffentlichung vorgesehene Crate aus
      [Lektion 5](05-crates-io-checkliste.md)) ohne Fehler.
- [ ] Die Transferaufgabe aus [Lektion 2](02-feature-flags.md) (neue
      Provider-Integration ohne Änderung an bestehenden Nutzer-Codebeispielen) ist gelöst
      — prüfe das konkret, indem du das Zielbild-Beispiel dieser Lektion unverändert gegen
      deinen erweiterten `mein_core` kompilierst.
- [ ] `README.md` des Repositorys beschreibt das fertige Framework, nicht mehr den
      Ausgangszustand aus Phase 1.

## Der Release

```bash
cargo fmt
cargo clippy --workspace --all-features
cargo test --workspace --all-features
git add .
git commit -m "Phase 7: Öffentliche API mit Builder, Feature Flags, Doctests, SemVer-Vorbereitung"
git tag ai-framework-0.1.0
```

`0.1.0` ist bewusst keine `1.0.0`. Nach SemVer (siehe
[Lektion 4](04-semver-breaking-changes.md)) signalisiert eine `0.x`-Version: "Die API
funktioniert, wurde aber noch nicht durch echten, breiten Praxiseinsatz gehärtet." Das ist
ehrlich, nicht bescheiden — `1.0.0` zu setzen, bevor eine Schnittstelle sich in der Praxis
bewährt hat, ist einer der häufigsten Fehler in veröffentlichten Rust-Crates.

## Zusammenfassung — was wir wirklich gebaut haben

Wir sind nicht bei "Was ist Programmieren überhaupt?" ([Kapitel 0](../01-grundlagen/00-was-ist-programmieren.md))
stehengeblieben. Wir haben Ownership und Borrowing nicht nur verstanden, sondern an
echten Design-Entscheidungen angewendet — von der ersten `Nachricht`, die keinen leeren
Inhalt haben darf, bis zum letzten Trait-Objekt, das zwischen Threads wandern muss. Wir
haben gelernt, dass ein Compilerfehler kein Rückschlag ist, sondern eine der genauesten
Erklärungen, die man in irgendeiner Programmiersprache bekommen kann, warum etwas gerade
*nicht* passieren darf. Wir haben ein System gebaut, das mit echten LLMs spricht, Tools
aufruft, Dokumente durchsucht, sich selbst vermisst und am Ende so verpackt ist, dass
fremde Menschen es benutzen können, ohne je unseren Quellcode gesehen zu haben. Das ist
kein kleines Ergebnis für einen Kurs, der mit einem leeren `cargo new` begonnen hat.

## Übung

Nimm dir das Zielbild-Beispiel ganz oben in dieser Lektion und prüfe es tatsächlich gegen
deinen fertigen Workspace: Kompiliert es unverändert, nachdem du in
[Lektion 2](02-feature-flags.md) einen zweiten Provider-Adapter hinter einem eigenen
Feature ergänzt hast? Wenn ja, hast du die Transferaufgabe der Phase — und damit einen der
zentralen Werte eines guten öffentlichen Crates — tatsächlich eingelöst, nicht nur
behauptet.

## Was jetzt?

Der Kurs endet hier, das Framework nicht. Ein paar ehrliche nächste Schritte, falls du
weitermachen willst:

- **Zurück ins echte Repository.** Der ganze Kurs begleitet
  [github.com/thorstenkloehn/Ai_Framework](https://github.com/thorstenkloehn/Ai_Framework)
  — schau nach, wie sich das echte Projekt seit deinem letzten Blick weiterentwickelt hat,
  und vergleiche es mit deinem eigenen Stand.
- **Baue etwas Eigenes darauf.** Ein CLI-Tool für eine Aufgabe, die dich wirklich
  interessiert, eine kleine RAG-Anwendung über deine eigenen Notizen, ein zweiter,
  spezialisierter Agent. Die beste Art, eine API wirklich zu verstehen, ist, sie unter
  echtem eigenem Bedarf zu benutzen.
- **Trage etwas zurück bei.** Ein gefundener Bug, eine verbesserte Fehlermeldung, ein
  zusätzlicher Adapter — echte Open-Source-Beiträge beginnen oft klein.
- **Vertiefe, was dich am meisten gepackt hat.** Vielleicht war es Ownership, vielleicht
  async/await, vielleicht Prompt-Engineering, vielleicht Systemarchitektur. Der
  [Anhang](../09-anhang/README.md) — Glossar, Rust-Spickzettel, Fehlermeldungen
  verstehen, weiterführende Ressourcen — ist ein guter Ort, um offene Fragen zu klären
  oder tiefer einzusteigen.

Du hast ein KI-Framework in Rust gebaut, Zeile für Zeile, Fehler für Fehler, Test für
Test. Das war kein Copy-Paste-Kurs. Gut gemacht.

[Weiter: Anhang](../09-anhang/README.md)
