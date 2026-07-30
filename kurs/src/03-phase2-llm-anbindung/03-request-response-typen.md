# Lektion 3: Request- und Response-Typen trennen

## Problem

Lektion 2 hat gezeigt: Unsere Domain-Typen `Rolle`/`Nachricht` sprechen ein anderes
Vokabular (`rolle`/`inhalt`, deutsch) als die HTTP-API eines LLM-Anbieters (`role`/
`content`, englisch, kleingeschrieben). Wir könnten `Nachricht` mit `#[serde(rename =
"...")]` biegen, bis es passt — aber dann trägt unser Domain-Modell für immer die
Altlasten eines einzelnen, austauschbaren Anbieterformats. Was, wenn wir später (Phase 3)
den Anbieter wechseln und der ein anderes JSON-Schema spricht? Die Lösung: eigene Typen,
die **nur** das Vertragsformat der HTTP-Grenze abbilden, komplett getrennt von `Rolle`/
`Nachricht`/`Konversation`.

## Code (Zielbild)

```rust
#[derive(Debug, Serialize)]
struct ChatNachricht {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatNachricht>,
    temperature: f64,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatAntwortNachricht,
}

#[derive(Debug, Deserialize)]
struct ChatAntwortNachricht {
    content: String,
}
```

```rust
impl OpenAiKompatiblerClient {
    pub fn chat(&self, konversation: &Konversation) -> Result<String, Box<dyn std::error::Error>> {
        // sendet konversation.verlauf() als ChatRequest, liest die Antwort typisiert zurück
    }
}
```

## Dekonstruktion

### Warum englische Namen für diese Typen, obwohl wir sonst deutsch benennen?

Nach unserer Namenskonvention (siehe [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md))
bleiben Domain-Begriffe deutsch. `ChatNachricht`, `ChatRequest`, `ChatResponse` sind aber
**keine Domain-Typen** — sie bilden ausschließlich das *externe* Vertragsformat einer
fremden HTTP-API ab, Feld für Feld an deren JSON-Schlüssel angelehnt (`role`, `content`,
`model`, `choices`). Deshalb wählen wir hier bewusst Namen nahe am englischen Original.
Der entscheidende Punkt: Diese Typen leben **ausschließlich** innerhalb von
`mein_core::provider` (kein `pub` nötig — gleich mehr dazu) und werden nach außen, an
`mein_cli`, nie sichtbar. Die öffentliche Schnittstelle von `provider` bleibt
`OpenAiKompatiblerClient::chat(&Konversation) -> Result<String, ...>` — nimmt eine
`Konversation`, gibt am Ende nur Text zurück.

### `struct` ohne `pub` — Modul-interne Helfer

Fällt dir auf, dass `ChatNachricht`, `ChatRequest` & Co. **kein** `pub` vor `struct`
haben? Das ist Absicht: Sie sind reine Übersetzungshilfen zwischen `Konversation` und
JSON, die außerhalb von `provider.rs` niemand braucht. Ohne `pub` verhindert der
Compiler sogar, dass sie versehentlich irgendwo anders im Projekt auftauchen und die
Kapselung aus Lektion 1 durchbrechen. Kleinstmögliche Sichtbarkeit ist ein Rust-Idiom,
das du dir merken solltest: **Mache etwas nur so öffentlich, wie es tatsächlich gebraucht
wird — nicht öffentlicher "auf Verdacht".**

### `role: &'static str` statt `Rolle`

```rust
struct ChatNachricht {
    role: &'static str,
    content: String,
}
```

`role` ist ein `&'static str` (ein Textverweis, der für die *gesamte* Programmlaufzeit
gültig bleibt — `'static` ist ein sogenanntes **Lifetime**, mehr dazu bei Bedarf in
[Kapitel 0](../01-grundlagen/01-variablen-und-typen.md) und vertieft, sobald wir
Lifetimes aktiv brauchen). Wir nutzen hier feste String-Literale wie `"system"`,
`"user"`, `"assistant"`, die zur Kompilierzeit feststehen — kein `String`, der zur
Laufzeit alloziert werden müsste. Die Übersetzung von `Rolle` übernimmt eine kleine freie
Funktion:

