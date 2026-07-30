# Lektion 8 · Release 2: typed-provider-boundary

## Rückblick

Aus einer `Konversation`, die nur im Speicher lebte, ist in sieben Lektionen eine echte
Verbindung nach außen geworden:

- `mein_core::provider` mit `OpenAiKompatiblerClient` — ein konkreter HTTP-Client
  (bewusst noch **kein** Trait, kein `dyn Trait`), der `Konversation` über eine
  OpenAI-kompatible Chat-API verschickt und typisiert wieder einliest.
- Eigene Request-/Response-Typen (`ChatRequest`, `ChatResponse`, `ChatNachricht`, ...),
  strikt getrennt von `Rolle`/`Nachricht`/`Konversation` — die Domäne bleibt sauber,
  egal wie das externe JSON-Format aussieht.
- `mein_core::error` mit `ProviderFehler` (`thiserror`) — typisierte, gezielt
  behandelbare Fehler für Netzwerk, Format und API-Statuscodes.
- `mein_cli` nutzt `anyhow` für menschenlesbare Fehlerketten mit Kontext, statt jeden
  Fehlertyp einzeln zu behandeln.
- `mein_core::prompt` mit `PromptTemplate` — Variablen in Prompts werden **vor** jedem
  Netzwerkaufruf vollständig validiert.
- Structured Output über `schemars`: LLM-Antworten lassen sich als typisiertes JSON
  statt als freier Text einlesen.
- `mein_core::persistence` mit `KonversationsSpeicher` — eine Skizze, wie eine
  `Konversation` eine Datenbank überleben könnte (als async-Vorgriff auf Phase 4).

## Definition of Done

- [ ] `cargo build` läuft ohne Fehler und ohne Warnungen.
- [ ] `cargo test` — alle Tests aus Lektion 2 bis 7 sind grün, inklusive der
      `#[tokio::test]`-Tests aus Lektion 7.
- [ ] `cargo clippy --workspace` läuft ohne Beanstandungen.
- [ ] `cargo fmt --check` zeigt keine Abweichungen.
- [ ] Der Fehlerpfad wurde mindestens einmal bewusst ausgeführt: ein `ProviderFehler`
      (z. B. `ApiFehler` gegen einen absichtlich falschen Endpunkt, siehe
      [Lektion 4](04-fehlerbehandlung.md)) *und* ein `PromptFehler`
      (fehlende Variable, siehe [Lektion 5](05-prompt-templating.md)).
- [ ] **Transferaufgabe der Phase gelöst und verifiziert:** Eine ungültige
      Prompt-Variable erscheint als verständlicher Fehler, **bevor** irgendein
      Netzwerkaufruf stattfindet — konkret bearbeitet in
      [Lektion 5](05-prompt-templating.md), verdrahtet mit `mein_cli` in deren Übung.
      Prüfe es noch einmal explizit: Rufe `mein_cli` mit einer absichtlich fehlenden
      Prompt-Variable auf, idealerweise ganz ohne Internetverbindung — die Fehlermeldung
      muss trotzdem sofort erscheinen.
- [ ] `mein_core::provider`, `mein_core::error`, `mein_core::prompt`,
      `mein_core::persistence` existieren als eigene Module (`mod`/`pub mod`), keine
      Vermischung mehr in einer einzigen `lib.rs`.
- [ ] Keine `reqwest`-, `sqlx`- oder `serde_json::Value`-Typen tauchen in
      `mein_cli/src/main.rs` auf — `mein_cli` kennt weiterhin nur die öffentliche API
      von `mein_core`.

## Aufräumen vor dem Commit

```bash
cargo fmt
cargo clippy --workspace
cargo test
```

> **⚠️ Warnung**
>
> Prüfe vor dem Commit, dass kein echter API-Key irgendwo im Code oder in einer
> versehentlich getrackten `.env`-Datei landet. Nutze `.gitignore` für lokale
> Umgebungsdateien und lies API-Keys ausschließlich über Umgebungsvariablen
> (`std::env::var(...)`, wie in [Lektion 4](04-fehlerbehandlung.md) gezeigt).

## Der Release

```bash
git add .
git commit -m "Phase 2: typisierte HTTP-Grenze zum LLM-Anbieter, Fehlerbehandlung, Prompt-Templating, Structured Output, Persistenz-Skizze"
git tag typed-provider-boundary
```

Der Tag-Name `typed-provider-boundary` beschreibt wieder das fachliche Ergebnis, nicht
die einzelnen Technologien — konsistent mit `conversation-in-memory` aus
[Phase 1](../02-phase1-fundament/07-release-1.md).

## Ausblick auf Phase 3

Ein wichtiger Punkt, der dir vielleicht schon während dieser Phase aufgefallen ist:
`OpenAiKompatiblerClient` ist der **einzige** Provider, den `mein_cli` kennt. Es gibt
kein Trait, keine Möglichkeit, ihn im Test gegen einen Fake auszutauschen, keine
Möglichkeit, zur Laufzeit zwischen zwei Anbietern zu wechseln. Das war die ganze Phase
über eine bewusste Entscheidung: Wir haben zuerst verstanden, *was* ein Provider
überhaupt braucht (typisierte Requests, typisierte Responses, typisierte Fehler,
Prompt-Validierung), bevor wir uns Gedanken über Austauschbarkeit machen.

[Phase 3](../04-phase3-architektur/README.md) holt genau das nach: Wir ziehen ein
`trait LlmProvider` als **Port** (in
[Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md)) aus dem, was wir
hier gebaut haben, richten das Projekt nach **Hexagonal Architecture** aus
(`domain/`, `port/`, `adapter/`), lernen `dyn Trait` und Ownership an dieser neuen
Grenze bewusst kennen, und bauen einen `Fake`-Provider ausschließlich für Tests. Die gute
Nachricht: `ChatRequest`, `ChatResponse`, `ProviderFehler`, `PromptTemplate` und
`KonversationsSpeicher` bleiben inhaltlich fast unverändert — Phase 3 zieht eine
Abstraktionsschicht **um** unseren bestehenden Code, sie schreibt ihn nicht neu. Auch das
ist ein Beweis dafür, dass sich die Sorgfalt dieser Phase auszahlt, genau wie schon beim
Übergang von Phase 1 zu Phase 2.

[Weiter: Phase 3 — Architektur & Qualitätssicherung](../04-phase3-architektur/README.md)
