# Phase 2 — Core & LLM-Anbindung

**Ziel dieser Phase** (aus der Roadmap, wörtlich): *"Ein Provider nimmt typisierte
Requests entgegen und liefert typisierte Responses."*

**Ergebnis dieser Phase:** `mein_cli` schickt eine echte `Konversation` über HTTP an einen
LLM-Anbieter und bekommt eine echte Antwort zurück — vollständig typisiert, mit
verständlichen Fehlermeldungen bei allem, was dabei schiefgehen kann, und mit einer
Skizze dafür, wie eine Konversation dauerhaft gespeichert werden könnte.

## Warum diese Phase jetzt?

Phase 1 hat ein sauberes Fundament gebaut: `Rolle`, `Nachricht`, `Konversation`,
`Konfiguration`. Aber bisher passiert alles nur im Arbeitsspeicher eines einzelnen
CLI-Aufrufs — es antwortet niemand wirklich. Phase 2 überquert die erste **echte
Grenze** unseres Programms: den Sprung von "Daten, die in unserem eigenen Prozess
leben" zu "Daten, die über das Netzwerk zu einem fremden System reisen und wieder
zurückkommen". Jede Grenze dieser Art bringt neue Fragen mit: Wie sieht das Format
drüben aus? Was, wenn die Antwort nicht kommt, nicht passt oder der Anbieter "Nein"
sagt? Wie verhindern wir, dass Netzwerk- und JSON-Details unsere saubere Domäne aus
Phase 1 verschmutzen?

Wichtig für diese Phase: Wir bauen **noch keinen austauschbaren** Provider. Es gibt
genau einen konkreten HTTP-Client, kein `trait LlmProvider`, kein `dyn Trait`. Das ist
Absicht, keine Abkürzung — dazu mehr in
[Lektion 1](01-http-grenze-reqwest.md) und im Ausblick von
[Lektion 8](08-release-2.md). Abstraktion lohnt sich erst, wenn wir einen zweiten
Anwendungsfall kennen; den bauen wir erst in Phase 3.

## Lektionen

1. [HTTP-Grenze mit reqwest](01-http-grenze-reqwest.md) — `mein_core` bekommt ein
   eigenes `provider`-Modul und spricht zum ersten Mal über das Netzwerk mit einer
   fremden Gegenstelle.
2. [JSON-Schema mit serde_json](02-json-schema.md) — wir erkunden, wie das
   JSON-Format eines LLM-Anbieters tatsächlich aussieht, und warum unsere
   Phase-1-Typen nicht einfach so hineinpassen.
3. [Request- und Response-Typen trennen](03-request-response-typen.md) — eigene,
   von der Domäne getrennte Typen für das, was über die Leitung geht.
4. [Fehler mit thiserror und anyhow](04-fehlerbehandlung.md) — Bibliotheksfehler
   typisiert mit `thiserror`, Anwendungskontext mit `anyhow`.
5. [Prompt-Templating](05-prompt-templating.md) — Variablen in Prompts einsetzen und
   *vor* dem Netzwerkaufruf prüfen, ob sie überhaupt gültig sind.
6. [Structured Output mit schemars](06-structured-output.md) — ein LLM zwingen,
   strukturiertes JSON statt Fließtext zu liefern, und diese Antwort typisiert
   einlesen.
7. [Persistenz mit sqlx](07-persistenz-sqlx.md) — eine Skizze dafür, wie eine
   `Konversation` in einer echten Datenbank überleben könnte.
8. [Release 2: typed-provider-boundary](08-release-2.md) — Definition of Done,
   Git-Tag, Ausblick auf Phase 3.

## Transferaufgabe der Phase

**Eine ungültige Prompt-Variable soll vor dem Netzwerkaufruf als verständlicher Fehler
erscheinen.** Du bearbeitest diese Aufgabe konkret in
[Lektion 5](05-prompt-templating.md) und überprüfst deine Lösung anhand der Definition
of Done in [Lektion 8](08-release-2.md). Die Idee dahinter: Ein API-Aufruf kostet Zeit
und (bei echten Anbietern) Geld — ein Fehler, der sich schon vorher, rein lokal,
erkennen lässt, darf niemals erst als kryptische HTTP-Antwort beim Anbieter auffallen.

## Bewertungsraster für diese Phase

- **Rust-Grundlagen:** Module über mehrere Dateien (`mod`), `Result`-Verkettung mit
  `?` über mehrere Fehlertypen hinweg, generische Funktionen (`schemars`/`serde`
  zusammen mit einem Typparameter `T`).
- **Design:** `mein_core::provider` kennt nur HTTP und JSON, nicht die Kommandozeile;
  `mein_cli` kennt nur die öffentliche API von `mein_core`, nie `reqwest`- oder
  `sqlx`-Typen direkt. Request-/Response-Typen bleiben getrennt von `Rolle`/
  `Nachricht`/`Konversation`.
- **Qualität:** jeder neue Fehlerfall hat einen Test oder zumindest eine bewusst
  provozierte, dokumentierte Fehlermeldung; `cargo fmt` und `cargo clippy --workspace`
  laufen sauber durch.

Vorheriger Release: [`conversation-in-memory`](../02-phase1-fundament/07-release-1.md).
Bereit? Los geht's mit [Lektion 1: HTTP-Grenze mit reqwest](01-http-grenze-reqwest.md).
