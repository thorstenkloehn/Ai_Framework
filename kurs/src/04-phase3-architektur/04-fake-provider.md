# Lektion 4: Fake-Provider für Unit-Tests

## Problem

Wir haben jetzt einen Port (`LlmProvider`) und einen echten Adapter
(`OpenAiKompatiblerClient`). Aber wie testen wir Code, der einen `LlmProvider` benutzt — zum
Beispiel eine spätere Funktion, die bei einem Netzwerkfehler dreimal erneut versucht, oder
den [Chain-Pipeline-Code aus Lektion 7](07-chain-pattern-runnable.md)? Mit dem echten
Adapter bräuchten wir für jeden Testlauf eine Internetverbindung, einen gültigen API-Key,
und würden bei jedem `cargo test` echte Kosten verursachen. Einen Timeout mit einer echten
API zu provozieren wäre besonders unangenehm: langsam, unzuverlässig, und wir müssten aktiv
darauf warten, dass etwas *nicht* rechtzeitig antwortet.

Das ist genau der Moment, an dem sich die Arbeit aus den letzten drei Lektionen auszahlt:
Weil unser Code nur gegen das **Trait** `LlmProvider` programmiert, nicht gegen den
konkreten `OpenAiKompatiblerClient`, können wir einen zweiten, winzigen Adapter schreiben,
der überhaupt kein Netzwerk anfasst — einen **Fake-Provider**, der auf Zuruf genau die
Antwort oder genau den Fehler liefert, den ein Test braucht.

## Code (Zielbild)

```rust
#[cfg(test)]
pub struct FakeProvider {
    verhalten: FakeVerhalten,
}

#[cfg(test)]
enum FakeVerhalten {
    Antwort(String),
    Timeout,
}

#[cfg(test)]
impl FakeProvider {
    pub fn antwortet_mit(text: impl Into<String>) -> Self {
        FakeProvider {
            verhalten: FakeVerhalten::Antwort(text.into()),
        }
    }

    pub fn simuliert_timeout() -> Self {
        FakeProvider {
            verhalten: FakeVerhalten::Timeout,
        }
    }
}

#[cfg(test)]
impl LlmProvider for FakeProvider {
    fn chat(&self, _anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        match &self.verhalten {
            FakeVerhalten::Antwort(text) => Ok(ChatAntwort {
                inhalt: text.clone(),
            }),
            FakeVerhalten::Timeout => Err(ProviderFehler::Timeout),
        }
    }
}
```

## Dekonstruktion

### `#[cfg(test)]` auf einem ganzen Modul, nicht nur auf Testfunktionen

Du kennst `#[cfg(test)]` bereits von `mod tests { ... }` aus
[Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md). Hier setzen wir es auf den
kompletten Adapter: `FakeProvider` soll **niemals** Teil eines regulären Release-Builds von
`mein_core` sein — er hat in echtem, laufendem Code nichts verloren, nur in Tests. In
`mein_core/src/adapter.rs` sieht das entsprechend aus:

```rust
pub mod openai_kompatibel;

#[cfg(test)]
pub mod fake;
```

`openai_kompatibel` ist immer dabei, `fake` nur, wenn mit `cargo test` (oder allgemeiner:
mit dem `test`-Flag) kompiliert wird. Das entspricht exakt der Ordnerstruktur aus
[Lektion 2](02-hexagonal-architecture.md) — CANON für dieses Framework nennt genau das den
Test-Adapter.

### Warum `FakeVerhalten` und nicht direkt `Result<ChatAntwort, ProviderFehler>` speichern?

Der naheliegendste erste Versuch wäre, den kompletten Rückgabewert direkt im
`FakeProvider` zu speichern:

```rust
#[cfg(test)]
pub struct FakeProvider {
    antwort: Result<ChatAntwort, ProviderFehler>,
}

impl LlmProvider for FakeProvider {
    fn chat(&self, _anfrage: ChatAnfrage) -> Result<ChatAntwort, ProviderFehler> {
        self.antwort.clone()
    }
}
```

