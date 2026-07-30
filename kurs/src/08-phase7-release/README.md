# Phase 7 — Öffentliches Release

**Ergebnis dieser Phase:** Das Framework besitzt eine dokumentierte, stabile und
veröffentlichbare API.

## Warum diese Phase zuletzt?

Sechs Phasen lang haben wir für uns selbst gebaut — wir kannten jeden Aufrufer unseres
Codes, weil wir ihn selbst geschrieben haben. Ein veröffentlichtes Crate auf
[crates.io](https://crates.io) kennt seine Aufrufer*innen nicht. Fremder Code verlässt
sich auf unsere öffentliche Schnittstelle, ohne unseren internen Aufbau zu kennen — und
genau das verändert die Spielregeln: Eine unbedacht geänderte Signatur bricht nicht mehr
nur unsere eigene `mein_cli`, sondern potenziell jedes Projekt, das unser Crate
einbindet. Phase 7 führt keine neue fachliche Fähigkeit mehr ein. Sie poliert das
Bestehende zu etwas, das andere Menschen gefahrlos benutzen können: eine ergonomische
Konstruktion (Builder Pattern), wählbare Compile-Zeit-Kosten (Feature Flags),
mitgeprüfte Dokumentation (rustdoc), ein verlässliches Versionsversprechen (SemVer) und
die formalen Voraussetzungen für eine echte Veröffentlichung (crates.io-Checkliste).

## Was bisher geschah

Diese Phase setzt den vollständigen, jetzt vermessenen und orchestrierbaren Stand aus
[Phase 6](../07-phase6-performance/README.md) voraus, der wiederum auf
[Phase 1](../02-phase1-fundament/README.md) bis
[Phase 5](../06-phase5-rag-betrieb/README.md) aufbaut. Falls du zwischendurch einsteigst:
Die Kurzfassung ist [Kapitel 0](../01-grundlagen/README.md) für Rust-Grundlagen, danach
Phase 1 für `Rolle`/`Nachricht`/`Konversation`, Phase 3 für den `LlmProvider`-Port, Phase 4
für `mein_agent`, Phase 5 für `mein_rag`/`mein_server`.

## Lektionen

1. [Builder Pattern und Defaults](01-builder-pattern.md) — eine Konfiguration mit vielen
   optionalen Werten ergonomisch aufbauen, statt einen Konstruktor mit zehn Parametern zu
   erzwingen.
2. [Feature Flags](02-feature-flags.md) — RAG und Agenten optional machen, damit niemand
   dafür bezahlt (an Compile-Zeit und Binärgröße), was er nicht nutzt.
3. [Rustdoc und Beispiele](03-rustdoc-beispiele.md) — Dokumentation, die `cargo test`
   automatisch mitprüft.
4. [SemVer und Breaking Changes](04-semver-breaking-changes.md) — verstehen, was an einer
   öffentlichen Schnittstelle wie `LlmProvider` als Bruch zählt, und was nicht.
5. [crates.io-Checkliste](05-crates-io-checkliste.md) — die Cargo.toml-Metadaten und
   Vorprüfungen, die eine echte Veröffentlichung braucht.
6. [Abschluss-Release: ai-framework-0.1.0](06-abschluss-release.md) — Rückblick über den
   gesamten Kurs und der letzte, finale Git-Tag.

## Transferaufgabe der Phase

Eine neue Provider-Integration wird hinzugefügt, ohne bestehende Nutzer-Codebeispiele zu
ändern. Du bearbeitest diese Aufgabe konkret am Ende von
[Lektion 2](02-feature-flags.md) und überprüfst sie noch einmal im großen Rückblick von
[Lektion 6](06-abschluss-release.md).

## Bewertungsraster für diese Phase

- **Rust-Grundlagen:** Ownership beim Builder (`self` vs. `&mut self`), `#[cfg(feature =
  "...")]`, Trait-Erweiterung mit Default-Implementierungen.
- **Design:** Öffentliche API bleibt stabil, während sich die interne Implementierung
  weiterentwickeln darf — genau die Trennung, die Hexagonal Architecture aus Phase 3
  bereits vorbereitet hat.
- **Qualität:** Doctests laufen mit, `cargo publish --dry-run` läuft ohne Fehler,
  SemVer-Entscheidungen sind begründet, nicht geraten.

## Der letzte Tag

Der vorherige Git-Tag bleibt
[`operable-rag-service`](../06-phase5-rag-betrieb/09-release-5.md) aus Release 5 — Phase 6
hat bewusst keinen eigenen Tag gesetzt (siehe
[Phase 6, README](../07-phase6-performance/README.md)). Phase 7 setzt am Ende in
[Lektion 6](06-abschluss-release.md) den letzten Tag dieses gesamten Kurses:
`ai-framework-0.1.0`.

Bereit? Los geht's mit
[Lektion 1: Builder Pattern und Defaults](01-builder-pattern.md).