```rust
fn rolle_zu_api_text(rolle: &Rolle) -> &'static str {
    match rolle {
        Rolle::System => "system",
        Rolle::Benutzer => "user",
        Rolle::Assistent => "assistant",
    }
}
```

Dieses `match` deckt alle drei `Rolle`-Varianten ab — lässt du eine weg, meldet der
Compiler `error[E0004]: non-exhaustive patterns`. Das ist derselbe Vollständigkeitsschutz,
den [Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md) schon
angekündigt hat: Käme später (Phase 4) eine vierte `Rolle` dazu (z. B. `Werkzeug` für
Tool-Antworten), würde dieser `match`-Block nicht mehr kompilieren, bis wir ihn
aktualisieren — der Compiler erinnert uns aktiv daran, statt dass wir es vergessen und
zur Laufzeit eine falsche Rolle verschicken.

### Von `Konversation` zu `Vec<ChatNachricht>`

```rust
fn verlauf_zu_api_nachrichten(konversation: &Konversation) -> Vec<ChatNachricht> {
    konversation
        .verlauf()
        .iter()
        .map(|nachricht| ChatNachricht {
            role: rolle_zu_api_text(&nachricht.rolle),
            content: nachricht.inhalt.clone(),
        })
        .collect()
}
```

`konversation.verlauf()` ist genau die Methode aus
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md), die einen `&[Nachricht]`
zurückgibt — `provider` kennt also weiterhin nur die öffentliche API von `Konversation`,
nie ihr internes `Vec`. `.iter().map(...).collect()` ist Rusts **Iterator**-Muster: über
jedes Element gehen, es transformieren, in eine neue Sammlung einsammeln — kompakter als
eine `for`-Schleife mit manuellem `push` in einen leeren `Vec`. `.clone()` bei
`nachricht.inhalt` ist nötig, weil wir nur eine **Referenz** auf die `Konversation`
bekommen (`&Konversation`) — wir dürfen die Originaldaten nicht wegnehmen (*ownen*), also
kopieren wir den Text für die eigene, kurzlebige `ChatNachricht`.

### `#[derive(Serialize)]` fehlt — ein Compilerfehler mit Ansage

Angenommen, du vergisst `#[derive(Serialize)]` auf `ChatRequest` und versuchst trotzdem,
es als Request-Body zu verschicken:

```rust
#[derive(Debug)] // Serialize fehlt!
struct ChatRequest {
    model: String,
    messages: Vec<ChatNachricht>,
    temperature: f64,
}
```

```rust
self.http.post(url).json(&anfrage).send()?;
```

```
error[E0277]: the trait bound `ChatRequest: Serialize` is not satisfied
   --> mein_core/src/provider.rs:52:29
    |
52  |         self.http.post(url).json(&anfrage).send()?;
    |                             ^^^^ the trait `Serialize` is not implemented for `ChatRequest`
```

`.json(&wert)` von `reqwest` verlangt, dass `wert` das Trait `Serialize` implementiert —
sonst wüsste `reqwest` nicht, wie es den Wert in JSON-Text umwandeln soll. Der Compiler
prüft das schon beim Kompilieren, nicht erst beim Ausführen: Ein vergessenes `derive`
fällt sofort auf, nicht erst live gegen einen echten Server. Korrektur: `#[derive(Debug,
Serialize)]` ergänzen.

### `ChatResponse` einlesen

```rust
let antwort: ChatResponse = reqwest_antwort.json()?;
let text = antwort
    .choices
    .into_iter()
    .next()
    .map(|choice| choice.message.content)
    .ok_or("Antwort enthielt keine choices")?;
```

