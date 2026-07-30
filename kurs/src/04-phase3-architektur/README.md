# Phase 3 — Architektur & Qualitätssicherung

**Ziel dieser Phase:** Anbieter und externe Systeme sind austauschbar und testbar.

## Warum diese Phase jetzt?

Am Ende von [Phase 2](../03-phase2-llm-anbindung/README.md) spricht `mein_core` mit genau
**einem** LLM-Anbieter, über einen konkreten Client, der `reqwest` direkt kennt. Das
funktioniert — aber es hat zwei Haken. Erstens: Jeder Test, der diesen Client benutzt,
braucht entweder eine echte Internetverbindung und einen echten API-Key, oder er testet gar
nichts. Zweitens: Wollen wir morgen einen zweiten Anbieter unterstützen, oder in
[Phase 4](../05-phase4-agenten/README.md) einen Agenten bauen, der mehrere Anfragen
orchestriert, müsste jede Stelle im Code, die bisher den konkreten Client kennt, angefasst
werden.

Die Lösung aus der Softwarearchitektur heißt **Hexagonal Architecture** (auch *Ports and
Adapters* genannt): Wir ziehen eine klare Grenze zwischen dem, *was* unser Framework mit
einem LLM-Anbieter tun können muss (ein **Port** — ein Vertrag, ausgedrückt als Rust-Trait),
und *wie* das im Detail passiert (ein **Adapter** — eine konkrete Implementierung dieses
Vertrags). Der große Gewinn: Sobald der Vertrag feststeht, können wir für Tests einen
**Fake-Adapter** schreiben, der gar keine echte API anspricht — und trotzdem exakt denselben
Code testen, der später mit dem echten Adapter läuft.

## Lektionen

1. [LlmProvider als Port](01-llmprovider-port.md) — den konkreten Phase-2-Client hinter
   einem Trait verstecken.
2. [Hexagonal Architecture](02-hexagonal-architecture.md) — `domain/`, `port/`, `adapter/`:
   die Ordnerstruktur, die die Architektur sichtbar macht.
3. [dyn Trait und Ownership an der Grenze](03-dyn-trait-ownership.md) — generische
   Parameter vs. Trait-Objekte, und wer einen Provider eigentlich *besitzt*.
4. [Fake-Provider für Unit-Tests](04-fake-provider.md) — ein Test-Adapter, der nie das
   Netzwerk anfasst.
5. [Integrationstests und clippy](05-tests-und-clippy.md) — die öffentliche API von außen
   prüfen, und automatisierte Codequalität als Gate statt als guten Vorsatz.
6. [Golden Set und LLM-as-Judge](06-golden-set-llm-judge.md) — wie man die Qualität von
   LLM-Antworten überhaupt testen kann, und wo dieser Ansatz an seine Grenzen stößt.
7. [Chain Pattern mit Runnable](07-chain-pattern-runnable.md) — Prompt, LLM-Aufruf und
   Parser zu einer wiederverwendbaren Pipeline verketten.
8. [Release 3: provider-agnostic-core](08-release-3.md) — Definition of Done, Git-Tag,
   Ausblick auf Phase 4.

## Transferaufgabe der Phase

**Wir testen einen Timeout, ohne eine echte API aufzurufen.** Ein Netzwerk-Timeout ist einer
der häufigsten Fehlerfälle, die ein LLM-Framework im Betrieb erlebt — und einer der
unangenehmsten, um ihn zu testen: Ihn mit einer echten API zu provozieren wäre langsam,
unzuverlässig und würde Kosten verursachen. Mit dem Fake-Provider aus
[Lektion 4](04-fake-provider.md) lösen wir das, ohne je das Netzwerk zu berühren, und
prüfen die fertige Lösung als Integrationstest in
[Lektion 5](05-tests-und-clippy.md). Die Definition of Done in
[Lektion 8](08-release-3.md) verlangt diesen Test ausdrücklich.

## Bewertungsraster für diese Phase

- **Rust-Grundlagen:** ein selbst geschriebenes (nicht per `derive` erzeugtes) Trait,
  `dyn Trait` vs. generische Parameter, `Box<dyn Trait>`, ein Modulsystem über mehrere
  Dateien und Ordner hinweg, assoziierte Typen (`type Input`, `type Output`).
- **Design:** die Trennung zwischen `domain/`, `port/` und `adapter/` bleibt während der
  ganzen Phase sauber — nichts in `domain/` weiß etwas von `reqwest` oder HTTP.
- **Qualität:** ein Fake-Adapter deckt Erfolgs- *und* Fehlerfälle ab (insbesondere Timeout),
  Integrationstests prüfen die öffentliche API von außen, `cargo clippy --workspace
  --all-targets -- -D warnings` läuft ohne Beanstandungen.

Der vorherige Release-Tag war `typed-provider-boundary` (siehe
[Phase 2, Release 2](../03-phase2-llm-anbindung/08-release-2.md)). Bereit? Los geht's mit
[Lektion 1: LlmProvider als Port](01-llmprovider-port.md).
