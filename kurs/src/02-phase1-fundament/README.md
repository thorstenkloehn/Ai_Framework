# Phase 1 — Fundament

**Ergebnis dieser Phase:** Eine Konversation kann im Speicher angelegt, erweitert und
ausgegeben werden. Aus dem Cargo-Standardtemplate wird ein echtes, wenn auch kleines,
Domain-Modell.

## Warum diese Phase zuerst?

Jedes KI-Framework dreht sich im Kern um eine Sache: **Nachrichten zwischen Rollen** —
System, Nutzer, Assistent. Egal ob wir später mit einem echten LLM sprechen (Phase 2),
Agenten bauen (Phase 4) oder Dokumente einbinden (Phase 5) — überall taucht dieselbe
Grundfrage wieder auf: Wie modelliere ich eine Nachricht, und wie halte ich einen
Verlauf davon? Wer das Fundament hier sauber baut, muss es später nicht mehr anfassen.

## Lektionen

1. [Den Workspace lesen](01-workspace-lesen.md) — Verantwortung von `core` und `cli`
   trennen, den echten Ist-Zustand des Repos verstehen.
2. [Rolle und Nachricht als Domain Types](02-rolle-und-nachricht.md) — den vorhandenen
   `Rolle`/`Nachricht`-Code Zeile für Zeile entschlüsseln.
3. [Invarianten schützen](03-invarianten.md) — leere Inhalte verhindern, Sichtbarkeit von
   Feldern bewusst wählen.
4. [Konversation mit Vec\<Nachricht\>](04-konversation.md) — einen Verlauf im Speicher
   aufbauen.
5. [Konfiguration mit serde](05-serde-konfiguration.md) — Einstellungen aus einer Datei
   statt hartkodiert.
6. [CLI mit clap](06-cli-mit-clap.md) — `mein_cli` wird ein echtes Kommandozeilenprogramm
   mit Subcommands.
7. [Release 1: conversation-in-memory](07-release-1.md) — Definition of Done, Git-Tag,
   Ausblick auf Phase 2.

## Transferaufgabe der Phase

Eine Systemnachricht soll beim Erzeugen einer Konversation **optional** gesetzt werden
können, ohne dass die CLI die interne Speicherung von `Konversation` kennen muss. Du
bearbeitest diese Aufgabe konkret am Ende von [Lektion 4](04-konversation.md) und
überprüfst deine Lösung anhand der Definition of Done in
[Lektion 7](07-release-1.md).

## Bewertungsraster für diese Phase

- **Rust-Grundlagen:** Ownership bei `String`/`&str`, `enum` + `match`, `Vec<T>`.
- **Design:** Trennung zwischen `mein_core` (Domäne) und `mein_cli` (Anwendung) bleibt
  während der ganzen Phase sauber.
- **Qualität:** mindestens ein Unit-Test pro neuem Typ, `cargo fmt` und `cargo clippy`
  laufen sauber durch.

Bereit? Los geht's mit [Lektion 1: Den Workspace lesen](01-workspace-lesen.md).
