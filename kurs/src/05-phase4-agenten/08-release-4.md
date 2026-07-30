# Lektion 8 · Release 4: tool-using-agent

## Rückblick

Aus einem synchronen Framework, das eine Anfrage stellt und auf eine Antwort wartet, ist
in sieben Lektionen ein Agent geworden, der selbstständig zwischen Denken und Handeln
wechselt:

- Ein neues Crate, `mein_agent`, das `tokio` und alle Agent-spezifischen Abhängigkeiten
  von der schlanken `mein_core`-Bibliothek fernhält.
- `async`/`.await` und die Tokio-Runtime als Grundlage für alles Weitere
  ([Lektion 1](01-async-und-tokio.md)).
- Ein Verständnis von SSE-Streaming als Strom von Ereignissen, konsumiert über
  `Stream`/`StreamExt` ([Lektion 2](02-sse-streaming.md)).
- `agent::tool` — der `Tool`-Trait, JSON-Schema-Beschreibungen über `schemars`, und ein
  anbieter-unabhängiges Function-Calling-Format ([Lektion 3](03-tool-schema-function-calling.md)).
- `agent::loop` (als `r#loop` gebaut) — die Schleife Plan → Tool-Aufruf → Beobachtung →
  nächste Aktion, mit `Box<dyn LlmProvider>` und `Vec<Box<dyn Tool>>` als Ports
  ([Lektion 4](04-agent-loop.md)).
- `agent::state` — `AgentState`, der Konversation und Schrittzähler kapselt, plus die
  Unterscheidung zwischen `Send`-sicherem geteiltem Zustand (`Arc<Mutex<...>>`) und dem
  einfacheren, nicht geteilten Fall ([Lektion 5](05-state-und-memory.md)).
- Vier saubere Abbruchwege — Erfolg, Schrittlimit, unbekanntes Werkzeug,
  Zeitüberschreitung — jeder ohne Panic ([Lektion 6](06-abbruchbedingungen-limits.md)).
- Einen Ausblick auf das Model Context Protocol als optionale Erweiterung, ohne den
  `AgentLoop` selbst anzufassen ([Lektion 7](07-mcp-client.md)).

## Definition of Done

Bevor wir taggen, prüfen wir dieselben Kriterien wie bei jedem Release in diesem Kurs
(siehe [Wie dieser Kurs funktioniert](../00-einleitung/02-wie-dieser-kurs-funktioniert.md)):

- [ ] `cargo build --workspace` läuft ohne Fehler und ohne Warnungen — `mein_agent`
      eingeschlossen.
- [ ] `cargo test -p mein_agent` — alle Tests aus Lektion 3-6 sind grün, inklusive der
      beiden Tests aus der Transferaufgabe (Schrittlimit, unbekanntes Werkzeug).
- [ ] `cargo clippy --workspace` läuft ohne Beanstandungen.
- [ ] `cargo fmt --check` zeigt keine Abweichungen.
- [ ] Alle vier Fehlerpfade aus [Lektion 6](06-abbruchbedingungen-limits.md)
      (Schrittlimit, unbekanntes Werkzeug, Provider-Fehler, Zeitüberschreitung) sind
      mindestens einmal bewusst ausgelöst und beobachtet worden — keiner davon endet in
      einem Panic.
- [ ] Die Transferaufgabe aus [Lektion 6](06-abbruchbedingungen-limits.md) — höchstens
      fünf Schritte, sicherer Abbruch bei unbekanntem Tool — ist mit zwei eigenen Tests
      belegt.
- [ ] Temporäre `examples/`-Dateien aus [Lektion 1](01-async-und-tokio.md) und
      [Lektion 2](02-sse-streaming.md), die nur zum Ausprobieren dienten, sind entweder
      aufgeräumt (gelöscht) oder bewusst als Lernbeispiel behalten — beides ist in
      Ordnung, solange es eine bewusste Entscheidung ist, kein Liegengebliebenes.

## Aufräumen vor dem Commit

```bash
cargo fmt
cargo clippy --workspace
cargo test --workspace
```

Wirf zusätzlich einen Blick in `mein_agent/Cargo.toml`: Ab jetzt sammeln sich dort
`tokio`, `futures-util`, `async-trait`, `schemars`, `serde`, `serde_json` und
`thiserror`. Prüfe mit `cargo tree -p mein_agent`, ob sich darunter etwas eingeschlichen
hat, das du gar nicht mehr brauchst (z. B. ein Tokio-Feature, das du in
[Lektion 1](01-async-und-tokio.md) probeweise aktiviert, aber nie benutzt hast) —
dasselbe YAGNI-Prinzip wie beim `derive`-Attribut in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md).

## Der Release

```bash
git add .
git commit -m "Phase 4: Agent Loop mit Tool-Use, State und Abbruchbedingungen in mein_agent"
git tag tool-using-agent
```

Der Tag-Name `tool-using-agent` beschreibt wieder das **fachliche** Ergebnis, nicht die
Technik — konsistent mit `conversation-in-memory` (Phase 1),
[`typed-provider-boundary`](../03-phase2-llm-anbindung/08-release-2.md) (Phase 2) und
[`provider-agnostic-core`](../04-phase3-architektur/08-release-3.md) (Phase 3).

## Ausblick auf Phase 5

Unser Agent kann jetzt denken, Werkzeuge aufrufen und sicher aufhören — aber er weiß
nur das, was im Modell selbst steckt oder was ihm ein Werkzeug im Moment liefert. Er hat
kein eigenes Wissen über *deine* Dokumente, *deine* Datenbasis.
[Phase 5](../06-phase5-rag-betrieb/README.md) schließt genau diese Lücke: Wir laden
Dokumente, zerlegen sie in durchsuchbare Abschnitte (**Chunking**), verwandeln sie in
Embeddings, legen sie in einem `VectorStore` ab und geben einem `Retriever` die
Möglichkeit, zur Laufzeit die relevantesten Abschnitte in den Prompt einzubinden —
nachvollziehbar, mit Quellenangaben. Am Ende von Phase 5 steht außerdem der Sprung vom
CLI-Werkzeug zu einem echten, betreibbaren Dienst: eine REST-Schnittstelle mit Axum,
Retry- und Backoff-Strategien für unzuverlässige Netzwerke, und die ersten
Sicherheits- und Betriebsthemen (Tracing, Kosten-Tracking, Prompt-Injection-Schutz,
Docker, CI).

Die gute Nachricht, wie schon nach jeder vorherigen Phase: `Konversation`,
`LlmProvider`, `Runnable` und jetzt auch `AgentLoop` bleiben, wie sie sind. Phase 5 baut
**um** sie herum — ein Retriever liefert am Ende nur zusätzlichen Kontext für dieselbe
`Konversation`, die du seit Phase 1 kennst.

[Weiter: Phase 5 — RAG, Deployment und Betrieb](../06-phase5-rag-betrieb/README.md)
