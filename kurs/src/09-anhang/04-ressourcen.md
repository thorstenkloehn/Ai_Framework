# Weiterführende Ressourcen

Dieser Kurs deckt Rust und den Aufbau eines KI-Frameworks praxisnah ab, aber nicht
vollständig. Diese Liste hilft weiter, wenn eine Frage über den Kursinhalt hinausgeht.

## Offizielle Rust-Dokumentation

- **The Rust Programming Language ("The Book")** — `doc.rust-lang.org/book` — das
  offizielle, kostenlose Einführungsbuch zu Rust. Deckt jedes Grundkonzept aus diesem Kurs
  (Ownership, Traits, Enums, Fehlerbehandlung) noch einmal ausführlicher und
  sprachfokussierter ab, ohne den KI-Framework-Kontext.
- **Rust by Example** — `doc.rust-lang.org/rust-by-example` — dieselben Konzepte, aber
  anhand vieler kurzer, lauffähiger Codebeispiele statt Fließtext. Gut, um eine
  Rust-Syntax schnell in Aktion zu sehen.
- **The Rust Standard Library Documentation (std-Doku)** — `doc.rust-lang.org/std` — die
  vollständige Referenz aller eingebauten Typen und Funktionen (`String`, `Vec`, `Option`,
  `Result` und alles andere aus `std`). Der erste Anlaufpunkt bei der Frage "welche
  Methoden hat dieser Typ eigentlich?".
- **docs.rs** — `docs.rs` — automatisch generierte Dokumentation für praktisch jedes auf
  crates.io veröffentlichte Crate. Für jedes im Kurs verwendete Crate (siehe unten) lohnt
  sich der direkte Aufruf `docs.rs/<crate-name>`.
- **crates.io** — `crates.io` — das offizielle Rust-Package-Registry. Zeigt zu jedem Crate
  Versionshistorie, Download-Zahlen, Lizenz und Link zur Dokumentation.
- **rustc Fehlercode-Index** — im Terminal per `rustc --explain <code>` (z. B.
  `rustc --explain E0308`) abrufbar, ergänzend auch online über die offizielle
  Rust-Fehlerindex-Seite auf `doc.rust-lang.org` auffindbar. Siehe auch
  [Fehlermeldungen verstehen](03-fehlermeldungen.md).

## Im Kurs verwendete Crates

Für jedes dieser Crates lohnt sich die Suche nach `docs.rs/<name>` für die aktuelle,
vollständige API-Referenz:

- **serde** / **serde_json** — Serialisierung/Deserialisierung, ab
  [Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md).
- **clap** — Kommandozeilen-Parsing, ab
  [Phase 1, Lektion 6](../02-phase1-fundament/06-cli-mit-clap.md).
- **reqwest** — HTTP-Client, ab
  [Phase 2, Lektion 1](../03-phase2-llm-anbindung/01-http-grenze-reqwest.md).
- **thiserror** / **anyhow** — Fehlertypen und Fehlerkontext, ab
  [Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md).
- **schemars** — JSON-Schema-Generierung für Structured Output, ab
  [Phase 2, Lektion 6](../03-phase2-llm-anbindung/06-structured-output.md).
- **sqlx** — asynchrone, typsichere Datenbankanbindung, ab
  [Phase 2, Lektion 7](../03-phase2-llm-anbindung/07-persistenz-sqlx.md).
- **tokio** — asynchrone Laufzeitumgebung, ab
  [Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md).
- **axum** — Webframework für `mein_server`, ab
  [Phase 5, Lektion 5](../06-phase5-rag-betrieb/05-rest-axum-oder-tui.md).
- **tracing** — strukturiertes Logging, ab
  [Phase 5, Lektion 7](../06-phase5-rag-betrieb/07-tracing-kosten-secrets.md).
- **zeroize** — sicheres Überschreiben sensibler Daten im Speicher, ab
  [Phase 5, Lektion 7](../06-phase5-rag-betrieb/07-tracing-kosten-secrets.md).
- **criterion** — Benchmarking, ab
  [Phase 6, Lektion 1](../07-phase6-performance/01-benchmarks-criterion.md).
- **proptest** — Property-Testing, ab
  [Phase 6, Lektion 2](../07-phase6-performance/02-fuzzing-proptest.md).

> **💡 Tipp**
>
> Bei jeder unbekannten Methode eines dieser Crates lohnt sich zuerst `cargo doc --open`
> im eigenen Projekt — das zeigt die Dokumentation der tatsächlich installierten Version,
> inklusive aller lokalen Typen, an einem Ort.

## Das Ai_Framework-Repository

- **github.com/thorstenkloehn/Ai_Framework** — das echte Repository, das diesem Kurs
  zugrunde liegt. `roadmap.md` und `AGENTS.md` im Repo-Root geben einen kompakten
  Überblick über Zielarchitektur und Konventionen, ergänzend zu den Kurslektionen. Die
  Release-Tags (`conversation-in-memory`, `typed-provider-boundary`, ...) markieren jeweils
  den Stand am Ende einer Kursphase.

## Community-Ressourcen

- **This Week in Rust** — ein wöchentlicher Newsletter mit aktuellen Neuigkeiten,
  Blogposts und neuen Crates aus dem Rust-Ökosystem. Guter Weg, um nach Kursende am Ball
  zu bleiben.
- **users.rust-lang.org** — das offizielle Rust-Nutzerforum. Der richtige Ort für
  konkrete "warum kompiliert das bei mir nicht"-Fragen, wenn [Fehlermeldungen
  verstehen](03-fehlermeldungen.md) nicht weiterhilft.
- **r/rust** (Reddit) — informelle Diskussionen, Projektvorstellungen und News rund um
  Rust.

> **⚠️ Warnung**
>
> Rust und sein Ökosystem entwickeln sich schnell. Versionsnummern und exakte APIs von
> Crates ändern sich — im Zweifel gilt immer die aktuelle Version auf docs.rs/crates.io,
> nicht eine im Kurs zitierte Zahl.
