# Lektion 6: Structured Output mit schemars

## Problem

`client.chat(&konversation)` liefert bisher genau eine Sache: freien Text (`String`).
Für ein Gespräch reicht das. Aber sobald wir eine LLM-Antwort **programmatisch
weiterverarbeiten** wollen — z. B. eine Liste von Stichpunkten, ein strukturiertes
Suchergebnis, später (Phase 4) einen Werkzeugaufruf — reicht Fließtext nicht mehr. Wir
bräuchten ihn mit Regex oder Rateheuristiken zerlegen, was fragil ist: Das Modell
formuliert nie zweimal exakt gleich. Wir wollen stattdessen, dass das LLM uns direkt
**JSON in einer festen Form** liefert, die wir typisiert einlesen können — genau wie wir
es in [Lektion 3](03-request-response-typen.md) schon mit der API-Antwort selbst gemacht
haben, nur jetzt mit dem *Inhalt*, den das Modell frei formuliert.

## Code (Zielbild)

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Zusammenfassung {
    pub titel: String,
    pub kernpunkte: Vec<String>,
}
```

```rust
let anweisung = format!(
    "Antworte AUSSCHLIESSLICH mit JSON nach exakt diesem Schema:\n{}",
    schema_als_text::<Zusammenfassung>()
);

let mut konversation = Konversation::mit_systemnachricht(anweisung)?;
konversation.hinzufuegen(Rolle::Benutzer, "Fasse zusammen: ...")?;

let antwort_text = client.chat(&konversation)?;
let zusammenfassung: Zusammenfassung = strukturiert_einlesen(&antwort_text)?;
```

## Dekonstruktion

### Was ist ein JSON Schema?

Ein **JSON Schema** ist selbst wieder JSON — es beschreibt aber nicht Daten, sondern die
**Form**, die gültige Daten haben müssen: welche Felder es gibt, welchen Typ sie haben,
welche Pflicht sind. Es ist zu JSON, was `struct Zusammenfassung { ... }` zu einem
Rust-Wert ist: eine Beschreibung der Struktur, keine Instanz davon. Viele LLM-Anbieter
können ein solches Schema als Teil des Prompts oder als eigenen API-Parameter
entgegennehmen und ihre Antwort daran ausrichten ("Structured Output" bzw. teils
"Function Calling" genannt — wir vertiefen Function Calling in
[Phase 4, Lektion 3](../05-phase4-agenten/03-tool-schema-function-calling.md); hier geht
es uns nur um die *Form* der Antwort, noch nicht um Werkzeugaufrufe).

### `schemars` — JSON Schema aus einem Rust-Typ ableiten

Genauso wie `#[derive(Serialize, Deserialize)]` aus [Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md)
Code zum Umwandeln zwischen Rust-Werten und JSON-*Daten* generiert, generiert
`#[derive(JsonSchema)]` (aus dem Crate `schemars`) Code, der die JSON-*Form* eines Typs
beschreibt. `schemars::schema_for!(T)` baut daraus konkret ein `Schema`-Objekt, das wir
selbst wieder mit `serde_json::to_string_pretty` in lesbaren JSON-Text verwandeln
können:

```rust
pub fn schema_als_text<T: JsonSchema>() -> String {
    let schema = schemars::schema_for!(T);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}
```

### Ein **Trait Bound**: `T: JsonSchema`

```rust
pub fn schema_als_text<T: JsonSchema>() -> String { ... }
```

Das ist eine **generische Funktion** — sie funktioniert für *jeden* Typ `T`, nicht nur
für `Zusammenfassung`. `<T: JsonSchema>` heißt: "T kann jeder beliebige Typ sein, solange
er das Trait `JsonSchema` implementiert" — das nennt man einen **Trait Bound**
(*Bound* = Einschränkung). Erinnere dich an Traits als "Vertrag" aus
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md): Ohne den Bound
wüsste der Compiler bei `schemars::schema_for!(T)` nicht, *dass* `T` überhaupt ein Schema
erzeugen kann — mit `T: JsonSchema` garantiert der Aufrufer das schon beim Kompilieren.
Rufst du `schema_als_text::<Zusammenfassung>()` auf, muss `Zusammenfassung` also
`#[derive(JsonSchema)]` haben, sonst lehnt der Compiler den Aufruf ab (siehe unten).

