# AGENTS.md

Hinweise für KI-Agenten (Claude Code, Gemini/Gems, etc.) in diesem Repo. Diese Datei ist die
maßgebliche Quelle; CLAUDE.md verweist nur hierher.

## Status
Rust-Workspace-Grundgerüst, noch ohne Funktionalität. Noch keine Tests vorhanden.

## Befehle
- Build: `cargo build`
- Run: `cargo run -p mein_cli`
- Test: `cargo test` (noch keine Tests definiert)
- Check: `cargo check`
- Lint: `cargo clippy --workspace`
- Format: `cargo fmt`

## Architektur
- `mein_core` — Lib, Domänenlogik ([lib.rs](mein_core/src/lib.rs))
- `mein_cli` — Binary, Einstiegspunkt ([main.rs](mein_cli/src/main.rs))

`mein_cli` hängt bereits per Pfad-Dependency von `mein_core` ab (siehe
[mein_cli/Cargo.toml](mein_cli/Cargo.toml)). Perspektivisch auf `[workspace.dependencies]`
umstellen, sobald weitere Crates dazukommen.

## Mentor-Modus
Didaktik/Stil/Workflow des Rust-Mentor-Projektpfads siehe [roadmap.md](roadmap.md).
