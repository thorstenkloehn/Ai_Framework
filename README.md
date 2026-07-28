# Ai_Framework

Projekt-Übersicht: Ziel, Roadmap-Phasen und aktueller Stand dieses Repos. Details zu
Didaktik/Workflow des Mentor-Modus stehen in roadmap.md, Hinweise für Claude Code in
CLAUDE.md.

# KI-Framework Praxis-Projektpfad — Fahrplan
Ziel: eigenständiges Rust-Crate/KI-Framework (LLM, Agenten, RAG) als Dependency für andere
Rust-Apps — wie LangChain/Spring AI, aber Rust.

## Phase 1 — Fundament
Workspace (core/cli/server), Config via serde, CLI (clap), Message/Role-Typen,
Konversationsverlauf als Vec<Message>.

## Phase 2 — Core & LLM-Anbindung
API-Client (reqwest + serde_json), Request/Response-Typen, Fehler mit thiserror + anyhow,
Prompt-Templating, Structured Output (schemars), Persistenz (sqlx).

## Phase 3 — Architektur & QS
LlmProvider-Trait, Hexagonal-Architektur (Ports & Adapters), dyn Trait, Unit-/Integrationstests
+ clippy, Eval (Golden-Set, LLM-as-Judge), Chain-Pattern (Runnable-Trait, LangChain-Prinzip).

## Phase 4 — Agent & Concurrency
SSE-Streaming, Tool-Use/Function-Calling, Agenten-Loop (Denken→Tool→Beobachtung), Gedächtnis/
State, tokio, optional MCP-Client.

## Phase 5 — RAG, Deployment & Betrieb
RAG (qdrant/lancedb, Document-Loader, Chunking, Retriever), REST (axum) oder TUI,
Rate-Limit/Retry, Tracing + Kosten-Tracking, zeroize, Prompt-Injection-Schutz, Docker/CI.

## Phase 6 — Experte: Performance
Benchmarking (criterion), Fuzzing (proptest), Model-Routing/Fallback, Multi-Agent-
Orchestrierung.

## Phase 7 — Release
Öffentliches API-Design (Builder-Pattern), Feature-Flags, rustdoc + Beispiele, SemVer,
crates.io-Publishing, Contribution-Guidelines.

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
