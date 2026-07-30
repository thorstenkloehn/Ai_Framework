# Phase 6 — Experte: Performance & Orchestrierung

**Ergebnis dieser Phase:** Wir messen statt zu raten und können mehrere Modelle oder
Agenten koordinieren.

## Warum diese Phase jetzt?

Bis hierhin haben wir vor allem **Fähigkeiten** gebaut: eine Domäne (Phase 1), eine
LLM-Anbindung (Phase 2), austauschbare Anbieter (Phase 3), einen Agenten (Phase 4) und
RAG samt Betrieb (Phase 5). Was bisher fehlt: harte Zahlen. Ist unser Chunking aus
[Phase 5](../06-phase5-rag-betrieb/README.md) eigentlich schnell genug? Hält unsere
Invariante "Nachricht hat nie leeren Inhalt" aus
[Phase 1](../02-phase1-fundament/03-invarianten.md) wirklich für *jede* denkbare
Eingabe, nicht nur die drei, die wir uns beim Testen ausgedacht haben? Und: Muss wirklich
jede Anfrage an das teuerste, langsamste Modell gehen, nur weil wir kein Kriterium haben,
das zu entscheiden?

Phase 6 beantwortet das. Wir führen keine einzige neue fachliche Fähigkeit ein — kein
neues Crate, kein neues Feature für Nutzer*innen. Stattdessen bauen wir die Werkzeuge, mit
denen wir das Bestehende **messen, beweisen und koordinieren**: Benchmarks,
Property-Based-Testing, Model-Routing und Multi-Agent-Orchestrierung.

## Was bisher geschah

Diese Phase baut auf dem vollständigen Stand von Release 5 auf:
[Phase 1](../02-phase1-fundament/README.md) (Domäne),
[Phase 2](../03-phase2-llm-anbindung/README.md) (HTTP/JSON/Fehler),
[Phase 3](../04-phase3-architektur/README.md) (`LlmProvider`-Port, Hexagonal
Architecture), [Phase 4](../05-phase4-agenten/README.md) (`mein_agent`, Agent Loop) und
[Phase 5](../06-phase5-rag-betrieb/README.md) (`mein_rag`, `mein_server`, Betrieb). Wenn
dir ein Begriff aus einer dieser Phasen unklar ist, lohnt sich ein kurzer Rücksprung,
bevor du hier weitermachst.

## Lektionen

1. [Benchmarks mit criterion](01-benchmarks-criterion.md) — Performance messen statt
   schätzen, am Beispiel des Chunkings aus Phase 5.
2. [Eigenschaften und Fuzzing mit proptest](02-fuzzing-proptest.md) — Invarianten gegen
   tausende zufällige Eingaben prüfen, statt nur gegen Beispiele.
3. [Model Routing und Fallback](03-model-routing-fallback.md) — ein günstiges Modell für
   einfache Aufgaben wählen, mit kontrolliertem Rückfall auf ein stärkeres.
4. [Multi-Agent-Orchestrierung](04-multi-agent-orchestrierung.md) — mehrere
   `mein_agent`-Instanzen mit klar getrennten Verantwortlichkeiten koordinieren.
5. [Kosten, Latenz und Qualität abwägen](05-kosten-latenz-qualitaet.md) — die drei
   Achsen gemeinsam bewerten, statt eine davon isoliert zu optimieren.

## Transferaufgabe der Phase

Für kurze Klassifikationen wird ein günstiges Modell gewählt; bei Unsicherheit erfolgt
ein kontrollierter Fallback. Du bearbeitest diese Aufgabe konkret am Ende von
[Lektion 3](03-model-routing-fallback.md) und ordnest das Ergebnis am Ende von
[Lektion 5](05-kosten-latenz-qualitaet.md) noch einmal in die
Kosten-Latenz-Qualität-Abwägung ein.

## Bewertungsraster für diese Phase

- **Rust-Grundlagen:** Trait-Objekte (`dyn LlmProvider`) als austauschbare Strategien,
  generische Testinfrastruktur (`#[cfg(test)]`, `benches/`, `tests/`).
- **Design:** Strategy Pattern für Model-Routing, klare Verantwortungsgrenzen zwischen
  koordinierten Agenten — keine geteilte, gleichzeitig veränderbare Zustandsvariable
  zwischen ihnen.
- **Qualität:** Mindestens ein Benchmark, mindestens ein Property-Test, Routing- und
  Orchestrierungs-Entscheidungen sind durch Zahlen begründet, nicht durch Bauchgefühl.

## Kein eigenes Release

Anders als jede vorherige Phase endet Phase 6 **ohne eigenen Git-Tag**. Der Grund: Wir
verändern in dieser Phase keine öffentliche Schnittstelle und liefern kein neues
fachliches Ergebnis aus — wir instrumentieren und optimieren das, was seit
[Release 5: `operable-rag-service`](../06-phase5-rag-betrieb/09-release-5.md) bereits
läuft. Der nächste Tag, den wir setzen, ist gleich der große
Abschluss-Release in [Phase 7](../08-phase7-release/README.md). Lektion 5 dieser Phase
schließt entsprechend nicht mit einer Release-Checkliste, sondern mit einem kurzen
Übergang.

Bereit? Los geht's mit
[Lektion 1: Benchmarks mit criterion](01-benchmarks-criterion.md).
