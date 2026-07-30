# Lektion 2: SSE-Streaming als Eventfolge

## Problem

Bisher wartet unser Framework, bis ein LLM seine **komplette** Antwort geschrieben hat,
bevor wir überhaupt etwas sehen — bei einer langen Antwort können das mehrere Sekunden
Stille sein. Fast jede moderne LLM-API bietet stattdessen einen Streaming-Modus: Statt
einer Antwort am Stück schickt der Server viele kleine Häppchen, sobald sie fertig sind
— "H", "allo", ", wie", " kann ich", " helfen?" — die wir sofort anzeigen können. Das
Protokoll dahinter heißt in aller Regel **Server-Sent Events (SSE)**: eine offene
HTTP-Verbindung, über die der Server fortlaufend Textereignisse schickt, bis er fertig
ist oder die Verbindung schließt.

## Code (Zielbild)

```rust
use futures_util::StreamExt;

async fn zeige_antwort_live(mut ereignisse: impl futures_util::Stream<Item = String> + Unpin) {
    while let Some(stueck) = ereignisse.next().await {
        print!("{stueck}");
    }
    println!();
}
```

## Dekonstruktion

### SSE — ein Textformat, keine Magie

Ein SSE-Ereignis ist reiner Text, zeilenweise aufgebaut, durch eine Leerzeile beendet:

```
data: {"token":"Hallo"}

data: {"token":", wie"}

data: [DONE]

```

Jede Zeile beginnt mit `data:`, gefolgt vom eigentlichen Inhalt (hier: JSON-Schnipsel
mit einem Token). Viele APIs beenden den Strom mit einer Sentinel-Zeile wie `data:
[DONE]`. SSE ist bewusst simpel gehalten — im Kern ein `text/event-stream`, das über eine
normale, aber offen gehaltene HTTP-Antwort läuft. Das unterscheidet es von WebSockets
(bidirektional, eigenes Binärprotokoll): SSE ist reine **Einbahnstraße** vom Server zu
uns, textbasiert, und genügt für "das LLM schickt uns Tokens" vollständig.

### Ein `Stream` ist ein asynchroner Iterator

Aus [Kapitel 0](../01-grundlagen/03-kontrollfluss.md) kennst du `for`-Schleifen über
`Vec`s — im Hintergrund holt sich Rust dort über `Iterator::next()` ein Element nach dem
anderen, bis `None` kommt. Ein `Stream` (aus dem Crate `futures_util`, das wir in
[Lektion 1](01-async-und-tokio.md) noch nicht brauchten) ist genau dieselbe Idee, nur
asynchron: `next()` gibt kein `Option<T>` **sofort** zurück, sondern ein `Future`, das
sich zu einem `Option<T>` auflöst — es kann ja sein, dass das nächste Stück noch gar
nicht angekommen ist und wir darauf warten müssen. Deshalb `.next().await` statt nur
`.next()`.

```rust
while let Some(stueck) = ereignisse.next().await {
    print!("{stueck}");
}
```

`while let Some(...) = ...` kennst du als Muster schon — hier läuft die Schleife, solange
noch Ereignisse kommen, und endet automatisch, sobald der Stream `None` liefert (Verbindung
zu Ende oder `[DONE]` erreicht).

### Wo ein solcher Stream in unserem Framework herkäme

`reqwest` (aus [Phase 2](../03-phase2-llm-anbindung/01-http-grenze-reqwest.md)) bietet
neben der einfachen "warte auf die ganze Antwort"-Methode auch einen **asynchronen**
Client, dessen Antwortkörper sich als Byte-Strom abholen lässt
(`response.bytes_stream()`). Ein solcher Byte-Strom liefert rohe Netzwerk-Häppchen, die
nicht zwingend an Zeilengrenzen enden — die eigentliche Arbeit ist, daraus vollständige
SSE-Zeilen (`data: ...\n\n`) zusammenzusetzen, jede Zeile zu parsen und nur den
eigentlichen Inhalt (z. B. das Token-Feld aus dem JSON) weiterzureichen. Skizziert sieht
das etwa so aus:

```rust
use futures_util::StreamExt;

async fn stream_antwort(response: reqwest::Response) -> impl futures_util::Stream<Item = String> {
    response
        .bytes_stream()
        .filter_map(|ergebnis| async move {
            let bytes = ergebnis.ok()?;
            let text = String::from_utf8_lossy(&bytes);
            text.strip_prefix("data: ")
                .map(|inhalt| inhalt.trim().to_string())
        })
}
```

> **💡 Tipp**
>
> Dieser Code ist bewusst eine **Skizze**, kein produktionsreifer Parser — echte
> SSE-Daten können über mehrere Netzwerk-Häppchen verteilt ankommen, mehrzeilig sein oder
> `[DONE]` enthalten, was wir hier separat abfangen müssten. Die Idee zählt: Aus einem
> rohen Byte-Strom wird über `.filter_map(...)` ein Strom bedeutungsvoller Text-Ereignisse.
> Wo genau dieser Code landet — als neue Methode auf `LlmProvider` aus
> [Phase 3](../04-phase3-architektur/01-llmprovider-port.md), oder als eigene
> Erweiterung dort — ist eine Entscheidung, die über den Rahmen dieser Lektion
> hinausgeht; hier lernst du das Konsumieren eines Streams, nicht seinen kompletten
> Aufbau.

