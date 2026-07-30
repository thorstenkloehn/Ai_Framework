# Lektion 4: Retriever und Quellenangaben

## Problem

`Embedder` und `VectorStore` aus [Lektion 3](03-embeddings-vector-store.md) sind zwei
separate Bausteine. Aufrufer-Code (später: `mein_agent` oder eine Axum-Route in
[Lektion 5](05-rest-axum-oder-tui.md)) will aber nicht bei jeder Anfrage selbst erst
`embedder.embed(...)` und dann `store.query(...)` verdrahten — das ist ein
Implementierungsdetail, kein Anliegen der aufrufenden Seite. Wir bündeln beides hinter
einem dritten Port: `Retriever`. Nimmt eine Textanfrage entgegen, liefert die
passendsten Chunks zurück.

Genauso wichtig: Ein Ergebnis ohne Herkunftsangabe ist für Nutzer:innen kaum zu
überprüfen. Wenn ein Assistent behauptet "Urlaub muss zwei Wochen vorher beantragt
werden", wollen wir sagen können, **woher** diese Aussage stammt — welches Dokument,
welcher Abschnitt. Das ist die Grundlage von Nachvollziehbarkeit, und es bereitet die
Sicherheitsbetrachtung in [Lektion 8](08-security-docker-ci.md) vor: Nur wenn wir wissen,
was aus dem Retrieval kommt und was nicht, können wir es später sauber als **nicht
vertrauenswürdige Daten** vom System- und Nutzeranteil des Prompts trennen.

## Code (Zielbild)

```rust
#[async_trait]
pub trait Retriever: Send + Sync {
    async fn retrieve(&self, anfrage: &str, top_k: usize) -> Result<Vec<RetrievedChunk>, RagFehler>;
}
```

```rust
let retriever = SimpleRetriever::neu(embedder, store);
let treffer = retriever.retrieve("Wie beantrage ich Urlaub?", 3).await?;
for t in &treffer {
    println!("[{}] (score {:.2}) {}", t.source, t.score, t.content);
}
```

## Dekonstruktion

### `RetrievedChunk` — Inhalt plus Herkunft plus Score

```rust
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub content: String,
    pub source: String,
    pub score: f32,
}
```

`source` übernehmen wir aus `Chunk::document_id`, die wiederum in Lektion 1 aus dem
Dateinamen stammt. Damit trägt jeder Treffer seine Herkunft von Anfang bis Ende der
Pipeline mit sich — nichts geht auf dem Weg verloren.

### `SimpleRetriever<E, V>` — Komposition statt Vererbung

```rust
pub struct SimpleRetriever<E: Embedder, V: VectorStore> {
    embedder: E,
    store: V,
}
```

Rust kennt keine Vererbung zwischen Structs. Stattdessen **hält** `SimpleRetriever`
einen `Embedder` und einen `VectorStore` als Felder und **implementiert** `Retriever`,
indem es beide intern nacheinander aufruft. Die generischen Parameter `E: Embedder, V:
VectorStore` (statt `Box<dyn Embedder>`, `Box<dyn VectorStore>`) sind hier bewusst
statisch gebunden: Für die eine feste Kombination "unser Embedder + unser Store", die wir
beim Erzeugen eines `SimpleRetriever` einmal festlegen, spart das die kleine zusätzliche
Laufzeitindirektion eines `dyn Trait`-Aufrufs. Nach außen — dort, wo wir mehrere
`Retriever`-Implementierungen austauschbar halten wollen (Lektion 5) — verwenden wir
weiterhin `dyn Retriever`.

### `Send + Sync` zahlt sich aus

Erinnerst du dich an die `Send + Sync`-Supertraits, die wir bei `DocumentLoader` in
Lektion 1 "auf Vorrat" ergänzt haben? Jetzt zeigt sich, warum. Angenommen, wir hätten sie
vergessen:

```rust
#[async_trait]
pub trait Retriever {
    async fn retrieve(&self, anfrage: &str) -> String;
}
```

Und wir wollten einen `Arc<dyn Retriever>` — wie es später in Lektion 5 als geteilter
Anwendungszustand nötig ist — in eine nebenläufige Tokio-Aufgabe hineinreichen:

```rust
pub async fn benutze(r: Arc<dyn Retriever>) {
    tokio::spawn(async move {
        let _ = r.retrieve("hallo").await;
    });
}
```

`cargo check`:

```
error: future cannot be sent between threads safely
   --> src/lib.rs:19:5
    |
 19 | /     tokio::spawn(async move {
 20 | |         let _ = r.retrieve("hallo").await;
 21 | |     });
    | |______^ future created by async block is not `Send`
    |
    = help: the trait `Send` is not implemented for `dyn Retriever`
note: required by a bound in `tokio::spawn`
    |
    |         F: Future + Send + 'static,
    |                     ^^^^ required by this bound in `spawn`
```

`tokio::spawn` verlangt, dass die übergebene Aufgabe zwischen Threads wandern darf
(`Send`) — schließlich weiß der Scheduler nicht im Voraus, auf welchem Worker-Thread sie
läuft. Weil unser `r: Arc<dyn Retriever>` in die Aufgabe hineinwandert, muss auch `dyn
Retriever` selbst `Send` (und für gemeinsamen Zugriff über `Arc` zusätzlich `Sync`) sein.
Die Reparatur ist eine einzige Zeile:

