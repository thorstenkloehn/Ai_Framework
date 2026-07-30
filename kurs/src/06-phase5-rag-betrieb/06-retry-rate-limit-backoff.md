# Lektion 6: Retry, Rate Limit und Backoff

## Problem

Seit [Phase 2, Lektion 1](../03-phase2-llm-anbindung/01-http-grenze-reqwest.md) spricht
`mein_core::provider` über `reqwest` mit einem echten LLM-Anbieter. Netzwerkaufrufe
scheitern gelegentlich — nicht weil unser Code falsch ist, sondern weil die Gegenseite
kurzzeitig überlastet ist (HTTP-Status `429 Too Many Requests`, wenn wir ein
**Rate Limit** — eine Obergrenze an Anfragen pro Zeitfenster — überschreiten) oder einen
Serverfehler meldet (`5xx`). Ein einzelner fehlgeschlagener Aufruf sollte nicht sofort die
ganze Anfrage an unsere Nutzer:innen scheitern lassen — aber ein sofortiges,
ungebremstes Wiederholen wäre auch keine Lösung: Es würde einen bereits überlasteten
Anbieter noch weiter belasten. Die Lösung heißt **exponentielles Backoff**: Nach jedem
Fehlschlag warten wir länger als beim vorigen Versuch, bevor wir es erneut probieren.

## Code (Zielbild)

```rust
pub async fn mit_backoff<T, E, F, Fut>(
    max_versuche: u32,
    basis_wartezeit: Duration,
    mut aktion: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
```

```rust
let antwort = mit_backoff(4, Duration::from_millis(200), || {
    provider.chat_anfrage(&konversation)
}).await?;
```

## Dekonstruktion

### Nicht jeder Fehler verdient einen erneuten Versuch

Ein Fehlerpfad, den wir bewusst besprechen müssen (Definition of Done aus
[Kapitel 0](../00-einleitung/02-wie-dieser-kurs-funktioniert.md)): Ein `401 Unauthorized`
(ungültiger API-Key) wird durch Wiederholen nicht besser — der Key bleibt ungültig. Ein
`429` oder `503` dagegen ist oft nur vorübergehend. Wir unterscheiden deshalb:

```rust
pub enum RetryEntscheidung {
    NochEinmalVersuchen,
    Aufgeben,
}

pub fn einordnen(status: u16) -> RetryEntscheidung {
    match status {
        429 | 500..=599 => RetryEntscheidung::NochEinmalVersuchen,
        _ => RetryEntscheidung::Aufgeben,
    }
}
```

`500..=599` ist ein **Bereichsmuster** (*range pattern*) im `match` — jeder
Server-seitige Fehlerstatus zwischen 500 und 599 fällt hindurch in denselben Zweig, ohne
dass wir jeden einzelnen Code (500, 501, 502, ...) aufzählen müssen.

### Die generische `mit_backoff`-Funktion

```rust
pub async fn mit_backoff<T, E, F, Fut>(
    max_versuche: u32,
    basis_wartezeit: Duration,
    mut aktion: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
```

Vier Generics tauchen hier auf einmal auf — das ist mehr, als du bisher in diesem Kurs
gesehen hast, also Schritt für Schritt:

- `T` — der Erfolgstyp (z. B. eine Provider-Antwort).
- `E` — der Fehlertyp.
- `F: FnMut() -> Fut` — `aktion` ist ein **aufrufbarer Wert** (typischerweise eine
  Closure), der bei jedem Aufruf einen neuen `Fut` erzeugt. `FnMut` statt `Fn`, weil ein
  echter Netzwerkaufruf üblicherweise selbst veränderlichen Zustand mitführt (z. B. einen
  Zähler in einem `reqwest::Client`); `mut aktion` in der Signatur erlaubt uns, sie
  mehrfach in der Schleife aufzurufen.
- `Fut: Future<Output = Result<T, E>>` — der von `aktion()` zurückgegebene Wert ist selbst
  ein Future (schließlich rufen wir eine `async`-Aktion mehrfach auf), dessen Ergebnis ein
  `Result<T, E>` ist.

Warum ein generischer Wrapper statt fest in `mein_core::provider` verdrahteter
Retry-Logik? Damit dieselbe Backoff-Funktion später auch für Embedding-Aufrufe
([Lektion 3](03-embeddings-vector-store.md)) oder andere HTTP-Grenzen wiederverwendbar
ist, ohne Code zu duplizieren.

### Die Wartezeit-Formel

```rust
let wartezeit = basis_wartezeit * 2u32.pow(versuch - 1);
```

Versuch 1 scheitert → warte `basis_wartezeit * 2^0 = basis_wartezeit`. Versuch 2
scheitert → warte `basis_wartezeit * 2^1`, also doppelt so lange. Versuch 3 → `* 2^2`,
viermal so lange. Das ist **exponentielles** Wachstum: Bei `basis_wartezeit = 200ms` und
vier Versuchen warten wir 200ms, 400ms, 800ms — statt bei jedem der potenziell vielen
gleichzeitig retryenden Clients sofort erneut anzufragen.