`chat` nimmt `&self` entgegen (siehe Trait-Definition aus
[Lektion 1](01-llmprovider-port.md)) — wir dürfen `self.antwort` also nicht *herausbewegen*
(**Move**, siehe [Ownership in Phase 1](../02-phase1-fundament/02-rolle-und-nachricht.md)),
sondern müssen es kopieren. `.clone()` verlangt aber, dass `Result<ChatAntwort,
ProviderFehler>` das `Clone`-Trait implementiert — und dafür müssen **beide** Typparameter,
`ChatAntwort` *und* `ProviderFehler`, `Clone` implementieren. Versuchen wir, das für
`ProviderFehler` (unseren `thiserror`-Fehlertyp aus Phase 2) einfach abzuleiten:

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderFehler {
    #[error("Netzwerkfehler: {0}")]
    Netzwerk(#[from] reqwest::Error),
    #[error("Zeitüberschreitung beim Warten auf die Antwort")]
    Timeout,
    #[error("Anbieter antwortete mit Status {status}: {meldung}")]
    Api { status: u16, meldung: String },
}
```

```
error[E0277]: the trait bound `reqwest::Error: Clone` is not satisfied
  --> src/error.rs:1:17
   |
 1 | #[derive(Debug, Clone, thiserror::Error)]
   |                 ^^^^^ the trait `Clone` is not implemented for `reqwest::Error`
   |
   = note: required for `ProviderFehler` to implement `Clone`
```

Das ist eine echte Grenze, keine Nachlässigkeit: `reqwest::Error` kann eine zugrundeliegende
`std::io`-Fehlerursache enthalten, die selbst nicht dupliziert werden kann — die
`reqwest`-Autor*innen haben bewusst kein `Clone` dafür implementiert. Wir können das nicht
mit einem weiteren `derive` reparieren, weil wir `reqwest::Error` nicht selbst definiert
haben (fremder Typ, fremde Regeln). Genau deshalb speichert unser `FakeProvider` **keinen
fertigen `ProviderFehler`**, sondern die einfache, selbst definierte `FakeVerhalten`-`enum`
aus dem Zielbild oben — sie enthält nur `String` und keinen `reqwest::Error`, ist also
problemlos klonbar (oder braucht, wie hier, überhaupt kein `Clone`: Wir bauen bei jedem
Aufruf von `chat` einen frischen `ProviderFehler::Timeout` — der hat gar keine Felder, es
gibt nichts zu klonen).

> **⚠️ Warnung**
>
> Wenn ein `derive`-Fehler wie oben eine Bibliotheks-Fehlermeldung über einen fremden Typ
> zeigt (`reqwest::Error: Clone`), ist der Reflex "einfach überall `Clone` ableiten" der
> falsche Weg. Frag stattdessen: Brauche ich wirklich eine Kopie des *gesamten* Werts, oder
> reicht es, die für meinen Zweck relevante Information (hier: "es soll ein Timeout sein")
> separat, in einer eigenen, einfacheren Struktur zu halten?

### `antwortet_mit` und `simuliert_timeout` — sprechende Konstruktoren statt eines generischen `neu`

Statt eines einzigen `FakeProvider::neu(verhalten: FakeVerhalten)` geben wir zwei benannte
Konstruktoren. Das ist reine Lesbarkeit: `FakeProvider::simuliert_timeout()` an einer
Testzeile sagt sofort, *was* der Test prüft, ohne dass man erst `FakeVerhalten::Timeout`
nachschlagen muss. `FakeVerhalten` selbst bleibt bewusst **nicht** `pub` (kein `pub enum`) —
Aufrufer*innen sollen den `FakeProvider` nur über diese beiden Konstruktoren bauen, nicht
das interne Verhalten direkt zusammenstecken.

## Schritt-Reveal

**Schritt 1** — Lege `mein_core/src/adapter/fake.rs` an, zunächst mit der naiven
`Result`-Variante von oben, um den E0277-Fehler bewusst zu provozieren. Beobachte ihn, dann
verwirf diesen Ansatz.

**Schritt 2** — Baue `FakeProvider` mit `FakeVerhalten` wie im Zielbild. `cargo check -p
mein_core` sollte jetzt sauber durchlaufen (`FakeProvider` wird nur unter `#[cfg(test)]`
kompiliert, `cargo check` ohne `--tests` prüft ihn also noch gar nicht mit — dazu gleich
mehr).

**Schritt 3** — Schreibe den ersten Test, der den Erfolgsfall prüft, ans Ende von
`adapter/fake.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erfolgreiche_antwort_wird_durchgereicht() {
        let provider = FakeProvider::antwortet_mit("Hallo zurück!");
        let anfrage = ChatAnfrage {
            nachrichten: vec![],
            modell: "irgendein-modell".into(),
        };

        let antwort = provider.chat(anfrage).unwrap();

        assert_eq!(antwort.inhalt, "Hallo zurück!");
    }
}
```

