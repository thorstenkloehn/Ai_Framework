# Ai_Framework

## Über dieses Dokument
Diese README ist die zentrale Projekt-Übersicht des Repos und beantwortet: Was wird
gebaut, in welchen Schritten, und wo steht das Projekt gerade. Sie enthält:
- **Ziel** — was das Framework können soll und wofür (Abschnitt "Fahrplan").
- **Phase 1–7** — die geplanten Ausbaustufen von Workspace-Grundgerüst bis
  crates.io-Release.
- **Agiler Zyklus** — der Prozessrahmen, den jede einzelne Aufgabe durchläuft.
- **Mentor-Modus** — Kurzhinweis, dass dies ein Lernprojekt mit eigenem Workflow ist
  (Details in roadmap.md).
- **Ist-Zustand** — was im Repo tatsächlich schon existiert vs. was noch fehlt.

Verwandte Dateien: roadmap.md (Didaktik/Stil/Workflow des Mentor-Modus im Detail),
CLAUDE.md (Hinweise für Claude Code: Befehle, Architektur, Repo-Eigenheiten).

# KI-Framework Praxis-Projektpfad — Fahrplan
Ziel: eigenständiges Rust-Crate/KI-Framework (LLM, Agenten, RAG) als Dependency für andere
Rust-Apps — wie LangChain/Spring AI, aber Rust.

## Phase 1 — Fundament
Ziel: lauffähiges Grundgerüst, das eine einfache Konversation im Speicher abbildet.
- **Workspace (core/cli/server)** — Cargo-Workspace mit mehreren Crates statt einem
  Monolithen: Domänenlogik (`core`), Kommandozeile (`cli`) und späterer Server (`server`)
  bleiben getrennt.
- **Config via serde** — Konfiguration (API-Keys, Modellnamen, Parameter) aus Datei/Env
  deserialisieren statt hartkodiert.
- **CLI (clap)** — Kommandozeilen-Parsing für Subcommands und Flags.
- **Message/Role-Typen** — Datentyp für eine einzelne Chat-Nachricht (Rolle
  System/User/Assistant + Inhalt).
- **Konversationsverlauf als Vec<Message>** — Verlauf als einfache Liste im Speicher,
  Grundlage für spätere Persistenz.

## Phase 2 — Core & LLM-Anbindung
Ziel: echte Anfragen an ein LLM schicken und Antworten typisiert verarbeiten.
- **API-Client (reqwest + serde_json)** — HTTP-Client für die LLM-API inkl. JSON-
  (De-)Serialisierung.
- **Request/Response-Typen** — typisierte Strukturen statt rohem JSON, damit Fehler beim
  Kompilieren auffallen statt erst zur Laufzeit.
- **Fehler mit thiserror + anyhow** — eigene Fehlertypen in der Bibliothek (thiserror),
  pragmatische Fehlerweitergabe in der Anwendung (anyhow).
- **Prompt-Templating** — Prompts aus Vorlagen mit Variablen zusammensetzen statt per
  String-Concatenation.
- **Structured Output (schemars)** — JSON-Schema aus Rust-Typen generieren, damit das LLM
  strukturierte statt freie Antworten liefert.
- **Persistenz (sqlx)** — Konversationen/Ergebnisse in einer Datenbank ablegen statt nur im
  Speicher zu halten.

## Phase 3 — Architektur & QS
Ziel: Code austauschbar und testbar machen, Qualität systematisch absichern.
- **LlmProvider-Trait** — Abstraktion über konkrete LLM-Anbieter, damit sich der Anbieter
  wechseln lässt, ohne Aufrufer-Code zu ändern.
- **Hexagonal-Architektur (Ports & Adapters)** — Domänenlogik von externen Systemen (API,
  DB) entkoppeln.
- **dyn Trait** — Laufzeit-Polymorphismus für austauschbare Implementierungen (z. B.
  mehrere Provider gleichzeitig).
- **Unit-/Integrationstests + clippy** — automatisierte Tests und Linting als Qualitäts-
  Gate vor jedem Merge.
- **Eval (Golden-Set, LLM-as-Judge)** — Antwortqualität systematisch gegen Referenzfälle
  prüfen, teils durch ein weiteres LLM bewertet.
- **Chain-Pattern (Runnable-Trait, LangChain-Prinzip)** — verkettbare Verarbeitungsschritte
  (Prompt → LLM → Parser) nach dem Vorbild von LangChains "Runnables".

## Phase 4 — Agent & Concurrency
Ziel: das LLM eigenständig handeln lassen (Tools aufrufen, mehrschrittig planen),
nebenläufig.
- **SSE-Streaming** — Antworten token-weise streamen (Server-Sent Events) statt auf die
  komplette Antwort zu warten.
