# Lektion 5: crates.io-Checkliste

## Problem

Ein Rust-Crate zu veröffentlichen ist nicht "einfach `git push`" — [crates.io](https://crates.io),
die zentrale Paket-Registry für Rust, verlangt bestimmte Mindestangaben in der
`Cargo.toml`, bevor `cargo publish` überhaupt akzeptiert wird. Fehlt eine Lizenzangabe
oder eine Beschreibung, weiß niemand, unter welchen Bedingungen er unser Crate benutzen
darf oder wofür es überhaupt gedacht ist — crates.io verweigert die Veröffentlichung
konsequent, statt ein halbdokumentiertes Paket anzunehmen. Wir wollen diese
Voraussetzungen kennen und **vor** dem echten `cargo publish` prüfen können, ohne
tatsächlich schon etwas hochzuladen.

## Code (Zielbild)

```toml
[package]
name = "mein_core"
version = "0.1.0"
edition = "2024"
description = "Ein modulares KI-Framework für LLM-Anbindung, Agenten und RAG in Rust."
license = "MIT OR Apache-2.0"
repository = "https://github.com/thorstenkloehn/Ai_Framework"
readme = "README.md"
keywords = ["llm", "ai", "agent", "rag"]
categories = ["asynchronous"]
```

## Dekonstruktion

### Die Pflicht- und Empfehlungsfelder

`cargo publish` verweigert den Vorgang ganz ohne eine `description` und eine `license`
(oder `license-file`) — das sind die einzigen wirklich harten Pflichtfelder. Die übrigen
Felder sind nicht technisch erzwungen, aber für ein seriöses öffentliches Crate praktisch
Pflicht:

- **`description`** — ein Satz, der auf der crates.io-Suchergebnisseite erscheint.
- **`license`** — ein [SPDX-Ausdruck](https://spdx.org/licenses/) wie `"MIT"`, `"Apache-2.0"`
  oder `"MIT OR Apache-2.0"` (die in der Rust-Community übliche doppelte Lizenzierung —
  Nutzer*innen dürfen sich eine der beiden aussuchen).
- **`repository`** — der Link zum Quellcode, hier
  [github.com/thorstenkloehn/Ai_Framework](https://github.com/thorstenkloehn/Ai_Framework).
- **`readme`** — verweist auf eine `README.md`, deren Inhalt crates.io direkt auf der
  Crate-Seite anzeigt.
- **`keywords`** / **`categories`** — verbessern die Auffindbarkeit über die
  crates.io-Suche (maximal 5 Keywords, `categories` muss aus einer von crates.io
  vorgegebenen Liste stammen).

### `cargo publish --dry-run`

`--dry-run` durchläuft den kompletten Veröffentlichungsprozess — Paketieren,
Metadaten-Validierung, Kompilieren des gepackten Ergebnisses — **ohne** tatsächlich etwas
zur Registry hochzuladen. Das ist der sichere Weg, alle Fehlermeldungen zu sehen, die ein
echtes `cargo publish` werfen würde, ohne das Risiko einer verfrühten (und bei crates.io
grundsätzlich **unlöschbaren**, nur "yankbaren") Veröffentlichung.

### Path-Dependencies brauchen eine Versionsangabe

Unser Workspace referenziert `mein_core` bisher intern oft nur über einen Pfad:

```toml
mein_core = { path = "../mein_core" }
```

Das funktioniert lokal, aber crates.io kennt keine lokalen Pfade — jede Dependency eines
veröffentlichten Crates muss selbst auf der Registry liegen und über eine Versionsnummer
referenziert werden:

```toml
mein_core = { path = "../mein_core", version = "0.1" }
```

Cargo nutzt beim lokalen Bauen weiterhin den `path`, verpackt aber beim Publish die
`version`-Angabe mit — genau diese Kombination erlaubt es, dass dasselbe Cargo.toml sowohl
im lokalen Workspace als auch nach der Veröffentlichung funktioniert.

## Schritt-Reveal

**Schritt 1 — Provoziere den Validierungsfehler bewusst.** Entferne testweise `license`
und `description` aus `mein_core/Cargo.toml` und führe aus:

```bash
cargo publish -p mein_core --dry-run
```

```
error: manifest has no description, license, license-file, documentation, homepage
or repository. At least one of these fields must be present.
```

crates.io verlangt nicht *jedes* Feld einzeln, sondern mindestens eines aus dieser Gruppe
— trotzdem ist ein Crate ganz ohne Beschreibung und Lizenz für niemanden vertrauenswürdig
benutzbar. Ergänze `description` und `license` wieder.

**Schritt 2 — Metadaten vollständig ergänzen** (siehe Zielbild oben).

**Schritt 3 — Path-Dependencies mit Versionsangabe versehen**, für jede interne
Workspace-Abhängigkeit, die `mein_core` nutzt (siehe Dekonstruktion). Ohne diese Angabe:

```bash
cargo publish -p mein_core --dry-run
```

```
error: all dependencies must have a version specified when publishing.
dependency `mein_rag` does not specify a version
```

**Schritt 4 — Erneuter Dry-Run.**

```bash
cargo publish -p mein_core --dry-run
```

```
    Updating crates.io index
   Packaging mein_core v0.1.0
   Verifying mein_core v0.1.0
   Compiling mein_core v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.31s
   Uploading mein_core v0.1.0 (dry run)
```

`Uploading ... (dry run)` — der letzte Schritt wird simuliert, nicht ausgeführt. Wir wissen
jetzt sicher: Ein echtes `cargo publish -p mein_core` (ohne `--dry-run`, nach `cargo login`
mit einem crates.io-API-Token) würde technisch durchlaufen.

## Ausführung

```bash
cargo publish -p mein_core --dry-run
```

Erwartete Ausgabe wie in Schritt 4 oben, ohne Fehlermeldungen.

> **⚠️ Warnung**
>
> Eine auf crates.io veröffentlichte Version kann **nicht gelöscht** werden — nur
> "yanken" (`cargo yank`), was verhindert, dass *neue* Projekte diese Version als
> Abhängigkeit auflösen, ohne bereits darauf aufbauende Builds zu brechen. Prüfe deshalb
> `--dry-run` gründlich und lies das Diff gegen die letzte Version, bevor du wirklich
> veröffentlichst — Unumkehrbarkeit ist hier keine Übertreibung.

> **💡 Tipp**
>
> Für ein reales, gemeinschaftlich entwickeltes Projekt gehört neben den
> Cargo.toml-Metadaten auch eine `CONTRIBUTING.md` (Ablauf für Pull Requests, verlangte
> Checks wie `cargo fmt --check` und `cargo clippy --workspace`, Commit-Konventionen) und
> oft eine `CODE_OF_CONDUCT.md` ins Repository — beides keine Cargo-Pflicht, aber
> Standard-Erwartung im Rust-Ökosystem, sobald Fremde beitragen sollen.

## Zusammenfassung

- `cargo publish` verlangt mindestens eines aus `description`, `license`/`license-file`,
  `documentation`, `homepage`, `repository` — praktisch sind alle davon sinnvoll, nicht
  nur das Minimum.
- `license = "MIT OR Apache-2.0"` ist die in der Rust-Community übliche
  Doppellizenzierung.
- Path-Dependencies innerhalb eines Workspace brauchen für die Veröffentlichung
  zusätzlich eine `version`-Angabe neben dem `path`.
- `cargo publish --dry-run` prüft den gesamten Veröffentlichungsprozess, ohne
  tatsächlich etwas hochzuladen — der sichere erste Schritt vor jedem echten Release.
- Eine veröffentlichte Version ist praktisch unumkehrbar (nur "yankbar") — Sorgfalt vor
  dem echten `cargo publish` zahlt sich aus.

## Übung

Vervollständige die Cargo.toml-Metadaten für die übrigen Workspace-Crates
(`mein_agent`, `mein_rag`, `mein_cli`, `mein_server`), soweit sie ebenfalls veröffentlicht
werden sollen, und führe für jedes einen `cargo publish --dry-run` aus. Überlege dabei: Muss
wirklich jedes Crate im Workspace einzeln auf crates.io veröffentlicht werden, oder gibt es
Crates (z. B. `mein_cli` als reines Binary für den Eigengebrauch), für die sich eine
Veröffentlichung nicht lohnt? Begründe deine Entscheidung kurz schriftlich, bevor du zur
letzten Lektion weitergehst.

[Weiter: Lektion 6 — Abschluss-Release: ai-framework-0.1.0](06-abschluss-release.md)
