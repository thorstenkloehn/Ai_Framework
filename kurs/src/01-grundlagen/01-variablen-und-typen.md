# Variablen, Werte und Typen

## Ein Wert braucht einen Namen

Ein **Wert** ist ein konkretes Stück Information: die Zahl `42`, der Text `"Hallo"`, die
Wahrheitsaussage `true`. Damit wir einen Wert später wiederverwenden können, geben wir ihm
einen Namen — eine **Variable**. In Rust:

```rust
let alter = 42;
let name = "Ada";
```

`let` bedeutet "binde den folgenden Namen an den folgenden Wert". Ab jetzt kannst du im
Code `alter` schreiben, und der Computer setzt dafür `42` ein.

> **⚠️ Warnung**
>
> In Rust sind Variablen **standardmäßig unveränderlich** (*immutable*). Das hier
> kompiliert NICHT:
>
> ```rust
> let alter = 42;
> alter = 43; // Fehler!
> ```
>
> Das ist Absicht, nicht Einschränkung: Wenn ein Wert sich nie ändert, kannst du (und der
> Compiler) das an jeder Stelle im Code darauf verlassen — eine ganze Fehlerklasse
> ("irgendwo im Code wurde der Wert versehentlich überschrieben") verschwindet dadurch.
> Willst du eine Variable ändern können, musst du das explizit sagen:
>
> ```rust
> let mut alter = 42;
> alter = 43; // ok, weil `mut`
> ```
>
> `mut` steht für *mutable*, veränderlich. Die Grundhaltung in Rust (und in diesem Kurs):
> unveränderlich ist der Normalfall, veränderlich die bewusste Ausnahme.

## Typen: Was für eine Art von Wert ist das?

Jeder Wert in Rust hat einen **Typ** — eine Klassifikation, die festlegt, welche Form die
Daten haben und welche Operationen damit erlaubt sind. Die wichtigsten Grundtypen:

| Typ | Beispielwert | Bedeutung |
|-----|--------------|-----------|
| `i32` | `42`, `-7` | Ganze Zahl (*integer*), 32 Bit, kann negativ sein |
| `u32` | `42` | Ganze Zahl, 32 Bit, **nur positiv** (*unsigned*) |
| `f64` | `3.14` | Kommazahl (*float*), 64 Bit |
| `bool` | `true`, `false` | Wahrheitswert |
| `char` | `'a'` | Ein einzelnes Unicode-Zeichen (einfache Anführungszeichen!) |
| `String` | `String::from("Hallo")` | Text, veränderbar, "besitzt" seinen Speicher |
| `&str` | `"Hallo"` | Text, meist ein Verweis auf bereits vorhandenen Text |

Die letzten beiden — `String` und `&str` — verwirren fast alle Rust-Anfänger*innen am
Anfang, weil andere Sprachen meist nur einen Text-Typ kennen. Der Unterschied hat mit
**Ownership** zu tun, Rusts Konzept dafür, wer im Programm gerade "Besitzer" eines Werts
im Speicher ist. Wir gehen darauf in [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)
im Detail ein, wenn wir `Nachricht::neu(rolle, inhalt: impl Into<String>)` aus dem echten
Framework-Code lesen. Für jetzt reicht: `&str` ist meist ein wörtlich im Code stehender
Text (`"Hallo"`), `String` ist Text, den du zur Laufzeit bauen, verändern und
weitergeben kannst.

## Typen werden meist automatisch erkannt

Rust ist **statisch typisiert**: Jede Variable hat einen festen Typ, der schon beim
Kompilieren feststeht (im Gegensatz zu z. B. Python, wo sich der Typ zur Laufzeit ändern
kann). Trotzdem musst du den Typ fast nie hinschreiben — der Compiler **erschließt** ihn
aus dem Wert (*Type Inference*):

```rust
let alter = 42;        // Compiler erkennt: i32 (Standard für ganze Zahlen)
let pi = 3.14;          // Compiler erkennt: f64
let name = "Ada";       // Compiler erkennt: &str
```

Du kannst den Typ trotzdem explizit angeben — nützlich, um Code lesbarer zu machen oder
wenn der Compiler nicht genug Information hat:

```rust
let anzahl: u32 = 42;
let anteil: f64 = 0.5;
```

Das Muster `let name: Typ = wert;` wirst du ab [Phase 1](../02-phase1-fundament/README.md)
ständig sehen, z. B. bei `pub inhalt: String` in der `Nachricht`-Struct.

## Warum ist das wichtig?

Weil der Compiler mit dieser Typinformation die meisten Fehler schon vor dem Ausführen
findet: Wenn eine Funktion eine Zahl erwartet und du versehentlich Text übergibst,
bekommst du eine Fehlermeldung beim Kompilieren — nicht einen Absturz beim Kunden. Diese
Garantie ist einer der Hauptgründe, warum wir Rust für ein Framework wählen, das andere
als Abhängigkeit einbinden sollen ([mehr dazu hier](../00-einleitung/01-ueber-dieses-projekt.md)).

[Weiter: Funktionen](02-funktionen.md)
