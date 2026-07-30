# Lektion 3: dyn Trait und Ownership an der Grenze

## Problem

`LlmProvider` ist jetzt ein Trait — aber Traits sind, anders als konkrete Typen wie `struct
OpenAiKompatiblerClient`, **nicht direkt als Wert benutzbar**. Du kannst keine Variable vom
Typ `LlmProvider` deklarieren, so wie du eine Variable vom Typ `Nachricht` deklarieren
kannst — der Grund dazu gleich. Trotzdem wollen wir Funktionen und später ganze Structs
(z. B. eine `Sitzung`, die einen Provider für eine laufende Konversation hält) schreiben, die
"irgendeinen `LlmProvider`" entgegennehmen, ohne den konkreten Typ zu kennen.

Rust bietet dafür **zwei grundsätzlich verschiedene Wege**, und die Wahl zwischen ihnen ist
eine der wichtigsten Architekturentscheidungen an jeder Systemgrenze: generische Parameter
(`<P: LlmProvider>`) und Trait-Objekte (`dyn LlmProvider`). Beide lösen "ich will einen
LlmProvider, egal welchen" — aber mit unterschiedlichen Kosten und Freiheiten.

## Code (Zielbild)

```rust
// Weg 1: generischer Parameter — "Compile-Zeit", static dispatch
fn chatte_generisch<P: LlmProvider>(
    provider: &P,
    anfrage: ChatAnfrage,
) -> Result<ChatAntwort, ProviderFehler> {
    provider.chat(anfrage)
}

// Weg 2: Trait-Objekt — "Laufzeit", dynamic dispatch
fn chatte_dynamisch(
    provider: &dyn LlmProvider,
    anfrage: ChatAnfrage,
) -> Result<ChatAntwort, ProviderFehler> {
    provider.chat(anfrage)
}

// Ownership an der Grenze: ein Typ, der "irgendeinen" Provider BESITZT
pub struct Sitzung {
    provider: Box<dyn LlmProvider>,
}
```

## Dekonstruktion

### Weg 1: `<P: LlmProvider>` — generischer Parameter

`<P: LlmProvider>` heißt: "`P` ist ein Platzhalter für *irgendeinen* Typ, solange dieser Typ
`LlmProvider` implementiert." Das nennt man einen **Trait Bound** — eine Einschränkung, die
das Trait an den generischen Typ stellt. Der entscheidende Punkt: Der Compiler kennt beim
Kompilieren zwar noch nicht den *konkreten* Typ von `P`, aber er weiß, dass es *einen* gibt,
und erzeugt für **jeden** tatsächlich benutzten konkreten Typ eine eigene, spezialisierte
Kopie der Funktion. Dieser Vorgang heißt **Monomorphisierung** (aus dem Griechischen, sinngemäß
"eine Form geben"): Rufst du `chatte_generisch` einmal mit `OpenAiKompatiblerClient` und
einmal mit einem Fake-Provider auf, entstehen beim Kompilieren zwei separate
Maschinencode-Versionen der Funktion — jede so schnell, als hättest du sie von Hand für
genau diesen einen Typ geschrieben. Das ist **static dispatch**: Welche konkrete
`chat`-Implementierung aufgerufen wird, steht schon beim Kompilieren fest, es gibt zur
Laufzeit keinen Nachschlage-Schritt.

Der Preis: Jede Aufrufstelle von `chatte_generisch` muss ihren konkreten Typ zur Compile-Zeit
festlegen. Du kannst keine `Vec<P>` mit *gemischten* Providern bauen — ein `Vec<T>` braucht
für alle Elemente denselben, konkreten Typ `T`. Und mehr benutzte konkrete Typen bedeuten
mehr generierten Code (größere Binärdatei) — in der Praxis meist vernachlässigbar, aber
nicht kostenlos.

### Weg 2: `dyn LlmProvider` — Trait-Objekt

