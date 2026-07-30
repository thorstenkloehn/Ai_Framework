# Lektion 3: Model Routing und Fallback

## Problem

Nicht jede Anfrage an ein LLM ist gleich anspruchsvoll. "Ist dieser Satz Deutsch oder
Englisch?" braucht kein Spitzenmodell mit tiefem Reasoning — ein kleines, günstiges Modell
beantwortet das zuverlässig und in Millisekunden. Eine komplexe Zusammenfassung eines
Fachtexts dagegen schon eher. Würden wir *jede* Anfrage pauschal an das teuerste,
leistungsfähigste Modell schicken, zahlen wir für Klassifikationsaufgaben drauf, die ein
Bruchteil kosten könnten. Schicken wir dagegen *alles* pauschal an das günstige Modell,
scheitern die schwierigen Fälle. Die Metapher aus dem echten Leben: Der Hausarzt behandelt
die meisten Fälle selbst — ist er sich unsicher, überweist er kontrolliert an eine
Facharztpraxis, statt zu raten. Wir brauchen genau diese Überweisungslogik in Code.

## Code (Zielbild)

```rust
pub struct RoutingProvider {
    guenstig: Box<dyn LlmProvider>,
    hochwertig: Box<dyn LlmProvider>,
    konfidenz_schwelle: f64,
}

#[async_trait::async_trait]
impl LlmProvider for RoutingProvider {
    async fn chat(&self, konversation: &Konversation) -> Result<Nachricht, ProviderFehler> {
        let versuch = self.guenstig.chat(konversation).await?;

        if versuch.konfidenz() >= self.konfidenz_schwelle {
            Ok(versuch)
        } else {
            self.hochwertig.chat(konversation).await
        }
    }
}
```

## Dekonstruktion

### Strategy Pattern über `LlmProvider`

Das ist der entscheidende Trick: `RoutingProvider` implementiert **denselben** Trait
`LlmProvider`, den auch `guenstig` und `hochwertig` implementieren (den Port aus
[Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md)). Für jeden
aufrufenden Code — `mein_agent`, `mein_cli`, `mein_server` — ist `RoutingProvider`
ununterscheidbar von einem einzelnen, "normalen" Provider. Das ist das **Strategy
Pattern**: austauschbares Verhalten hinter einer gemeinsamen Schnittstelle, wobei die
konkrete Strategie (hier: "welches Modell wählen wir?") zur Laufzeit entschieden wird,
ohne dass Aufrufer*innen davon wissen müssen. Wir bauen hier keine neue Abstraktion — wir
nutzen die aus Phase 3 bereits vorhandene ein zweites Mal, jetzt zur *Komposition* statt
nur zum *Austausch*.

### Woher kommt "Konfidenz"?

Ein LLM liefert nicht von Natur aus einen "Ich bin mir zu 80 % sicher"-Wert mit. Wir
müssen ihn selbst erzeugen. Der pragmatischste Weg mit unseren bisherigen Werkzeugen: Wir
bitten das günstige Modell, strukturiert zu antworten — mit `schemars` aus
[Phase 2, Lektion 6](../03-phase2-llm-anbindung/06-structured-output.md) definieren wir
ein Antwortschema mit einem Konfidenzfeld, z. B. `{ "antwort": "...", "konfidenz": 0.92
}`. Das Modell schätzt seine eigene Sicherheit ein — unzuverlässig im Detail, aber
brauchbar genug für eine grobe Weiche. Alternativen, die wir hier bewusst *nicht* nutzen:
echte Token-Log-Wahrscheinlichkeiten (nicht jeder Anbieter liefert sie über die API) oder
ein separates, noch kleineres "Ist das schwierig?"-Modell (zusätzliche Komplexität, die
sich für unseren Kurs nicht lohnt).

### Warum `Box<dyn LlmProvider>` und nicht Generics?

[Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md) hat den
Unterschied bereits eingeführt: Mit Generics (`RoutingProvider<G: LlmProvider, H:
LlmProvider>`) müsste der konkrete Typ jedes Providers zur Kompilierzeit feststehen — wir
könnten also nicht zur Laufzeit (z. B. aus einer Konfigurationsdatei) entscheiden, welcher
Adapter als "günstig" eingesetzt wird. `Box<dyn LlmProvider>` erkauft sich diese
Laufzeit-Flexibilität mit einem kleinen, hier völlig vernachlässigbaren
Laufzeit-Overhead (virtueller Funktionsaufruf statt statischer). Für ein
Routing-Szenario, das ohnehin auf einen Netzwerkaufruf wartet, fällt das nicht ins
Gewicht.

## Schritt-Reveal

**Schritt 1 — `RoutingProvider` als Struct anlegen.** In `mein_core/src/routing.rs`
(neues Modul, wie in [Phase 3](../04-phase3-architektur/02-hexagonal-architecture.md)
festgelegt: Erweiterungen landen als eigenes Modul neben `domain/`, `port/`, `adapter/`):

```rust
pub struct RoutingProvider {
    guenstig: Box<dyn LlmProvider>,
    hochwertig: Box<dyn LlmProvider>,
    konfidenz_schwelle: f64,
}

impl RoutingProvider {
    pub fn neu(
        guenstig: Box<dyn LlmProvider>,
        hochwertig: Box<dyn LlmProvider>,
        konfidenz_schwelle: f64,
    ) -> Self {
        RoutingProvider { guenstig, hochwertig, konfidenz_schwelle }
    }
}
```

