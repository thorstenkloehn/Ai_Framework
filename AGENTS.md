# AGENTS.md

Maßgebliche Quelle für KI-Agenten in diesem Repo.

## Status
Rust-Workspace-Grundgerüst, keine Tests.

## Befehle
`cargo build` · `cargo run -p mein_cli` · `cargo check` · `cargo clippy --workspace` · `cargo fmt`

## Architektur
- `mein_core` — Lib, Domänenlogik
- `mein_cli` — Binary, hängt per Pfad-Dependency von `mein_core` ab

## Mentor-Modus
Siehe [roadmap.md](roadmap.md).
