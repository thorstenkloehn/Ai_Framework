# Lektion 4: SemVer und Breaking Changes

## Problem

Sobald `mein_core` auf [crates.io](https://crates.io) liegt, hängen fremde Projekte mit
einer Zeile wie `mein_core = "0.1"` daran. Wenn wir jetzt einfach eine neue Fähigkeit zum
`LlmProvider`-Trait hinzufügen — sagen wir, eine Methode, die den verwendeten Modellnamen
zurückgibt — kann das für jede außenstehende Implementierung dieses Traits (z. B. der
Provider-Adapter, den du in [Lektion 2](02-feature-flags.md) als Übung selbst gebaut
hast) bedeuten, dass sie plötzlich nicht mehr kompiliert. Wir brauchen eine Sprache, mit
der wir Nutzer*innen *im Voraus*, allein an einer Versionsnummer, mitteilen können: "Diese
neue Version ist sicher zu übernehmen" oder "Achtung, hier kann etwas brechen."

## Code (Zielbild)

```rust
// Version 0.1.0
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, konversation: &Konversation) -> Result<Nachricht, ProviderFehler>;
}
```

```rust
// Version 0.2.0 — zusätzliche Methode MIT Default-Implementierung
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, konversation: &Konversation) -> Result<Nachricht, ProviderFehler>;

    fn modellname(&self) -> &str {
        "unbekannt"
    }
}
```

## Dekonstruktion

### SemVer: Major.Minor.Patch

**SemVer** (*Semantic Versioning*) ist eine Konvention für Versionsnummern der Form
`MAJOR.MINOR.PATCH` (z. B. `0.1.0`), bei der jede Stelle eine feste Bedeutung trägt:

- **PATCH** (letzte Ziffer) — reine Fehlerkorrekturen, kein Unterschied in der
  öffentlichen API. Bestehender Aufrufcode funktioniert unverändert weiter.
- **MINOR** (mittlere Ziffer) — neue Funktionalität, die **abwärtskompatibel** ist:
  Bestehender Code kompiliert unverändert, es kommt nur etwas Neues hinzu.
- **MAJOR** (erste Ziffer) — mindestens eine **Breaking Change**: Bestehender Code kann
  nach dem Update nicht mehr kompilieren oder sich anders verhalten.

Solange die MAJOR-Version `0` ist (wie unser `ai-framework-0.1.0`), gilt bei SemVer eine
Sonderregel: Schon ein MINOR-Sprung (`0.1.0` → `0.2.0`) darf Breaking Changes enthalten —
Version `0.x` gilt als "noch nicht stabil versprochen". Erst ab `1.0.0` gilt die volle
Garantie: Nur ein MAJOR-Sprung darf etwas brechen.

### Was zählt bei einem `pub trait` als Breaking Change?

Das ist der Kern dieser Lektion, weil Traits wie `LlmProvider` besonders leicht brechen:

| Änderung | Breaking? | Warum |
|---|---|---|
| Neue Methode **ohne** Default-Implementierung hinzufügen | Ja | Jede externe Implementierung fehlt jetzt ein Trait-Element. |
| Neue Methode **mit** Default-Implementierung hinzufügen | Nein | Externe Implementierungen kompilieren unverändert weiter, nutzen einfach den Default. |
| Eine bestehende Methode entfernen oder umbenennen | Ja | Aufrufender Code, der die Methode benutzt hat, kompiliert nicht mehr. |
| Ein `pub`-Struct-Feld hinzufügen (bei einem Struct ohne `#[non_exhaustive]`) | Meist ja | Konstruktion per `Struct { feld1, feld2 }`-Literal bricht, weil ein Feld fehlt. |
| Eine neue Variante zu einem `pub enum` hinzufügen | Ja (ohne `#[non_exhaustive]`) | Ein `match` ohne `_`-Zweig bei Aufrufer*innen deckt die neue Variante nicht ab und kompiliert nicht mehr. |

Das Beispiel im Zielbild oben zeigt die einzige sichere Art, ein Trait wie `LlmProvider`
zu erweitern, ohne die MAJOR-Version anheben zu müssen: eine **Default-Implementierung**
für die neue Methode. Wer sie nicht überschreibt, bekommt automatisch das Standardverhalten
— kein bestehender Adapter aus [Phase 3](../04-phase3-architektur/README.md) oder
[Lektion 2](02-feature-flags.md) muss angefasst werden.

### `#[non_exhaustive]` — Zukunftssicherheit einbauen

```rust
#[non_exhaustive]
pub enum ProviderFehler {
    Timeout,
    UngueltigerApiKey,
}
```

`#[non_exhaustive]` zwingt jeden externen `match` über `ProviderFehler`, einen `_`-Zweig
zu haben — auch wenn aktuell alle Varianten abgedeckt sind. Der Vorteil: Fügen wir später
eine dritte Variante hinzu (z. B. `RateLimitUeberschritten`), bricht kein externer Code,
weil er den Fall ohnehin schon über `_` behandelt. Der Preis: Aufrufer*innen können nie
ganz sicher sein, "alle" Fälle erschöpfend behandelt zu haben — eine bewusste Abwägung
zwischen Erweiterbarkeit und Vollständigkeitsgarantie.

## Schritt-Reveal

**Schritt 1 — Provoziere den Breaking Change bewusst.** Füge `modellname` **ohne**
Default-Implementierung zum `LlmProvider`-Trait hinzu:

```rust
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, konversation: &Konversation) -> Result<Nachricht, ProviderFehler>;
    fn modellname(&self) -> &str;
}
```

`cargo check -p mein_core` — kompiliert (der Trait selbst ist nur eine Definition). Prüfe
jetzt eine bestehende Implementierung, z. B. den Fake-Provider aus
[Phase 3, Lektion 4](../04-phase3-architektur/04-fake-provider.md):

```bash
cargo check -p mein_core --tests
```

```
error[E0046]: not all trait items implemented, missing: `modellname`
  --> mein_core/src/adapter/fake.rs:12:1
   |
12 | impl LlmProvider for FakeProvider {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `modellname` in implementation
```

Exakt die Situation aus dem Problem-Abschnitt oben — nur dass wir sie hier selbst
kontrolliert auslösen, statt dass eine fremde Nutzerin sie beim Update erlebt. Das ist der
Sinn dieser Übung: den Fehler *vor* der Veröffentlichung erleben, nicht danach.

**Schritt 2 — Korrektur mit Default-Implementierung** (siehe Zielbild, Version 0.2.0).
`cargo check -p mein_core --tests` — kompiliert jetzt wieder, ohne dass `FakeProvider`
angefasst werden musste.

**Schritt 3 — `#[non_exhaustive]` an `ProviderFehler` ergänzen** und einen `match` ohne
`_`-Zweig testen:

```
error[E0004]: non-exhaustive patterns: `_` not covered
  --> mein_cli/src/main.rs:8:11
   |
 8 |     match fehler {
   |           ^^^^^^ pattern `_` not covered
   = note: `ProviderFehler` is marked as non-exhaustive, so a wildcard `_` arm is necessary
```

Ergänze den fehlenden `_`-Zweig — die Fehlermeldung sagt bereits exakt, was zu tun ist.

## Ausführung

```bash
cargo check -p mein_core --all-features --tests
```

```
    Checking mein_core v0.2.0 (mein_core)
    Finished dev [unoptimized + debuginfo] target(s) in 1.84s
```

Kein Fehler mehr — die neue Fähigkeit wurde additiv, nicht brechend eingeführt.

> **⚠️ Warnung**
>
> SemVer ist ein **Versprechen**, keine automatische Garantie — Cargo prüft nicht selbst,
> ob deine Änderung wirklich zur gewählten Versionsnummer passt. Es gibt das externe Tool
> `cargo-semver-checks`, das API-Diffs gegen die letzte veröffentlichte Version prüft und
> genau solche Verstöße automatisch findet; für unseren Kurs reicht das bewusste
> Durchdenken jeder Änderung anhand der Tabelle oben.

## Zusammenfassung

- SemVer kodiert die Bedeutung einer Versionsänderung in drei Zahlen: PATCH
  (Fehlerkorrektur), MINOR (additiv, abwärtskompatibel), MAJOR (Breaking Change).
- Solange die MAJOR-Version `0` ist, gilt schon ein MINOR-Sprung als potenziell brechend —
  echte Stabilität beginnt erst ab `1.0.0`.
- Ein `pub trait` bricht durch neue Methoden ohne Default-Implementierung, entfernte
  Methoden, oder geänderte Signaturen — eine Default-Implementierung macht eine neue
  Methode additiv statt brechend.
- `#[non_exhaustive]` an `enum`/`struct` erzwingt zukunftsoffene `match`-Ausdrücke bei
  Aufrufer*innen und erlaubt so, später neue Varianten hinzuzufügen, ohne die MAJOR-Version
  zu erhöhen.

## Übung

Entscheide für die folgenden drei hypothetischen Änderungen jeweils, ob sie PATCH, MINOR
oder MAJOR wären, und begründe kurz warum: (1) Ein öffentliches Feld `temperatur: f64` auf
`Konfiguration` wird in `temperatur: Option<f64>` geändert. (2) Der `ClientBuilder` aus
[Lektion 1](01-builder-pattern.md) bekommt eine zusätzliche Setter-Methode
`.timeout(Duration)`. (3) Ein interner Implementierungsfehler in `RoutingProvider` aus
[Phase 6, Lektion 3](../07-phase6-performance/03-model-routing-fallback.md) wird
behoben, ohne dass sich Signaturen ändern. Prüfe deine Antworten anhand der Tabelle oben.

[Weiter: Lektion 5 — crates.io-Checkliste](05-crates-io-checkliste.md)