### Die Antwort typisiert einlesen

```rust
pub fn strukturiert_einlesen<T: serde::de::DeserializeOwned>(
    antwort_text: &str,
) -> Result<T, ProviderFehler> {
    Ok(serde_json::from_str(antwort_text)?)
}
```

Auch das ist generisch (`T: DeserializeOwned` — eine Variante von `Deserialize`, die
speziell für Werte gedacht ist, die *keine* Referenzen auf die Eingabe mehr halten,
üblich bei `serde_json::from_str`). Das `?` am Ende funktioniert **nur**, weil wir in
[Lektion 4](04-fehlerbehandlung.md) `ProviderFehler::UngueltigesFormat(#[from]
serde_json::Error)` bereits eingerichtet haben — ein `serde_json::Error` beim Parsen
verwandelt sich hier automatisch in unseren eigenen, typisierten Fehler. Das ist
derselbe Vorteil aus Lektion 4, nur ein zweites Mal genutzt, ohne zusätzlichen Aufwand.

### Warum keine neue Client-Methode?

Auffällig: `schema_als_text` und `strukturiert_einlesen` sind **freie Funktionen**, keine
neue Methode auf `OpenAiKompatiblerClient`. Structured Output ist konzeptionell nichts
Neues — es ist eine **Kombination** aus Bausteinen, die wir schon haben:
`PromptTemplate`/eine Systemanweisung ([Lektion 5](05-prompt-templating.md)) zum
Einbetten des Schemas, `client.chat(&konversation)` ([Lektion 3](03-request-response-typen.md))
zum Senden, und generisches Parsen der Antwort. Eine eigene Methode
`chat_strukturiert(...)` wäre eine zusätzliche, redundante API-Fläche für etwas, das sich
genauso gut aus vorhandenen Teilen zusammensetzen lässt — auch das ist YAGNI in Aktion.

### Ein bewusst provozierter Trait-Bound-Fehler

Vergisst du `#[derive(JsonSchema)]` auf `Zusammenfassung`:

```rust
#[derive(Debug, Deserialize)] // JsonSchema fehlt!
pub struct Zusammenfassung {
    pub titel: String,
    pub kernpunkte: Vec<String>,
}
```

```rust
let text = schema_als_text::<Zusammenfassung>();
```

```
error[E0277]: the trait bound `Zusammenfassung: JsonSchema` is not satisfied
   --> mein_core/src/provider.rs:88:38
    |
88  |     let text = schema_als_text::<Zusammenfassung>();
    |                                  ^^^^^^^^^^^^^^^^ the trait `JsonSchema` is not implemented for `Zusammenfassung`
```

Exakt dasselbe Fehlermuster wie beim fehlenden `Serialize` in
[Lektion 3](03-request-response-typen.md) — nur diesmal an einem Trait Bound statt an
einem konkreten Funktionsaufruf wie `.json(...)`. Der Compiler prüft Trait Bounds an
jeder Aufrufstelle, nicht nur einmal irgendwo zentral.

> **⚠️ Warnung**
>
> `schemars` ist ein eigenständiges, sich weiterentwickelndes Crate — prüfe beim
> Einrichten (`cargo add schemars`) in der aktuellen Dokumentation auf docs.rs, ob
> `schema_for!` bei deiner installierten Version exakt so heißt bzw. ob `serde`-Ableitung
> zusätzlich ein Feature-Flag braucht. Die Grundidee (Typ → Schema → Text) bleibt über
> Versionen hinweg stabil, auch wenn sich Details am Rand mal verschieben.

## Schritt-Reveal

**Schritt 1** — Abhängigkeit ergänzen, `mein_core/Cargo.toml`:

```toml
[dependencies]
schemars = "..." # aktuelle stabile Version, z. B. via `cargo add schemars`
```

**Schritt 2** — `Zusammenfassung` (oder einen eigenen Beispieltyp) mit `#[derive(Debug,
Deserialize, JsonSchema)]` in `provider.rs` (oder einer neuen Datei, falls du mehrere
Structured-Output-Typen sammeln willst) anlegen.

