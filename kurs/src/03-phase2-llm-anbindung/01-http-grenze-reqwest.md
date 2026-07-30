# Lektion 1: HTTP-Grenze mit reqwest

## Problem

Eine `Konversation` aus Phase 1 lebt bisher ausschließlich im Arbeitsspeicher eines
einzelnen `mein_cli`-Aufrufs — sie antwortet niemand. Ein echtes Sprachmodell läuft auf
einem fremden Server, den wir nur über eine **HTTP-Schnittstelle** erreichen: Wir
schicken eine Anfrage (*Request*) über das Netzwerk, der Server schickt eine Antwort
(*Response*) zurück. Stell es dir wie einen Brief vor: Du schreibst ihn nach einem
festen Format (Adresse, Absender, Inhalt), gibst ihn bei der Post ab, und irgendwann
kommt eine Antwort zurück — oder auch nicht, oder eine, die du nicht lesen kannst. Genau
diese Unsicherheiten muss unser Code aushalten.

Konkrete Frage dieser Lektion: Wie bauen wir diese Verbindung nach außen so, dass sie an
**einer** klar benannten Stelle in `mein_core` sitzt — und nirgendwo sonst im Projekt
auftaucht?

## Code (Zielbild)

```rust
use reqwest::blocking::Client;

pub struct OpenAiKompatiblerClient {
    basis_url: String,
    api_key: String,
    modell: String,
    http: Client,
}

impl OpenAiKompatiblerClient {
    pub fn neu(
        basis_url: impl Into<String>,
        api_key: impl Into<String>,
        modell: impl Into<String>,
    ) -> Self {
        OpenAiKompatiblerClient {
            basis_url: basis_url.into(),
            api_key: api_key.into(),
            modell: modell.into(),
            http: Client::new(),
        }
    }
}
```

## Dekonstruktion

### Ein neues Modul: `mein_core::provider`

Bisher lebte der gesamte `mein_core`-Code in einer einzigen Datei,
`mein_core/src/lib.rs`. Das war für `Rolle`, `Nachricht`, `Konversation` und
`Konfiguration` noch übersichtlich — ab jetzt wächst `mein_core` spürbar, und alles in
eine Datei zu packen würde schnell unlesbar. Rust teilt Code mit dem **Modul**-System in
mehrere Dateien auf. Wir legen eine neue Datei `mein_core/src/provider.rs` an und
verbinden sie in `lib.rs` mit:

```rust
pub mod provider;
```

`mod provider;` sagt dem Compiler: "Es gibt ein Modul namens `provider`, sein Inhalt
steht in `provider.rs` (oder `provider/mod.rs`)." `pub` davor macht dieses Modul selbst
nach außen sichtbar — ohne `pub` könnte `mein_cli` nichts daraus benutzen, egal wie viele
einzelne Typen darin `pub` sind (Sichtbarkeit vererbt sich in Rust nicht automatisch nach
oben). Von außen sprechen wir den neuen Typ dann als `mein_core::provider::
OpenAiKompatiblerClient` an, oder kürzer nach einem `use mein_core::provider::
OpenAiKompatiblerClient;`.

### Warum ein eigenes Modul, statt `reqwest` überall zu benutzen?

Wir könnten `reqwest`-Aufrufe auch direkt in `mein_cli/src/main.rs` schreiben. Tun wir
bewusst nicht: `mein_core::provider` ist die **einzige** Stelle im gesamten Projekt, die
weiß, dass wir gerade HTTP sprechen. Der Rest von `mein_core` (`Rolle`, `Nachricht`,
`Konversation`) bleibt komplett frei von Netzwerkdetails, und `mein_cli` bekommt später
nur eine simple Methode wie `client.chat(&konversation)` zu sehen — keine URLs, keine
Header, keine JSON-Bibliothek. Diese Kapselung zahlt sich in
[Phase 3](../04-phase3-architektur/README.md) direkt aus, wenn wir denselben Provider
hinter einem `trait LlmProvider` verstecken: Je weniger Code heute weiß, *dass* es
`reqwest` gibt, desto weniger Code muss sich später ändern.

> **💡 Tipp**
>
> Merke dir diese Faustregel für den ganzen Kurs: Eine externe Abhängigkeit (hier
> `reqwest`) taucht in Typsignaturen möglichst nur innerhalb des Moduls auf, das sie
> einführt — nicht in `mein_cli`, nicht in anderen `mein_core`-Modulen.

### Bewusst *kein* austauschbarer Provider — noch nicht