`cargo check -p mein_core` — kompiliert; `RoutingProvider` implementiert noch nicht
`LlmProvider`, das folgt im nächsten Schritt.

**Schritt 2 — `LlmProvider` implementieren** (siehe Zielbild oben) und Modul in
`mein_core/src/lib.rs` einbinden: `pub mod routing;`.

**Schritt 3 — Provoziere einen typischen Async-Trait-Fehler.** Entferne testweise `Send +
Sync` aus der `LlmProvider`-Trait-Definition (falls dein Trait sie wie in Phase 3 fordert)
und versuche, `RoutingProvider` in einem `tokio::spawn`-Kontext zu benutzen:

```
error: future cannot be sent between threads safely
  = help: within `impl Future<Output = Result<Nachricht, ProviderFehler>>`,
    the trait `Send` is not implemented for `dyn LlmProvider`
```

Diese Fehlermeldung begegnet dir in echten Rust-Codebasen mit `async`/`dyn Trait` sehr
häufig. Sie bedeutet: tokio darf eine asynchrone Aufgabe potenziell auf einem anderen
Thread fortsetzen — dafür muss alles, was über einen `.await`-Punkt hinweg "lebt"
(hier: unser `Box<dyn LlmProvider>`), sicher zwischen Threads wandern dürfen. Ohne `Send +
Sync` als Trait-Bound weiß der Compiler das nicht und verweigert die Kompilierung, statt
zur Laufzeit ein unsicheres Verhalten zu riskieren. Setze `Send + Sync` zurück — Phase 3
hat es aus genau diesem Grund von Anfang an gefordert.

**Schritt 4 — Test mit zwei Fake-Providern.** Nutze den `adapter::fake`-Testadapter aus
[Phase 3, Lektion 4](../04-phase3-architektur/04-fake-provider.md), einmal konfiguriert
mit hoher, einmal mit niedriger Konfidenz:

```rust
#[tokio::test]
async fn niedrige_konfidenz_loest_fallback_aus() {
    let guenstig = FakeProvider::mit_konfidenz(0.4);
    let hochwertig = FakeProvider::mit_konfidenz(0.95);
    let routing = RoutingProvider::neu(Box::new(guenstig), Box::new(hochwertig), 0.7);

    let konversation = Konversation::neu();
    let antwort = routing.chat(&konversation).await.unwrap();

    assert_eq!(antwort.inhalt, "antwort-vom-hochwertigen-modell");
}
```

## Ausführung

```bash
cargo test -p mein_core routing
```

```
running 2 tests
test routing::tests::hohe_konfidenz_bleibt_beim_guenstigen_modell ... ok
test routing::tests::niedrige_konfidenz_loest_fallback_aus ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Kein einziger echter Netzwerkaufruf nötig — genau der Vorteil des Fake-Providers aus
Phase 3, jetzt ein zweites Mal genutzt, um Routing-Logik ohne API-Kosten zu testen.

> **⚠️ Warnung**
>
> `RoutingProvider` ruft im schlechtesten Fall **zwei** Modelle statt eines auf (erst das
> günstige, dann bei Unsicherheit das teure) — das kostet zusätzliche Latenz. Für
> Anwendungen mit harten Zeitbudgets kann es sinnvoller sein, beide Aufrufe parallel zu
> starten und nur bei Bedarf auf das Ergebnis des teuren Modells zu warten. Wir bleiben in
> dieser Lektion bewusst beim einfacheren sequenziellen Ablauf; [Lektion 5](05-kosten-latenz-qualitaet.md)
> ordnet diesen Trade-off zwischen Kosten und Latenz genauer ein.

## Zusammenfassung

- Model-Routing ist eine Anwendung des Strategy Patterns: `RoutingProvider` implementiert
  `LlmProvider` und wrappt zwei weitere `LlmProvider`-Implementierungen — für Aufrufer*innen
  unsichtbar.
- "Konfidenz" ist kein eingebautes LLM-Feature, sondern ein von uns über Structured
  Output (Phase 2) angefordertes, selbst eingeschätztes Signal.
- `Box<dyn LlmProvider>` erlaubt Laufzeit-Flexibilität bei der Wahl der konkreten
  Provider — der Preis dafür ist ein vernachlässigbarer Overhead und die Pflicht zu
  `Send + Sync` in einem `async`-Kontext.
- Fallback-Logik lässt sich mit dem Fake-Provider aus Phase 3 vollständig ohne
  Netzwerkzugriff testen.

## Übung — Transferaufgabe der Phase

Für kurze Klassifikationen wird ein günstiges Modell gewählt; bei Unsicherheit erfolgt ein
kontrollierter Fallback. Baue dafür konkret einen Anwendungsfall: eine
Sprach-Erkennung ("Ist dieser Text Deutsch oder Englisch?") über `RoutingProvider`, bei
der das günstige Modell die Mehrheit der Fälle übernimmt und nur bei Konfidenzwerten
unter einer von dir gewählten Schwelle an ein stärkeres Modell eskaliert wird. Zwei
Leitfragen: Wie wählst du die Schwelle — und wie würdest du (ohne echte API-Kosten zu
zahlen) mit einem Golden Set aus
[Phase 3, Lektion 6](../04-phase3-architektur/06-golden-set-llm-judge.md) messen, ob
deine Schwelle zu hoch oder zu niedrig angesetzt ist? [Lektion 5](05-kosten-latenz-qualitaet.md)
greift diese Frage systematisch wieder auf.

[Weiter: Lektion 4 — Multi-Agent-Orchestrierung](04-multi-agent-orchestrierung.md)
