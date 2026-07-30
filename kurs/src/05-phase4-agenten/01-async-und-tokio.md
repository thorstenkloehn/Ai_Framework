# Lektion 1: Async-Grundlagen und Tokio

## Problem

Stell dir einen Kellner vor, der pro Tisch arbeitet wie ein Anfänger: Bestellung
aufnehmen, dann **wartend** vor der Küche stehen bleiben, bis das Essen fertig ist, erst
dann zum nächsten Tisch gehen. Ein erfahrener Kellner nimmt stattdessen an Tisch 1 die
Bestellung auf, gibt sie in der Küche ab, geht **währenddessen** zu Tisch 2, nimmt dort
auf — und holt das Essen für Tisch 1 ab, sobald es fertig ist, ohne dass er die ganze
Zeit wartend davorstand.

Bis jetzt programmiert unser Framework wie der Anfänger: Ein Aufruf an ein LLM (Phase 2)
war ein synchroner, blockierender Funktionsaufruf — das Programm stand still, bis die
Antwort da war. Für eine einzelne Anfrage-Antwort-Runde reicht das. Ein Agent aber
braucht mehr: Er will Antworten **Token für Token streamen** (nächste Lektion), er will
vielleicht ein Zeitlimit setzen, während er auf eine Antwort wartet, und in späteren
Phasen soll er mehrere Dinge gleichzeitig koordinieren können. Dafür brauchen wir
**asynchrones Rust**.

## Code (Zielbild)

```rust
use tokio::time::{sleep, Duration};

async fn stelle_frage(text: &str) -> String {
    // steht hier stellvertretend für einen echten, wartenden Netzwerkaufruf
    sleep(Duration::from_millis(500)).await;
    format!("Antwort auf: {text}")
}

#[tokio::main]
async fn main() {
    let antwort = stelle_frage("Wie spät ist es?").await;
    println!("{antwort}");
}
```

## Dekonstruktion

### `async fn` — eine Funktion, die eine Pause einlegen darf

`async` vor `fn` verändert, was die Funktion zurückgibt: `stelle_frage` gibt **nicht**
direkt einen `String` zurück, sondern einen Wert vom Typ `impl Future<Output = String>`.
Ein **`Future`** ("Zukünftiges") ist ein Wert, der *noch nicht fertig* ist — ein
Versprechen, dass irgendwann, wenn man ihn ausreichend oft "anstößt" (man nennt das
*pollen*), ein `String` dabei herauskommt. Beim Aufruf von `stelle_frage(...)` passiert
noch **gar nichts** — der Funktionskörper startet erst, wenn das zurückgegebene `Future`
tatsächlich ausgeführt wird. Das ist ein wichtiger Unterschied zu jeder Funktion, die du
bisher geschrieben hast: `async fn` ist *träge* (*lazy*), eine normale `fn` ist sofort
aktiv.

### `.await` — "hol mir das fertige Ergebnis, und gib in der Zwischenzeit ab"

```rust
let antwort = stelle_frage("Wie spät ist es?").await;
```

`.await` treibt ein `Future` tatsächlich an, bis es fertig ist, und entpackt dann den
Wert (hier: `String`) daraus — ganz ähnlich wie `?` ein `Result` entpackt (siehe
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)). Der entscheidende
Unterschied zu einem blockierenden Aufruf: Während dieses `Future` auf etwas wartet (z. B.
eine Netzwerkantwort oder, wie hier, `sleep`), kann der Rest des Programms — andere
Futures, andere Aufgaben — in der Zwischenzeit weiterlaufen. Der Kellner steht nicht
wartend vor der Küche, er bedient Tisch 2.

> **💡 Tipp**
>
> `async`/`await` in Rust sind reine Syntax, keine Magie: Der Compiler übersetzt eine
> `async fn` in eine Zustandsmaschine (*state machine*), die bei jedem `.await`-Punkt
> anhalten und später genau dort weitermachen kann. Du musst diese Übersetzung nicht im
> Detail verstehen — aber es erklärt, warum ein `Future` "kalt" ist, bis es angetrieben
> wird: Die Zustandsmaschine existiert, aber niemand hat sie noch einen Schritt
> weiterlaufen lassen.

### Warum Rust async **explizit** macht — anders als Go

