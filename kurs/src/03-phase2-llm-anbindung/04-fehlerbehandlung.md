# Lektion 4: Fehler mit thiserror und anyhow

## Problem

`chat()` gibt aktuell `Result<String, Box<dyn std::error::Error>>` zurück — ein grober
Sammelbehälter (siehe [Lektion 3](03-request-response-typen.md)). Das Problem: Ruft
`mein_cli` `chat()` auf und bekommt einen `Err` zurück, kann es **nicht gezielt**
unterscheiden, ob das Netzwerk ausgefallen ist, der API-Key falsch war oder die Antwort
ein unerwartetes Format hatte — alles sieht für den Aufrufer gleich aus: irgendein
`Box<dyn Error>`. Genau wie bei `NachrichtFehler` in
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) brauchen wir einen
eigenen, typisierten Fehlertyp — nur jetzt mit **mehreren** Fehlerquellen (Netzwerk,
JSON, API-Statuscode), nicht nur einer.

## Code (Zielbild)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderFehler {
    #[error("Netzwerkfehler beim Erreichen des Anbieters: {0}")]
    Netzwerk(#[from] reqwest::Error),

    #[error("Antwort des Anbieters hatte kein gültiges JSON-Format: {0}")]
    UngueltigesFormat(#[from] serde_json::Error),

    #[error("Antwort des Anbieters enthielt keine Auswahl (choices)")]
    LeereAntwort,

    #[error("Anbieter meldete Statuscode {status}: {nachricht}")]
    ApiFehler { status: u16, nachricht: String },
}
```

```rust
impl OpenAiKompatiblerClient {
    pub fn chat(&self, konversation: &Konversation) -> Result<String, ProviderFehler> {
        // wie in Lektion 3, aber jetzt mit ProviderFehler statt Box<dyn Error>
    }
}
```

## Dekonstruktion

### `thiserror` — Fehlertypen für Bibliotheken

`mein_core` ist eine **Bibliothek**. Bibliotheken sollten Aufrufer*innen nicht zwingen,
Fehlertexte zu parsen, um zu wissen, was schiefging (dasselbe Argument wie bei
`NachrichtFehler` in Phase 1) — sie sollten stattdessen **typisierte** Fehler anbieten,
über die man `match`en kann. `thiserror` ist ein `derive`-Makro (dasselbe Konzept wie
`#[derive(Debug, Clone, ...)]`, das wir seit
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md) kennen), das genau
das mit minimalem Schreibaufwand ermöglicht: Du beschreibst die Fehlerfälle als `enum`,
`thiserror` generiert daraus automatisch eine `Display`-Implementierung (für
menschenlesbaren Text, z. B. in `eprintln!("{fehler}")`) und eine `std::error::Error`-
Implementierung.

### `#[error("...")]` — die Fehlermeldung pro Variante

```rust
#[error("Netzwerkfehler beim Erreichen des Anbieters: {0}")]
Netzwerk(#[from] reqwest::Error),
```

Jede Variante bekommt ihr eigenes `#[error("...")]`-Attribut mit der Meldung, die
Nutzer*innen letztlich sehen. `{0}` verweist auf das erste (und hier einzige) Feld der
Variante — bei benannten Feldern wie `ApiFehler { status, nachricht }` schreibst du
stattdessen `{status}`/`{nachricht}` (genau wie bei `println!`-Formatstrings, die du
schon aus [Kapitel 0](../01-grundlagen/02-funktionen.md) kennst).

### `#[from]` — automatische Umwandlung für `?`

```rust
Netzwerk(#[from] reqwest::Error),
```

`#[from]` generiert automatisch eine `impl From<reqwest::Error> for ProviderFehler`.
Das ist entscheidend für den `?`-Operator (den du seit
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) kennst): `?` reicht einen
Fehler nur dann automatisch weiter, wenn er sich per `From` in den Rückgabetyp der
Funktion umwandeln lässt. Ohne `#[from]` müsstest du jeden `reqwest::Error` von Hand in
`ProviderFehler::Netzwerk(...)` verpacken.