```rust
pub trait Retriever: Send + Sync {
    async fn retrieve(&self, anfrage: &str) -> String;
}
```

`cargo check` — kompiliert. Das ist derselbe Mechanismus, der uns in
[Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md) beim
`LlmProvider`-Port schon begegnet ist: Supertraits an einer Systemgrenze sind meist
billiger, früh ergänzt zu werden, als spät nachgerüstet.

## Schritt-Reveal

**Schritt 1 — `RetrievedChunk` und `Retriever` in `mein_rag/src/retriever.rs`:**

```rust
use crate::{Embedder, RagFehler, VectorStore};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub content: String,
    pub source: String,
    pub score: f32,
}

#[async_trait]
pub trait Retriever: Send + Sync {
    async fn retrieve(&self, anfrage: &str, top_k: usize) -> Result<Vec<RetrievedChunk>, RagFehler>;
}
```

**Schritt 2 — `SimpleRetriever` implementieren:**

```rust
pub struct SimpleRetriever<E: Embedder, V: VectorStore> {
    embedder: E,
    store: V,
}

impl<E: Embedder, V: VectorStore> SimpleRetriever<E, V> {
    pub fn neu(embedder: E, store: V) -> Self {
        SimpleRetriever { embedder, store }
    }
}

#[async_trait]
impl<E: Embedder, V: VectorStore> Retriever for SimpleRetriever<E, V> {
    async fn retrieve(&self, anfrage: &str, top_k: usize) -> Result<Vec<RetrievedChunk>, RagFehler> {
        let anfrage_vektor = self.embedder.embed(anfrage).await?;
        let treffer = self.store.query(&anfrage_vektor, top_k).await?;

        Ok(treffer
            .into_iter()
            .map(|t| RetrievedChunk {
                content: t.chunk.content,
                source: t.chunk.document_id,
                score: t.score,
            })
            .collect())
    }
}
```

Beachte die beiden `?`-Operatoren: Sowohl `embed` als auch `query` können fehlschlagen
(z. B. wenn ein echter Embedder-Adapter einen Netzwerkfehler meldet) — `RagFehler`
deckt beide Fälle bereits ab (Lektion 1).

## Ausführung

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, HashEmbedder, InMemoryVectorStore};

    #[tokio::test]
    async fn retriever_liefert_relevanten_chunk_zuerst() {
        let embedder = HashEmbedder { dimensionen: 32 };
        let store = InMemoryVectorStore::neu();

        let chunk_a = Chunk {
            document_id: "katzen.txt".to_string(),
            index: 0,
            content: "Katzen mögen Fisch".to_string(),
        };
        let chunk_b = Chunk {
            document_id: "autos.txt".to_string(),
            index: 0,
            content: "Autos brauchen Benzin".to_string(),
        };

        store.upsert(chunk_a.clone(), embedder.embed(&chunk_a.content).await.unwrap()).await.unwrap();
        store.upsert(chunk_b.clone(), embedder.embed(&chunk_b.content).await.unwrap()).await.unwrap();

        let retriever = SimpleRetriever::neu(embedder, store);
        let ergebnisse = retriever.retrieve("Was mögen Katzen?", 1).await.unwrap();

        assert_eq!(ergebnisse[0].source, "katzen.txt");
    }
}
```

```bash
cargo test -p mein_rag
```

```
running 1 test
test retriever::tests::retriever_liefert_relevanten_chunk_zuerst ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

> **⚠️ Warnung**
>
> Ein `Retriever` liefert Textinhalte, die **aus fremden Dokumenten** stammen — nicht aus
> deinem eigenen Prompt-Design. Ab hier gilt eine Regel, die wir in
> [Lektion 8](08-security-docker-ci.md) konkret umsetzen: Der Inhalt eines
> `RetrievedChunk` ist **untrusted data** (nicht vertrauenswürdige Daten). Er darf beim
> Zusammenbau eines Prompts niemals so behandelt werden, als käme er von dir als
> Entwickler:in — sonst kann ein präpariertes Dokument eigene "Anweisungen" einschleusen.

## Zusammenfassung

- `Retriever` bündelt `Embedder` und `VectorStore` hinter einer einzigen, einfachen
  Schnittstelle: Text rein, bewertete, quellenannotierte Chunks raus.
- `SimpleRetriever<E, V>` nutzt statische Generics für die eine feste
  Embedder/Store-Kombination; nach außen bleibt `dyn Retriever` austauschbar.
- `Send + Sync` als Supertrait ist keine Formsache — ohne sie verweigert `tokio::spawn`
  den Dienst, mit einer präzisen Fehlermeldung, die genau das benennt.
- Jeder `RetrievedChunk` trägt seine Quelle (`source`) mit sich — die Grundlage für
  nachvollziehbare Antworten und für die Sicherheitsbetrachtung in Lektion 8.

## Übung

Erweitere `RetrievedChunk` um eine Methode `zitat(&self) -> String`, die eine
menschenlesbare Quellenangabe erzeugt, z. B. `"[Quelle: urlaub.txt, Chunk 2, Score
0.87]"`. Dafür brauchst du den `index` aus `Chunk` — erweitere `RetrievedChunk`
entsprechend um ein `chunk_index: usize`-Feld und passe `SimpleRetriever::retrieve` an.
Schreibe einen Test, der das erwartete Zitat-Format prüft.

[Weiter: Lektion 5 — REST mit Axum oder TUI](05-rest-axum-oder-tui.md)