Sprachen wie Go starten "grüne Threads" (*goroutines*) automatisch: Jeder Funktionsaufruf
*könnte* pausieren und den Prozessor an etwas anderes abgeben, ohne dass das im Code
sichtbar ist. Rust geht bewusst den anderen Weg: Ob eine Funktion pausieren kann, steht
im Typ (`async fn`, Rückgabetyp `impl Future`), und *wo* sie pausiert, steht im Code
(jedes `.await`). Das hat einen Grund, der zu Rusts Gesamtphilosophie passt (siehe
[Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md)): Rust will,
dass Kosten und Verhalten **sichtbar** sind, nicht implizit im Hintergrund geschehen.
Ein zusätzlicher praktischer Vorteil: Rust selbst liefert **keine** eingebaute Laufzeit
(*runtime*) für Futures mit — anders als Go, wo der Goroutine-Scheduler fest im Compiler
und Laufzeitsystem verankert ist. Das bedeutet: Rust-Code, der `async`/`await` benutzt,
kann grundsätzlich auf ganz unterschiedlichen Umgebungen laufen (ein Server mit vielen
CPU-Kernen, ein eingebettetes Gerät, sogar der Browser via WebAssembly) — solange es
irgendeinen **Executor** gibt, der die Futures tatsächlich antreibt. Diesen Executor
liefern wir selbst dazu: **Tokio**.

### Tokio — die Laufzeit, die Futures tatsächlich antreibt

Ein `Future`, das niemand pollt, tut nichts — für immer. **Tokio** ist die im
Rust-Ökosystem verbreitetste **Async-Runtime**: Sie bringt einen Scheduler mit, der
Futures antreibt, mehrere davon nebenläufig verwaltet (auch über mehrere
Betriebssystem-Threads verteilt) und Werkzeuge wie `sleep`, Timeouts, Netzwerk-Sockets
und Synchronisationsprimitive bereitstellt, die selbst async-fähig sind.

```rust
#[tokio::main]
async fn main() {
    // ...
}
```

`#[tokio::main]` ist ein **Attribut-Makro**: Es baut aus deiner `async fn main()` beim
Kompilieren eine ganz normale, synchrone `fn main()`, die im Hintergrund eine
Tokio-Runtime erzeugt und darauf **wartet** ("blockiert"), bis dein
`main`-Future fertig ist. `main()` selbst darf in Rust nämlich nicht `async` sein — es
braucht immer irgendetwas, das die Runtime startet und den allerersten Future antreibt.
Das Makro nimmt dir genau diese Handarbeit ab.

> **⚠️ Warnung**
>
> `.await` funktioniert **nur** innerhalb einer `async fn` oder eines `async`-Blocks.
> Das ist keine stilistische Empfehlung, sondern eine Compiler-Regel — du siehst sie
> gleich im Schritt-Reveal in Aktion.

## Schritt-Reveal

**Schritt 1 — Das neue Crate `mein_agent` anlegen.**

```bash
cargo new --lib mein_agent
```

Trage es im Workspace-Root in `Cargo.toml` unter `members` ein (analog zu
[Phase 1, Lektion 1](../02-phase1-fundament/01-workspace-lesen.md)):

```toml
[workspace]
resolver = "2"
members = [
    "mein_core",
    "mein_cli",
    "mein_agent",
]
```

**Schritt 2 — Tokio als Abhängigkeit ergänzen.** In `mein_agent/Cargo.toml`:

```toml
[package]
name = "mein_agent"
version = "0.1.0"
edition = "2024"

[dependencies]
mein_core = { path = "../mein_core" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

Oder per Kommandozeile:

```bash
cargo add tokio --features macros,rt-multi-thread,time -p mein_agent
```

> **💡 Tipp**
>
> Ab dieser Phase kommen mehrere neue Crates dazu (`tokio`, und in den nächsten
> Lektionen noch weitere). Wir zeigen jeweils die Major-Version, die zum Zeitpunkt
> dieses Kurses stabil ist — lass dir von `cargo add <crate>` die tatsächlich aktuelle
> Version eintragen, statt eine Zahl aus einem Buch abzutippen; Patch- und teils auch
> Minor-Versionen ändern sich laufend.

Die Features sind bewusst einzeln gewählt statt pauschal `features = ["full"]`: `macros`
gibt uns `#[tokio::main]`, `rt-multi-thread` die eigentliche Runtime mit mehreren
Worker-Threads, `time` Funktionen wie `sleep` und (später, [Lektion 6](06-abbruchbedingungen-limits.md))
`timeout`. Nur einzubinden, was wir tatsächlich brauchen, hält Kompilierzeit und
Abhängigkeitsbaum klein — dasselbe Prinzip wie YAGNI in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md).

