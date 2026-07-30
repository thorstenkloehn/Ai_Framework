# Über dieses Projekt: Was wir bauen

## Die Ausgangsfrage

Stell dir vor, du willst in einer Rust-Anwendung ein Sprachmodell (ein "LLM", Large
Language Model, wie es z. B. hinter ChatGPT oder Claude steckt) nutzen — Nachrichten
schicken, Antworten bekommen, vielleicht später eigene Dokumente durchsuchbar machen oder
einen Agenten bauen, der selbstständig Werkzeuge benutzt.

Du könntest das jedes Mal von Grund auf neu schreiben: HTTP-Aufrufe zusammenbauen, JSON von
Hand parsen, Fehlerfälle irgendwie abfangen. Funktioniert — aber sobald du den
LLM-Anbieter wechselst, eine Datenbank austauschst oder ein zweites Projekt mit denselben
Anforderungen startest, schreibst du alles noch einmal.

Genau dieses Problem lösen **Frameworks**. In der Python-Welt heißt die bekannteste
Antwort darauf [LangChain](https://www.langchain.com/), in der Java-Welt
[Spring AI](https://spring.io/projects/spring-ai). Beide bieten wiederverwendbare
Bausteine: eine einheitliche Schnittstelle zu verschiedenen Anbietern, Werkzeuge für
Prompt-Templates, Speicher für Gesprächsverläufe, Anbindungen an Vektordatenbanken für
Retrieval-Augmented Generation (RAG), und vieles mehr.

**Ai_Framework** ist der Versuch, so etwas für Rust zu bauen — und dieser Kurs ist der
Weg dorthin, Schritt für Schritt, mit dir am Keyboard.

## Warum Rust?

Falls du neu in der Programmierwelt bist, wunderst du dich vielleicht, warum wir nicht
gleich mit einer "einfacheren" Sprache anfangen. Kurz zusammengefasst — mehr dazu in
[Kapitel 0](../01-grundlagen/README.md):

- Rust hat einen ungewöhnlich strengen Compiler, der ganze Fehlerklassen (abstürzende
  Programme durch ungültigen Speicherzugriff, Datenwettläufe bei Nebenläufigkeit) schon
  beim Kompilieren verhindert, statt sie erst beim Kunden auffallen zu lassen.
- Diese Strenge ist am Anfang mühsam, aber pädagogisch wertvoll: Der Compiler zwingt dich,
  über Dinge nachzudenken, die andere Sprachen wegabstrahieren — Speicher, Besitz,
  Nebenläufigkeit. Wer das einmal in Rust verstanden hat, versteht es in jeder anderen
  Sprache mit.
- Für ein Framework, das andere als Abhängigkeit einbinden, ist Performance und
  Zuverlässigkeit ohne Laufzeitüberraschungen ein echter Vorteil.

Wir nutzen diese Strenge aktiv: Ein wiederkehrendes Element in diesem Kurs sind bewusst
provozierte Compilerfehler — wir schreiben absichtlich Code, der nicht kompiliert, lesen
gemeinsam die Fehlermeldung, und lernen daraus. Der Rust-Compiler ist in diesem Kurs
weniger Gegner als Co-Lehrer.

## Die sieben Phasen im Überblick

Das Framework wächst in sieben klar abgegrenzten Ausbaustufen. Jede endet mit einem
lauffähigen, git-getaggten Release.

| Phase | Ergebnis am Ende | Release-Tag |
|-------|-------------------|-------------|
| 1 — Fundament | Konversation im Speicher anlegen, erweitern, ausgeben | `conversation-in-memory` |
| 2 — Core & LLM-Anbindung | Typisierte Requests an einen echten LLM-Anbieter, typisierte Antworten | `typed-provider-boundary` |
| 3 — Architektur & QS | Anbieter austauschbar, Code getestet und lint-sauber | `provider-agnostic-core` |
| 4 — Agenten & Concurrency | Agent ruft Tools auf, beobachtet Ergebnisse, plant weiter | `tool-using-agent` |
| 5 — RAG, Deployment & Betrieb | Eigene Dokumente durchsuchbar, Framework läuft als Service | `operable-rag-service` |
| 6 — Performance | Gemessen statt geraten, mehrere Modelle/Agenten koordiniert | — |
| 7 — Öffentliches Release | Stabile, dokumentierte, auf crates.io veröffentlichbare API | `ai-framework-0.1.0` |

Jede Phase baut zwingend auf der vorherigen auf — wir überspringen keine Stufe, auch wenn
sie dir am Anfang trivial vorkommt. Gerade Phase 1 sieht "zu einfach" aus, legt aber die
Architekturentscheidungen (Trennung von `core` und `cli`, saubere Domain-Typen), die uns
in Phase 3–5 vor größeren Umbauten bewahren.

## Der Ausgangspunkt

Das echte Repository beginnt bewusst klein: ein Cargo-Workspace mit zwei Crates,
`mein_core` (Bibliothek) und `mein_cli` (Kommandozeilenprogramm), und ganz am Anfang genau
zwei Typen: `Rolle` und `Nachricht`. Kein Netzwerkcode, kein Agent, kein RAG — noch nicht.

Das ist kein Zufall, sondern der erste Lerngegenstand: Bevor wir Features anhäufen,
verstehen wir, welche Architektur das Projekt überhaupt braucht.
[Weiter zu Phase 1](../02-phase1-fundament/README.md), oder zuerst
[Kapitel 0](../01-grundlagen/README.md), falls du Programmier-Neuling bist.
