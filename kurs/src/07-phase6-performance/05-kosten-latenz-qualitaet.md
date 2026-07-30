# Lektion 5: Kosten, Latenz und Qualität abwägen

## Problem

Wir haben jetzt zwei mächtige Werkzeuge gebaut: Model-Routing ([Lektion 3](03-model-routing-fallback.md))
und Multi-Agent-Orchestrierung ([Lektion 4](04-multi-agent-orchestrierung.md)). Beide
verändern das Verhalten unseres Systems entlang dreier Achsen gleichzeitig — **Kosten**
(wie viel kostet ein Aufruf), **Latenz** (wie lange dauert er) und **Qualität** (wie gut
ist das Ergebnis). Das Problem: Diese drei Achsen stehen oft im Widerspruch zueinander.
Das teuerste Modell liefert meist die beste Qualität, aber auch die höchste Latenz und
die höchsten Kosten. Ein Fallback wie in Lektion 3 verbessert Qualität in Unsicherheitsfällen,
kostet aber im schlechtesten Fall zwei Aufrufe statt einem. Eine Multi-Agent-Pipeline aus
Lektion 4 verbessert oft die Ergebnisqualität, vervielfacht aber Latenz und Kosten pro
Anfrage. Wer nur eine Achse optimiert, verschlechtert typischerweise die anderen — wir
brauchen einen Weg, alle drei **gemeinsam** zu bewerten.

## Code (Zielbild)

```rust
pub struct Bewertung {
    pub kosten_usd: f64,
    pub latenz_ms: u128,
    pub qualitaet: f64, // 0.0 .. 1.0, aus dem LLM-as-Judge-Score
}

pub struct Gewichtung {
    pub kosten: f64,
    pub latenz: f64,
    pub qualitaet: f64,
}

impl Bewertung {
    pub fn punkte(&self, gewichtung: &Gewichtung) -> f64 {
        let kosten_score = 1.0 - self.kosten_usd.min(1.0);
        let latenz_score = 1.0 - (self.latenz_ms as f64 / 10_000.0).min(1.0);
        gewichtung.kosten * kosten_score
            + gewichtung.latenz * latenz_score
            + gewichtung.qualitaet * self.qualitaet
    }
}
```

## Dekonstruktion

### Drei Achsen, drei Quellen, die wir schon haben

Der entscheidende Punkt dieser Lektion: Wir bauen **keine neue Messtechnik**. Alle drei
Werte kommen aus bereits vorhandenen Bausteinen:

- **Kosten** — aus dem Kosten-Tracking (`Kostenschaetzung`), das
  [Phase 5, Lektion 7](../06-phase5-rag-betrieb/07-tracing-kosten-secrets.md) eingeführt
  hat.
- **Latenz** — mit derselben Denkweise wie in
  [Lektion 1](01-benchmarks-criterion.md) gemessen, hier aber nicht als isolierter
  Micro-Benchmark, sondern als End-zu-Ende-Zeit eines echten Aufrufs (`std::time::Instant`
  um den gesamten `chat`-Aufruf).
- **Qualität** — aus dem Golden-Set-Verfahren mit LLM-as-Judge aus
  [Phase 3, Lektion 6](../04-phase3-architektur/06-golden-set-llm-judge.md), das schon
  einen Score zwischen 0 und 1 pro Antwort liefert.

Diese Lektion verbindet drei bestehende Werkzeuge zu einer gemeinsamen Entscheidungsbasis
— genau das ist der rote Faden von Phase 6: kein neues fachliches Feature, sondern
Messbarkeit über das Bestehende hinweg.

### Warum eine gewichtete Summe, und warum keine "richtige" Gewichtung?

`Gewichtung` macht explizit, dass es keine objektiv "beste" Balance zwischen Kosten,
Latenz und Qualität gibt — sie hängt vom Produkt ab. Ein interner Klassifikator, der
tausendfach pro Sekunde läuft, gewichtet Kosten und Latenz hoch und Qualität niedriger
(kleine Fehlerquote ist tolerierbar). Eine juristische Zusammenfassung, die einmal pro
Anfrage läuft und bei Fehlern reale Konsequenzen hat, gewichtet Qualität fast
ausschließlich. Der Code macht diese Entscheidung **sichtbar und parametrisierbar** —
statt sie implizit im Kopf einer einzelnen Person zu verstecken, die einmal "das teure
Modell" gewählt hat und nie wieder hinterfragt, warum.

### Warum `.min(1.0)` bei den Scores?

`kosten_score` und `latenz_score` sind bewusst auf den Bereich `0.0..=1.0` begrenzt (ein
Wert über einer gewählten Obergrenze zählt als "genauso schlecht wie die Obergrenze",
nicht als "unendlich schlecht"). Ohne diese Begrenzung könnte ein einziger extrem teurer
oder extrem langsamer Ausreißer die gesamte gewichtete Summe dominieren und alle anderen
Kriterien wertlos machen — ein Klassiker bei jeder Kennzahlen-Kombination, nicht nur bei
LLM-Aufrufen.

## Schritt-Reveal

**Schritt 1 — `Bewertung` und `Gewichtung` anlegen** (siehe Zielbild), in einem neuen
Modul, z. B. `mein_core::evaluation` (ergänzt die bestehende Golden-Set-Infrastruktur aus
Phase 3, nicht ersetzt sie).

**Schritt 2 — `punkte` implementieren** wie im Zielbild.

**Schritt 3 — Provoziere eine typische Clippy-Warnung.** Vergleiche testweise zwei
`f64`-Bewertungen direkt mit `==`:

