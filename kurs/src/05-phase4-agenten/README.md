# Phase 4 — Agenten & Concurrency

**Ergebnis dieser Phase:** Ein Agent kann einen Tool-Aufruf anfordern, das Ergebnis
beobachten und weiterarbeiten.

## Warum diese Phase jetzt?

Bisher hat unser Framework immer nur **einmal** angefragt und **einmal** geantwortet:
Konversation rein, eine Antwort raus (Phase 2), über einen sauber getrennten Port
(Phase 3). Ein Agent ist etwas anderes: Er darf sich selbst mehrfach hintereinander
Antworten holen, dazwischen Werkzeuge (**Tools**) aufrufen — eine Websuche, einen
Taschenrechner, eine Datenbankabfrage — und das Ergebnis wieder in seine nächste
Überlegung einfließen lassen. Diese Schleife (**Plan → Tool-Aufruf → Beobachtung →
nächste Aktion**) ist der Kern dessen, was ein "Agent" von einem einfachen Chat-Wrapper
unterscheidet.

Technisch bringt das zwei neue Anforderungen mit, die wir in Phase 1-3 bewusst
zurückgestellt haben: Wir wollen Antworten **streamen** können (Token für Token, statt
auf den ganzen Text zu warten), und wir wollen mehrere unabhängige Dinge (LLM-Aufruf,
Werkzeugausführung, Zeitlimits) **nebenläufig** koordinieren können, ohne dass unser
Programm dabei blockiert. Beides verlangt **asynchrones Rust** — Thema von Lektion 1,
und Voraussetzung für den Rest der Phase.

## Warum ein neues Crate, `mein_agent`?

Wir könnten den Agenten als weiteres Modul in `mein_core` unterbringen. Wir tun das
bewusst **nicht**. `mein_core` ist bislang eine schlanke Bibliothek ohne
Laufzeitabhängigkeit von `tokio` — jede Anwendung, die `mein_core` einbindet (auch eine
zukünftige Web-API in [Phase 5](../06-phase5-rag-betrieb/README.md), die vielleicht gar
keine volle Agenten-Maschinerie braucht, nur `Konversation` und `LlmProvider`), zahlt
sonst die Kompilierzeit und die Abhängigkeitslast von `tokio` und allem, was ein Agent
sonst noch braucht, mit — auch wenn sie nie einen Agenten benutzt. Ein eigenes Crate
`mein_agent` isoliert das: Es hängt von `mein_core` ab, nicht umgekehrt, und bleibt
optional. Wer nur `Konversation` und `LlmProvider` will, fügt `mein_core` hinzu und ist
fertig.

## Lektionen

1. [Async-Grundlagen und Tokio](01-async-und-tokio.md) — was `async`/`.await` wirklich
   bedeuten, und warum Rust dabei explizit bleibt, wo andere Sprachen automatisch
   "grüne Threads" starten.
2. [SSE-Streaming als Eventfolge](02-sse-streaming.md) — eine Antwort als Strom von
   Ereignissen statt als ein fertiges Ergebnis konsumieren.
3. [Tool-Schema und Function Calling](03-tool-schema-function-calling.md) — Werkzeuge so
   beschreiben, dass ein Sprachmodell sie anfordern kann.
4. [Der Agent Loop](04-agent-loop.md) — die Schleife Plan → Tool → Beobachtung → nächste
   Aktion, der Kern dieser Phase.
5. [State und Memory](05-state-und-memory.md) — was der Agent sich merkt, und wie dieser
   Zustand sauber über Schritte (und über nebenläufige Tasks) hinweg wandert.
6. [Abbruchbedingungen und Limits](06-abbruchbedingungen-limits.md) — wann ein Agent
   aufhören **muss**, nicht nur, wann er aufhören darf.
7. [Optionaler MCP-Client](07-mcp-client.md) — ein kurzer, bewusst optionaler Ausblick auf
   das Model Context Protocol.
8. [Release 4: tool-using-agent](08-release-4.md) — Definition of Done, Git-Tag, Ausblick
   auf Phase 5.

## Transferaufgabe der Phase

**Der Agent darf höchstens fünf Schritte ausführen und muss bei einem unbekannten Tool
sicher abbrechen.** Du bearbeitest diese Aufgabe konkret am Ende von
[Lektion 6](06-abbruchbedingungen-limits.md) und überprüfst deine Lösung anhand der
Definition of Done in [Lektion 8](08-release-4.md).

## Bewertungsraster für diese Phase

- **Rust-Grundlagen:** `async`/`.await`, Futures als "noch nicht fertige Werte", die
  Tokio-Runtime, `Send`/`Sync` bei geteiltem Zustand über Tasks hinweg.
- **Design:** saubere Grenze zwischen `mein_core` (schlank, kein `tokio`) und
  `mein_agent` (Agent-spezifisch, mit `tokio`); der Agent Loop baut auf `LlmProvider` und
  `Runnable` aus [Phase 3](../04-phase3-architektur/README.md) auf, statt sie zu
  ersetzen.
- **Qualität:** mindestens ein Test für den erfolgreichen Pfad, einer für das
  Schrittlimit und einer für ein unbekanntes Werkzeug — alle drei ohne Panic, alle drei
  mit einem sprechenden `Err`-Wert.

Vorheriger Release: [`provider-agnostic-core`, Phase 3](../04-phase3-architektur/08-release-3.md).

Bereit? Los geht's mit [Lektion 1: Async-Grundlagen und Tokio](01-async-und-tokio.md).
