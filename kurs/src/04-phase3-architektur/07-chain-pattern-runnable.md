# Lektion 7: Chain Pattern mit Runnable

## Problem

Ein typischer Ablauf in unserem Framework hat mehrere Schritte hintereinander: aus einer
`Konversation` eine `ChatAnfrage` bauen, die Anfrage über einen `LlmProvider` schicken, die
`ChatAntwort` in eine für die Anwendung brauchbare Form weiterverarbeiten. Schreiben wir das
jedes Mal von Hand in `mein_cli` aus, entsteht schnell Code, der Reihenfolge und
Zwischenschritte fest verdrahtet — schwer wiederzuverwenden, wenn [Phase 5](../06-phase5-rag-betrieb/README.md)
einen vierten Schritt (Dokumente aus einem `Retriever` holen) davorsetzen will.

Das Python-Framework LangChain hat für genau dieses Problem ein Muster populär gemacht:
**Runnable** — eine einheitliche Schnittstelle "nimmt einen Input, liefert einen Output",
die sich beliebig verketten lässt, ohne dass jeder Schritt die anderen Schritte kennen muss.
Wir bauen eine minimale, eigenständige Rust-Version davon.

## Code (Zielbild)

```rust
pub trait Runnable {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> Self::Output;
}

pub struct Chain<A, B> {
    pub erster: A,
    pub zweiter: B,
}

impl<A, B> Runnable for Chain<A, B>
where
    A: Runnable,
    B: Runnable<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn run(&self, input: Self::Input) -> Self::Output {
        let zwischenergebnis = self.erster.run(input);
        self.zweiter.run(zwischenergebnis)
    }
}
```

## Dekonstruktion

### `type Input; type Output;` — assoziierte Typen

```rust
pub trait Runnable {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> Self::Output;
}
```

`type Input;` innerhalb eines Traits ist ein **assoziierter Typ** — ein Platzhalter für
einen konkreten Typ, den jede Implementierung von `Runnable` selbst festlegt, genau einmal.
Das unterscheidet sich von einem generischen Trait-Parameter wie bei `LlmProvider` (das
selbst gar keinen generischen Parameter hat) oder wie es hier alternativ aussähe:

```rust
pub trait RunnableGenerisch<I, O> {
    fn run(&self, input: I) -> O;
}
```

Der Unterschied wird sichtbar, sobald man fragt: Kann ein einzelner Typ das Trait
**mehrfach** implementieren, mit unterschiedlichen Typen? Bei `RunnableGenerisch<I, O>`:
ja, ein Typ könnte `RunnableGenerisch<String, ChatAnfrage>` **und**
`RunnableGenerisch<i32, bool>` gleichzeitig implementieren — Rust erlaubt mehrere
`impl`-Blöcke desselben generischen Traits mit unterschiedlichen Parametern für denselben
Typ. Bei `Runnable` mit assoziierten Typen: **nein**, ein Typ legt sich mit `impl Runnable
for MeinSchritt { type Input = ...; type Output = ...; }` **genau einmal** auf ein
Input/Output-Paar fest. Für unser Kettenglied-Konzept ist das die richtige Eigenschaft: Ein
Pipeline-Schritt hat eine feste Form ("aus einer `Konversation` wird eine `ChatAnfrage`"),
keine mehreren gleichzeitig gültigen Formen.

### Drei konkrete Schritte

```rust
pub struct AnfrageSchritt {
    pub modell: String,
}

impl Runnable for AnfrageSchritt {
    type Input = Konversation;
    type Output = ChatAnfrage;

    fn run(&self, konversation: Konversation) -> ChatAnfrage {
        ChatAnfrage {
            nachrichten: konversation.verlauf().to_vec(),
            modell: self.modell.clone(),
        }
    }
}

pub struct LlmSchritt {
    pub provider: Box<dyn LlmProvider>,
}

impl Runnable for LlmSchritt {
    type Input = ChatAnfrage;
    type Output = ChatAntwort;

    fn run(&self, anfrage: ChatAnfrage) -> ChatAntwort {
        self.provider
            .chat(anfrage)
            .expect("Provider-Aufruf ist fehlgeschlagen")
    }
}

pub struct GrossschreibungsSchritt;

impl Runnable for GrossschreibungsSchritt {
    type Input = ChatAntwort;
    type Output = String;

    fn run(&self, antwort: ChatAntwort) -> String {
        antwort.inhalt.trim().to_string()
    }
}
```

