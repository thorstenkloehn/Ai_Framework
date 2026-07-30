# Lektion 5: Prompt-Templating

## Problem

Ein Prompt (der Text, den wir als Nutzernachricht an ein LLM schicken) ist selten
komplett fest — meistens hat er feste Teile ("Fasse den folgenden Text auf {{sprache}}
zusammen:") und variable Teile, die je nach Aufruf wechseln ({{text}}, {{sprache}}). Bauen
wir diese Variablen naiv mit `format!`/String-Verkettung zusammen, fällt ein Tippfehler
oder eine vergessene Variable erst auf, wenn entweder ein falscher Prompt beim Anbieter
ankommt (teuer und schwer zu debuggen — die Antwort ist einfach seltsam, ohne dass ein
Fehler auftritt) oder wenn ein Platzhalter wie `{{sprache}}` **unersetzt** im Text an das
LLM geschickt wird. Wir wollen das Gegenteil: **Eine ungültige Prompt-Variable soll vor
dem Netzwerkaufruf als verständlicher Fehler erscheinen.** Das ist die Transferaufgabe
dieser Phase (siehe [README](README.md)) — wir lösen sie in dieser Lektion vollständig.

## Code (Zielbild)

```rust
use std::collections::HashMap;

pub struct PromptTemplate {
    text: String,
}

impl PromptTemplate {
    pub fn neu(text: impl Into<String>) -> Self {
        PromptTemplate { text: text.into() }
    }

    pub fn rendern(&self, variablen: &HashMap<String, String>) -> Result<String, PromptFehler> {
        // prüft zuerst ALLE Variablen, ersetzt erst danach
    }
}
```

```rust
let vorlage = PromptTemplate::neu("Fasse folgenden Text auf {{sprache}} zusammen: {{text}}");
let mut variablen = HashMap::new();
variablen.insert("text".to_string(), "Rust ist eine Systemprogrammiersprache.".to_string());
// "sprache" wird vergessen!

let ergebnis = vorlage.rendern(&variablen);
// Err(PromptFehler::FehlendeVariable("sprache")) — KEIN Netzwerkaufruf hat stattgefunden
```

## Dekonstruktion

### Ein neues Modul: `mein_core::prompt`

Genau wie `provider` in [Lektion 1](01-http-grenze-reqwest.md) bekommt auch das
Templating eine eigene Datei, `mein_core/src/prompt.rs`, verbunden über `pub mod
prompt;` in `lib.rs`. Templating hat mit HTTP nichts zu tun — es ist reine
Textverarbeitung, die genauso gut ohne jedes Netzwerk funktionieren und getestet werden
muss. Diese Trennung ist wichtig: `PromptTemplate::rendern` braucht **keinen**
`OpenAiKompatiblerClient`, um zu funktionieren oder getestet zu werden.

### Warum zwei Durchgänge — erst validieren, dann ersetzen?

Der naheliegende, aber falsche Ansatz wäre, Platzhalter direkt beim ersten Finden zu
ersetzen und bei einem fehlenden Wert sofort abzubrechen. Das Problem: Findet man den
*ersten* fehlenden Platzhalter erst nach der Hälfte des Texts, hat man vorher schon
Zeit in nutzlose Teilarbeit gesteckt, und schlimmer: Man merkt nicht, ob es *noch mehr*
fehlende Variablen gäbe. Wir gehen deshalb zweistufig vor:

1. **Alle** benötigten Platzhalter aus dem Template extrahieren und **alle** gegen die
   übergebenen Variablen prüfen — bricht hier etwas, ist der Fehler eindeutig benannt.
2. Erst wenn Schritt 1 vollständig durchläuft, tatsächlich ersetzen.

Das ist dieselbe Grundidee wie **Invarianten prüfen, bevor man einen Wert konstruiert**
aus [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) — nur jetzt nicht bei
der Konstruktion eines Typs, sondern bei der Konstruktion eines Prompt-Texts.

### Platzhalter finden: `split("{{")` und `split_once("}}")`

```rust
fn platzhalter_namen(text: &str) -> Vec<String> {
    let mut namen = Vec::new();
    for teil in text.split("{{").skip(1) {
        if let Some((name, _rest)) = teil.split_once("}}") {
            namen.push(name.trim().to_string());
        }
    }
    namen
}
```

`text.split("{{")` zerlegt den Text an jedem Vorkommen von `"{{"` in Stücke.
`.skip(1)` überspringt das allererste Stück (das, was **vor** dem ersten `{{` steht —
dort kann kein Platzhaltername beginnen). Für jedes verbleibende Stück versucht
`split_once("}}")` es an der **ersten** `"}}"` in zwei Teile zu spalten: den
Platzhalternamen davor, den Rest danach. `.trim()` entfernt versehentliche Leerzeichen
wie in `{{ sprache }}`. Wir verzichten hier bewusst auf eine externe Regex-Bibliothek —
für ein so einfaches Muster reicht Rusts eingebaute `str`-API vollständig, eine weitere
Abhängigkeit wäre unnötig ([YAGNI](../09-anhang/01-glossar.md), wie schon in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md) erwähnt).

