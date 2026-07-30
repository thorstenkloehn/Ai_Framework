# Lektion 1: LlmProvider als Port

## Problem

Am Ende von [Phase 2](../03-phase2-llm-anbindung/README.md) hat `mein_core` einen
funktionierenden, aber **konkreten** HTTP-Client für genau einen LLM-Anbieter gebaut —
nennen wir ihn hier `OpenAiKompatiblerClient` (falls du ihn anders genannt hast, ist das
kein Problem, das Prinzip dieser Lektion ändert sich nicht). Typischerweise sieht das
ungefähr so aus:

```rust
pub struct ChatAnfrage {
    pub nachrichten: Vec<Nachricht>,
    pub modell: String,
}

pub struct ChatAntwort {
    pub inhalt: String,
}

pub struct OpenAiKompatiblerClient {
    client: reqwest::blocking::Client,
    api_key: String,
    basis_url: String,
}

impl OpenAiKompatiblerClient {
    pub fn anfrage_senden(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        // ... der reqwest-Aufruf aus Phase 2, unverändert ...
        # unimplemented!()
    }
}
```

`mein_cli` (oder später ein Agent) ruft `client.anfrage_senden(...)` direkt auf. Das
Problem: Jeder Aufrufer ist damit fest an genau diesen einen Typ gekettet. Ein Unit-Test für
`mein_cli`s Logik müsste einen echten `OpenAiKompatiblerClient` mit echtem API-Key
instanziieren — ein Test, der ohne Internetverbindung nicht einmal startet. Und wollten wir
später einen zweiten Anbieter unterstützen, bräuchte jede Aufrufstelle eine Fallunterscheidung.

Wir brauchen eine Abstraktion: nicht *"dieser eine Client kann chatten"*, sondern *"irgendein
Typ, der chatten kann"*. Genau dafür sind Rusts **Traits** gemacht — du kennst sie bereits
indirekt über `#[derive(Debug, Clone, PartialEq)]` aus
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md). Dort hat der
Compiler die Implementierung für uns generiert. Diesmal schreiben wir ein Trait **selbst**
— und lernen dabei, was ein Trait eigentlich ist.

## Code (Zielbild)

```rust
pub trait LlmProvider {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler>;
}

impl LlmProvider for OpenAiKompatiblerClient {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        self.anfrage_senden(anfrage)
    }
}
```

## Dekonstruktion

### Was ein Trait wirklich ist

Ein **Trait** ist ein Vertrag: eine Menge von Methoden-Signaturen, die ein Typ
implementieren muss, um zu behaupten, er könne "das". `#[derive(Debug)]` lässt den Compiler
automatisch eine passende Implementierung für ein bekanntes, eingebautes Trait (`Debug`)
schreiben. `LlmProvider` dagegen ist ein Trait, das **wir** definieren — es gehört zu keiner
Bibliothek, es beschreibt ein Konzept aus unserer eigenen Domäne: *"Etwas, mit dem man
chatten kann."*

```rust
pub trait LlmProvider {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler>;
}
```

Das ist die komplette Definition — nur eine Signatur, kein Körper. Ein Trait legt fest,
**welche** Methoden ein Typ anbieten muss und mit welcher Signatur, aber nicht, **wie** sie
implementiert sind. Das "Wie" liefert jeder Typ, der das Trait implementiert, selbst — hier
`OpenAiKompatiblerClient`, später ein Fake-Adapter, der überhaupt kein Netzwerk anfasst.

`&self` (nicht `&mut self`): `chat` verändert den Provider selbst nicht (er hält höchstens
unveränderliche Konfiguration wie den API-Key) — nur lesenden Zugriff nötig, genau wie
`Konversation::verlauf()` aus
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md).

### Warum ein Trait und nicht einfach eine `enum`?

