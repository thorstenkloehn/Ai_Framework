# Lektion 2: JSON-Schema mit serde_json

## Problem

In Lektion 1 haben wir bewiesen, dass wir überhaupt etwas über HTTP verschicken und
zurücklesen können — aber mit beliebigem JSON an einen Test-Dienst. Ein echter LLM-Anbieter
erwartet ein **festes Format**: bestimmte Feldnamen, bestimmte Verschachtelung, bestimmte
Groß-/Kleinschreibung bei Werten. Bevor wir eigene Rust-Typen dafür bauen (das ist Thema
von Lektion 3), müssen wir erst verstehen, wie dieses Format überhaupt aussieht — und ob
unsere Phase-1-Typen `Rolle`/`Nachricht` nicht vielleicht schon reichen.

## Code (Zielbild)

Ein typischer Chat-Completion-Request, wie ihn die meisten OpenAI-kompatiblen
HTTP-APIs erwarten (die exakten Felder unterscheiden sich leicht je nach Anbieter — das
Grundmuster aber kaum):

```json
{
  "model": "irgendein-modell",
  "messages": [
    { "role": "system", "content": "Du bist hilfreich." },
    { "role": "user", "content": "Hallo, wer bist du?" }
  ],
  "temperature": 0.7
}
```

Und eine typische Antwort:

```json
{
  "choices": [
    {
      "message": { "role": "assistant", "content": "Ich bin ein Sprachmodell." }
    }
  ]
}
```

## Dekonstruktion

### JSON kurz erklärt

**JSON** (*JavaScript Object Notation*) ist ein textbasiertes Datenformat mit genau
fünf Bausteinen: Objekte (`{ "schlüssel": wert }`), Arrays (`[wert, wert]`), Strings
(`"text"`), Zahlen (`42`, `0.7`) und die Literale `true`/`false`/`null`. Das war's — kein
Kommentar-Syntax, keine Trailing-Commas, jeder Schlüssel ein String in doppelten
Anführungszeichen. JSON ist heute der De-facto-Standard für HTTP-APIs, auch weil es so
klein und eindeutig spezifiziert ist.

Beachte im Beispiel oben: `role` (nicht `rolle`), `content` (nicht `inhalt`), Werte wie
`"user"` klein geschrieben (nicht `"Benutzer"`). Das ist kein Zufall — es ist einfach ein
anderes Vokabular als unser deutsches Domain-Modell aus Phase 1.

### `serde_json::Value` — JSON ohne festen Rust-Typ

`serde_json` (seit
[Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md) in
`mein_core/Cargo.toml`) bietet neben `#[derive(Serialize/Deserialize)]` auch einen
dynamischen Typ an: `serde_json::Value`. Er kann **jedes** gültige JSON-Dokument
darstellen, ohne dass du vorher einen passenden `struct` definierst — praktisch, um
fremdes JSON zu erkunden, bevor du weißt, wie es vollständig aussieht:

```rust
use serde_json::{json, Value};

let anfrage: Value = json!({
    "model": "irgendein-modell",
    "messages": [
        { "role": "system", "content": "Du bist hilfreich." },
        { "role": "user", "content": "Hallo, wer bist du?" }
    ],
    "temperature": 0.7
});

println!("{}", anfrage["messages"][0]["role"]); // "system"
```

`Value` lässt sich wie ein verschachteltes Array/Objekt indizieren (`anfrage["messages"]
[0]["role"]`), fast wie in JavaScript oder Python. Genau das ist aber auch das Problem:
Der Compiler weiß **nichts** über die Struktur — ob der Schlüssel `"role"` existiert, ob
er wirklich ein String ist, prüft er nicht beim Kompilieren, sondern erst zur Laufzeit.

### Ein bewusster Griff daneben: Feldzugriff auf `Value`

Nehmen wir an, wir haben eine Beispiel-Antwort (fest im Code, weil wir noch keinen
echten Anbieter ansprechen) und wollen den Antworttext extrahieren:

```rust
let beispiel_antwort = r#"{
    "choices": [
        { "message": { "role": "assistant", "content": "Ich bin ein Sprachmodell." } }
    ]
}"#;

let werte: Value = serde_json::from_str(beispiel_antwort).unwrap();
let text: &str = werte["choices"][0]["message"]["content"].as_str();
```

`.as_str()` gibt **nicht** `&str` zurück, sondern `Option<&str>` — "vielleicht ein
String, falls der Wert an dieser Stelle überhaupt existiert und ein String ist". Der
Compiler meldet:

```
error[E0308]: mismatched types
  |
  | let text: &str = werte["choices"][0]["message"]["content"].as_str();
  |           ----   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `&str`, found `Option<&str>`
  |           |
  |           expected due to this
```

Das ist der Kern des Problems mit `Value`: Jeder einzelne Feldzugriff *könnte*
fehlschlagen (falscher Schlüssel, falscher Typ, fehlendes Feld) — und der Compiler
zwingt dich, das über `Option`/`Result` zuzugeben, aber er kann dir **nicht** vorher
sagen, welche Schlüssel überhaupt gültig sind. Ein Tippfehler wie `"conent"` statt
`"content"` fällt erst zur Laufzeit auf (dann als `None`), nicht beim Kompilieren.