- **Tool-Use/Function-Calling** — dem LLM erlauben, definierte Funktionen/Tools
  aufzurufen.
- **Agenten-Loop (Denken→Tool→Beobachtung)** — Schleife, in der das LLM plant, ein Tool
  aufruft, das Ergebnis beobachtet und weiterplant.
- **Gedächtnis/State** — Zustand, der über mehrere Agentenschritte/-läufe hinweg erhalten
  bleibt.
- **tokio** — asynchrone Laufzeit für nebenläufige I/O (API-Calls, Tool-Aufrufe parallel).
- **optional MCP-Client** — Anbindung an Tools/Server über das Model Context Protocol.

## Phase 5 — RAG, Deployment & Betrieb
Ziel: eigene Daten einbinden (Retrieval-Augmented Generation) und das Framework
betriebsfähig machen.
- **RAG (qdrant/lancedb, Document-Loader, Chunking, Retriever)** — Dokumente laden, in
  Abschnitte zerlegen, in einer Vektordatenbank ablegen und zur Anfragezeit relevante
  Abschnitte abrufen.
- **REST (axum) oder TUI** — Framework über eine HTTP-API oder ein Terminal-UI nutzbar
  machen.
- **Rate-Limit/Retry** — Anfragen drosseln und bei transienten Fehlern automatisch
  wiederholen.
- **Tracing + Kosten-Tracking** — strukturiertes Logging und Nachverfolgung von
  Token-/API-Kosten.
- **zeroize** — sensible Daten (Keys, Secrets) beim Verwerfen aktiv aus dem Speicher
  löschen.
- **Prompt-Injection-Schutz** — Eingaben und Retrieval-Inhalte gegen Manipulationsversuche
  absichern.
- **Docker/CI** — containerisiertes Deployment und automatisierte Build/Test-Pipeline.

## Phase 6 — Experte: Performance
Ziel: Performance messen und verbessern, mehrere Agenten koordinieren.
- **Benchmarking (criterion)** — Performance-Regressionen systematisch messen.
- **Fuzzing (proptest)** — Eigenschaften mit zufällig generierten Eingaben testen.
- **Model-Routing/Fallback** — je nach Anfrage unterschiedliche Modelle wählen, bei Ausfall
  auf Alternativen ausweichen.
- **Multi-Agent-Orchestrierung** — mehrere spezialisierte Agenten koordiniert
  zusammenarbeiten lassen.

## Phase 7 — Release
Ziel: das Framework als öffentliches Crate veröffentlichen.
- **Öffentliches API-Design (Builder-Pattern)** — stabile, ergonomische öffentliche
  Schnittstelle.
- **Feature-Flags** — optionale Funktionalität über Cargo-Features zu-/abschaltbar
  machen.
- **rustdoc + Beispiele** — Dokumentation und Beispielcode für Nutzer der Bibliothek.
- **SemVer** — Versionierung nach Semantic Versioning, damit Breaking Changes erkennbar
  sind.
- **crates.io-Publishing** — Veröffentlichung auf dem offiziellen Rust-Package-Registry.
- **Contribution-Guidelines** — Richtlinien für externe Beiträge.

## Agiler Zyklus
Planung → Analyse → Entwurf → Implementierung → Test → Deployment → Betrieb → Wartung →
Review → Dokumentation.

## Mentor-Modus
Lernprojekt: Didaktik/Stil/Workflow des Rust-Mentor-Projektpfads siehe roadmap.md.
Code wird im Chat besprochen und selbst getippt, nicht automatisch eingefügt — erst
verstehen und testen, dann Git-Release.

## Ist-Zustand
Rust-Workspace-Grundgerüst (Cargo.toml, resolver "2", edition 2024), noch ohne
Domänenlogik:
- `crates/core` und `crates/cli` enthalten noch das `cargo new --lib`-Standardtemplate
  (`add(left, right)` + ein Test) — keine Message/Role-Typen, keine Config, kein CLI-
  Parsing.
- `crates/cli` soll laut Architektur ein Binary mit `main.rs` werden (`cli` soll künftig
  von `core` abhängen); aktuell existiert dort nur ein `lib.rs`.
- `crates/core` enthält ein verschachteltes `.git` (kein Submodul) → erscheint bei
  `git status` als unversioniert. Vor jedem `git add` entfernen.

Nächster sinnvoller Schritt (Phase 1): Message/Role-Typen und `Vec<Message>`-
Konversationsverlauf in `core`, danach `clap`-basierte CLI (`main.rs`) in `cli`.