Bei `Rolle` haben wir uns für `enum` entschieden, weil es *endlich viele, feste* Varianten
gibt (System, Benutzer, Assistent). Bei Anbietern ist das anders: Wir wissen heute noch
nicht, wie viele es am Ende geben wird — vielleicht kommt nächstes Jahr ein neuer Anbieter
mit eigener API dazu, vielleicht schreibst du selbst einen zweiten für einen anderen Dienst.
Ein Trait beschreibt *Verhalten*, das **beliebig viele**, auch zukünftige, unbekannte Typen
haben können — ohne dass wir den bestehenden Code dafür anfassen müssen. Das ist der
entscheidende Unterschied zu `enum`: Eine `enum` ist geschlossen (der Compiler kennt alle
Fälle), ein Trait ist offen (jeder darf es implementieren, auch Code, den wir noch gar nicht
geschrieben haben).

### `impl LlmProvider for OpenAiKompatiblerClient` — der Vertrag wird erfüllt

```rust
impl LlmProvider for OpenAiKompatiblerClient {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        self.anfrage_senden(anfrage)
    }
}
```

`impl LlmProvider for OpenAiKompatiblerClient` heißt: "`OpenAiKompatiblerClient` erfüllt ab
jetzt den `LlmProvider`-Vertrag." Die Signatur der Methode `chat` muss **exakt** zur
Trait-Definition passen (Name, Parameter, Rückgabetyp) — der Compiler prüft das beim
Kompilieren, nicht erst zur Laufzeit. Der Methodenkörper selbst delegiert hier nur an die
bereits vorhandene `anfrage_senden`-Methode aus Phase 2 — wir bauen also nichts Neues, wir
verpacken Bestehendes hinter einem neuen, generischen Namen.

> **💡 Tipp**
>
> Warum heißt die Trait-Methode `chat` und nicht `anfrage_senden` wie die bisherige,
> konkrete Methode? Weil der Trait-Name die *Sprache des Vertrags* ist, nicht die Sprache
> einer einzelnen Implementierung. `chat` ist der Begriff, den auch ein ganz anderer
> Adapter (zum Beispiel der Fake-Provider aus [Lektion 4](04-fake-provider.md)) verwenden
> wird — er hat vielleicht gar keine `anfrage_senden`-Methode, weil er nichts sendet.

## Schritt-Reveal

**Schritt 1** — Lege in `mein_core/src/lib.rs` ganz oben eine Modul-Deklaration an, *bevor*
die Datei existiert:

```rust
pub mod port;
```

`cargo check -p mein_core`:

```
error[E0583]: file not found for module `port`
 --> src/lib.rs:1:1
  |
1 | pub mod port;
  | ^^^^^^^^^^^^^
  |
  = help: to create the module `port`, create file "src/port.rs" or "src/port/mod.rs"
```

So liest du das: `pub mod port;` sagt dem Compiler "irgendwo gibt es eine Datei, die zu
diesem Modulnamen gehört" — Rust erwartet dafür standardmäßig entweder `src/port.rs` oder
`src/port/mod.rs`, hat aber (noch) keine von beiden gefunden. Anders als bei einem fehlenden
Typ ist das kein Logikfehler, sondern reine Buchführung: Wir haben dem Compiler von der
Datei erzählt, bevor sie existiert. Der `help`-Hinweis sagt uns exakt, was zu tun ist.

**Schritt 2** — Lege `mein_core/src/port.rs` an:

```rust
use crate::error::ProviderFehler;
use crate::Nachricht;

pub struct ChatAnfrage {
    pub nachrichten: Vec<Nachricht>,
    pub modell: String,
}

pub struct ChatAntwort {
    pub inhalt: String,
}

pub trait LlmProvider {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler>;
}
```

`cargo check -p mein_core` — der E0583-Fehler ist verschwunden, `port` existiert jetzt als
Modul. Falls `ChatAnfrage`/`ChatAntwort` bei dir schon in `mein_core::provider` aus Phase 2
liegen: Verschiebe sie hierher, wir begründen in
[Lektion 2](02-hexagonal-architecture.md), warum diese Typen zum Port gehören, nicht zum
Adapter.

**Schritt 3** — Implementiere `LlmProvider` für deinen Phase-2-Client, in derselben Datei
wie der Client (vorerst — auch das ordnen wir in Lektion 2 neu):