Der Name `OpenAiKompatiblerClient` ist absichtlich **konkret**, kein `trait
LlmProvider`. Viele LLM-Anbieter (und lokale Server wie Ollama oder LM Studio) sprechen
inzwischen ein sehr ähnliches, am OpenAI-Format orientiertes HTTP/JSON-Protokoll — daher
der Name: "ein Client für Anbieter, die dieses verbreitete Format sprechen", nicht "der
Client für Anbieter X". Trotzdem bauen wir hier **einen einzigen, konkreten Typ**, kein
Trait, kein `dyn Trait`. Das ist eine bewusste Design-Entscheidung, keine Faulheit: Wir
bauen erst einen konkreten Client, bevor wir ihn in
[Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md) hinter einem Trait
verstecken — Abstraktion kommt, wenn wir den zweiten Anwendungsfall kennen, nicht vorher.
Ein Trait für genau eine Implementierung zu bauen, hieße, Komplexität auf Vorrat zu
kaufen, ohne zu wissen, welche Form sie wirklich braucht.

### `reqwest::blocking::Client` statt `async`

`reqwest` ist Rusts verbreitetste HTTP-Bibliothek. Sie bietet zwei Varianten an: eine
**asynchrone** (`reqwest::Client`, braucht `async`/`await` und einen Async-Laufzeit wie
Tokio) und eine **blockierende** (`reqwest::blocking::Client`, sieht aus wie ganz
normaler, synchroner Code). Wir nutzen hier bewusst die blockierende Variante.
`async`/`await` und Tokio sind eigenständiges Thema von
[Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md) — bis dahin würde uns
Async nur zusätzliche Komplexität aufbürden, ohne dass wir davon profitieren: Unser
`mein_cli` macht ohnehin genau einen HTTP-Aufruf nach dem anderen, nichts läuft parallel.
"Blocking" heißt: Der Programmablauf **wartet**, bis die Antwort da ist, bevor er
weitermacht — für ein einzelnes CLI-Kommando ist das genau das gewünschte Verhalten.

> **⚠️ Warnung**
>
> `reqwest::blocking` ist ein **Cargo-Feature**, kein Standardverhalten. Ohne es explizit
> zu aktivieren, existiert das Modul `blocking` in `reqwest` schlicht nicht — das siehst
> du gleich im provozierten Compilerfehler.

### Die Felder des Clients

```rust
pub struct OpenAiKompatiblerClient {
    basis_url: String,
    api_key: String,
    modell: String,
    http: Client,
}
```

Alle vier Felder sind **privat** (kein `pub` davor) — anders als bei `Nachricht` in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md). Ein API-Key ist
ein Geheimnis; es soll nicht von außen gelesen (und versehentlich geloggt oder
weitergegeben) werden können. `http: Client` hält den eigentlichen `reqwest`-Client, der
intern Verbindungen wiederverwendet (*Connection Pooling*) — deshalb legen wir ihn
**einmal** in `neu()` an und halten ihn im Struct, statt bei jedem Aufruf einen neuen
`Client::new()` zu erzeugen.

## Schritt-Reveal

**Schritt 1 — Abhängigkeit ergänzen.** In `mein_core/Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "...", features = ["blocking", "json"] }
```

Ersetze `"..."` mit der aktuellen stabilen Version, z. B. per
`cargo add reqwest --features blocking,json` im Ordner `mein_core` ausgeführt — das
trägt Versionsnummer und Features automatisch korrekt ein.

**Schritt 2 — Modul anlegen.** Erstelle `mein_core/src/provider.rs`, ergänze in
`mein_core/src/lib.rs` ganz oben:

```rust
pub mod provider;
```

**Schritt 3 — Provoziere den Fehler bewusst.** Schreibe in `provider.rs` testweise
zuerst *ohne* das `blocking`-Feature in der `Cargo.toml` (entferne es kurz wieder):

```rust
use reqwest::blocking::Client;
```

```bash
cargo check -p mein_core
```

```
error[E0433]: failed to resolve: could not find `blocking` in `reqwest`
 --> mein_core/src/provider.rs:1:20
  |
1 | use reqwest::blocking::Client;
  |                    ^^^^^^^^ could not find `blocking` in `reqwest`
```

Das ist kein Tippfehler-Symptom, sondern ein fehlendes Feature: `reqwest` kompiliert das
`blocking`-Modul nur mit, wenn du es in `Cargo.toml` explizit anforderst — so bleibt die
Bibliothek schlank für alle, die nur die async-Variante brauchen. Ergänze
`features = ["blocking", "json"]` wieder, `cargo check -p mein_core` läuft jetzt sauber
durch.

**Schritt 4 — `OpenAiKompatiblerClient` mit `neu()` anlegen** (siehe Zielbild oben).