`reqwest_antwort.json::<ChatResponse>()` (der Typ wird hier über die
Ziel-Variable `antwort: ChatResponse` abgeleitet, sog. **Typinferenz**) parst den
Antwortkörper direkt in unseren typisierten `struct` — kein `Value`, kein `.as_str()`
mehr nötig. Passt das JSON nicht zur Struktur (fehlendes Feld, falscher Typ), bekommst du
sofort einen `Err` mit einer sprechenden Fehlermeldung, statt eines stillen `None`
irgendwo tief in einem `Value`-Baum.

`.into_iter().next()` holt das erste Element aus `choices` (als `Option<ChatChoice>`,
`None` falls das Array leer ist — genau der Fall, den du in der Übung von Lektion 2
schon einmal behandelt hast). `.ok_or("...")` wandelt ein `Option` in ein `Result` um:
"Ist es `Some(wert)`, mach weiter mit `wert`; ist es `None`, wird daraus ein `Err` mit
dieser Nachricht." Das ist ein sehr verbreitetes Umwandlungsmuster zwischen `Option` und
`Result`, das du dir merken solltest.

### `Box<dyn std::error::Error>` — ein provisorischer Sammelbehälter

`chat()` kann aus **mehreren** Gründen fehlschlagen: das Netzwerk (`reqwest::Error`),
eine falsch geformte Antwort (auch `reqwest::Error`, da `.json()` intern
deserialisiert), oder unser eigenes `.ok_or(...)` (ein `&str`-Fehler). Diese
verschiedenen Fehlertypen alle über `?` weiterzureichen, braucht **einen** gemeinsamen
Rückgabetyp. `Box<dyn std::error::Error>` heißt: "irgendein Fehlertyp, solange er das
Standard-`Error`-Trait implementiert, in einer `Box` verpackt" (eine `Box` ist ein Zeiger
auf Daten auf dem Heap — Speicher, der zur Laufzeit reserviert wird; das brauchen wir
hier, weil unterschiedliche Fehlertypen unterschiedlich groß sind und der Compiler zur
Kompilierzeit eine feste Größe für den Rückgabetyp kennen muss). `dyn` steht für
**dynamischer Trait-Dispatch** — welcher konkrete Fehlertyp wirklich drinsteckt, wird
erst zur Laufzeit entschieden. Wir vertiefen `dyn Trait` bewusst erst in
[Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md); für jetzt reicht:
Es ist ein pragmatischer, aber grober Sammelbehälter. Genau seine Grobheit ist unser
Problem für [Lektion 4](04-fehlerbehandlung.md): Aufrufer*innen von `chat()` können mit
`Box<dyn Error>` nicht gezielt per `match` reagieren ("war es das Netzwerk oder ein
falsches Format?") — das lösen wir dort mit einem eigenen, typisierten Fehler.

## Schritt-Reveal

**Schritt 1** — Lege die vier Typen `ChatNachricht`, `ChatRequest`, `ChatResponse`,
`ChatChoice` (samt `ChatAntwortNachricht`) in `provider.rs` an, ergänze
`use serde::{Deserialize, Serialize};` und `use crate::{Konversation, Rolle};` am
Dateianfang.

**Schritt 2** — Provoziere den `Serialize`-Fehler bewusst (siehe oben), korrigiere ihn.

**Schritt 3** — Implementiere `rolle_zu_api_text` und `verlauf_zu_api_nachrichten`.

**Schritt 4** — Implementiere `chat`:

```rust
impl OpenAiKompatiblerClient {
    pub fn chat(&self, konversation: &Konversation) -> Result<String, Box<dyn std::error::Error>> {
        let anfrage = ChatRequest {
            model: self.modell.clone(),
            messages: verlauf_zu_api_nachrichten(konversation),
            temperature: 0.7,
        };

        let url = format!("{}/chat/completions", self.basis_url);

        let antwort: ChatResponse = self
            .http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&anfrage)
            .send()?
            .json()?;

        antwort
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| "Antwort enthielt keine choices".into())
    }
}
```

`.bearer_auth(&self.api_key)` setzt den Header `Authorization: Bearer <api_key>` — das
verbreitete Authentifizierungsschema für LLM-HTTP-APIs. `.ok_or_else(|| "...".into())`
statt `.ok_or("...")`: `into()` wandelt den `&str` explizit in den `Box<dyn Error>`-Typ
um, den unsere Funktion als Fehlertyp erwartet (ein `&str` implementiert `Error` nicht
direkt, aber `From<&str>` existiert für `Box<dyn Error>` — deshalb funktioniert `.into()`
hier).

`cargo check -p mein_core` sollte jetzt sauber durchlaufen.

## Ausführung

```bash
cargo test -p mein_core
```

Ergänze einen Test, der nur die *reine* Umwandlung prüft (kein echter Netzwerkaufruf
nötig — genau der Vorteil, `verlauf_zu_api_nachrichten` als eigene, testbare Funktion
herauszuziehen):

```rust
#[test]
fn konversation_wird_zu_api_nachrichten_uebersetzt() {
    let mut k = Konversation::neu();
    k.hinzufuegen(Rolle::System, "Du bist hilfreich.").unwrap();
    k.hinzufuegen(Rolle::Benutzer, "Hallo!").unwrap();

    let nachrichten = verlauf_zu_api_nachrichten(&k);

    assert_eq!(nachrichten[0].role, "system");
    assert_eq!(nachrichten[1].role, "user");
    assert_eq!(nachrichten[1].content, "Hallo!");
}
```

```
running 1 test
test provider::tests::konversation_wird_zu_api_nachrichten_uebersetzt ... ok
```

> **⚠️ Warnung**
>
> `chat()` selbst testest du hier noch **nicht** automatisiert — das würde einen echten
> Netzwerkaufruf bedeuten (langsam, braucht einen echten Endpunkt, kostet bei manchen
> Anbietern Geld). Wir trennen deshalb bewusst die reine, leicht testbare Übersetzung
> (`verlauf_zu_api_nachrichten`) von der unreinen Netzwerk-Methode (`chat`) — ein Muster,
> das du dir merken solltest: Zieh so viel Logik wie möglich aus einer Funktion heraus,
> die nebenbei Netzwerk/Datei/Zeit anfasst, in reine, leicht testbare Funktionen.

## Zusammenfassung

- Request-/Response-Typen (`ChatRequest`, `ChatResponse`, ...) bilden das *externe*
  API-Format ab und bleiben strikt getrennt von `Rolle`/`Nachricht`/`Konversation`.
- Kleinstmögliche Sichtbarkeit: Diese Typen sind nicht `pub`, weil sie ausschließlich
  intern in `provider` gebraucht werden.
- `.json::<T>()` auf einer `reqwest`-Antwort deserialisiert direkt in einen typisierten
  Rust-Wert — mit sprechenden Fehlern bei Formatabweichungen, statt stiller `None`-Werte
  wie bei `Value`.
- `Box<dyn std::error::Error>` ist ein pragmatischer, aber grober Sammelbehälter für
  mehrere Fehlerquellen — ausreichend für jetzt, aber nicht gezielt behandelbar.
- Reine Übersetzungsfunktionen (`verlauf_zu_api_nachrichten`) trennen wir bewusst von
  unreinen Netzwerkaufrufen (`chat`), um sie ohne echten Server testen zu können.

## Übung

`temperature` steht in `chat()` aktuell fest auf `0.7`, ganz ähnlich wie in
`Konfiguration` aus [Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md).
Erweitere `OpenAiKompatiblerClient::neu` um einen zusätzlichen Parameter, der eine
`Konfiguration` (oder zumindest deren `temperatur`-Feld) entgegennimmt, und nutze diesen
Wert in `chat()` statt der festen `0.7`. Was ändert sich an der öffentlichen Signatur von
`chat(&self, konversation: &Konversation)`? Idealerweise: nichts — genau das ist der
Punkt dieser Übung, vergleichbar mit der Transferaufgabe aus
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md).

[Weiter: Lektion 4 — Fehler mit thiserror und anyhow](04-fehlerbehandlung.md)