`dyn LlmProvider` ("dynamic", Trait-Objekt) ist grundsätzlich anders: Statt für jeden
konkreten Typ eigenen Code zu erzeugen, legt Rust zur Laufzeit eine **Vtable** an (eine
kleine Tabelle mit Funktionszeigern, eine Art Inhaltsverzeichnis: "hier ist die `chat`-
Implementierung für *diesen* konkreten Wert"). Ein Aufruf über `dyn LlmProvider` schlägt zur
Laufzeit in dieser Tabelle nach, statt beim Kompilieren festzustehen — das ist **dynamic
dispatch**, minimal langsamer als static dispatch (ein zusätzlicher Zeiger-Sprung), aber
dafür mit einer entscheidenden Freiheit: `Vec<Box<dyn LlmProvider>>` funktioniert, weil
jedes Element nur noch "ein Trait-Objekt, das `LlmProvider` erfüllt" ist — der konkrete
darunterliegende Typ ist für den `Vec` gar nicht mehr sichtbar.

### Warum `dyn LlmProvider` nicht als Wert existiert, nur als Referenz

`dyn LlmProvider` hat keine feste, beim Kompilieren bekannte Größe im Speicher — schließlich
könnte dahinter `OpenAiKompatiblerClient` (mit einem `reqwest::blocking::Client` und zwei
`String`-Feldern) stecken, oder ein winziger Fake-Provider, oder ein dritter, uns unbekannter
Typ. Rust nennt so etwas **unsized**. Deshalb muss `dyn LlmProvider` immer *hinter einem
Zeiger* auftreten: als `&dyn LlmProvider` (geliehen, siehe
[Borrowing in Phase 1](../02-phase1-fundament/04-konversation.md)) oder als `Box<dyn
LlmProvider>` (siehe unten). Ein nackter Parameter `provider: dyn LlmProvider` (ohne `&`
oder `Box`) lässt der Compiler gar nicht erst zu:

```
error[E0277]: the size for values of type `dyn LlmProvider` cannot be known at compilation time
```

### `Box<dyn LlmProvider>` — Ownership an der Systemgrenze

```rust
pub struct Sitzung {
    provider: Box<dyn LlmProvider>,
}
```

Hier kommt die eigentliche Pointe dieser Lektion: `&dyn LlmProvider` ist eine **geliehene**
Referenz — sie braucht immer einen Ort, an dem der eigentliche Wert lebt und lange genug
existiert (bei einem Struct-Feld hieße das, eine explizite **Lifetime**-Annotation wie `&'a
dyn LlmProvider` anzugeben — ein Konzept, das wir in diesem Kurs bewusst nicht vertiefen).
Für ein Struct wie `Sitzung`, das einen Provider über seine gesamte Lebensdauer **besitzen**
soll (kein anderer Teil des Programms soll sich um dessen Lebensdauer kümmern müssen), ist
`Box<dyn LlmProvider>` das richtige Werkzeug: `Box<T>` ist ein Zeiger auf einen Wert, der auf
dem **Heap** liegt (dem Teil des Speichers für Werte unbekannter oder variabler Größe zur
Compile-Zeit — im Gegensatz zum **Stack**, wo Werte fester, bekannter Größe liegen) und den
die `Box` **besitzt**. `Box<dyn LlmProvider>` verbindet beides: "ein Wert unbekannter Größe
und unbekannten konkreten Typs, aber mit klarer Ownership — wenn die `Sitzung` stirbt, stirbt
mit ihr auch der Provider."

Das ist der Grund, warum wir dieses Kapitel "Ownership an der Grenze" nennen: Genau an
Systemgrenzen — wo eine Komponente (hier `Sitzung`) einen austauschbaren Baustein (einen
Provider) für sich beansprucht, ohne dessen konkreten Typ zu kennen — ist `Box<dyn Trait>`
das Standardwerkzeug in Rust.

> **💡 Tipp**
>
> Faustregel für die Wahl: Brauchst du maximale Geschwindigkeit und kennst alle beteiligten
> Typen beim Kompilieren (z. B. in einer Bibliotheksfunktion, die generisch bleiben soll)?
> Nimm `<P: LlmProvider>`. Brauchst du zur Laufzeit **entscheidbare** Flexibilität — welcher
> Provider verwendet wird, hängt z. B. von einer Kommandozeilen-Option oder einer
> Konfigurationsdatei ab — oder willst du verschiedene Provider in einer Sammlung mischen?
> Nimm `dyn LlmProvider`, meist als `Box<dyn LlmProvider>`.

### Objektsicherheit: Warum nicht jedes Trait `dyn`-fähig ist

Nicht jedes Trait lässt sich als `dyn Trait` verwenden — ein Trait muss **objektsicher**
(*object safe*) sein. Eine der Regeln: Trait-Methoden dürfen keine eigenen generischen
Parameter haben. Der Grund: Eine Vtable braucht für jede Methode **eine feste** Adresse —
eine generische Methode müsste aber für jeden möglichen Typ eine eigene Adresse haben, was
der Vtable-Mechanismus nicht abbilden kann. Probiere es aus:

```rust
pub trait LlmProviderExperimentell {
    fn chat<T: Into<ChatAnfrage>>(&self, anfrage: T) -> Result<ChatAntwort, ProviderFehler>;
}

fn nimm_dyn(p: Box<dyn LlmProviderExperimentell>) {}
```

```
error[E0038]: the trait `LlmProviderExperimentell` cannot be made into an object
 --> src/main.rs:5:17
  |
5 | fn nimm_dyn(p: Box<dyn LlmProviderExperimentell>) {}
  |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `LlmProviderExperimentell` cannot be made into an object
  |
note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically
note: method `chat` has generic type parameters
```

Das erklärt rückblickend eine Design-Entscheidung aus [Lektion 1](01-llmprovider-port.md):
`LlmProvider::chat` nimmt bewusst den konkreten Typ `ChatAnfrage` entgegen, keinen
generischen Parameter — genau, damit `dyn LlmProvider` überhaupt möglich ist. Diese
Fehlermeldung ist kein Zufallsfund, sondern der Grund, warum viele Rust-Traits, die als
Trait-Objekt genutzt werden sollen, bewusst auf generische Methoden verzichten.

## Schritt-Reveal

**Schritt 1** — Füge `chatte_generisch` und `chatte_dynamisch` aus dem Zielbild irgendwo in
`mein_core` oder testweise in `mein_cli` ein. `cargo check` — beide kompilieren, solange
`OpenAiKompatiblerClient: LlmProvider` gilt.

**Schritt 2** — Rufe beide mit demselben konkreten Client auf:

```rust
let client = OpenAiKompatiblerClient::neu(/* ... */);
let anfrage = ChatAnfrage { nachrichten: vec![], modell: "irgendein-modell".into() };

let _ = chatte_generisch(&client, anfrage_klon_1);
let _ = chatte_dynamisch(&client, anfrage_klon_2);
```

Beachte: An der Aufrufstelle sieht man äußerlich kaum einen Unterschied — `&client` passt
für beide. Der Unterschied liegt ausschließlich in der **Signatur der Funktion**, nicht im
Aufruf.

**Schritt 3** — Provoziere den E0038-Fehler von oben bewusst mit
`LlmProviderExperimentell`, lies die Fehlermeldung, mach die Änderung rückgängig.

**Schritt 4** — Baue `Sitzung` mit `Box<dyn LlmProvider>` und einem kleinen Konstruktor:

```rust
impl Sitzung {
    pub fn neu(provider: Box<dyn LlmProvider>) -> Self {
        Sitzung { provider }
    }

    pub fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        self.provider.chat(anfrage)
    }
}
```

```rust
let sitzung = Sitzung::neu(Box::new(OpenAiKompatiblerClient::neu(/* ... */)));
```

`Box::new(wert)` legt `wert` auf dem Heap ab und gibt eine `Box`, die ihn besitzt, zurück —
der Compiler "vergisst" dabei automatisch den konkreten Typ `OpenAiKompatiblerClient` und
merkt sich nur noch "etwas, das `LlmProvider` erfüllt", sobald du das Ergebnis einer
Variablen vom Typ `Box<dyn LlmProvider>` zuweist (oder es, wie hier, direkt an `Sitzung::neu`
übergibst, dessen Parametertyp das vorschreibt).

## Ausführung

```bash
cargo check -p mein_core
cargo test -p mein_core
```

## Zusammenfassung

- `<P: LlmProvider>` (generisch): static dispatch, eine spezialisierte Codekopie pro
  konkretem Typ (Monomorphisierung), maximale Laufzeitgeschwindigkeit, aber Typ muss beim
  Kompilieren feststehen — kein Mischen verschiedener Provider in einer Sammlung.
- `dyn LlmProvider` (Trait-Objekt): dynamic dispatch über eine Vtable, minimal langsamer,
  dafür zur Laufzeit flexibel, mischbar in Sammlungen wie `Vec<Box<dyn LlmProvider>>`.
- `dyn Trait` ist *unsized* — er muss immer hinter `&` oder `Box` stehen.
- `Box<dyn Trait>` ist das Standardwerkzeug, wenn ein Typ einen austauschbaren Baustein
  **besitzen** soll, ohne dessen konkreten Typ zu kennen — typisch an Systemgrenzen.
- Nicht jedes Trait ist objektsicher: generische Methoden verhindern `dyn Trait`, weil eine
  Vtable keine generische Methode abbilden kann (E0038).

## Übung

Schreibe eine Funktion `fn provider_liste(provider: Vec<Box<dyn LlmProvider>>) -> usize`,
die einfach `provider.len()` zurückgibt, und rufe sie mit einem `Vec`, das **zwei
verschiedene** `LlmProvider`-Implementierungen enthält (nutze für den zweiten notfalls
vorerst wieder einen zweiten `OpenAiKompatiblerClient` mit anderer Konfiguration — einen
echten zweiten Typ bauen wir in [Lektion 4](04-fake-provider.md)). Versuche anschließend,
dieselbe Funktion mit einem generischen Parameter `<P: LlmProvider>(provider: Vec<P>)` statt
`Vec<Box<dyn LlmProvider>>` zu schreiben, und beobachte, warum sich zwei *unterschiedliche*
konkrete Provider-Typen nicht in denselben `Vec<P>` packen lassen. Das ist genau der
Praxisunterschied zwischen den beiden Wegen aus dieser Lektion, nicht nur graue Theorie.

[Weiter: Lektion 4 — Fake-Provider für Unit-Tests](04-fake-provider.md)
