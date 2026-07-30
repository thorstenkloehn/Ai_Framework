# Lektion 7: Persistenz mit sqlx

## Problem

Jede `Konversation` lebt bisher ausschließlich im Arbeitsspeicher eines einzelnen
`mein_cli`-Aufrufs — beendest du das Programm, ist der Verlauf weg. Ein echtes
KI-Framework muss Konversationen über einzelne Programmläufe hinweg **aufbewahren**
können. Diese Lektion ist bewusst eine **Skizze**, kein vollständiges
Datenbank-Setup: Wir zeichnen die Grenze zwischen `mein_core` und einer echten Datenbank
sauber vor, ohne uns schon in Migrations-Tooling, Schema-Versionierung oder
Verbindungspools für Produktionslast zu vertiefen — das wäre für Phase 2 zu viel auf
einmal. Was wir hier bauen, ist tragfähig genug, um später (Phase 5, Betrieb) ausgebaut
zu werden, ohne die öffentliche Form noch einmal umzuwerfen.

## Code (Zielbild)

```rust
pub struct KonversationsSpeicher {
    pool: sqlx::SqlitePool,
}

impl KonversationsSpeicher {
    pub async fn neu(datenbank_url: &str) -> Result<Self, PersistenzFehler> {
        // Verbindung aufbauen, Tabelle anlegen, falls sie noch nicht existiert
    }

    pub async fn speichern(&self, konversation: &Konversation) -> Result<i64, PersistenzFehler> {
        // Konversation als JSON in die Datenbank schreiben, ID zurückgeben
    }

    pub async fn laden(&self, id: i64) -> Result<Konversation, PersistenzFehler> {
        // JSON aus der Datenbank lesen, als Konversation zurückgeben
    }
}
```

## Dekonstruktion

### Ein Vorgriff: `async`/`await`

`sqlx` ist eine **asynchrone** Bibliothek — anders als `reqwest::blocking` aus
[Lektion 1](01-http-grenze-reqwest.md) gibt es keine blockierende Variante. Das bedeutet,
wir müssen hier zum ersten Mal `async fn` und `.await` benutzen, obwohl wir
`async`/Tokio erst vollständig in
[Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md) erklären. Für diese
Lektion reicht ein pragmatisches Verständnis, das du dir einfach merkst:

- `async fn` markiert eine Funktion als **asynchron** — sie liefert nicht sofort ihr
  Ergebnis, sondern etwas, das man **abwarten** (*awaiten*) muss.
- `.await` hinter einem Aufruf einer `async fn` heißt: "Warte hier, bis das Ergebnis
  wirklich da ist, dann mach weiter." Ohne `.await` bekommst du nicht das Ergebnis,
  sondern nur das *Versprechen* auf ein Ergebnis (siehe der provozierte Fehler unten).
- Um eine `async fn` überhaupt auszuführen, brauchst du eine **Async-Laufzeit**
  (*runtime*) — wir nutzen dafür (nur in dieser Lektion, als Muster zum Abschreiben)
  `#[tokio::test]` auf Testfunktionen.

Betrachte das als Vorschau, kein vollständiges Verständnis — wir lösen das Versprechen
in Phase 4 vollständig ein. Bis dahin gilt: Kopiere das Muster, es funktioniert, auch
ohne dass du schon jedes Detail durchdringst.

