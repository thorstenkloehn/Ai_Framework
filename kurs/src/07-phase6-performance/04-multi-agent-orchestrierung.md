# Lektion 4: Multi-Agent-Orchestrierung

## Problem

Unser Agent aus [Phase 4](../05-phase4-agenten/04-agent-loop.md) kann recherchieren,
Tools aufrufen und antworten — alles mit einem einzigen System-Prompt, einem einzigen
Tool-Set, einer einzigen Verantwortung. Solange die Aufgabe klein bleibt, funktioniert
das gut. Aber stell dir eine Aufgabe wie "Recherchiere Thema X und schreibe einen
strukturierten Bericht darüber" vor: Ein einzelner Agent müsste gleichzeitig gut im
Web-Suchen, gut im Bewerten von Quellen *und* gut im Schreiben sein — sein System-Prompt
wird zu einem Sammelsurium widersprüchlicher Anweisungen, und ein Fehler an einer Stelle
(schlechte Quelle) verschmutzt das Endergebnis, ohne dass klar ist, wo es passiert ist.
Die Lösung, die sich in der Praxis durchgesetzt hat: mehrere kleinere Agenten mit klar
getrennten Zuständigkeiten, so wie ein kleines Redaktionsteam aus Rechercheur*in und
Autor*in statt einer Person, die alles allein macht.

## Code (Zielbild)

```rust
pub struct Orchestrator {
    recherche_agent: Agent,
    schreib_agent: Agent,
}

impl Orchestrator {
    pub async fn bearbeiten(&mut self, auftrag: &str) -> Result<String, OrchestratorFehler> {
        let rechercheergebnis = self.recherche_agent.ausfuehren(auftrag).await?;
        let bericht = self
            .schreib_agent
            .ausfuehren(&format!("Schreibe einen Bericht auf Basis von:\n{rechercheergebnis}"))
            .await?;
        Ok(bericht)
    }
}
```

## Dekonstruktion

### Verantwortungsgrenzen statt einem Alleskönner

Der Kerngedanke von Multi-Agent-Orchestrierung ist nicht "mehr Agenten sind besser",
sondern **klare Verantwortungsgrenzen**. Jeder `Agent` (Struct aus
[Phase 4](../05-phase4-agenten/04-agent-loop.md)) bekommt: einen eigenen, fokussierten
System-Prompt, nur die Tools, die er für seine eine Aufgabe braucht (kein Schreib-Agent
braucht Websuche-Zugriff), und ein klar definiertes Ein-/Ausgabeformat zu seinen
Nachbarn. Das ist dieselbe Idee wie die Trennung zwischen `mein_core` und `mein_cli` aus
[Phase 1](../02-phase1-fundament/01-workspace-lesen.md) — nur auf Ebene von Agenten statt
Modulen: Jede Einheit hat eine Aufgabe, die sie versteht, und eine schmale Schnittstelle
nach außen.

### Warum hier ein einfaches sequenzielles Pipeline-Muster?

Es gibt anspruchsvollere Orchestrierungs-Muster: ein zentraler "Leitagent", der dynamisch
entscheidet, welcher Sub-Agent als Nächstes drankommt (Orchestrator-Worker-Muster), oder
mehrere Agenten, die sich gegenseitig kritisieren (Debate-Muster). Wir bauen bewusst das
einfachste tragfähige Muster: eine feste **Pipeline** — Recherche-Agent liefert an
Schreib-Agent, fertig. Der Grund: Komplexere Orchestrierung bringt mehr LLM-Aufrufe, mehr
Fehlerquellen und mehr Kosten (dazu mehr in [Lektion 5](05-kosten-latenz-qualitaet.md)) —
sie lohnt sich erst, wenn die einfache Pipeline nachweislich nicht reicht. Genau diese
Zurückhaltung ist selbst eine Design-Entscheidung, keine Verlegenheitslösung.

### Warum keine geteilte, gleichzeitig veränderbare Zustandsvariable?

Ein naheliegender, aber gefährlicher Ansatz wäre ein gemeinsamer, veränderbarer Zustand,
auf den beide Agenten schreibend zugreifen (z. B. ein `Arc<Mutex<Verlauf>>`, den beide
"nebenbei" aktualisieren). Das mag nach Zeitersparnis aussehen, verwischt aber genau die
Verantwortungsgrenze, die wir gerade eingezogen haben — und öffnet die Tür für
Nebenläufigkeitsfehler, die [Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md)
bereits als eine der größten Fehlerquellen in nebenläufigem Code beschrieben hat. Wir
übergeben stattdessen bewusst **Werte** zwischen Agenten (hier: ein `String` als
Rechercheergebnis) — jeder Agent besitzt seinen eigenen Zustand exklusiv, Kommunikation
läuft nur über explizite Rückgabewerte.

## Schritt-Reveal

**Schritt 1 — `Orchestrator` als Struct anlegen**, das zwei `Agent`-Instanzen besitzt
(siehe Zielbild). Jede Instanz wird beim Erzeugen des Orchestrators mit eigenem
System-Prompt und eigenem Tool-Set aus [Phase 4, Lektion 3](../05-phase4-agenten/03-tool-schema-function-calling.md)
konfiguriert — der Recherche-Agent bekommt z. B. ein Websuche-Tool, der Schreib-Agent
keines.

