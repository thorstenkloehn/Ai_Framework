# Lektion 7 · Release 1: conversation-in-memory

## Rückblick

Aus dem leeren Cargo-Standardtemplate ist in sechs Lektionen ein kleines, aber
vollständiges Fundament geworden:

- Ein Cargo-Workspace mit klar getrennten Verantwortlichkeiten: `mein_core` (Domäne),
  `mein_cli` (Kommandozeilen-Interface).
- `Rolle` und `Nachricht` als Domain Types, die ungültige Zustände so weit wie möglich
  unmöglich machen (`enum` statt `String`).
- Eine erzwungene Invariante: keine `Nachricht` mit leerem Inhalt, durchgesetzt über
  `Result<Self, NachrichtFehler>` in der Konstruktion.
- `Konversation`, die ihren `Vec<Nachricht>`-Verlauf kapselt und nur über geprüfte
  Methoden verändern lässt.
- `serde`-Unterstützung für `Rolle`, `Nachricht` und eine eigene `Konfiguration`.
- Eine echte CLI mit `clap`: Subcommands, Flags, automatische Hilfe.

## Definition of Done

Bevor wir taggen, prüfen wir die Kriterien, die für **jede** Lektion in diesem Kurs gelten
(siehe [Wie dieser Kurs funktioniert](../00-einleitung/02-wie-dieser-kurs-funktioniert.md)):

- [ ] `cargo build` läuft ohne Fehler und ohne Warnungen.
- [ ] `cargo test` — alle Tests aus Lektion 3, 4 und 5 sind grün.
- [ ] `cargo clippy --workspace` läuft ohne Beanstandungen (falls Clippy neue Hinweise
      zeigt: lies sie, sie sind meist berechtigt — Clippy vertiefen wir formal in
      [Phase 3, Lektion 5](../04-phase3-architektur/05-tests-und-clippy.md), nutzen es
      aber schon ab hier als Gewohnheit).
- [ ] `cargo fmt --check` zeigt keine Abweichungen.
- [ ] Der Fehlerpfad wurde mindestens einmal bewusst ausgeführt (leerer Inhalt, fehlendes
      Pflichtfeld in der Konfiguration, falscher CLI-Befehl).
- [ ] Die Transferaufgabe aus [Lektion 4](04-konversation.md)
      (`mit_systemnachricht`, ohne dass `mein_cli` die interne Speicherung kennt) ist
      gelöst.

## Aufräumen vor dem Commit

```bash
cargo fmt
cargo clippy --workspace
cargo test
```

Und, falls noch nicht geschehen (siehe die Warnung in
[Lektion 1](01-workspace-lesen.md)):

```bash
rm -rf mein_core/.git
```

Prüfe mit `git status`, dass jetzt alle Dateien normal (nicht als "Submodul") erscheinen.

## Der Release

```bash
git add .
git commit -m "Phase 1: Konversation im Speicher mit Rolle, Nachricht, Konversation, Konfiguration, CLI"
git tag conversation-in-memory
```

Der Tag-Name `conversation-in-memory` beschreibt bewusst das **fachliche Ergebnis** der
Phase, nicht die Technik ("hat clap, serde") — so bleibt die Release-Historie auch für
jemanden lesbar, der später nur die Tags überfliegt, ohne den Code zu kennen. Diesem Muster
folgen wir für alle sieben Releases dieses Kurses.

## Ausblick auf Phase 2

Bisher lebt jede Konversation nur im Arbeitsspeicher eines einzelnen CLI-Aufrufs — startest
du `mein_cli` neu, ist der Verlauf weg, und niemand antwortet wirklich auf die
Nutzernachricht. [Phase 2](../03-phase2-llm-anbindung/README.md) ändert genau das: Wir
schicken die `Konversation` über eine echte HTTP-Verbindung an einen LLM-Anbieter, lesen
die Antwort typisiert zurück, und bereiten mit `thiserror`/`anyhow` sauberes
Fehlerhandling für alles vor, was bei einem Netzwerkaufruf schiefgehen kann — vom
abgelehnten API-Key bis zum Timeout.

Die gute Nachricht: `Rolle`, `Nachricht` und `Konversation` bleiben, wie sie sind. Phase 2
baut **um** sie herum, nicht **durch** sie hindurch — ein erster Beweis dafür, dass sich
die Sorgfalt aus dieser Phase auszahlt.

[Weiter: Phase 2 — Core & LLM-Anbindung](../03-phase2-llm-anbindung/README.md)