### `PersistenzFehler` — wieder `thiserror`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenzFehler {
    #[error("Datenbankfehler: {0}")]
    Datenbank(#[from] sqlx::Error),

    #[error("Gespeicherte Konversation hatte kein gültiges JSON-Format: {0}")]
    UngueltigesFormat(#[from] serde_json::Error),
}
```

Dasselbe Muster wie `ProviderFehler` in [Lektion 4](04-fehlerbehandlung.md): zwei
Fehlerquellen (die Datenbank selbst, und unser eigenes JSON-Format), beide über
`#[from]` automatisch für `?` nutzbar. Ein neues Modul `mein_core::persistence`
(`mein_core/src/persistence.rs`, `pub mod persistence;` in `lib.rs`) bekommt diesen
Fehlertyp und `KonversationsSpeicher`.

### Design-Entscheidung: `Konversation` als JSON-Text speichern, nicht Zeile für Zeile

Wir könnten eine relationale Tabellenstruktur bauen: eine Tabelle `konversationen`, eine
Tabelle `nachrichten` mit einem Fremdschlüssel zurück zur Konversation, eine Zeile pro
`Nachricht`. Das wäre "sauberer" im klassischen Datenbank-Sinn — aber für eine Skizze
in Phase 2 unnötig komplex (zwei Tabellen, Joins, eine echte Migration). Wir wählen
bewusst die einfachere Variante: **Die gesamte `Konversation` wird als ein einziger
JSON-Text in einer Spalte gespeichert.** Das nutzt genau aus, was wir in
[Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md) bei `Rolle`/
`Nachricht` schon angelegt haben — und was jetzt auch `Konversation` selbst braucht:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Konversation {
    verlauf: Vec<Nachricht>,
}
```

`Serialize`/`Deserialize` ergänzen wir hier zum ersten Mal auch auf `Konversation`
selbst (bisher hatte nur `Nachricht` sie). Das funktioniert trotz des **privaten**
Felds `verlauf` (siehe [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md))
problemlos: `#[derive(...)]` erzeugt seinen Code direkt an der Stelle, an der
`Konversation` definiert ist — innerhalb desselben Moduls hat generierter Code
denselben Zugriff wie handgeschriebener.

> **💡 Tipp**
>
> Diese Entscheidung — ein ganzes Aggregat als JSON-Blob statt normalisiert über mehrere
> Tabellen — ist ein verbreitetes, legitimes Muster (manchmal *"Document Store in einer
> relationalen Datenbank"* genannt), solange du nie **Teile** der `Konversation** direkt
> per SQL abfragen musst (z. B. "alle Nachrichten mit Rolle X über alle Konversationen").
> Bräuchten wir das später, wäre der Zeitpunkt gekommen, doch zu normalisieren — bis
> dahin sparen wir uns die Komplexität. Auch das ist YAGNI.

### Tabelle anlegen ohne echtes Migrations-Tool

```rust
pub async fn neu(datenbank_url: &str) -> Result<Self, PersistenzFehler> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(datenbank_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS konversationen (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            inhalt TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(KonversationsSpeicher { pool })
}
```

`sqlx` bietet ein echtes Migrations-System (`sqlx::migrate!`, versionierte `.sql`-Dateien,
die nacheinander angewendet werden) — für eine einzelne Tabelle in einer Lern-Skizze ist
das noch übertrieben. Stattdessen legen wir die Tabelle beim Verbindungsaufbau direkt an,
mit `CREATE TABLE IF NOT EXISTS` ("nur anlegen, falls noch nicht vorhanden" — mehrfaches
Starten von `mein_cli` schlägt so nicht beim zweiten Mal fehl). Wenn `mein_core` später
(Phase 5) produktionsreif wird, ist `sqlx::migrate!` der nächste logische Schritt — wir
erwähnen es hier bewusst nur, ohne es einzuführen.

> **⚠️ Warnung**
>
> `.max_connections(1)` ist hier **kein** Performance-Detail, sondern nötig für
> Korrektheit: Eine In-Memory-SQLite-Datenbank (`sqlite::memory:`) existiert nur so
> lange, wie *eine bestimmte* Verbindung offen ist — jede neue Verbindung aus einem Pool
> mit mehr als einer Verbindung bekäme ihre **eigene, leere** Datenbank. Mit genau einer
> Verbindung im Pool ist das kein Thema. Nutzt du stattdessen eine echte Datei
> (`sqlite://pfad/zur/datei.db`), ist diese Einschränkung nicht nötig — für unsere
> Lern-Skizze bleiben wir bei `sqlite::memory:`, um keine Dateileichen zu hinterlassen.

### Speichern und Laden

```rust
pub async fn speichern(&self, konversation: &Konversation) -> Result<i64, PersistenzFehler> {
    let json = serde_json::to_string(konversation)?;

    let ergebnis = sqlx::query("INSERT INTO konversationen (inhalt) VALUES (?)")
        .bind(json)
        .execute(&self.pool)
        .await?;

    Ok(ergebnis.last_insert_rowid())
}

pub async fn laden(&self, id: i64) -> Result<Konversation, PersistenzFehler> {
    use sqlx::Row;

    let zeile = sqlx::query("SELECT inhalt FROM konversationen WHERE id = ?")
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

    let json: String = zeile.try_get("inhalt")?;
    Ok(serde_json::from_str(&json)?)
}
```

`.bind(wert)` setzt einen `?`-Platzhalter in der SQL-Anweisung sicher — **niemals**
Nutzereingaben direkt in einen SQL-String einfügen (Stichwort *SQL-Injection*, ein
klassisches Sicherheitsproblem: Würdest du z. B. `format!("... WHERE id = {id}")`
schreiben und `id` stammt aus Nutzereingabe, könnte jemand darüber beliebige SQL-Befehle
einschleusen). `.bind(...)` verhindert das strukturell, indem Wert und Anweisung getrennt
bleiben. `zeile.try_get("inhalt")` liest die Spalte `inhalt` typisiert aus der
Ergebniszeile — auch das kann fehlschlagen (Spalte fehlt, falscher Typ), daher wieder ein
`Result`, wieder über `#[from] sqlx::Error` automatisch mit `?` nutzbar.

### Ein bewusst provozierter Fehler: `.await` vergessen

```rust
pub async fn speichern(&self, konversation: &Konversation) -> Result<i64, PersistenzFehler> {
    let json = serde_json::to_string(konversation)?;

    let ergebnis = sqlx::query("INSERT INTO konversationen (inhalt) VALUES (?)")
        .bind(json)
        .execute(&self.pool); // .await fehlt!

    Ok(ergebnis.last_insert_rowid())
}
```

Der Compiler meldet (sinngemäß, der genaue Typname ist lang und hier gekürzt):

```
error[E0599]: no method named `last_insert_rowid` found for opaque type
              `impl Future<Output = Result<SqliteQueryResult, sqlx::Error>>`
              in the current scope
```

`.execute(&self.pool)` **ohne** `.await` gibt nicht das Ergebnis zurück, sondern ein
`Future` — ein "Versprechen", dass irgendwann ein Ergebnis vorliegen wird, sobald es
tatsächlich ausgeführt (*gepollt*) wird. `ergebnis` ist an dieser Stelle also gar kein
`SqliteQueryResult`, sondern dieses Versprechen — und ein Versprechen hat keine Methode
`last_insert_rowid()`. Der Compiler zeigt dir das exakt so: "diese Methode existiert
nicht auf diesem (opaken, also nicht direkt benennbaren) Future-Typ". Korrektur: `.await`
ergänzen, dann liegt wirklich das `Result` vor.

## Schritt-Reveal

**Schritt 1** — Abhängigkeiten ergänzen. `mein_core/Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "...", features = ["sqlite", "runtime-tokio"] }

[dev-dependencies]
tokio = { version = "...", features = ["rt-multi-thread", "macros"] }
```

Nutze `cargo add sqlx --features sqlite,runtime-tokio` bzw. `cargo add tokio --dev
--features rt-multi-thread,macros` für die jeweils aktuellen, korrekten Feature-Namen —
sqlx benennt Runtime-Features gelegentlich um, prüfe im Zweifel die Crate-Dokumentation
auf docs.rs.

**Schritt 2** — Modul `mein_core/src/persistence.rs` anlegen, `pub mod persistence;` in
`lib.rs`. `PersistenzFehler` anlegen.

**Schritt 3** — `Konversation` um `Serialize, Deserialize` in der `derive`-Liste
ergänzen (siehe oben).

**Schritt 4** — `KonversationsSpeicher` mit `neu`, `speichern`, `laden` anlegen.

**Schritt 5** — Provoziere den fehlenden-`.await`-Fehler bewusst, korrigiere ihn.

## Ausführung

```bash
cargo test -p mein_core
```

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rolle;

    #[tokio::test]
    async fn konversation_uebersteht_speichern_und_laden() {
        let speicher = KonversationsSpeicher::neu("sqlite::memory:").await.unwrap();

        let mut konversation = Konversation::neu();
        konversation.hinzufuegen(Rolle::Benutzer, "Hallo!").unwrap();

        let id = speicher.speichern(&konversation).await.unwrap();
        let geladen = speicher.laden(id).await.unwrap();

        assert_eq!(geladen.verlauf().len(), 1);
        assert_eq!(geladen.verlauf()[0].inhalt, "Hallo!");
    }
}
```

```
running 1 test
test persistence::tests::konversation_uebersteht_speichern_und_laden ... ok
```

`#[tokio::test]` ersetzt hier `#[test]` — es startet vor dem eigentlichen Test eine
Tokio-Laufzeit, damit `.await` innerhalb der (jetzt `async fn`) Testfunktion überhaupt
funktioniert. Auch das: ein Muster zum Abschreiben, vollständig erklärt in
[Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md).

## Zusammenfassung

- `mein_core::persistence` skizziert die Speichergrenze — ein `KonversationsSpeicher`
  mit `speichern`/`laden`, ohne vollständiges Migrations-Setup.
- `sqlx` ist asynchron; wir nutzen `async`/`.await`/`#[tokio::test]` hier bewusst als
  Vorgriff, vollständig erklärt erst in Phase 4.
- `Konversation` bekommt jetzt `Serialize`/`Deserialize`, um komplett als JSON-Text
  gespeichert zu werden — bewusst unnormalisiert, um die Skizze einfach zu halten.
- `.bind(...)` statt String-Verkettung verhindert SQL-Injection strukturell.
- Ein vergessenes `.await` zeigt sich als "Methode nicht gefunden auf `impl Future`" —
  ein typischer erster Rust-Async-Stolperstein, den du jetzt schon kennst, bevor
  Phase 4 async formal einführt.

## Übung

Ergänze `KonversationsSpeicher` um eine Methode `pub async fn alle_ids(&self) ->
Result<Vec<i64>, PersistenzFehler>`, die alle gespeicherten `id`-Werte zurückgibt (nutze
`sqlx::query("SELECT id FROM konversationen").fetch_all(&self.pool).await?` und
extrahiere aus jeder Zeile die Spalte `id` per `try_get`). Schreibe einen Test, der zwei
Konversationen speichert und prüft, dass `alle_ids()` beide IDs enthält. Überlege dir
zum Schluss (ohne es umzusetzen): Was müsste sich ändern, wenn `mein_cli` diesen Speicher
tatsächlich nutzen wollte — `mein_cli` ist bisher komplett synchron (kein `async fn
main`). Notiere dir diese offene Frage; wir lösen sie in
[Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md).

[Weiter: Lektion 8 · Release 2 — typed-provider-boundary](08-release-2.md)
