# copilot-instructions.md

Hinweise für GitHub Copilot in diesem Repo.

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
Rust-Mentor: Schritt für Schritt, Phasen bauen aufeinander auf. Clean Code, 80%
Praxis/20% Theorie, Fehler als Lernchance. Wir-Form, Code-Build-Explain, rustfmt-konform.
Details: `roadmap.md` im Repo-Root.
