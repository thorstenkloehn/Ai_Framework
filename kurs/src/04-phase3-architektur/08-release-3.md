# Lektion 8 · Release 3: provider-agnostic-core

## Rückblick

Aus einem `mein_core`, das genau einen LLM-Anbieter fest verdrahtet kannte, ist in sieben
Lektionen ein Framework geworden, das Anbieter und externe Systeme austauschen kann, ohne
seine eigene Logik anzufassen:

- `LlmProvider` als **Port**: ein selbst geschriebenes Trait, das den Vertrag "kann chatten"
  von jeder konkreten Technik trennt ([Lektion 1](01-llmprovider-port.md)).
- Eine Ordnerstruktur nach **Hexagonal Architecture** — `domain/`, `port/`, `adapter/` —, die
  diese Trennung im Dateisystem sichtbar macht, mit Re-Exports, die alte Importpfade am
  Leben halten ([Lektion 2](02-hexagonal-architecture.md)).
- Zwei Wege zu Polymorphismus, bewusst gewählt statt zufällig gemischt: generische Parameter
  für static dispatch, `Box<dyn LlmProvider>` für Ownership an Systemgrenzen
  ([Lektion 3](03-dyn-trait-ownership.md)).
- Ein `FakeProvider`, der Erfolg *und* Fehlerfälle simuliert, ohne je das Netzwerk zu
  berühren — inklusive eines vollständig deterministischen Timeout-Tests
  ([Lektion 4](04-fake-provider.md)).
- Integrationstests, die `mein_core` von außen prüfen, und `cargo clippy --workspace
  --all-targets -- -D warnings` als hartes Quality Gate statt gutem Vorsatz
  ([Lektion 5](05-tests-und-clippy.md)).
- Ein Golden Set mit Eigenschafts-Prüfungen statt exakter Textvergleiche, und ein kritischer
  Blick auf die Grenzen von LLM-as-Judge ([Lektion 6](06-golden-set-llm-judge.md)).
- `Runnable` mit assoziierten Typen als Chain Pattern, das Prompt-Aufbau, LLM-Aufruf und
  Nachbearbeitung zu einer typsicheren Pipeline verkettet
  ([Lektion 7](07-chain-pattern-runnable.md)).

## Definition of Done

Bevor wir taggen, prüfen wir wieder die Kriterien, die für **jede** Lektion in diesem Kurs
gelten (siehe [Wie dieser Kurs funktioniert](../00-einleitung/02-wie-dieser-kurs-funktioniert.md)):

- [ ] `cargo build --workspace` läuft ohne Fehler und ohne Warnungen.
- [ ] `cargo test --workspace` — alle Unit-Tests aus `mein_core` **und** die
      Integrationstests aus `mein_core/tests/` sind grün.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` läuft ohne Beanstandungen.
- [ ] `cargo fmt --check` zeigt keine Abweichungen.
- [ ] Der Fehlerpfad wurde bewusst ausgeführt: ein simulierter Timeout über `FakeProvider`,
      sowohl als Unit-Test ([Lektion 4](04-fake-provider.md)) als auch als Integrationstest
      ([Lektion 5](05-tests-und-clippy.md)).
- [ ] Die **Transferaufgabe der Phase** — *"Wir testen einen Timeout, ohne eine echte API
      aufzurufen"* — ist gelöst und läuft ohne Internetverbindung durch
      (`timeout_wird_ohne_echte_api_erkannt` bzw. `timeout_wird_von_aussen_sichtbar`).
- [ ] Mindestens ein Golden-Set-Fall mit mindestens zwei `Eigenschaft`-Prüfungen existiert
      und läuft (regulär gegen `FakeProvider`, optional `#[ignore]`-markiert gegen ein
      echtes Modell).
- [ ] Eine `Runnable`-Kette aus mindestens zwei Schritten kompiliert und liefert im Test das
      erwartete Ergebnis.

## Aufräumen vor dem Commit

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Der Release

```bash
git add .
git commit -m "Phase 3: LlmProvider als Port, Hexagonal Architecture, Fake-Provider, Golden Set, Runnable"
git tag provider-agnostic-core
```

Wie schon bei `conversation-in-memory`
([Phase 1](../02-phase1-fundament/07-release-1.md)) und `typed-provider-boundary`
([Phase 2](../03-phase2-llm-anbindung/08-release-2.md)) beschreibt der Tag-Name das
**fachliche** Ergebnis, nicht die verwendete Technik: `provider-agnostic-core` — ein Kern,
dem es egal ist, welcher Anbieter (oder welches Test-Double) hinter dem `LlmProvider`-Port
steckt.

## Ausblick auf Phase 4

Bisher ist jeder Aufruf an einen `LlmProvider` **synchron** — `chat` blockiert den
aufrufenden Thread, bis eine Antwort (oder ein Fehler) da ist. Das war bewusst so: Ein `dyn
LlmProvider` mit einer `async fn`-Methode bringt zusätzliche Komplexität mit sich (async
Traits sind in Rust bis heute nicht ganz so einfach wie synchrone), die wir uns für diese
Phase bewusst gespart haben, um uns auf Architektur statt Nebenläufigkeit zu konzentrieren.

[Phase 4](../05-phase4-agenten/README.md) ändert genau das: Wir lernen `async`/`await` und
`tokio` von Grund auf und machen `LlmProvider` asynchron. Dabei bleibt es nicht bei einem
reinen `async`-Anstrich auf `chat`: Ein Agent Loop denkt in ganzen Gesprächsverläufen, nicht
in einzelnen Anfragen, deshalb wandert die Methode zu einer Form, die direkt eine
`Konversation` entgegennimmt und eine `Nachricht` zurückgibt (Details und Begründung dazu in
[Phase 4, Lektion 4](../05-phase4-agenten/04-agent-loop.md)). Die gute Nachricht: Der
`FakeProvider` aus [Lektion 4](04-fake-provider.md) und das `dyn LlmProvider`-Muster aus
[Lektion 3](03-dyn-trait-ownership.md) bleiben als **Testphilosophie** bestehen — wir werden
in Phase 4 denselben Trick anwenden, um einen Agenten zu testen, ohne einen echten Agenten
loszulassen.

[Weiter: Phase 4 — Agenten & Concurrency](../05-phase4-agenten/README.md)