### Der Fehlertyp: `PromptFehler`

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PromptFehler {
    #[error("Variable '{0}' wird im Template verwendet, aber beim Aufruf nicht angegeben")]
    FehlendeVariable(String),

    #[error("Variable '{0}' wurde angegeben, kommt aber im Template nicht vor")]
    UnbenutzteVariable(String),
}
```

Wir nutzen `thiserror`, das wir gerade in [Lektion 4](04-fehlerbehandlung.md)
kennengelernt haben — derselbe Mechanismus, ein neuer, eigenständiger Fehlertyp. Zwei
Fälle statt einem: `FehlendeVariable` (das Template braucht etwas, das nicht geliefert
wurde — das ist unser Kernfall) und `UnbenutzteVariable` (umgekehrt: es wurde etwas
geliefert, das das Template gar nicht kennt — meist ein Tippfehler im Variablennamen
beim Aufruf, z. B. `"sprchae"` statt `"sprache"`). Beide Fälle sind "eine ungültige
Prompt-Variable", nur aus entgegengesetzter Richtung.

### `rendern` vollständig

```rust
pub fn rendern(&self, variablen: &HashMap<String, String>) -> Result<String, PromptFehler> {
    let benoetigt = platzhalter_namen(&self.text);

    for name in &benoetigt {
        if !variablen.contains_key(name) {
            return Err(PromptFehler::FehlendeVariable(name.clone()));
        }
    }

    for name in variablen.keys() {
        if !benoetigt.contains(name) {
            return Err(PromptFehler::UnbenutzteVariable(name.clone()));
        }
    }

    let mut ergebnis = self.text.clone();
    for (name, wert) in variablen {
        let platzhalter = "{{".to_string() + name + "}}";
        ergebnis = ergebnis.replace(&platzhalter, wert);
    }

    Ok(ergebnis)
}
```

`"{{".to_string() + name + "}}"` baut den vollständigen Platzhaltertext (z. B.
`"{{sprache}}"`) durch String-Verkettung mit `+` — für `String + &str` überladen Rusts
Standardbibliothek den `+`-Operator genau dafür. `.replace(&platzhalter, wert)` tauscht
**jedes** Vorkommen des Platzhalters im Text gegen den Wert aus.

> **💡 Tipp**
>
> Du wirst in echtem Rust-Code oft `format!("{{{{{name}}}}}")` statt der
> String-Verkettung oben sehen — `{{` und `}}` sind in `format!`-Strings die Escape-
> Sequenz für ein einzelnes, *literales* `{` bzw. `}` (weil `{` und `}` sonst als
> Platzhalter interpretiert würden). Zwei literale geschweifte Klammern brauchen also
> `{{{{`/`}}}}` (verdoppelt, verdoppelt). Das ist correct, aber schwer auf den ersten
> Blick zu lesen — die String-Verkettung oben ist für den Einstieg klarer, merke dir die
> `format!`-Variante aber für später.

### Die Transferaufgabe konkret gelöst

```rust
let vorlage = PromptTemplate::neu("Fasse folgenden Text auf {{sprache}} zusammen: {{text}}");
let mut variablen = HashMap::new();
variablen.insert("text".to_string(), "Rust ist eine Systemprogrammiersprache.".to_string());