**Schritt 2 — `bearbeiten` implementieren** wie im Zielbild: sequenziell, Ergebnis des
einen als Eingabe des nächsten.

**Schritt 3 — Provoziere einen typischen Ownership-Fehler.** Versuche testweise, *einen
einzigen* `Agent` gleichzeitig für zwei unabhängige Teilaufgaben in zwei parallelen
`tokio::spawn`-Tasks zu verwenden:

```rust
let handle1 = tokio::spawn(async move { recherche_agent.ausfuehren("Thema A").await });
let handle2 = tokio::spawn(async move { recherche_agent.ausfuehren("Thema B").await });
```

```
error[E0382]: use of moved value: `recherche_agent`
  --> src/orchestrator.rs:12:47
   |
10 |     let handle1 = tokio::spawn(async move { recherche_agent.ausfuehren("Thema A").await });
   |                                              --------------- value moved here
11 |
12 |     let handle2 = tokio::spawn(async move { recherche_agent.ausfuehren("Thema B").await });
   |                                              ^^^^^^^^^^^^^^^ value used here after move
```

Der Compiler verweigert das zu Recht: `async move` nimmt den `Agent` in die erste Closure
auf — danach existiert er im umgebenden Scope nicht mehr. Das ist Ownership
([Kapitel 0](../01-grundlagen/04-daten-buendeln.md), vertieft in
[Phase 1](../02-phase1-fundament/README.md)) an einer neuen Stelle: Ein `Agent` kann
nicht gleichzeitig zwei unabhängigen Aufgaben "gehören". Die richtige Lösung ist nicht,
den Fehler mit `Arc<Mutex<Agent>>` zu übertünchen, sondern **zwei eigene
Agent-Instanzen** zu erzeugen, wenn wirklich parallelisiert werden soll — was exakt der
Verantwortungsgrenzen-Gedanke von oben noch einmal bestätigt.

**Schritt 4 — Korrektur: zwei unabhängige Instanzen, wenn Parallelität gewünscht ist.**

```rust
let handle1 = tokio::spawn(async move { recherche_agent_a.ausfuehren("Thema A").await });
let handle2 = tokio::spawn(async move { recherche_agent_b.ausfuehren("Thema B").await });
let (ergebnis_a, ergebnis_b) = tokio::join!(handle1, handle2);
```

`tokio::join!` (aus [Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md))
wartet auf beide Tasks gleichzeitig, statt nacheinander — sinnvoll genau dann, wenn zwei
Teilaufgaben *unabhängig* voneinander sind (zwei verschiedene Recherche-Themen), im
Gegensatz zu unserer Pipeline oben, wo der Schreib-Agent zwingend auf das
Rechercheergebnis wartet.

## Ausführung

```bash
cargo test -p mein_agent orchestrator
```

```
running 1 test
test orchestrator::tests::pipeline_reicht_rechercheergebnis_an_schreib_agent_weiter ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Nutze für den Test wieder Fake-Provider aus Phase 3 hinter beiden Agenten, damit keine
echten API-Kosten für den Testlauf anfallen — ein Muster, das dir jetzt zum dritten Mal in
diesem Kurs begegnet (Fehlerpfad-Tests in Phase 3, Routing-Tests in Lektion 3, jetzt hier).

## Zusammenfassung

- Multi-Agent-Orchestrierung teilt eine große, vage Aufgabe in mehrere kleine Agenten mit
  klaren, engen Verantwortungsgrenzen auf — nicht Selbstzweck, sondern Mittel gegen
  Prompt-Überladung und schwer lokalisierbare Fehler.
- Das einfachste Muster ist eine feste Pipeline; aufwändigere Muster (dynamischer
  Leitagent, Debate) lohnen sich erst, wenn die Pipeline nachweislich nicht ausreicht.
- Kommunikation zwischen Agenten läuft über explizit übergebene Werte, nicht über
  geteilten veränderbaren Zustand — das vermeidet Nebenläufigkeitsfehler und hält
  Verantwortungsgrenzen sauber.
- Wirkliche Parallelität (`tokio::join!`) braucht unabhängige Agent-Instanzen, keine
  geteilte einzelne Instanz — der Compiler erzwingt das über Ownership.

## Übung

Erweitere die Pipeline um einen dritten Agenten, einen `kritiker_agent`, der den Bericht
des `schreib_agent` bewertet und entweder freigibt oder mit konkretem Verbesserungshinweis
zurückschickt. Baue eine Schleife, die den Schreib-Agent mit dem Kritikpunkt erneut
aufruft — mit einer festen Obergrenze an Wiederholungen, nach dem Muster der
Abbruchbedingungen aus
[Phase 4, Lektion 6](../05-phase4-agenten/06-abbruchbedingungen-limits.md). Leitfrage: Wo
genau greift hier die dort gelernte Limit-Logik — beim einzelnen Agent Loop, oder auf der
Ebene deines neuen `Orchestrator`?

[Weiter: Lektion 5 — Kosten, Latenz und Qualität abwägen](05-kosten-latenz-qualitaet.md)