Korrigiert, mit `if let`:

```rust
if let Some(text) = werte["choices"][0]["message"]["content"].as_str() {
    println!("{text}");
} else {
    eprintln!("Antwort hatte nicht das erwartete Format.");
}
```

### Reicht nicht vielleicht `Nachricht` direkt?

`Nachricht` ist seit [Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md)
bereits `Serialize`. Probieren wir es aus:

```rust
let nachricht = Nachricht::neu(Rolle::Benutzer, "Hallo, wer bist du?").unwrap();
println!("{}", serde_json::to_string(&nachricht).unwrap());
```

Ausgabe:

```
{"rolle":"Benutzer","inhalt":"Hallo, wer bist du?"}
```

Vergleiche das mit dem, was die API erwartet: `{"role":"user","content":"Hallo, wer bist
du?"}`. Zwei Unterschiede — die Feldnamen (`rolle`/`inhalt` vs. `role`/`content`) *und*
die Groß-/Kleinschreibung des Rollenwerts (`"Benutzer"` vs. `"user"`, `"Rolle::Benutzer"`
serialisiert außerdem nicht 1:1 zu `"user"`, sondern zu `"Benutzer"` — unser deutscher
Enum-Variantenname). `Nachricht` direkt loszuschicken würde beim echten Anbieter mit
einer Fehlermeldung wie *"unknown role: Benutzer"* zurückkommen. Wir könnten das mit
`#[serde(rename = "...")]` pro Feld reparieren — aber dann trägt unser sauberer,
deutscher Domain-Typ aus Phase 1 auf einmal API-spezifische Übersetzungsattribute. Genau
dieses Problem lösen wir in [Lektion 3](03-request-response-typen.md) sauber: mit
eigenen, von der Domäne getrennten Typen für das, was über die Leitung geht.

> **💡 Tipp**
>
> `serde_json::to_string_pretty(&wert)` statt `to_string` gibt eingerücktes, lesbares
> JSON zurück — praktisch beim Debuggen, aber unnötig größer für den tatsächlichen
> Netzwerkversand.

## Schritt-Reveal

**Schritt 1** — Lege testweise (z. B. in einem `#[cfg(test)]`-Modul in
`mein_core/src/provider.rs`) den `Value`-Zugriff aus dem Zielbild an, inklusive des
absichtlichen Fehlers mit `let text: &str = ...as_str();`. Beobachte den
Compilerfehler wörtlich, korrigiere mit `if let Some(text) = ...`.

**Schritt 2** — Schreibe einen Test, der zeigt, dass `serde_json::to_string(&nachricht)`
tatsächlich `rolle`/`inhalt` statt `role`/`content` produziert:

```rust
#[test]
fn nachricht_serialisiert_noch_nicht_im_api_format() {
    let nachricht = Nachricht::neu(Rolle::Benutzer, "Hallo").unwrap();
    let json = serde_json::to_string(&nachricht).unwrap();
    assert!(json.contains("\"rolle\""));
    assert!(!json.contains("\"role\""));
}
```

Dieser Test ist bewusst kein "Fehler", den wir beheben — er **dokumentiert** die Lücke,
die Lektion 3 schließt. Lösche ihn wieder, sobald du Lektion 3 abgeschlossen hast, oder
markiere ihn als `#[ignore]` mit einem Kommentar, warum.

## Ausführung

```bash
cargo test -p mein_core
```

```
running 1 test
test provider::tests::nachricht_serialisiert_noch_nicht_im_api_format ... ok
```

## Zusammenfassung

- JSON kennt fünf Bausteine: Objekt, Array, String, Zahl, Bool/Null — kein festes
  Rust-Vokabular.
- `serde_json::Value` stellt beliebiges JSON dar, ohne vorher einen Rust-Typ zu
  definieren — praktisch zum Erkunden, riskant für dauerhaften Code, weil der Compiler
  Schlüssel und Typen nicht prüfen kann.
- `.as_str()`, `.as_i64()` & Co. geben auf `Value` immer `Option<...>` zurück, nie den
  Wert direkt — ein Feldzugriff kann immer "daneben gehen".
- Unsere Phase-1-Typen `Rolle`/`Nachricht` serialisieren aktuell **nicht** im Format,
  das eine echte API erwartet (`rolle`/`inhalt` statt `role`/`content`, andere
  Groß-/Kleinschreibung bei den Werten).

## Übung

Erweitere den `Value`-Zugriff aus dieser Lektion um einen zweiten, ebenfalls bewusst
provozierten Fehlerfall: Was passiert, wenn `beispiel_antwort` ein **leeres**
`"choices"`-Array enthält (`"choices": []`)? Greife testweise trotzdem auf `[0]` zu und
beobachte, was `Value` in diesem Fall zurückgibt (Tipp: es ist kein Compilerfehler mehr,
sondern etwas, das du erst mit `if let`/`match` bemerkst — genau das macht `Value` so
tückisch für echten Code). Schreibe einen Test, der diesen Fall sauber als `Option::None`
behandelt, statt ihn zu ignorieren.

[Weiter: Lektion 3 — Request- und Response-Typen trennen](03-request-response-typen.md)
