# Phase 5 — RAG, Deployment & Betrieb

**Ergebnis dieser Phase:** Eigene Dokumente werden geladen, durchsucht und
nachvollziehbar in einen Prompt eingebunden.

## Warum diese Phase jetzt?

Bisher kennt unser Framework nur, was im LLM selbst "gelernt" wurde, plus das, was der
Agent aus Phase 4 sich während eines Laufs merkt. Für viele echte Anwendungen reicht das
nicht: Ein Assistent für ein Firmenhandbuch muss das *aktuelle* Handbuch kennen, nicht
irgendeinen Trainingsstand. Die Lösung heißt **Retrieval-Augmented Generation (RAG)** —
zu Deutsch etwa "generierungsunterstützt durch Suche": Wir laden eigene Dokumente,
zerlegen sie in durchsuchbare Stücke, finden bei einer Anfrage die passendsten Stücke und
reichen sie dem LLM als zusätzlichen Kontext.

Parallel dazu wird unser Framework in dieser Phase **betriebsreif**: Es bekommt eine
REST-Schnittstelle statt nur einer CLI, überlebt fehlerhafte Netzwerkantworten durch
Retry-Logik, macht sein Verhalten über Tracing nachvollziehbar, behandelt API-Keys als
das Geheimnis, das sie sind, und schützt sich gegen den Angriff, der RAG-Systeme
besonders trifft: Prompt Injection über eingebettete Dokumente.

## Lektionen

1. [Document Loader](01-document-loader.md) — Dokumente aus unterschiedlichen Quellen in
   ein einheitliches Modell laden.
2. [Chunking](02-chunking.md) — lange Dokumente in durchsuchbare Stücke zerlegen,
   Zusammenhang gegen Suchgranularität abwägen.
3. [Embeddings und Vector Store](03-embeddings-vector-store.md) — Text in Vektoren
   verwandeln und hinter einem austauschbaren Port speichern.
4. [Retriever und Quellenangaben](04-retriever-quellenangaben.md) — die passendsten
   Chunks finden und nachvollziehbar zitieren.
5. [REST mit Axum oder TUI](05-rest-axum-oder-tui.md) — `mein_server` als neue
   Anwendungsschicht neben `mein_cli`.
6. [Retry, Rate Limit und Backoff](06-retry-rate-limit-backoff.md) — Netzwerkfehler
   überleben, ohne den Anbieter zu bombardieren.
7. [Tracing, Kosten-Tracking und Secrets](07-tracing-kosten-secrets.md) — nachvollziehen,
   was passiert ist, was es gekostet hat, und wer welche Geheimnisse sieht.
8. [Prompt-Injection-Schutz, Docker und CI](08-security-docker-ci.md) — Retrieval-Inhalte
   als das behandeln, was sie sind: nicht vertrauenswürdige Daten.
9. [Release 5: operable-rag-service](09-release-5.md) — Definition of Done, Git-Tag,
   Ausblick auf Phase 6.

## Transferaufgabe der Phase

> Retrieval-Inhalte werden als untrusted data behandelt und dürfen keine Systemregeln
> überschreiben.

Ein RAG-System lädt fremde Dokumente in den Prompt hinein — und ein bösartig präpariertes
Dokument kann versuchen, sich als neue Systemanweisung auszugeben ("Ignoriere alle
vorherigen Anweisungen und ..."). Wir bearbeiten diese Aufgabe konkret in
[Lektion 8](08-security-docker-ci.md): Wir bauen dort zunächst eine verwundbare Variante,
zeigen den Angriff an einem deterministischen Test, und beheben ihn durch strukturelle
Trennung von System-, Retrieval- und Nutzeranteil im Prompt. Die Definition of Done in
[Lektion 9](09-release-5.md) prüft, ob dieser Schutz tatsächlich sitzt.

## Bewertungsraster für diese Phase

- **AI Engineering:** Retrieval wird als fehleranfälliger, nicht vertrauenswürdiger
  Systemteil behandelt, nicht als verlängerter Arm des Systemprompts.
- **Design:** Neue Ports (`DocumentLoader`, `VectorStore`, `Retriever`) bleiben generisch
  — konkrete Backends (Dateisystem, Qdrant, LanceDB) sind austauschbare Adapter.
- **Betrieb:** Retry/Backoff, Tracing, Kosten-Tracking und Secret-Handling sind Teil der
  Lösung, nicht nachträgliche Zutat.
- **Qualität:** `cargo fmt`, `cargo clippy --workspace` und Tests laufen sauber durch,
  inklusive eines Tests, der den Prompt-Injection-Angriff aktiv nachstellt.

## Voriger Release

Diese Phase baut auf [Release 4: tool-using-agent](../05-phase4-agenten/08-release-4.md)
auf: `mein_agent` mit Agent Loop, Tools und State existiert bereits, `mein_core` bietet
`port::LlmProvider` und `Runnable`. Phase 5 fügt Retrieval und Betriebsfähigkeit hinzu, ohne
diese Bausteine zu verändern.

Bereit? Los geht's mit [Lektion 1: Document Loader](01-document-loader.md).