```rust
use crate::port::{ChatAnfrage, ChatAntwort, LlmProvider};

impl LlmProvider for OpenAiKompatiblerClient {
    fn chat(&self, anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        self.anfrage_senden(anfrage)
    }
}
```

**Schritt 4** — Provoziere jetzt bewusst einen zweiten, sehr aufschlussreichen Fehler.
Rufe in `mein_cli` probeweise `chat` **direkt** über den konkreten Typ auf, aber ohne den
Trait zu importieren:

```rust
use mein_core::OpenAiKompatiblerClient; // kein `use mein_core::port::LlmProvider;`

fn beispiel(client: &OpenAiKompatiblerClient, anfrage: mein_core::port::ChatAnfrage) {
    let _ = client.chat(anfrage);
}
```

```
error[E0599]: no method named `chat` found for reference `&OpenAiKompatiblerClient` in the current scope
  --> src/main.rs:5:20
   |
 5 |     let _ = client.chat(anfrage);
   |                    ^^^^ method not found in `&OpenAiKompatiblerClient`
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `LlmProvider` which provides `chat` is implemented but not in scope; perhaps you want to import it
   |
 1 + use mein_core::port::LlmProvider;
   |
```

Das ist eine der wichtigsten Rust-Eigenheiten überhaupt: **Trait-Methoden sind nur
aufrufbar, wenn das Trait selbst mit `use` importiert ist** — selbst wenn der konkrete Typ
die Methode längst implementiert. Das verhindert, dass zwei Bibliotheken mit gleichnamigen
Trait-Methoden sich stillschweigend in die Quere kommen. Der Compiler schlägt die Lösung
sogar vor: `use mein_core::port::LlmProvider;` ergänzen — dann kompiliert derselbe Aufruf.

## Ausführung

```bash
cargo check -p mein_core
cargo check -p mein_cli
```

Beide sollten sauber durchlaufen, sobald `LlmProvider` überall importiert ist, wo `chat`
aufgerufen wird.

```bash
cargo test -p mein_core
```

Noch keine neuen Tests — die folgen ab [Lektion 4](04-fake-provider.md), sobald wir einen
zweiten `LlmProvider` haben, der sich lohnt zu testen.

## Zusammenfassung

- Ein **Trait** ist ein selbst definierbarer Vertrag: Methoden-Signaturen ohne
  Implementierung, den beliebig viele, auch zukünftige Typen erfüllen können.
- `enum` für geschlossene, feste Alternativen (`Rolle`); `trait` für offenes,
  austauschbares Verhalten (`LlmProvider`).
- `impl Trait for Typ` erfüllt den Vertrag; die Methodensignatur muss exakt passen, der
  Compiler prüft das beim Kompilieren.
- Trait-Methoden brauchen den Trait **im Scope** (`use ...::LlmProvider;`), sonst findet der
  Compiler die Methode nicht — auch wenn sie implementiert ist.
- In der Sprache der Hexagonal Architecture (vertieft in
  [Lektion 2](02-hexagonal-architecture.md)) ist `LlmProvider` ein **Port**: die von außen
  sichtbare Schnittstelle, hinter der beliebige **Adapter** stecken können.

## Übung

Erweitere das `LlmProvider`-Trait probeweise um eine zweite Methode, z. B. `fn
modellname(&self) -> &str`, die den Namen des aktuell konfigurierten Modells zurückgibt.
Kompiliere neu (`cargo check -p mein_core`) und beobachte die Fehlermeldung für
`OpenAiKompatiblerClient` — der Compiler zeigt dir exakt, dass jetzt **jeder** Typ, der
`LlmProvider` implementiert, auch `modellname` implementieren muss. Das ist die Kehrseite
eines offenen Vertrags: Jede Erweiterung des Traits verpflichtet alle vorhandenen und
künftigen Implementierungen. Mach die Änderung danach wieder rückgängig, wir brauchen sie
für den Kursverlauf nicht.

[Weiter: Lektion 2 — Hexagonal Architecture](02-hexagonal-architecture.md)