**Schritt 3 — Ein Beispiel zum Ausprobieren anlegen.** `mein_agent` ist wie `mein_core`
eine **Bibliothek** (`src/lib.rs`, kein `main`) — Agent-Code soll später von `mein_cli`
oder einem künftigen `mein_server` genutzt werden, nicht selbst ein eigenständiges
Programm sein. Zum Ausprobieren einzelner Konzepte während dieser Phase nutzen wir
deshalb Cargos **Beispiel-Mechanismus**: Jede Datei unter `examples/` ist ein kleines,
eigenständiges Binary, das die Bibliothek benutzt, aber nicht Teil davon ist. Lege
`mein_agent/examples/erste_frage.rs` an:

```rust
async fn stelle_frage(text: &str) -> String {
    format!("Antwort auf: {text}")
}

fn main() {
    let antwort = stelle_frage("Test").await;
    println!("{antwort}");
}
```

`cargo check -p mein_agent --example erste_frage` meldet:

```
error[E0728]: `await` is only allowed inside `async` functions and blocks
 --> mein_agent/examples/erste_frage.rs:5:33
  |
4 | fn main() {
  |    ---- this is not `async`
5 |     let antwort = stelle_frage("Test").await;
  |                                 ^^^^^^ only allowed inside `async` functions and blocks
```

Genau die Regel aus der Warnung oben, jetzt als echte Fehlermeldung: `main` ist eine
normale, synchrone Funktion — `.await` darf darin nicht stehen. Die Fehlermeldung zeigt
sogar exakt, welche Funktion "nicht async" ist (`this is not \`async\``).

**Schritt 4 — Korrektur mit `#[tokio::main]`.**

```rust
async fn stelle_frage(text: &str) -> String {
    format!("Antwort auf: {text}")
}

#[tokio::main]
async fn main() {
    let antwort = stelle_frage("Test").await;
    println!("{antwort}");
}
```

`cargo check -p mein_agent --example erste_frage` — sauber.

## Ausführung

```bash
cargo run -p mein_agent --example erste_frage
```

```
Antwort auf: Test
```

## Zusammenfassung

- `async fn` gibt ein `Future` zurück — einen noch nicht fertigen Wert, der erst bei
  `.await` (oder einem anderen Antreiben) tatsächlich läuft.
- `.await` entpackt ein `Future`, gibt dabei aber die Kontrolle ab, solange es wartet —
  andere Arbeit kann in der Zwischenzeit weiterlaufen.
- Rust macht Async-Fähigkeit und Pausenpunkte bewusst im Typsystem und im Code sichtbar,
  statt sie wie Go automatisch im Hintergrund zu verwalten.
- Rust selbst liefert keine Runtime mit — wir bringen sie über das Crate `tokio` mit,
  `#[tokio::main]` startet sie und treibt das erste `Future` an.
- `.await` ist nur innerhalb von `async fn`/`async`-Blöcken erlaubt — ein sehr häufiger
  Anfängerfehler, den der Compiler klar benennt (`E0728`).

## Übung

Schreibe eine zweite `async fn`, die zwei "wartende" Schritte **nacheinander** ausführt
(z. B. zwei `sleep`-Aufrufe mit `.await` dazwischen etwas Text ausgibt) und rufe sie aus
`main` auf. Miss danach mit `std::time::Instant` (vor und nach dem Aufruf), wie lange das
Programm insgesamt läuft. Überlege dir, ohne es zu implementieren: Was müsstest du
ändern, damit beide `sleep`-Aufrufe **gleichzeitig** statt nacheinander laufen (Hinweis:
Die Antwort liegt nicht in `.await` selbst, sondern darin, wie viele separate Futures du
erzeugst und ob du sie einzeln oder gemeinsam startest — `tokio::join!` ist das
Stichwort, das du in der Tokio-Dokumentation nachschlagen kannst). Wir brauchen dieses
Prinzip in [Lektion 5](05-state-und-memory.md) wieder, wenn wir Zustand über mehrere
gleichzeitig laufende Tasks hinweg teilen.

[Weiter: Lektion 2 — SSE-Streaming als Eventfolge](02-sse-streaming.md)