```rust
if bewertung.qualitaet == 0.9 { /* ... */ }
```

```bash
cargo clippy -p mein_core
```

```
warning: strict comparison of `f64` or `f32`
  --> src/evaluation.rs:23:8
   |
23 |     if bewertung.qualitaet == 0.9 {
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: `#[warn(clippy::float_cmp)]` on by default
   = help: consider comparing them within some margin of error
```

Gleitkommazahlen (`f64`) sind intern nicht exakt darstellbar — zwei rechnerisch
"gleiche" Werte können durch minimale Rundungsfehler unterschiedlich sein. clippy warnt
deshalb bei jedem direkten `==`-Vergleich von Fließkommazahlen. Die Korrektur: eine
Toleranzgrenze, z. B. `(bewertung.qualitaet - 0.9).abs() < 0.0001`.

**Schritt 4 — Zwei Konfigurationen gegenüberstellen.** Baue einen kleinen Vergleich, der
`RoutingProvider` aus Lektion 3 gegen einen Provider stellt, der immer nur das teure
Modell nutzt, über dasselbe Golden Set laufen lässt und beide `Bewertung`en ausgibt.

## Ausführung

```bash
cargo test -p mein_core evaluation -- --nocapture
```

```
Konfiguration          Kosten (USD)   Latenz (ms)   Qualität   Punkte (Standard-Gewichtung)
immer-hochwertig        0.0420          1840          0.94       0.71
routing-mit-fallback     0.0110           920           0.89       0.86
```

In diesem Beispiel gewinnt `routing-mit-fallback` trotz leicht niedrigerer Qualität, weil
die Standard-Gewichtung Kosten und Latenz stark mitzählt. Ändere probeweise die
`Gewichtung` so, dass `qualitaet` dominiert (z. B. `0.1, 0.1, 0.8`) und beobachte, wie sich
die Rangfolge umkehrt — genau das ist der Sinn dieser Lektion: **Es gibt keine feste
richtige Antwort**, nur eine explizit gemachte Entscheidung.

> **💡 Tipp**
>
> Führe diesen Vergleich regelmäßig gegen dein Golden Set aus, nicht nur einmalig — ändert
> sich ein Anbieter-Preis oder ein Modell-Update die Antwortqualität, verschiebt sich die
> Rangfolge unter Umständen völlig, ohne dass du eine Zeile eigenen Codes geändert hast.

## Zusammenfassung

- Kosten, Latenz und Qualität stehen typischerweise im Zielkonflikt — die Optimierung
  einer einzelnen Achse verschlechtert oft die anderen.
- Alle drei Messgrößen entstehen aus bereits vorhandenen Bausteinen (Kosten-Tracking aus
  Phase 5, Zeitmessung wie in Lektion 1, Golden-Set-Qualität aus Phase 3) — Phase 6 fügt
  keine neue Infrastruktur hinzu, sondern verbindet die vorhandene.
- Eine gewichtete Bewertung macht die Priorisierung zwischen den drei Achsen explizit und
  parametrisierbar, statt sie implizit einer einzelnen Modellwahl zu überlassen.
- `clippy::float_cmp` erinnert daran, Gleitkommazahlen nie mit `==`, sondern mit einer
  Toleranzgrenze zu vergleichen.

## Übung

Ergänze `Bewertung` um eine vierte Achse: **Sicherheit**, gemessen z. B. als Anteil der
Golden-Set-Fälle, bei denen der Prompt-Injection-Schutz aus
[Phase 5, Lektion 8](../06-phase5-rag-betrieb/08-security-docker-ci.md) korrekt gegriffen
hat. Erweitere `Gewichtung` entsprechend und wiederhole den Vergleich aus diesem
Kapitel — insbesondere für die `RoutingProvider`-Konfiguration aus
[Lektion 3](03-model-routing-fallback.md): Ändert eine neue vierte Achse ihre Bewertung
gegenüber der reinen Hochwertig-Konfiguration?

## Übergang zu Phase 7

Damit ist der fachliche Teil von Phase 6 abgeschlossen — und mit ihm die Transferaufgabe
der Phase (Lektion 3: ein günstiges Modell für kurze Klassifikationen, kontrollierter
Fallback bei Unsicherheit) ist gelöst und über die Kosten-Latenz-Qualität-Bewertung aus
dieser Lektion sogar belegbar, nicht nur behauptet.

Wie in der [Phasen-Übersicht](README.md) angekündigt, setzen wir hier **keinen Git-Tag**.
Wir haben keine neue öffentliche Fähigkeit ausgeliefert, sondern Messbarkeit — Benchmarks,
Property-Tests, Routing, Orchestrierung, eine gemeinsame Bewertung — über das bestehende
System aus Release 5 gelegt. Der Code funktioniert für Nutzer*innen nach außen exakt wie
zuvor, nur besser verstanden und begründet.

[Phase 7](../08-phase7-release/README.md) macht daraus jetzt den letzten Schritt: eine
öffentliche, stabile, dokumentierte API, die wir tatsächlich als Crate veröffentlichen
können. Wir bauen dafür ein ergonomisches Builder Pattern für die Konfiguration, führen
Feature Flags ein (damit niemand `mein_rag` oder `mein_agent` mitkompilieren muss, der sie
nicht braucht), dokumentieren mit rustdoc, klären SemVer-Regeln für zukünftige
Änderungen — und setzen am Ende den finalen Tag: `ai-framework-0.1.0`.

[Weiter: Phase 7 — Öffentliches Release](../08-phase7-release/README.md)
