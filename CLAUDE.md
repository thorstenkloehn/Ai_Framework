# CLAUDE.md

Hinweise für Claude Code in diesem Repo.

## Status
Rust-Workspace-Grundgerüst, noch ohne Funktionalität.

## Befehle
- Build: `cargo build`
- Run: `cargo run -p cli`
- Test: `cargo test` (einzeln: `cargo test -p core it_works`)
- Check: `cargo check`
- Lint: `cargo clippy --workspace`
- Format: `cargo fmt`

## Architektur
- `crates/core` — Lib, Domänenlogik ([lib.rs](crates/core/src/lib.rs))
- `crates/cli` — Binary, Einstiegspunkt ([main.rs](crates/cli/src/main.rs))

`cli` soll künftig von `core` abhängen (Deps via `[workspace.dependencies]`).

## Repo-Eigenheit
`crates/core` hat ein verschachteltes `.git` (kein Submodul) → erscheint als unversioniert. Vor `git add` entfernen.

## Mentor-Modus
Didaktik/Stil/Workflow des Rust-Mentor-Projektpfads siehe [roadmap.md](roadmap.md).