Lässt du `#[from]` versehentlich weg:

```rust
#[error("Netzwerkfehler: {0}")]
Netzwerk(reqwest::Error), // #[from] fehlt!
```

```rust
pub fn chat(&self, konversation: &Konversation) -> Result<String, ProviderFehler> {
    let antwort = self.http.post(url).json(&anfrage).send()?; // reqwest::Error
    // ...
}
```

```
error[E0277]: `?` couldn't convert the error to `ProviderFehler`
   --> mein_core/src/provider.rs:60:64
    |
60  |         let antwort = self.http.post(url).json(&anfrage).send()?;
    |                                                                ^ the trait `From<reqwest::Error>` is not implemented for `ProviderFehler`
```

Der Compiler sagt dir wörtlich, welches `From` fehlt — ein sehr direkter Hinweis, welches
`#[from]`-Attribut du vergessen hast. Ergänze es wieder, der Fehler verschwindet.

> **⚠️ Warnung**
>
> `#[from]` funktioniert nur, wenn **eine einzige** Variante für einen bestimmten
> Quelltyp zuständig ist. Hättest du zwei Varianten mit `#[from] reqwest::Error`, wüsste
> `?` nicht, welche gemeint ist — der Compiler lehnt das mit einem Fehler zur Definition
> der Varianten selbst ab, nicht erst bei der Benutzung.

### Ein Fehlerfall ohne automatische Herkunft: `LeereAntwort` und `ApiFehler`

Nicht jeder Fehlerfall stammt von einer fremden Bibliothek. `LeereAntwort` (leeres
`choices`-Array, aus Lektion 3) und `ApiFehler` (der Anbieter antwortet mit einem
Fehlerstatus wie `401 Unauthorized` oder `429 Too Many Requests`) sind Fälle, die **wir**
selbst erkennen und explizit erzeugen — ohne `#[from]`, weil es keinen fremden Fehlertyp
gibt, aus dem wir automatisch konvertieren:

```rust
let status = antwort.status();
if !status.is_success() {
    let text = antwort.text().unwrap_or_default();
    return Err(ProviderFehler::ApiFehler {
        status: status.as_u16(),
        nachricht: text,
    });
}
```

`status.is_success()` prüft, ob der HTTP-Statuscode im Bereich 200–299 liegt ("OK"). Das
gehört zu den wichtigsten Lektionen im Umgang mit HTTP-APIs überhaupt: **`send()`
schlägt bei einem 4xx/5xx-Statuscode nicht automatisch mit `Err` fehl** — aus Sicht von
`reqwest` war die Anfrage technisch erfolgreich zugestellt, nur die *Antwort* sagt
inhaltlich "Fehler". Prüfst du `status.is_success()` nicht selbst, verarbeitest du im
schlimmsten Fall eine Fehlermeldung des Anbieters, als wäre sie eine echte Antwort.

### `anyhow` — Fehlerkontext für Anwendungen

`mein_cli` ist keine Bibliothek, sondern eine **Anwendung**: Sie muss keine Fehler nach
außen typisiert anbieten (niemand "matcht" gegen Fehler von `mein_cli` — es gibt kein
Außen mehr). Was sie stattdessen braucht: **möglichst viel Kontext** in einer einzigen,
für Menschen lesbaren Fehlerkette. Dafür ist `anyhow` gemacht — das Gegenstück zu
`thiserror`, nicht dessen Ersatz:

```rust
use anyhow::{Context, Result};
use mein_core::provider::OpenAiKompatiblerClient;
use mein_core::Konversation;

fn frage_stellen(client: &OpenAiKompatiblerClient, konversation: &Konversation) -> Result<String> {
    client
        .chat(konversation)
        .context("Anfrage an den LLM-Anbieter ist fehlgeschlagen")
}

fn main() -> Result<()> {
    let client = OpenAiKompatiblerClient::neu(
        std::env::var("MEIN_BASIS_URL").context("MEIN_BASIS_URL ist nicht gesetzt")?,
        std::env::var("MEIN_API_KEY").context("MEIN_API_KEY ist nicht gesetzt")?,
        "irgendein-modell",
    );

    let mut konversation = Konversation::neu();
    konversation.hinzufuegen(mein_core::Rolle::Benutzer, "Hallo!")?;

    let antwort = frage_stellen(&client, &konversation)?;
    println!("{antwort}");
    Ok(())
}
```

`anyhow::Result<T>` ist eine Abkürzung für `Result<T, anyhow::Error>` — ein einziger,
universeller Fehlertyp, der **jeden** konkreten Fehler (egal ob `ProviderFehler`,
`std::env::VarError`, `NachrichtFehler`, ...) mit `?` aufnehmen kann, solange er
`std::error::Error` implementiert (das gilt für alle unsere thiserror-generierten
Fehler automatisch). `.context("...")` (aus dem `Context`-Trait) hängt eine zusätzliche,
menschliche Erklärung an einen Fehler, **ohne** die ursprüngliche Fehlerursache zu
verlieren — ruft `?` in `main` fehl, druckt `anyhow` am Ende beide Ebenen aus: deinen
Kontexttext *und* die ursprüngliche Fehlermeldung von `ProviderFehler`/`thiserror`.

> **💡 Tipp**
>
> Merke dir die Faustregel: **`thiserror` in Bibliotheken** (`mein_core`), wo Aufrufer
> gezielt auf einzelne Fehlerfälle reagieren könnten — **`anyhow` in Anwendungen**
> (`mein_cli`), wo am Ende nur noch eine verständliche Fehlermeldung für einen Menschen
> im Terminal zählt. Diese Zweiteilung ist in der Rust-Community weit verbreitet, nicht
> nur eine Vorliebe dieses Kurses.

### `main() -> Result<()>` statt `main()`

Bisher gab `main` in `mein_cli` nichts zurück (`fn main()`). Ab jetzt lassen wir `main`
selbst `anyhow::Result<()>` zurückgeben — dadurch können wir `?` direkt in `main`
benutzen, statt jeden Fehler manuell mit `match`/`if let Err(...)` abzufangen. Gibt
`main` am Ende `Err(fehler)` zurück, druckt Rust automatisch die Fehlerkette auf `stderr`
und beendet das Programm mit einem Exit-Code ungleich `0` (das übliche Signal an die
Shell: "hier ist etwas schiefgelaufen").

## Schritt-Reveal

**Schritt 1** — Abhängigkeit ergänzen, `mein_core/Cargo.toml`:

```toml
[dependencies]
thiserror = "..." # aktuelle stabile Version, z. B. via `cargo add thiserror`
```

**Schritt 2** — Neues Modul `mein_core::error` anlegen: Datei
`mein_core/src/error.rs` mit dem `ProviderFehler`-`enum` aus dem Zielbild, in `lib.rs`
ergänzen:

```rust
pub mod error;
```

**Schritt 3** — Provoziere den fehlenden `#[from]`-Fehler bewusst (siehe oben),
korrigiere ihn.

**Schritt 4** — Passe `OpenAiKompatiblerClient::chat` an: Rückgabetyp wird
`Result<String, ProviderFehler>`, die Statuscode-Prüfung und `LeereAntwort`/`ApiFehler`
werden ergänzt (siehe oben). `use crate::error::ProviderFehler;` in `provider.rs`.

`cargo check -p mein_core` — sollte sauber durchlaufen.

**Schritt 5** — Abhängigkeit ergänzen, `mein_cli/Cargo.toml`:

```toml
[dependencies]
anyhow = "..." # aktuelle stabile Version, z. B. via `cargo add anyhow`
```

