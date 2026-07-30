# Lektion 2: Rolle und Nachricht als Domain Types

## Problem

Jede Konversation mit einem Sprachmodell besteht aus Nachrichten, und jede Nachricht hat
zwei Dinge: *wer* spricht (System, Nutzer oder Assistent) und *was* gesagt wird. Diese Idee
klingt trivial — die eigentliche Frage ist: Wie modellieren wir sie so, dass ungültige
Zustände (eine Nachricht ohne Rolle, eine Rolle, die nicht System/Nutzer/Assistent ist) gar
nicht erst entstehen können?

## Code (Zielbild)

Das ist der reale, bereits im Repository vorhandene Code — unser Ausgangspunkt, nicht das
Ziel, das wir neu erfinden. Wir verstehen ihn in dieser Lektion vollständig:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Rolle {
    System,
    Benutzer,
    Assistent,
}

#[derive(Debug, Clone)]
pub struct Nachricht {
    pub rolle: Rolle,
    pub inhalt: String,
}

impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht {
            rolle,
            inhalt: inhalt.into(),
        }
    }
}
```

## Dekonstruktion

### `pub enum Rolle` — ungültige Zustände unmöglich machen

Wir kennen `enum` schon aus [Kapitel 0](../01-grundlagen/04-daten-buendeln.md). Die
Alternative wäre gewesen, die Rolle als `String` zu speichern (`"system"`, `"user"`,
`"assistant"`). Das würde auch funktionieren — bis jemand `"System"` (Großschreibung),
`"assitant"` (Tippfehler) oder `""` (leer) hineinschreibt, und das fällt erst zur Laufzeit
auf, oft weit entfernt von der Stelle, wo der Fehler entstand. Mit `enum Rolle { System,
Benutzer, Assistent }` sind das die **einzigen drei** Werte, die überhaupt existieren
können — der Compiler garantiert das für uns, kostenlos, bei jedem Kompilieren.

`pub` vor `enum` bedeutet: Dieser Typ ist **öffentlich**, andere Crates (wie `mein_cli`)
dürfen ihn benutzen. Ohne `pub` wäre `Rolle` nur innerhalb von `mein_core` sichtbar.
Sichtbarkeit ist in Rust immer explizit — nichts ist "aus Versehen" öffentlich.

### `#[derive(Debug, Clone, PartialEq)]` — Fähigkeiten automatisch generieren

Diese Zeile über `enum Rolle` heißt **Attribut**, konkret ein `derive`-Attribut. Es sagt
dem Compiler: "Generiere für `Rolle` automatisch die Standardimplementierung von drei
**Traits**." Ein Trait ist ein Vertrag — eine Menge von Fähigkeiten, die ein Typ hat. Wir
vertiefen Traits ausführlich in
[Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md), aber diese drei
brauchen wir jetzt schon:

- **`Debug`** — erlaubt `println!("{:?}", wert)`, eine für Entwickler*innen lesbare
  Darstellung. Genau das haben wir in [Lektion 1](01-workspace-lesen.md) in `mein_cli`
  gesehen.
- **`Clone`** — erlaubt `wert.clone()`, eine echte Kopie zu erzeugen. Ohne `Clone` könntest
  du einen `Rolle`-Wert nicht einfach duplizieren (Hintergrund: Ownership, siehe unten).
