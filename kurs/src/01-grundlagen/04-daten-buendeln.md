# Daten bündeln: Listen, Structs und Enums

Einzelne Werte reichen selten. Fast jedes echte Problem braucht **mehrere** zusammengehörige
Werte — eine Liste von Nachrichten, eine Nachricht mit Rolle und Inhalt, eine Rolle, die
genau eine von drei Möglichkeiten ist. Rust bietet dafür drei Grundwerkzeuge.

## Listen: `Vec<T>`

Ein `Vec` (*Vector*) ist eine veränderbare, wachsende Liste von Werten desselben Typs:

```rust
let mut zahlen: Vec<i32> = Vec::new();
zahlen.push(1);
zahlen.push(2);
zahlen.push(3);
println!("{}", zahlen.len()); // 3
```

`Vec<i32>` liest sich "ein Vec von i32" — das `<i32>` ist ein **generischer
Typparameter**: `Vec` selbst ist generisch, kann also eine Liste von *irgendetwas* sein,
und `<i32>` legt hier fest: konkret eine Liste von `i32`-Werten. Genauso wird
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) eine `Konversation` im
Kern aus einem `Vec<Nachricht>` bestehen — einer wachsenden Liste von `Nachricht`-Werten.

Kurzschreibweise mit dem `vec!`-Makro (ein Makro erzeugt zur Kompilierzeit Code aus einer
kürzeren Schreibweise — mehr dazu in [Lektion 6](06-erstes-programm.md)):

```rust
let zahlen = vec![1, 2, 3]; // gleichwertig zu den drei push()-Aufrufen oben
```

## Zusammengehörige Werte bündeln: `struct`

Ein `struct` (*structure*) fasst mehrere benannte Werte unterschiedlichen Typs zu einer
Einheit zusammen:

```rust
struct Nachricht {
    rolle: Rolle,
    inhalt: String,
}
```

Das ist — Feld für Feld — exakt die `Nachricht`-Struct aus dem echten Ai_Framework-Code,
den wir in [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md) lesen.
Ein Wert davon wird so erzeugt:

```rust
let n = Nachricht {
    rolle: Rolle::Benutzer,
    inhalt: String::from("Hallo"),
};
println!("{}", n.inhalt); // Zugriff auf ein Feld mit einem Punkt
```

## Eine von mehreren Möglichkeiten: `enum`

Ein `enum` (*enumeration*) beschreibt einen Wert, der genau **eine** von mehreren fest
benannten Möglichkeiten ist — nie mehrere gleichzeitig, nie etwas anderes:

```rust
enum Rolle {
    System,
    Benutzer,
    Assistent,
}
```

Auch das ist wörtlich der echte Framework-Code. Ein `Rolle`-Wert kann nur `Rolle::System`,
`Rolle::Benutzer` oder `Rolle::Assistent` sein — keine vierte Möglichkeit, kein leerer
oder ungültiger Zustand. Das ist der entscheidende Unterschied zu z. B. einem `String` mit
Text wie `"system"`: Bei einem `String` könnte jemand versehentlich `"Sytsem"` (Tippfehler)
oder `"admin"` (ungültiger Wert) hineinschreiben, und das würde erst zur Laufzeit auffallen
— wenn überhaupt. Bei `enum Rolle` verhindert der **Compiler** solche ungültigen Zustände
komplett. Diese Idee — ungültige Zustände gar nicht erst *darstellbar* zu machen, statt sie
zur Laufzeit abzufangen — ist eines der wichtigsten Rust-Prinzipien und zieht sich durch
den gesamten Kurs.

Mit `match` (siehe [Kontrollfluss](03-kontrollfluss.md)) fragst du ab, welche
Möglichkeit gerade vorliegt:

```rust
match rolle {
    Rolle::System => println!("Systemnachricht"),
    Rolle::Benutzer => println!("Nutzernachricht"),
    Rolle::Assistent => println!("Antwort des Assistenten"),
}
```

## `Option` und `Result`: zwei besonders wichtige Enums

Rust hat keinen `null`-Wert wie viele andere Sprachen (der berüchtigte "Milliarden-Dollar-
Fehler" der Softwaregeschichte). Stattdessen gibt es zwei eingebaute `enum`s, die dasselbe
Problem sicherer lösen:

```rust
enum Option<T> {
    Some(T), // ein Wert ist vorhanden
    None,    // kein Wert vorhanden
}

enum Result<T, E> {
    Ok(T),   // Operation erfolgreich, hier ist der Wert
    Err(E),  // Operation fehlgeschlagen, hier ist der Fehler
}
```

Statt dass ein "leerer" Wert unbemerkt durchs Programm wandert und irgendwann abstürzt,
zwingt dich der Compiler, den Fall "kein Wert" bzw. "Fehler" **explizit** zu behandeln,
bevor du an den eigentlichen Wert herankommst. Wir begegnen `Option` und `Result` ab
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) (z. B. wenn wir prüfen, ob
ein Nachrichteninhalt leer ist) und vertiefen `Result` in
[Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md), wo Netzwerk- und
API-Fehler typisiert behandelt werden.

[Weiter: Der Compiler als Lehrer](05-der-compiler-als-lehrer.md)