**Schritt 6** — `main.rs` wie im Zielbild oben umbauen.

## Ausführung

```bash
cargo build
```

Provoziere den `ApiFehler`-Fall bewusst, indem du testweise eine falsche `MEIN_API_KEY`
gegen einen echten Anbieter setzt (oder, ohne echten Anbieter, `basis_url` auf einen
Endpunkt zeigst, der garantiert `404` zurückgibt, z. B. `https://httpbin.org/status/404`):

```bash
MEIN_BASIS_URL=https://httpbin.org/status MEIN_API_KEY=irrelevant cargo run -p mein_cli
```

Erwartete Fehlerausgabe (sinngemäß, mit deinem Kontexttext):

```
Error: Anfrage an den LLM-Anbieter ist fehlgeschlagen

Caused by:
    Anbieter meldete Statuscode 404: ...
```

Provoziere zusätzlich den fehlenden-Umgebungsvariable-Fall:

```bash
cargo run -p mein_cli
```

```
Error: MEIN_BASIS_URL ist nicht gesetzt

Caused by:
    environment variable not found
```

Genau diese zweistufige Ausgabe — dein `.context(...)`-Text plus die technische Ursache
— ist der Mehrwert von `anyhow`.

```bash
cargo test -p mein_core
```

Ergänze einen Test für den API-Fehlerfall (kein echter Netzwerkaufruf nötig, da wir
`ProviderFehler` direkt konstruieren):

```rust
#[test]
fn api_fehler_zeigt_status_und_nachricht() {
    let fehler = ProviderFehler::ApiFehler {
        status: 401,
        nachricht: "invalid api key".to_string(),
    };
    assert_eq!(
        fehler.to_string(),
        "Anbieter meldete Statuscode 401: invalid api key"
    );
}
```

`fehler.to_string()` funktioniert, weil `thiserror` das `Display`-Trait implementiert —
genau die Meldung aus `#[error("...")]`, mit den Platzhaltern gefüllt.

## Zusammenfassung

- `thiserror` generiert `Display`/`Error`-Implementierungen aus einem einfachen `enum`
  mit `#[error("...")]`-Attributen — für Bibliotheken, die gezielt behandelbare Fehler
  anbieten wollen.
- `#[from]` erzeugt automatisch `impl From<QuellFehler>`, damit `?` fremde Fehlertypen
  automatisch umwandelt. Fehlt es, meldet der Compiler exakt, welches `From` fehlt.
- Ein HTTP-Statuscode `4xx`/`5xx` lässt `send()` nicht automatisch fehlschlagen — wir
  müssen `status.is_success()` selbst prüfen.
- `anyhow` ist das Gegenstück für Anwendungscode: ein universeller Fehlertyp plus
  `.context(...)` für menschliche Fehlerketten, ohne dass Aufrufer je einzelne Varianten
  matchen müssten.
- Faustregel: `thiserror` in Bibliotheken, `anyhow` in Anwendungen.

## Übung

Ergänze `ProviderFehler` um eine Variante `Timeout`, die auftritt, wenn eine Anfrage zu
lange dauert. Recherchiere in der `reqwest`-Dokumentation, wie man einen Timeout auf
einem `reqwest::blocking::Client` konfiguriert (Stichwort: `ClientBuilder`), und wie sich
ein Timeout-Fehler von einem gewöhnlichen `reqwest::Error` unterscheiden lässt (Tipp:
`reqwest::Error` hat eine Methode `is_timeout()`). Baue das so ein, dass `chat()`
speziell bei einem Timeout die `Timeout`-Variante zurückgibt, bei jedem anderen
Netzwerkfehler weiterhin `Netzwerk`. Was sagt dir das über die Grenzen von `#[from]` —
warum reicht ein einfaches `#[from] reqwest::Error` hier nicht mehr aus, um beide Fälle
zu unterscheiden?

[Weiter: Lektion 5 — Prompt-Templating](05-prompt-templating.md)