`LlmSchritt` hält `provider: Box<dyn LlmProvider>` — genau das Ownership-Muster aus
[Lektion 3](03-dyn-trait-ownership.md): `LlmSchritt` **besitzt** irgendeinen Provider, ohne
dessen konkreten Typ zu kennen, ohne dafür eine Lifetime-Annotation zu brauchen.

> **⚠️ Warnung**
>
> `LlmSchritt::run` nutzt `.expect(...)`, wirft also bei einem `Err` (z. B. einem Timeout)
> einen `panic!` — obwohl wir in
> [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) gelernt haben, `panic!`
> nur für Fälle zu nutzen, die nie passieren sollten. Das ist hier eine bewusste
> Vereinfachung für dieses Einführungsbeispiel, keine Empfehlung für echten Code: Ein
> produktionsreifes `Runnable` würde `type Output = Result<ChatAntwort, ProviderFehler>`
> verwenden und Fehler entlang der Kette durchreichen. Das hätten wir in dieser Lektion
> zeigen können — es hätte aber jeden Schritt mit zusätzlichem `Result`-Verpacken belastet
> und vom eigentlichen Kernprinzip (assoziierte Typen, Verkettung) abgelenkt. Die Übung am
> Ende dieser Lektion holt das nach.

### `Chain<A, B>` — zwei Schritte zu einem verschmelzen

```rust
pub struct Chain<A, B> {
    pub erster: A,
    pub zweiter: B,
}

impl<A, B> Runnable for Chain<A, B>
where
    A: Runnable,
    B: Runnable<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn run(&self, input: Self::Input) -> Self::Output {
        let zwischenergebnis = self.erster.run(input);
        self.zweiter.run(zwischenergebnis)
    }
}
```

`Chain<A, B>` ist selbst wieder ein `Runnable` — das ist der eigentliche Trick des
Chain-Patterns: Eine Kette aus zwei Schritten verhält sich nach außen genau wie ein
einzelner Schritt, mit eigenem `Input` (dem `Input` des ersten Glieds) und eigenem `Output`
(dem `Output` des zweiten Glieds). Das erlaubt, `Chain`s wieder in weitere `Chain`s zu
verschachteln (`Chain<Chain<A, B>, C>`) und so beliebig lange Pipelines zu bauen, ohne die
Struktur zu ändern.

Die `where`-Klausel ist der Kern der Typsicherheit: `B: Runnable<Input = A::Output>` sagt
dem Compiler: "Der `Input`-Typ von `B` muss **exakt** dem `Output`-Typ von `A`
entsprechen." Das ist eine Gleichheitsbedingung auf einem assoziierten Typ — passen die
beiden Schritte nicht zusammen, verweigert der Compiler das Kompilieren, bevor das Programm
je läuft.

## Schritt-Reveal

**Schritt 1** — Lege `Runnable` und `Chain<A, B>` wie im Zielbild an, z. B. in einem neuen
Modul `mein_core/src/runnable.rs` (ergänze `pub mod runnable;` in `lib.rs`).

**Schritt 2** — Implementiere `AnfrageSchritt`, `LlmSchritt`, `GrossschreibungsSchritt` wie
oben gezeigt. `cargo check -p mein_core` — alle drei sollten für sich allein kompilieren.

**Schritt 3** — Verkette `AnfrageSchritt` und `LlmSchritt`:

```rust
let kette = Chain {
    erster: AnfrageSchritt { modell: "irgendein-modell".into() },
    zweiter: LlmSchritt { provider: Box::new(FakeProvider::antwortet_mit("Hallo!")) },
};

let mut konversation = Konversation::neu();
konversation.hinzufuegen(Rolle::Benutzer, "Hi").unwrap();

let antwort: ChatAntwort = kette.run(konversation);
assert_eq!(antwort.inhalt, "Hallo!");
```

`cargo check -p mein_core` (im Testmodus, wegen `FakeProvider`) — kompiliert, weil
`AnfrageSchritt::Output` (`ChatAnfrage`) exakt zu `LlmSchritt::Input` (`ChatAnfrage`) passt.

**Schritt 4** — Provoziere jetzt bewusst einen Typfehler: Verkette `AnfrageSchritt` direkt
mit `GrossschreibungsSchritt`, überspringe `LlmSchritt`:

```rust
let kaputte_kette = Chain {
    erster: AnfrageSchritt { modell: "irgendein-modell".into() },
    zweiter: GrossschreibungsSchritt,
};
```

```
error[E0271]: type mismatch resolving `<GrossschreibungsSchritt as Runnable>::Input == ChatAnfrage`
  --> src/main.rs:12:14
   |
12 |         zweiter: GrossschreibungsSchritt,
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^ type mismatch resolving `<GrossschreibungsSchritt as Runnable>::Input == ChatAnfrage`
   |
   = note: expected type `ChatAntwort`
              found type `ChatAnfrage`
```

Das ist die `where`-Klausel `B: Runnable<Input = A::Output>` in Aktion: `A::Output`
(`AnfrageSchritt::Output`) ist `ChatAnfrage`, aber `GrossschreibungsSchritt::Input` ist
`ChatAntwort` — die beiden passen nicht zusammen, weil in der Pipeline der `LlmSchritt`
fehlt, der aus einer `ChatAnfrage` erst eine `ChatAntwort` macht. Der Compiler verhindert
damit exakt den Fehler, den man in einer dynamisch typisierten Pipeline (wie in Python ohne
Typannotationen) erst zur Laufzeit bemerken würde — hier bemerken wir ihn, bevor das
Programm je ausgeführt wird.

**Schritt 5** — Korrigiere zur vollständigen Drei-Schritte-Kette:

```rust
let vollstaendige_kette = Chain {
    erster: Chain {
        erster: AnfrageSchritt { modell: "irgendein-modell".into() },
        zweiter: LlmSchritt { provider: Box::new(FakeProvider::antwortet_mit("Hallo!")) },
    },
    zweiter: GrossschreibungsSchritt,
};

let ergebnis: String = vollstaendige_kette.run(konversation);
assert_eq!(ergebnis, "Hallo!");
```

## Ausführung

```bash
cargo test -p mein_core --features test-utils
```

```
running 1 test
test runnable::tests::vollstaendige_kette_liefert_erwarteten_text ... ok
```

## Zusammenfassung

- `Runnable` mit assoziierten Typen (`type Input`, `type Output`) beschreibt einen
  einzelnen Pipeline-Schritt mit genau einer festen Eingabe- und Ausgabeform.
- Assoziierte Typen unterscheiden sich von generischen Trait-Parametern: Ein Typ legt sich
  mit einem assoziierten Typ genau einmal fest, statt das Trait für mehrere
  Typkombinationen gleichzeitig zu implementieren.
- `Chain<A, B>` ist selbst wieder ein `Runnable` — Ketten lassen sich verschachteln.
- Die `where B: Runnable<Input = A::Output>`-Klausel erzwingt beim Kompilieren, dass
  aufeinanderfolgende Schritte zueinander passen (E0271 sonst).
- `Box<dyn LlmProvider>` als Feld in `LlmSchritt` verbindet das Chain Pattern direkt mit dem
  Ownership-Prinzip aus [Lektion 3](03-dyn-trait-ownership.md).

## Übung

Ergänze dem `Runnable`-Trait einen komfortablen Verkettungs-Helfer als **Default-Methode**
(eine Methode mit Körper direkt im `trait`-Block, die nicht jede Implementierung neu
schreiben muss):

```rust
pub trait Runnable {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> Self::Output;

    fn pipe<B>(self, naechster: B) -> Chain<Self, B>
    where
        Self: Sized,
        B: Runnable<Input = Self::Output>,
    {
        Chain { erster: self, zweiter: naechster }
    }
}
```

Baue damit dieselbe Drei-Schritte-Kette aus Schritt 5 ohne verschachtelte
`Chain { ... }`-Konstruktoren, sondern als `schritt1.pipe(schritt2).pipe(schritt3)`.
Überlege: Warum braucht `pipe` das zusätzliche `Self: Sized`-Bound (ein Hinweis: Erinnere
dich an [Lektion 3](03-dyn-trait-ownership.md) und die *unsized*-Eigenschaft von `dyn
Trait`)? Passe anschließend `LlmSchritt::run` so an, dass es `Result<ChatAntwort,
ProviderFehler>` statt `.expect(...)` verwendet — und überlege, wie sich die `where`-Klausel
in `Chain` ändern müsste, damit Fehler durch die ganze Pipeline durchgereicht werden können.

[Weiter: Lektion 8 · Release 3 — provider-agnostic-core](08-release-3.md)