- **`PartialEq`** — erlaubt den Vergleich mit `==`, z. B. `if rolle == Rolle::System { ...
  }`. Warum steht `PartialEq` bei `Nachricht` (noch) nicht dabei? Weil wir Nachrichten
  aktuell nicht vergleichen müssen — ungenutzte Fähigkeiten lassen wir bewusst weg, das
  ist kein Versehen, sondern [YAGNI](../09-anhang/01-glossar.md) ("You Aren't Gonna Need
  It").

> **💡 Tipp**
>
> Tippe `#[derive(Debug, Clone, PartialEq)]` versehentlich als `#[derive(debug, clone)]`
> (klein geschrieben) — der Compiler wird sich beschweren, dass er die Traits `debug`/
> `clone` nicht kennt. Traitnamen sind, wie Typnamen allgemein in Rust, per Konvention
> `PascalCase` (jedes Wort groß beginnend, keine Unterstriche).

### `pub struct Nachricht` — zwei Felder, beide öffentlich

```rust
pub struct Nachricht {
    pub rolle: Rolle,
    pub inhalt: String,
}
```

Beachte: `pub` steht **zweimal** — einmal vor `struct` (der Typ selbst ist öffentlich),
einmal vor jedem Feld (`rolle`, `inhalt` sind einzeln öffentlich lesbar/schreibbar von
außen). Das ist in Rust bewusst getrennt: Ein `struct` könnte öffentlich sein, aber
einzelne Felder privat halten (dazu genau in
[Lektion 3](03-invarianten.md) mehr, wenn wir überlegen, ob das für `inhalt` sinnvoll
wäre, um leere Nachrichten zu verhindern).

`inhalt: String` statt `inhalt: &str` — hier kommt Ownership ins Spiel
([Variablen und Typen](../01-grundlagen/01-variablen-und-typen.md) hat es kurz
angerissen). Eine `Nachricht` soll ihren Text **besitzen**: Wenn die Funktion, aus der der
Text ursprünglich kam, längst beendet ist, muss der Text trotzdem noch gültig sein, solange
die `Nachricht` existiert. `&str` wäre nur ein *Verweis* auf Text, der irgendwo anders
lebt — dieser "irgendwo anders"-Ort müsste garantiert länger leben als unsere `Nachricht`,
was wir hier nicht sinnvoll garantieren können/wollen. `String` löst das: Es kopiert den
Text in eigenen, von der `Nachricht` besessenen Speicher.

### `impl Nachricht { pub fn neu(...) }` — ein Konstruktor per Konvention

```rust
impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht {
            rolle,
            inhalt: inhalt.into(),
        }
    }
}
```

Rust hat kein eingebautes Konstruktor-Schlüsselwort wie `new` in Java oder `__init__` in
Python. Stattdessen ist es Konvention, eine assoziierte Funktion namens `new` (hier
verdeutscht: `neu`) zu schreiben, die einen fertigen Wert zurückgibt. "Assoziiert" heißt:
Sie gehört zum Typ, braucht aber (anders als eine Methode) noch kein existierendes
`Nachricht`-Objekt — deshalb wird sie mit `Nachricht::neu(...)` aufgerufen (Doppelpunkt),
nicht `nachricht.neu(...)` (Punkt).

`-> Self` — `Self` ist innerhalb eines `impl Nachricht`-Blocks eine Abkürzung für
`Nachricht`. Ändert sich später der Typname, muss dieser Rückgabetyp nicht mit angepasst
werden.

`inhalt: impl Into<String>` ist die interessanteste Zeile hier. `impl Into<String>`
bedeutet: "irgendein Typ, der sich in einen `String` umwandeln lässt." Sowohl `&str`
(`"Hallo"`) als auch bereits vorhandene `String`-Werte erfüllen das. Das macht die
Funktion **ergonomisch**: Du kannst sowohl

```rust
Nachricht::neu(Rolle::Benutzer, "Hallo")               // &str
```

als auch

```rust
let text = String::from("Hallo");
Nachricht::neu(Rolle::Benutzer, text)                    // String
```

schreiben, ohne dass die Aufrufer*innen sich um den genauen Texttyp kümmern müssen.
`inhalt.into()` im Funktionskörper führt die tatsächliche Umwandlung durch. Dieses Muster
— `impl Into<String>` als Parametertyp, `.into()` im Körper — wirst du in Rust-Bibliotheken
ständig wiedersehen; merke es dir als Standard-Rezept für "flexibel Text entgegennehmen".

## Schritt-Reveal

Wir haben in dieser Lektion nichts Neues implementiert — der Code existiert schon. Statt
eines Schritt-Reveals verifizierst du dein Verständnis, indem du dir selbst drei Fragen im
Code beantwortest (tippe die Antworten testweise als Kommentare direkt über die jeweilige
Zeile in `mein_core/src/lib.rs`, lösche sie danach wieder):

1. Warum kompiliert `Rolle::System == Rolle::System` nur, weil `PartialEq` abgeleitet ist?
   Entferne testweise `PartialEq` aus der `derive`-Liste, versuche `Rolle::System ==
   Rolle::System` irgendwo zu schreiben (z. B. in `mein_cli/src/main.rs`), beobachte den
   Compilerfehler, mach die Änderung rückgängig.
2. Warum würde `Nachricht::neu(Rolle::Benutzer, 42)` **nicht** kompilieren? (Antwort: `42`
   ist ein `i32`, kein Typ, der `Into<String>` implementiert.) Probiere es aus, lies die
   Fehlermeldung.
3. Was passiert, wenn du `pub` vor `inhalt: String` entfernst und danach in `mein_cli`
   versuchst, `nachricht.inhalt` zu lesen? Probiere es, lies die Fehlermeldung — sie nennt
   das Feld explizit "private".

## Ausführung

```bash
cargo check -p mein_core
cargo check -p mein_cli
```

Beide sollten sauber durchlaufen (`Finished`, keine Fehler). Falls du bei den drei
Experimenten oben etwas stehen gelassen hast: `git diff` zeigt dir, was noch nicht
zurückgesetzt ist.

## Zusammenfassung

- `enum Rolle` macht ungültige Rollen-Werte zur Compile-Zeit unmöglich — die robustere
  Alternative zu einem `String`-Feld mit "erlaubten" Werten.
- `#[derive(...)]` generiert Standardverhalten (`Debug`, `Clone`, `PartialEq`) ohne
  Schreibarbeit — nur die Fähigkeiten ableiten, die wir tatsächlich brauchen.
- `pub` ist zweistufig: Typ und einzelne Felder werden unabhängig sichtbar gemacht.
- `String` statt `&str` als Feldtyp, weil `Nachricht` ihren Inhalt **besitzen** soll.
- `impl Into<String>` + `.into()` ist das Standardmuster für ergonomische, flexible
  Text-Parameter.

## Übung

`Rolle` hat aktuell keine Möglichkeit, sich selbst als Text auszugeben (z. B. für ein
späteres JSON-Format oder eine Log-Ausgabe) außer über `{:?}` (Debug). Schreibe — auf
einem eigenen Branch oder testweise, ohne es sofort zu committen — eine Methode `fn
als_text(&self) -> &'static str` in einem `impl Rolle`-Block, die `"system"`,
`"benutzer"` bzw. `"assistent"` zurückgibt (klein geschrieben, wie es später ein
JSON-Feld erwarten könnte). Nutze dafür `match` (siehe
[Kontrollfluss](../01-grundlagen/03-kontrollfluss.md)). Was passiert mit deiner Methode,
wenn du testweise einen vierten `Rolle`-Fall wie `Werkzeug` hinzufügst, aber vergisst,
`als_text` anzupassen? Beobachte den Compilerfehler — er zeigt dir schon jetzt, warum wir
in [Lektion 5](05-serde-konfiguration.md) `serde` statt einer Handschreib-Lösung nutzen,
sobald es um echte JSON-Serialisierung geht.

[Weiter: Lektion 3 — Invarianten schützen](03-invarianten.md)