**Schritt 5 — Ein erster, roher Verbindungstest.** Wir wollen beweisen, dass die
Leitung grundsätzlich funktioniert, bevor wir uns in Lektion 2 und 3 um das exakte
JSON-Format eines LLM-Anbieters kümmern. Dafür nutzen wir testweise
[httpbin.org](https://httpbin.org) — einen öffentlichen Testdienst, der jede Anfrage
einfach als JSON zurückspiegelt. So kannst du diese Lektion durcharbeiten, **ohne**
schon einen echten API-Key zu besitzen:

```rust
use serde_json::json;

impl OpenAiKompatiblerClient {
    pub fn ping(&self) -> Result<String, reqwest::Error> {
        let antwort = self
            .http
            .post("https://httpbin.org/post")
            .json(&json!({ "modell": self.modell, "test": true }))
            .send()?;
        antwort.text()
    }
}
```

`json!({...})` ist ein Makro aus `serde_json` (das seit
[Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md) schon in
`mein_core/Cargo.toml` steht) — es baut ad hoc einen JSON-Wert, ohne dass wir dafür
einen eigenen Rust-Typ definieren müssten. `.json(&wert)` setzt den Request-Body und den
`Content-Type: application/json`-Header automatisch. `.send()` schickt die Anfrage
**los und wartet** (blockierend, siehe oben) auf die Antwort — daher `?`, es kann
fehlschlagen (kein Netzwerk, Timeout, DNS-Fehler). `antwort.text()` liest den Antwortkörper
als `String` — auch das kann fehlschlagen (z. B. ungültige Zeichenkodierung), daher wieder
ein `Result`.

## Ausführung

Ergänze testweise in `mein_cli/src/main.rs`:

```rust
use mein_core::provider::OpenAiKompatiblerClient;

fn main() {
    let client = OpenAiKompatiblerClient::neu(
        "https://api.beispiel-anbieter.invalid",
        "kein-echter-key",
        "irgendein-modell",
    );

    match client.ping() {
        Ok(antwort) => println!("{antwort}"),
        Err(fehler) => eprintln!("Verbindungsfehler: {fehler}"),
    }
}
```

```bash
cargo run -p mein_cli
```

Erwartete Ausgabe (gekürzt) — httpbin spiegelt genau das JSON zurück, das wir gesendet
haben:

```json
{
  "json": {
    "modell": "irgendein-modell",
    "test": true
  },
  ...
}
```

Setze `main.rs` danach zurück, wir bauen ab Lektion 3 die echte `chat`-Methode.

> **💡 Tipp**
>
> Läuft bei dir lokal ein OpenAI-kompatibler Server (z. B. Ollama unter
> `http://localhost:11434/v1`), kannst du `basis_url` schon jetzt darauf umstellen und
> brauchst dafür keinen bezahlten API-Key. Der Rest dieser Phase ist bewusst so gebaut,
> dass er mit jedem Anbieter funktioniert, der dieses verbreitete Format spricht.

## Zusammenfassung

- `mein_core` wächst ab dieser Lektion über mehrere Dateien, verbunden über `mod`/
  `pub mod` — nötig, sobald ein Modul (`provider`) eine klar eigene Verantwortung
  bekommt.
- `reqwest::blocking::Client` ist unsere HTTP-Grenze — bewusst synchron, weil
  `async`/Tokio erst in Phase 4 eingeführt wird und wir sie hier noch nicht brauchen.
- `OpenAiKompatiblerClient` ist ein einziger, konkreter Typ — kein Trait, kein
  `dyn Trait`. Abstraktion verschieben wir bewusst auf Phase 3.
- Externe Abhängigkeiten (`reqwest`) bleiben auf ein Modul begrenzt; `mein_cli` sieht
  später nur einfache Methodenaufrufe.

## Übung

Ergänze `OpenAiKompatiblerClient` um eine Methode `pub fn basis_url(&self) -> &str`, die
schreibgeschützt Einblick in das private Feld `basis_url` gibt (vergleiche das Muster
mit `Konversation::verlauf()` aus
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) — dort war der Grund
Kapselung des *internen* Speicherformats, hier ist der Grund ein anderer: Überlege dir,
welcher). Schreibe dazu einen kleinen Test, der einen Client erzeugt und die `basis_url`
zurückliest. Was wäre der Unterschied, hättest du stattdessen das Feld `basis_url`
direkt `pub` gemacht? Nutze dazu deine Antwort aus der Überlegung zu `api_key` oben.

[Weiter: Lektion 2 — JSON-Schema mit serde_json](02-json-schema.md)