> **💡 Tipp**
>
> Produktive Systeme fügen der Wartezeit oft **Jitter** hinzu — eine kleine zufällige
> Schwankung (z. B. `wartezeit * (0.5 + zufallszahl_zwischen_0_und_1())`). Grund: Wenn
> hundert Clients gleichzeitig denselben Fehler bekommen, würden sie ohne Jitter alle
> exakt zur selben Millisekunde erneut anfragen (*thundering herd* — "stampfende Herde")
> und den Anbieter erneut überlasten. Das ist die Übung dieser Lektion.

## Schritt-Reveal

**Schritt 1 — `RetryEntscheidung` und `einordnen` anlegen** (siehe Dekonstruktion oben,
z. B. in `mein_core/src/provider.rs` oder einem neuen `mein_core/src/retry.rs`).

**Schritt 2 — `mit_backoff` implementieren:**

```rust
use std::future::Future;
use std::time::Duration;

pub async fn mit_backoff<T, E, F, Fut>(
    max_versuche: u32,
    basis_wartezeit: Duration,
    mut aktion: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut versuch = 0;
    loop {
        match aktion().await {
            Ok(wert) => return Ok(wert),
            Err(fehler) => {
                versuch += 1;
                if versuch >= max_versuche {
                    return Err(fehler);
                }
                let wartezeit = basis_wartezeit * 2u32.pow(versuch - 1);
                tokio::time::sleep(wartezeit).await;
            }
        }
    }
}
```

Beachte: `mit_backoff` gibt bei endgültigem Scheitern den **letzten** Fehler zurück, nicht
etwa eine Sammlung aller Fehlversuche — für unsere Zwecke reicht das; Aufrufer:innen
interessiert vor allem, warum der *letzte* Versuch fehlschlug.

## Ausführung

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn gibt_nach_erfolgreichem_versuch_ergebnis_zurueck() {
        let zaehler = Arc::new(AtomicU32::new(0));
        let z2 = zaehler.clone();

        let ergebnis: Result<&str, &str> = mit_backoff(5, Duration::from_millis(1), || {
            let z = z2.clone();
            async move {
                let versuch = z.fetch_add(1, Ordering::SeqCst);
                if versuch < 2 { Err("503 Service Unavailable") } else { Ok("ok") }
            }
        }).await;

        assert_eq!(ergebnis, Ok("ok"));
        assert_eq!(zaehler.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn gibt_nach_max_versuchen_auf() {
        let ergebnis: Result<&str, &str> =
            mit_backoff(3, Duration::from_millis(1), || async { Err("immer kaputt") }).await;

        assert_eq!(ergebnis, Err("immer kaputt"));
    }
}
```

`AtomicU32` — ein Zähler, der threadsicher erhöht werden kann (`fetch_add`), ohne einen
expliziten Lock zu brauchen — simuliert hier "die ersten zwei Aufrufe scheitern, der
dritte klappt", damit der Test deterministisch ist, ohne echtes Netzwerk zu brauchen.

```bash
cargo test -p mein_core
```

```
running 2 tests
test provider::tests::gibt_nach_erfolgreichem_versuch_ergebnis_zurueck ... ok
test provider::tests::gibt_nach_max_versuchen_auf ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Provoziere den Fehlerpfad bewusst: Setze `max_versuche` auf `1` bei einer stets
fehlschlagenden `aktion` — `mit_backoff` gibt sofort nach dem ersten (einzigen) Versuch
den Fehler zurück, ohne zu warten, weil `versuch >= max_versuche` schon nach dem ersten
Fehlschlag zutrifft.

## Zusammenfassung

- Nicht jeder Fehler ist retry-würdig: `429`/`5xx` (vorübergehend) unterscheiden wir von
  `4xx`-Clientfehlern wie `401` (bleiben bestehen, egal wie oft wir es versuchen).
  Retry-Logik gehört bewusst hinter eine Einordnung, nicht vor sie.
- `mit_backoff<T, E, F, Fut>` ist generisch über beliebige asynchrone Aktionen — dieselbe
  Funktion bedient Provider- und Embedding-Aufrufe gleichermaßen.
- Exponentielles Wachstum der Wartezeit (`basis * 2^versuch`) entlastet einen bereits
  überlasteten Anbieter, statt ihn mit sofortigen Wiederholungen weiter zu belasten.
- `AtomicU32` erlaubt deterministische Tests für "beim n-ten Versuch klappt es", ganz
  ohne echtes Netzwerk oder Zeitverzögerung im Test.

## Übung

Ergänze `mit_backoff` um Jitter: Statt der reinen Formel `basis_wartezeit *
2^(versuch-1)` soll die tatsächliche Wartezeit zufällig zwischen 50 % und 100 % dieses
Werts liegen (Crate `rand`, `cargo add rand` — prüfe mit `cargo doc -p rand --open` oder
der aktuellen Doku auf docs.rs, welche Methode und welcher Trait-Import in der gerade
installierten Version für einen Zufallswert in einem Bereich nötig sind, das hat sich
zwischen `rand`-Versionen schon mehrfach geändert). Überlege dir vorher, warum ein Test
für Zufallsverhalten anders aussehen muss als die bisherigen Tests (Hinweis: Du kannst
nicht mehr auf einen exakten Wert prüfen — wohl aber auf einen plausiblen Bereich).

[Weiter: Lektion 7 — Tracing, Kosten-Tracking und Secrets](07-tracing-kosten-secrets.md)