### Warum überhaupt streamen, wenn wir doch sowieso auf die Antwort warten?

Zwei Gründe. Erstens, Nutzererfahrung: Menschen empfinden "das Modell tippt gerade"
spürbar schneller als dieselbe Gesamtzeit als eine Sekunde Stille gefolgt von einem
Textblock. Zweitens, und wichtiger für den [Agent Loop](04-agent-loop.md): Ein Agent
kann so schon reagieren, sobald er erkennt, dass ein Tool-Aufruf im Stream auftaucht,
statt zwingend auf das allerletzte Token zu warten.

## Schritt-Reveal

**Schritt 1 — Abhängigkeit ergänzen.** In `mein_agent/Cargo.toml`:

```toml
[dependencies]
mein_core = { path = "../mein_core" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
futures-util = "0.3"
```

**Schritt 2 — Provoziere den Fehler bewusst.** Lege
`mein_agent/examples/stream_lesen.rs` an, aber vergiss das `.await`:

```rust
use futures_util::{stream, StreamExt};

#[tokio::main]
async fn main() {
    let mut ereignisse = stream::iter(vec!["Hallo".to_string(), ", Welt".to_string()]);

    while let Some(stueck) = ereignisse.next() {
        print!("{stueck}");
    }
    println!();
}
```

`cargo check -p mein_agent --example stream_lesen` meldet:

```
error[E0308]: mismatched types
 --> mein_agent/examples/stream_lesen.rs:6:25
  |
6 |     while let Some(stueck) = ereignisse.next() {
  |                    ------      ^^^^^^^^^^^^^^^^ expected `Option<_>`, found `Next<'_, Iter<...>>`
  |                    |
  |                    expected due to this
  |
help: use `.await` to get the resolved value
  |
6 |     while let Some(stueck) = ereignisse.next().await {
  |                                                +++++++
```

`ereignisse.next()` liefert nicht direkt ein `Option<String>`, sondern ein `Future`, das
sich erst *nach* `.await` zu einem `Option<String>` auflöst — der Compiler erkennt das
Muster sogar und schlägt selbst `.await` als Korrektur vor (`help: use \`.await\` ...`).
Das ist derselbe Denkfehler wie in [Lektion 1](01-async-und-tokio.md), nur an einer
anderen Stelle: Ein asynchroner Wert bleibt ein "noch nicht fertiger" Wert, egal ob er
aus einer `async fn` oder aus `Stream::next()` kommt.

**Schritt 3 — Korrektur.**

```rust
use futures_util::{stream, StreamExt};

#[tokio::main]
async fn main() {
    let mut ereignisse = stream::iter(vec!["Hallo".to_string(), ", Welt".to_string()]);

    while let Some(stueck) = ereignisse.next().await {
        print!("{stueck}");
    }
    println!();
}
```

## Ausführung

```bash
cargo run -p mein_agent --example stream_lesen
```

```
Hallo, Welt
```

## Zusammenfassung

- SSE ist ein einfaches, textbasiertes Protokoll: Zeilen wie `data: ...`, durch eine
  Leerzeile getrennt, oft mit einer `[DONE]`-Sentinel-Zeile am Ende.
- `Stream` ist die asynchrone Entsprechung von `Iterator` — `next()` gibt ein `Future`
  zurück, `.await` treibt es an und entpackt das nächste `Option<Item>`.
- Aus rohen Netzwerk-Bytes (`reqwest`s `bytes_stream()`) wird über `.filter_map(...)` ein
  Strom bedeutungsvoller Ereignisse — dasselbe Grundmuster, egal wie kompliziert das
  Parsing im Detail wird.
- Streaming ist mehr als ein UX-Bonus: Der Agent Loop kann auf einen erkannten
  Tool-Aufruf reagieren, ohne auf den letzten Token zu warten.

## Übung

Erweitere `stream_lesen.rs` um einen Strom, der testweise auch ein Element `"[DONE]"`
enthält. Passe die Schleife so an, dass sie beim Erreichen von `"[DONE]"` sauber
**aufhört**, ohne das Sentinel-Element selbst noch auszugeben (Hinweis: `break` innerhalb
der `while let`-Schleife, sobald du das Sentinel erkennst — schau dir dafür noch einmal
[Kontrollfluss](../01-grundlagen/03-kontrollfluss.md) an). Überlege dir zusätzlich: Was
sollte passieren, wenn die Verbindung **mittendrin** abbricht, bevor `[DONE]` je kommt —
ist das derselbe Fall wie ein sauberes Ende, oder ein Fehlerfall, den du unterscheiden
solltest? Wir greifen genau diese Frage in
[Lektion 6](06-abbruchbedingungen-limits.md) wieder auf.

[Weiter: Lektion 3 — Tool-Schema und Function Calling](03-tool-schema-function-calling.md)
