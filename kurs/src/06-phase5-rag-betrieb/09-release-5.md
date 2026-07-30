# Lektion 9 · Release 5: operable-rag-service

## Rückblick

Aus einem Agenten, der Tools aufrufen konnte, aber nichts über die Welt außerhalb seines
Trainings wusste, ist in acht Lektionen ein betriebsfähiger RAG-Dienst geworden:

- Ein neues Crate `mein_rag` mit `DocumentLoader`, `Chunk`/`chunke_dokument`, `Embedder`,
  `VectorStore` und `Retriever` — jeder Schritt der Pipeline (Laden → Zerlegen →
  Einbetten → Speichern → Suchen) hinter einem eigenen, austauschbaren Port.
- `DateisystemLoader`, `HashEmbedder` und `InMemoryVectorStore` als konkrete, lokal
  lauffähige Implementierungen dieser Ports — austauschbar gegen produktive Backends wie
  Qdrant oder LanceDB, ohne dass `Retriever`-Aufrufer-Code sich ändert.
- Quellenangaben (`RetrievedChunk::source`), die von der geladenen Datei bis zur
  Antwort durchgereicht werden.
- Ein neues Binary-Crate `mein_server` mit Axum als REST-Anwendungsschicht neben
  `mein_cli`, mit geteiltem `AppState` für `Retriever` und `LlmProvider`.
- Exponentielles Backoff (`mit_backoff`) für robuste Provider- und Embedding-Aufrufe,
  aufbauend auf dem `reqwest`-Client aus Phase 2.
- `tracing` für strukturiertes Logging, eine `Kostenschaetzung` für Token-/Kosten-Tracking
  und `ApiSchluessel` mit `zeroize` für sicheres Secret-Handling.
- Strukturelle Trennung von System-, Retrieval- und Nutzeranteil im Prompt, mit einem
  aktiven Test, der den Prompt-Injection-Angriff nachstellt — sowie ein mehrstufiges
  Dockerfile und ein CI-Workflow, der all das bei jeder Änderung erneut prüft.

## Definition of Done

- [ ] `cargo build --workspace` läuft ohne Fehler und ohne Warnungen.
- [ ] `cargo test --workspace` — alle Tests aus Lektion 1–8 sind grün, **inklusive**
      `naiver_prompt_ist_verwundbar_fuer_prompt_injection` (dokumentiert den Angriff) und
      `retrieval_inhalte_koennen_keine_systemregeln_ueberschreiben` (beweist die
      Reparatur).
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` läuft ohne Beanstandungen.
- [ ] `cargo fmt --check` zeigt keine Abweichungen.
- [ ] Der Fehlerpfad wurde mindestens einmal bewusst ausgeführt (nicht existierendes
      Verzeichnis beim `DocumentLoader`, `max_versuche` erschöpft bei `mit_backoff`,
      fehlgeschlagene JSON-Deserialisierung bei `mein_server`).
- [ ] Die Transferaufgabe der Phase — *"Retrieval-Inhalte werden als untrusted data
      behandelt und dürfen keine Systemregeln überschreiben"* — ist in
      [Lektion 8](08-security-docker-ci.md) über `StrukturierterPrompt` gelöst, und der
      dazugehörige Test läuft in CI mit.
- [ ] `docker build` erzeugt ein lauffähiges Image von `mein_server`, ohne dass ein
      API-Key im Image selbst enthalten ist.
- [ ] `.github/workflows/ci.yml` (oder eine gleichwertige CI-Konfiguration) führt `fmt`,
      `clippy` und `test` bei jedem Push automatisch aus.

## Aufräumen vor dem Commit

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Der Release

```bash
git add .
git commit -m "Phase 5: RAG-Pipeline, Axum-Server, Retry/Tracing/Secrets, Prompt-Injection-Schutz"
git tag operable-rag-service
```

Wie schon bei den vorigen vier Releases beschreibt der Tag-Name das **fachliche**
Ergebnis: Unser Framework kann jetzt nicht nur mit einem LLM sprechen und Tools nutzen,
sondern eigene Dokumente einbinden und dabei betrieben werden — *operable*, betriebsfähig,
nicht nur lauffähig auf dem eigenen Rechner.

## Ausblick auf Phase 6

Bisher haben wir uns bei Design-Entscheidungen — Chunk-Größe, Overlap, Backoff-Basiszeit,
`top_k` — auf Plausibilität und kleine, handgeschriebene Tests verlassen. In
[Phase 6](../07-phase6-performance/README.md) messen wir stattdessen: Benchmarks mit
`criterion` zeigen, wie schnell Chunking und Ähnlichkeitssuche tatsächlich sind,
Property-Testing mit `proptest` sucht automatisch nach Randfällen, die uns beim
Handschreiben von Tests entgangen wären, und Model Routing wählt je nach Anfrage
automatisch zwischen mehreren Modellen — von einem günstigen Modell für einfache
Klassifikation bis zu einem leistungsfähigeren für komplexe Aufgaben.

Die Ports aus dieser Phase — `DocumentLoader`, `Embedder`, `VectorStore`, `Retriever` —
bleiben dabei unverändert. Phase 6 optimiert und orchestriert **um** sie herum, so wie
schon jede vorige Phase auf dem Fundament der vorigen aufgebaut hat, ohne es
umzureißen.

[Weiter: Phase 6 — Experte: Performance & Orchestrierung](../07-phase6-performance/README.md)
