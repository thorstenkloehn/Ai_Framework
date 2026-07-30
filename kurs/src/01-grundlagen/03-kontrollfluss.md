# Kontrollfluss: Entscheidungen und Wiederholungen

Ein Programm, das immer exakt dieselben Anweisungen in derselben Reihenfolge ausführt,
wäre kaum nützlich. **Kontrollfluss** ist die Fähigkeit, abhängig von Bedingungen
unterschiedlich zu reagieren oder etwas mehrfach zu tun.

## Entscheidungen: `if` / `else`

```rust
let temperatur = 18;

if temperatur < 10 {
    println!("Es ist kalt.");
} else if temperatur < 20 {
    println!("Es ist mild.");
} else {
    println!("Es ist warm.");
}
```

Die Bedingung nach `if` muss ein `bool` sein (`true` oder `false`) — anders als in manchen
Sprachen akzeptiert Rust keine Zahl oder Text als "wahrheitsähnlichen" Wert. Das ist wieder
typisch Rust: lieber ein Compilerfehler an dieser Stelle als eine Überraschung zur
Laufzeit.

`if` ist in Rust ein **Ausdruck**, kein reines Statement — er kann selbst einen Wert
liefern:

```rust
let einstufung = if temperatur < 10 { "kalt" } else { "warm" };
```

Das nutzen wir später ständig, z. B. um in einer Methode je nach Zustand einen anderen
Wert zurückzugeben, ohne eine Hilfsvariable zu brauchen.

## Mehrfachauswahl: `match`

Sobald du mehr als zwei, drei Fälle unterscheidest, wird `if`/`else if`/`else`
unübersichtlich. Rust bietet dafür `match` — mächtiger als ein einfaches `switch` aus
anderen Sprachen, weil der Compiler prüft, ob du **alle** Fälle abgedeckt hast:

```rust
enum Ampel {
    Rot,
    Gelb,
    Gruen,
}

fn anweisung(farbe: Ampel) -> &'static str {
    match farbe {
        Ampel::Rot => "stehen bleiben",
        Ampel::Gelb => "bereit machen",
        Ampel::Gruen => "fahren",
    }
}
```

Würdest du hier einen Fall vergessen (z. B. `Ampel::Gelb`), lehnt der Compiler das
Programm ab — mit einer Fehlermeldung, die genau sagt, welcher Fall fehlt. Das ist kein
Zufall: `enum` und `match` gehören zusammen, und genau diese Kombination sehen wir in
[Phase 1](../02-phase1-fundament/02-rolle-und-nachricht.md) wieder, sobald wir mit `Rolle`
(`System`, `Benutzer`, `Assistent`) arbeiten.

## Wiederholungen: `loop`, `while`, `for`

Drei Varianten, je nachdem, wie die Wiederholung enden soll:

```rust
// loop: wiederholt sich, bis du explizit `break` sagst
let mut zaehler = 0;
loop {
    zaehler += 1;
    if zaehler == 5 {
        break;
    }
}

// while: wiederholt sich, solange eine Bedingung wahr ist
let mut rest = 5;
while rest > 0 {
    rest -= 1;
}

// for: wiederholt sich einmal pro Element einer Sammlung
let zahlen = vec![1, 2, 3];
for zahl in &zahlen {
    println!("{zahl}");
}
```

Die `for`-Schleife ist in Rust die mit Abstand häufigste — sie iteriert über eine
Sammlung (mehr dazu im nächsten Kapitel, [Daten bündeln](04-daten-buendeln.md)), ohne
dass du einen Zähler manuell verwalten musst. Genau so werden wir in
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) über den
`Vec<Nachricht>`-Verlauf einer `Konversation` iterieren, um alle Nachrichten auszugeben.

> **💡 Tipp**
>
> `zaehler += 1` ist Kurzschreibweise für `zaehler = zaehler + 1`. Rust kennt `+=`, `-=`,
> `*=`, `/=` — praktisch, aber nur auf `mut`-Variablen erlaubt (siehe
> [Variablen und Typen](01-variablen-und-typen.md)).

[Weiter: Daten bündeln — Listen, Structs und Enums](04-daten-buendeln.md)
