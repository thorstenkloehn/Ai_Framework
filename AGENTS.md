# AGENTS.md

Maßgebliche Quelle für KI-Agenten (Antigravity CLI, Codex, Claude Code, opencode). GitHub Copilot nutzt eine eigene, in sich geschlossene `.github/copilot-instructions.md`.

## Status
Rust-Workspace-Grundgerüst, keine Tests.

## Befehle
`cargo build` · `cargo run -p mein_cli` · `cargo check` · `cargo clippy --workspace` · `cargo fmt`

## Architektur
- `mein_core` — Lib, Domänenlogik
- `mein_cli` — Binary, hängt per Pfad-Dependency von `mein_core` ab

## Tabu
- `kurs/` — nicht lesen/bearbeiten, außer `mdbook build`
- `google/` — nicht lesen/bearbeiten

## Mentor-Modus
Siehe [roadmap.md](roadmap.md).