**Schritt 4 — die Transferaufgabe der Phase.** Schreibe direkt daneben den Timeout-Test:

```rust
    #[test]
    fn timeout_wird_ohne_echte_api_erkannt() {
        let provider = FakeProvider::simuliert_timeout();
        let anfrage = ChatAnfrage {
            nachrichten: vec![],
            modell: "irgendein-modell".into(),
        };

        let ergebnis = provider.chat(anfrage);

        assert!(matches!(ergebnis, Err(ProviderFehler::Timeout)));
    }
```

Das ist die Transferaufgabe dieser Phase in Aktion: **Wir testen einen Timeout, ohne eine
echte API aufzurufen.** Kein Netzwerk, keine Wartezeit, kein API-Key — der Test läuft in
Millisekunden und ist trotzdem ein echter Test unseres Fehlerpfads: Ruft irgendein Aufrufer
später `provider.chat(...)` auf einem `FakeProvider::simuliert_timeout()` auf, muss er mit
`Err(ProviderFehler::Timeout)` umgehen können, genau wie mit einem echten Timeout.

`matches!(wert, muster)` ist ein Makro, das `true` zurückgibt, wenn `wert` zum `muster`
passt — kompakter als ein volles `match` mit zwei Zweigen, wenn du nur an einem einzigen
Fall interessiert bist. Wir haben eine sehr ähnliche Struktur schon als `if let` in
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) gesehen; `matches!` ist die
Ausdrucks-Variante davon.

## Ausführung

```bash
cargo test -p mein_core
```

```
running 2 tests
test adapter::fake::tests::erfolgreiche_antwort_wird_durchgereicht ... ok
test adapter::fake::tests::timeout_wird_ohne_echte_api_erkannt ... ok
```

Beachte: `cargo build -p mein_core` (ohne `test`) enthält `FakeProvider` **nicht** im
erzeugten Code — probiere versuchsweise, `FakeProvider` aus `mein_cli/src/main.rs` heraus zu
benutzen (das nicht unter `#[cfg(test)]` steht):

```
error[E0433]: failed to resolve: could not find `fake` in `adapter`
```

Das ist kein Bug, sondern genau die Absicht von `#[cfg(test)]`: Der Fake-Adapter existiert
für den Compiler schlicht nicht, sobald nicht im Testmodus kompiliert wird.

## Zusammenfassung

- Ein Fake-Adapter implementiert denselben Port (`LlmProvider`) wie der echte Adapter,
  antwortet aber ohne Netzwerkzugriff, konfigurierbar über sprechende Konstruktoren.
- `#[cfg(test)]` auf dem gesamten Modul stellt sicher, dass Test-Doubles nie in einem
  regulären Build landen.
- Nicht jedes `derive` lässt sich einfach ergänzen: `Clone` für einen Fehlertyp, der einen
  fremden, nicht klonbaren Typ (`reqwest::Error`) enthält, scheitert zu Recht — die Lösung
  ist ein eigenes, einfacheres Test-Datenmodell statt eines erzwungenen `derive`.
- Ein Timeout lässt sich vollständig deterministisch, ohne Netzwerk und ohne Wartezeit
  testen, sobald der Provider hinter einem Trait steckt — das ist der ganze Sinn dieser
  Phase.

## Übung

Erweitere `FakeVerhalten` um einen dritten Fall, `ApiFehler { status: u16, meldung: String
}`, und einen passenden Konstruktor `FakeProvider::antwortet_mit_fehler(status: u16,
meldung: impl Into<String>)`. Schreibe einen Test, der prüft, dass `chat` in diesem Fall
`Err(ProviderFehler::Api { status: 429, meldung: ... })` zurückgibt (429 ist der
HTTP-Status für "zu viele Anfragen" — ein realistischer Fehlerfall, den wir in
[Phase 5, Lektion 6](../06-phase5-rag-betrieb/06-retry-rate-limit-backoff.md) mit Retry- und
Backoff-Logik behandeln werden). Überlege beim Schreiben: Warum reicht auch hier ein
frisches `ProviderFehler::Api { status, meldung }` pro Aufruf, ohne dass `ProviderFehler`
`Clone` implementieren muss?

[Weiter: Lektion 5 — Integrationstests und clippy](05-tests-und-clippy.md)