**Schritt 3** — `schema_als_text` und `strukturiert_einlesen` wie oben ergänzen.

**Schritt 4** — Provoziere den Trait-Bound-Fehler bewusst (siehe oben), korrigiere ihn.

## Ausführung

```bash
cargo test -p mein_core
```

Da wir keinen echten Netzwerkaufruf brauchen, um das Einlesen zu testen, simulieren wir
eine Modellantwort als festen String — genau wie in [Lektion 2](02-json-schema.md) und
[Lektion 3](03-request-response-typen.md):

```rust
#[test]
fn gueltiges_json_wird_typisiert_eingelesen() {
    let antwort = r#"{
        "titel": "Rust",
        "kernpunkte": ["sicher", "schnell", "kein Garbage Collector"]
    }"#;

    let ergebnis: Zusammenfassung = strukturiert_einlesen(antwort).unwrap();

    assert_eq!(ergebnis.titel, "Rust");
    assert_eq!(ergebnis.kernpunkte.len(), 3);
}

#[test]
fn unpassendes_json_liefert_verstaendlichen_fehler() {
    let antwort = r#"{ "nur_falsche_felder": true }"#;

    let ergebnis: Result<Zusammenfassung, ProviderFehler> = strukturiert_einlesen(antwort);

    assert!(matches!(ergebnis, Err(ProviderFehler::UngueltigesFormat(_))));
}
```

```
running 2 tests
test provider::tests::gueltiges_json_wird_typisiert_eingelesen ... ok
test provider::tests::unpassendes_json_liefert_verstaendlichen_fehler ... ok
```

`matches!(wert, Muster)` ist ein Makro, das prüft, ob `wert` zu einem `match`-Muster
passt, und `true`/`false` zurückgibt — praktisch in Tests, wenn du nur die *Variante*
prüfen willst, ohne (wie bei `PromptFehler` in Lektion 5) `PartialEq` für den ganzen
Fehlertyp abzuleiten (`ProviderFehler` enthält u. a. `reqwest::Error`, das selbst kein
`PartialEq` implementiert — ein `#[derive(PartialEq)]` würde hier also gar nicht
kompilieren).

## Zusammenfassung

- Ein JSON Schema beschreibt die *Form* gültiger JSON-Daten — `#[derive(JsonSchema)]`
  leitet es automatisch aus einem Rust-Typ ab, analog zu `Serialize`/`Deserialize`.
- Trait Bounds (`T: JsonSchema`, `T: DeserializeOwned`) erlauben generische Funktionen,
  die für *jeden* passenden Typ funktionieren — der Compiler prüft die Voraussetzung an
  jeder Aufrufstelle.
- Structured Output braucht keine neue Client-Methode — es ist eine Kombination aus
  Prompt-Templating, `chat()` und generischem JSON-Parsing, alles bereits vorhanden.
- `ProviderFehler::UngueltigesFormat` (aus Lektion 4) fängt jetzt **zwei** Fehlerquellen
  ab: eine kaputte API-Antwort *und* eine Modellantwort, die nicht zum erwarteten Schema
  passt.

## Übung

Definiere einen eigenen Typ `Aufgabenliste` mit einem Feld `aufgaben: Vec<String>`
(`#[derive(Debug, Deserialize, JsonSchema)]`). Baue dir mit `PromptTemplate` aus
[Lektion 5](05-prompt-templating.md) eine Systemanweisung, die sowohl das per
`schema_als_text::<Aufgabenliste>()` erzeugte Schema **als auch** eine Variable für das
Thema enthält (z. B. `{{thema}}`). Rendere das Template, baue daraus eine
`Konversation`, rufe (echt oder simuliert) `chat` auf und lies die Antwort mit
`strukturiert_einlesen::<Aufgabenliste>` ein. Was passiert, testest du bewusst mit einer
Antwort, die zwar gültiges JSON ist, aber ein anderes Feld hat als erwartet (z. B.
`{"todos": [...]}` statt `{"aufgaben": [...]}`)? Vergleiche die Fehlermeldung mit der aus
dieser Lektion — worin unterscheidet sie sich, und warum?

[Weiter: Lektion 7 — Persistenz mit sqlx](07-persistenz-sqlx.md)
