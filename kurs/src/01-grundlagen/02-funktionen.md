# Funktionen

## Wiederholung vermeiden, indem wir Anweisungen bündeln

Stell dir vor, du müsstest an zehn Stellen im Programm dieselben drei Zeilen Code
schreiben, um eine Begrüßung zu erzeugen. Änderst du die Begrüßung später, musst du sie an
allen zehn Stellen ändern — fehleranfällig und mühsam. Eine **Funktion** löst das: Wir
schreiben den Code einmal, geben ihm einen Namen, und rufen ihn überall dort auf, wo wir
ihn brauchen.

```rust
fn begruessung(name: &str) -> String {
    format!("Hallo, {name}!")
}
```

Zerlegen wir das:

- `fn` leitet eine Funktionsdefinition ein.
- `begruessung` ist der Name, den wir uns ausdenken.
- `(name: &str)` sind die **Parameter** — hier einer, `name`, vom Typ `&str` (ein
  Textverweis, siehe [vorheriges Kapitel](01-variablen-und-typen.md)).
- `-> String` sagt: Diese Funktion **gibt** einen Wert vom Typ `String` **zurück**.
- `format!("Hallo, {name}!")` baut einen `String` und setzt den Wert von `name` an die
  Stelle `{name}` ein — das ist Rusts *string interpolation*.

Aufgerufen wird die Funktion so:

```rust
let text = begruessung("Ada");
println!("{text}"); // Hallo, Ada!
```

## Rückgabewerte ohne `return`

In Rust ist der letzte Ausdruck eines Funktionskörpers automatisch der Rückgabewert —
**wenn** er ohne Semikolon endet. Das ist gewöhnungsbedürftig, aber durchgängig:

```rust
fn verdoppeln(zahl: i32) -> i32 {
    zahl * 2   // kein Semikolon = das ist der Rückgabewert
}
```

Ein Semikolon am Ende würde daraus eine Anweisung machen, die "ins Leere" läuft — der
Compiler würde sich beschweren, dass die Funktion `i32` zurückgeben soll, aber nichts
zurückgibt. Das ist einer der bewussten Compilerfehler, die wir gleich in
[Das erste Programm](06-erstes-programm.md) selbst provozieren werden.

Du **kannst** auch explizit `return` schreiben, meist um früh aus einer Funktion
auszusteigen:

```rust
fn kategorie(alter: u32) -> &'static str {
    if alter < 18 {
        return "minderjährig";
    }
    "volljährig"
}
```

(Das `'static` bei `&'static str` ignorierst du für jetzt einfach — es ist eine
*Lifetime*-Angabe, dazu mehr, sobald wir sie in echtem Framework-Code brauchen.)

## Methoden: Funktionen, die zu einem Typ gehören

Eine Funktion, die auf einem bestimmten Typ "wohnt" und mit `punkt.funktion()`
aufgerufen wird, heißt **Methode**. Das siehst du sofort im echten Framework-Code —
`Nachricht::neu(...)` ist eine solche Methode:

```rust
impl Nachricht {
    pub fn neu(rolle: Rolle, inhalt: impl Into<String>) -> Self {
        Nachricht { rolle, inhalt: inhalt.into() }
    }
}
```

`impl Nachricht { ... }` bedeutet: "Hier folgen Funktionen, die zum Typ `Nachricht`
gehören." `Self` ist eine Abkürzung für "der Typ, zu dem dieser `impl`-Block gehört" —
hier also `Nachricht`. Genau das entschlüsseln wir Zeile für Zeile in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md); für jetzt reicht,
das Muster `fn name(parameter) -> rückgabetyp { ... }` sicher zu erkennen.

## Warum das wichtig ist

Funktionen sind der wichtigste Baustein, um Programme in verständliche, testbare Einheiten
zu zerlegen. Ein gut benannter Funktionsname (`begruessung`, `neu`, `system_nachricht`)
ersetzt einen Kommentar — wer die Funktion aufruft, muss nicht wissen, *wie* sie
funktioniert, nur *was* sie tut. Dieses Prinzip ("Was, nicht Wie") ziehen wir durch den
ganzen Kurs durch, bis hin zu den Traits in [Phase 3](../04-phase3-architektur/README.md),
die letztlich nichts anderes sind als Verträge über Funktionsnamen und -signaturen.

[Weiter: Kontrollfluss](03-kontrollfluss.md)