let gerendert = vorlage.rendern(&variablen)?;
// bis hierhin kommen wir NICHT, wenn "sprache" fehlt — chat() wird gar nicht erst aufgerufen
```

Weil `rendern` ein `Result<String, PromptFehler>` zurückgibt und wir es mit `?` (oder
`match`) behandeln, **bevor** wir `client.chat(&konversation)` überhaupt aufrufen, kann
eine fehlende oder überzählige Variable niemals bis zum Netzwerkaufruf durchsickern. Der
Fehler `PromptFehler::FehlendeVariable("sprache")` trägt (dank `thiserror`) eine
sprechende `Display`-Meldung: *"Variable 'sprache' wird im Template verwendet, aber beim
Aufruf nicht angegeben"* — verständlich für jeden, der den Fehler liest, auch ohne
Rust-Kenntnisse.

## Schritt-Reveal

**Schritt 1** — Modul anlegen: `mein_core/src/prompt.rs`, `pub mod prompt;` in
`lib.rs`. `use thiserror::Error;` und `use std::collections::HashMap;` ergänzen.

**Schritt 2** — `PromptFehler` und `platzhalter_namen` wie oben anlegen.

**Schritt 3** — `PromptTemplate` mit `neu` und `rendern` anlegen.

**Schritt 4** — Provoziere einen Compilerfehler bewusst: Rufe `rendern` versehentlich
ohne `?`/`match` auf, dort, wo eine Funktion selbst `String` (nicht `Result<String,
...>`) zurückgibt:

```rust
fn beispiel_prompt(text: String) -> String {
    let vorlage = PromptTemplate::neu("Text: {{text}}");
    let mut variablen = HashMap::new();
    variablen.insert("text".to_string(), text);

    vorlage.rendern(&variablen) // fehlendes `?`!
}
```

```
error[E0308]: mismatched types
  |
  | fn beispiel_prompt(text: String) -> String {
  |                                     ------ expected `String` because of return type
...
  |     vorlage.rendern(&variablen)
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `String`, found `Result<String, PromptFehler>`
```

Genau dasselbe Muster wie schon bei `Nachricht::neu` in
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md): `rendern` **kann**
fehlschlagen, also muss der Rückgabetyp das zeigen — der Compiler lässt uns das
`Result` nicht einfach ignorieren. Korrektur: entweder `-> Result<String, PromptFehler>`
als Rückgabetyp der Funktion, oder das `Result` an Ort und Stelle mit `match`/`?`
behandeln.

## Ausführung

```bash
cargo test -p mein_core
```

Ergänze Tests, die genau die Transferaufgabe abdecken:

```rust
#[test]
fn fehlende_variable_wird_als_verstaendlicher_fehler_erkannt() {
    let vorlage = PromptTemplate::neu("Hallo {{name}}, wie geht es {{ort}}?");
    let mut variablen = HashMap::new();
    variablen.insert("name".to_string(), "Welt".to_string());
    // "ort" fehlt absichtlich

    let ergebnis = vorlage.rendern(&variablen);

    assert_eq!(
        ergebnis,
        Err(PromptFehler::FehlendeVariable("ort".to_string()))
    );
}

#[test]
fn ungenutzte_variable_wird_erkannt() {
    let vorlage = PromptTemplate::neu("Hallo {{name}}!");
    let mut variablen = HashMap::new();
    variablen.insert("name".to_string(), "Welt".to_string());
    variablen.insert("tippfehler".to_string(), "übrig".to_string());

    let ergebnis = vorlage.rendern(&variablen);

    assert_eq!(
        ergebnis,
        Err(PromptFehler::UnbenutzteVariable("tippfehler".to_string()))
    );
}

#[test]
fn vollstaendige_variablen_werden_korrekt_ersetzt() {
    let vorlage = PromptTemplate::neu("Hallo {{name}}!");
    let mut variablen = HashMap::new();
    variablen.insert("name".to_string(), "Welt".to_string());

    assert_eq!(vorlage.rendern(&variablen), Ok("Hallo Welt!".to_string()));
}
```

```
running 3 tests
test prompt::tests::fehlende_variable_wird_als_verstaendlicher_fehler_erkannt ... ok
test prompt::tests::ungenutzte_variable_wird_erkannt ... ok
test prompt::tests::vollstaendige_variablen_werden_korrekt_ersetzt ... ok
```

Beachte: **Kein** einziger dieser Tests baut eine Netzwerkverbindung auf — genau das
zeigt, dass die Validierung vollständig *vor* der HTTP-Grenze aus
[Lektion 1](01-http-grenze-reqwest.md) stattfindet.

## Zusammenfassung

- `PromptTemplate::rendern` validiert **zuerst vollständig** (fehlende **und**
  überzählige Variablen), bevor es überhaupt einen Text zusammenbaut — dieselbe
  Grundidee wie Invarianten-Prüfung vor der Konstruktion aus Phase 1.
- `PromptFehler` (via `thiserror`) benennt den exakten Variablennamen im Fehlertext —
  verständlich, ohne dass jemand Rust-Code lesen muss.
- Textverarbeitung (`prompt`) bleibt bewusst getrennt von Netzwerk (`provider`) — beide
  Module kennen sich nicht gegenseitig.
- Damit ist die Transferaufgabe der Phase gelöst: Eine ungültige Prompt-Variable
  erscheint als verständlicher `PromptFehler`, **bevor** `client.chat(...)` je
  aufgerufen wird.

## Übung — Transferaufgabe der Phase

Verdrahte `PromptTemplate` jetzt tatsächlich mit `mein_cli`: Ergänze den `chat`-
Subcommand aus [Phase 1, Lektion 6](../02-phase1-fundament/06-cli-mit-clap.md) um ein
optionales Flag, mit dem Nutzer*innen ein Template und Variablen angeben können (z. B.
`--vorlage "Fasse {{text}} auf {{sprache}} zusammen"` plus mehrfach wiederholbares
`--var name=wert`, geparst über `clap`s `Vec<String>`-Unterstützung für wiederholte
Flags — schlage in der `clap`-Dokumentation nach, wie `#[arg(long)]` mit `Vec<String>`
zusammenspielt). Rufe `rendern(...)` auf, **bevor** du `konversation.hinzufuegen(...)`
oder gar `client.chat(...)` aufrufst. Prüfe von Hand: Lässt du bewusst eine Variable weg,
muss die Fehlermeldung im Terminal erscheinen — **und** `cargo run -p mein_cli` darf in
diesem Fall garantiert keine Netzwerkverbindung aufbauen (du kannst das z. B. daran
erkennen, dass das Programm auch ganz ohne Internetverbindung sofort mit der
Fehlermeldung reagiert). Genau das prüfst du auch noch einmal in der Definition of Done
von [Lektion 8](08-release-2.md).

[Weiter: Lektion 6 — Structured Output mit schemars](06-structured-output.md)
