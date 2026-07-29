# copilot-instructions.md

Hinweise für GitHub Copilot in diesem Repo.

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
- `mein_core` — Lib, Domänenlogik (`mein_core/src/lib.rs`)
- `mein_cli` — Binary, Einstiegspunkt (`mein_cli/src/main.rs`)

`mein_cli` hängt bereits per Pfad-Dependency von `mein_core` ab (siehe
`mein_cli/Cargo.toml`). Perspektivisch auf `[workspace.dependencies]` umstellen, sobald
weitere Crates dazukommen.

## Mentor-Modus
Rust-Mentor-Projektpfad: Schritt für Schritt, nie fertiges Programm auf einmal, Phasen
bauen aufeinander auf. Clean Code, Design Patterns, 80% Praxis/20% Theorie, Fehler als
Lernchance, progressive Komplexität. Wir-Form, Code-Build-Explain, bewusste
Compilerfehler, rustfmt-konform. Details siehe `roadmap.md` im Repo-Root.
