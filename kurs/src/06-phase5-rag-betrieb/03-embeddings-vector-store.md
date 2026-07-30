# Lektion 3: Embeddings und Vector Store

## Problem

Wir haben Chunks ([Lektion 2](02-chunking.md)) — aber wie finden wir bei der Anfrage "Wie
beantrage ich Urlaub?" den passenden Chunk, ohne stumpf nach den exakten Wörtern
"Urlaub" und "beantrage" zu suchen (eine Chunk mit "Ich nehme mir frei" träfe dann nicht,
obwohl sie inhaltlich passt)? Die Antwort in praktisch jedem modernen RAG-System:
**Embeddings** — ein Modell wandelt Text in einen Vektor (eine Liste von
Kommazahlen) um, sodass **bedeutungsähnlicher** Text auf **nahe beieinanderliegende**
Vektoren abgebildet wird. "Ich nehme mir frei" und "Wie beantrage ich Urlaub?" landen
dann nah beieinander, obwohl kein gemeinsames Wort vorkommt.

Diese Vektoren müssen wir irgendwo speichern und schnell nach den nächsten Nachbarn
durchsuchen können — das übernimmt ein **Vector Store**. Es gibt dafür spezialisierte
Datenbanken wie Qdrant oder LanceDB (beide in `roadmap.md` genannt), aber wir wollen uns
heute nicht an eine davon binden. Genau wie bei `LlmProvider` in Phase 3 bauen wir daher
zuerst den Port.

## Code (Zielbild)

```rust
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, RagFehler>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert(&self, chunk: Chunk, embedding: Vec<f32>) -> Result<(), RagFehler>;
    async fn query(&self, embedding: &[f32], top_k: usize) -> Result<Vec<ScoredChunk>, RagFehler>;
}
```

## Dekonstruktion

### Zwei Ports, eine Verantwortung pro Trait

Wir trennen bewusst **Embedder** (Text → Vektor) von **VectorStore** (Vektor speichern
und durchsuchen). Das folgt demselben Prinzip wie die Trennung von Request-/
Response-Typen in [Phase 2](../03-phase2-llm-anbindung/03-request-response-typen.md):
Jeder Baustein hat eine einzige Aufgabe. Ein produktives System könnte beide
Verantwortlichkeiten sogar bei verschiedenen Anbietern haben — Embeddings von OpenAI,
Speicherung in einer selbstgehosteten Qdrant-Instanz. Mit getrennten Traits ist das kein
Sonderfall, sondern der Normalfall.

### `Vec<f32>` als Vektor-Typ

Ein Embedding ist schlicht eine feste Anzahl Kommazahlen (die *Dimensionalität*, oft
mehrere hundert bis mehrere tausend). `f32` (32-Bit-Gleitkommazahl) statt `f64` ist der
in Embedding-Bibliotheken übliche Kompromiss: Die Genauigkeit von `f64` wird für
Ähnlichkeitsvergleiche nicht gebraucht, aber halb so viel Speicher pro Zahl zählt bei
tausenden Vektoren mit hunderten Dimensionen durchaus.

### `&self` statt `&mut self` bei `upsert`

Ein Vector Store, der hinter `Arc<dyn VectorStore>` von mehreren Orten gleichzeitig
genutzt wird (z. B. mehreren gleichzeitigen Anfragen in Lektion 5), kann keine exklusive
`&mut self`-Referenz herausgeben. Wir wählen deshalb `&self` für **beide** Methoden und
verlagern die notwendige Veränderlichkeit nach innen — *interior mutability*, ein Muster,
das du in [Phase 4, Lektion 5](../05-phase4-agenten/05-state-und-memory.md) mit
`Arc<tokio::sync::Mutex<AgentState>>` schon gesehen hast. Hier greifen wir zu
`tokio::sync::RwLock<Vec<...>>` statt `Mutex`: Ein `RwLock` erlaubt **beliebig viele
gleichzeitige Lesezugriffe** (mehrere `query`-Aufrufe parallel) und blockiert nur bei
einem Schreibzugriff (`upsert`) exklusiv — für einen Store, der viel häufiger gelesen als
beschrieben wird, ist das ein besserer Kompromiss als ein `Mutex`, das jeden Zugriff
(lesend oder schreibend) gleichermaßen exklusiv macht.

### Ein austauschbarer Embedder ohne API-Key: `HashEmbedder`

Für Übungszwecke wollen wir nicht bei jedem `cargo test` einen echten, kostenpflichtigen
Embedding-Aufruf machen (dasselbe Argument wie beim Fake-Provider aus
[Phase 3, Lektion 4](../04-phase3-architektur/04-fake-provider.md)). Wir bauen deshalb
einen deterministischen `HashEmbedder`: Er zählt für jedes (normalisierte) Wort einen
einfachen Hash-Wert in einen Vektor fester Länge. Das ist **kein** Ersatz für ein echtes
Embedding-Modell — semantisch verwandte, aber wortverschiedene Sätze werden damit nicht
zuverlässig erkannt —, reicht aber, um die komplette Pipeline (Chunking → Embedding →
Speichern → Suchen) offline zu testen.

> **⚠️ Warnung**
>
> Verwechsle `HashEmbedder` nicht mit einem echten Embedding-Modell. In einer echten
> Anwendung ersetzt du ihn durch einen Adapter, der z. B. die Embedding-API deines
> LLM-Anbieters aufruft — die Signatur des `Embedder`-Traits ändert sich dabei nicht, nur
> die Implementierung.

## Schritt-Reveal

**Schritt 1 — Traits und `ScoredChunk` anlegen** in `mein_rag/src/embedding.rs`:

```rust
use crate::{Chunk, RagFehler};
use async_trait::async_trait;

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, RagFehler>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert(&self, chunk: Chunk, embedding: Vec<f32>) -> Result<(), RagFehler>;
    async fn query(&self, embedding: &[f32], top_k: usize) -> Result<Vec<ScoredChunk>, RagFehler>;
}

#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub chunk: Chunk,
    pub score: f32,
}
```

**Schritt 2 — `HashEmbedder` implementieren:**

```rust
pub struct HashEmbedder {
    pub dimensionen: usize,
}

#[async_trait]
impl Embedder for HashEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, RagFehler> {
        let mut vektor = vec![0f32; self.dimensionen];
        for wort in text.split_whitespace() {
            let normalisiert: String = wort
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if normalisiert.is_empty() {
                continue;
            }
            let hash = einfacher_hash(&normalisiert);
            let index = (hash as usize) % self.dimensionen;
            vektor[index] += 1.0;
        }
        Ok(vektor)
    }
}

fn einfacher_hash(wort: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in wort.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}
```

Wir entfernen Satzzeichen und wandeln in Kleinbuchstaben um (`normalisiert`), bevor wir
hashen — sonst würden "Katzen" und "Katzen?" (mit Fragezeichen) unterschiedliche Hashes
erzeugen, obwohl es dasselbe Wort ist. `wrapping_mul`/`wrapping_add` statt `*`/`+`: Bei
sehr langen Wörtern könnte der Hash sonst überlaufen (*overflow*) und im Debug-Build mit
einem Panic abbrechen — `wrapping_*`-Operationen lassen den Wert stattdessen bewusst
"umlaufen", was für einen Hash unproblematisch ist.

**Schritt 3 — `InMemoryVectorStore` implementieren:**

```rust
use tokio::sync::RwLock;

pub struct InMemoryVectorStore {
    eintraege: RwLock<Vec<(Chunk, Vec<f32>)>>,
}

impl InMemoryVectorStore {
    pub fn neu() -> Self {
        InMemoryVectorStore { eintraege: RwLock::new(Vec::new()) }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(&self, chunk: Chunk, embedding: Vec<f32>) -> Result<(), RagFehler> {
        self.eintraege.write().await.push((chunk, embedding));
        Ok(())
    }

    async fn query(&self, embedding: &[f32], top_k: usize) -> Result<Vec<ScoredChunk>, RagFehler> {
        let eintraege = self.eintraege.read().await;
        let mut bewertet: Vec<ScoredChunk> = eintraege
            .iter()
            .map(|(chunk, vektor)| ScoredChunk {
                chunk: chunk.clone(),
                score: cosine_similarity(embedding, vektor),
            })
            .collect();

        bewertet.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        bewertet.truncate(top_k);
        Ok(bewertet)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
```

**Kosinus-Ähnlichkeit** (*cosine similarity*) misst den Winkel zwischen zwei Vektoren,
nicht ihre absolute Länge — zwei Vektoren, die in dieselbe Richtung zeigen, gelten als
ähnlich, egal wie lang sie sind. Das ist der Standard-Ähnlichkeitsmaßstab für Embeddings.
Der Wertebereich liegt zwischen `-1.0` (entgegengesetzt) und `1.0` (identische Richtung);
die Sonderbehandlung für `norm_a == 0.0` verhindert eine Division durch Null bei einem
Nullvektor (z. B. bei völlig leerem Text).

`InMemoryVectorStore` ist bewusst **kein** reiner Test-Adapter wie `adapter::fake` aus
Phase 3 (der nur unter `#[cfg(test)]` existiert) — er ist eine echte, wenn auch nicht
persistente, Implementierung, die für kleine Wissensbasen im Betrieb ausreicht.

## Ausführung

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &str, content: &str) -> Chunk {
        Chunk { document_id: id.to_string(), index: 0, content: content.to_string() }
    }

    #[tokio::test]
    async fn aehnlicher_text_bekommt_hoeheren_score() {
        let embedder = HashEmbedder { dimensionen: 32 };
        let store = InMemoryVectorStore::neu();

        let a = chunk("a", "Katzen mögen Fisch und schlafen viel");
        let b = chunk("b", "Autos brauchen Benzin und Öl");

        store.upsert(a.clone(), embedder.embed(&a.content).await.unwrap()).await.unwrap();
        store.upsert(b.clone(), embedder.embed(&b.content).await.unwrap()).await.unwrap();

        let anfrage = embedder.embed("Katzen und Fisch").await.unwrap();
        let ergebnisse = store.query(&anfrage, 2).await.unwrap();

        assert_eq!(ergebnisse[0].chunk.document_id, "a");
    }
}
```

```bash
cargo test -p mein_rag
```

```
running 1 test
test embedding::tests::aehnlicher_text_bekommt_hoeheren_score ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Zusammenfassung

- Embeddings bilden Text auf Vektoren ab, sodass Bedeutungsnähe zu räumlicher Nähe wird —
  damit funktioniert Suche über Formulierungsunterschiede hinweg.
- `Embedder` und `VectorStore` sind zwei getrennte Ports; ihre konkreten Implementierungen
  (`HashEmbedder`, `InMemoryVectorStore`) sind austauschbar, ohne Aufrufer-Code zu
  ändern — echte Backends wie Qdrant oder LanceDB ließen sich später als eigene Adapter
  ergänzen.
- `&self` statt `&mut self` plus *interior mutability* (`RwLock`) macht einen Store
  nebenläufig nutzbar, ohne den Trait mit Sperrlogik zu verunreinigen.
- Kosinus-Ähnlichkeit ist der Standardmaßstab, um Embedding-Vektoren zu vergleichen.
- Ein deterministischer Stand-in-Embedder erlaubt, die gesamte Pipeline offline und ohne
  API-Key zu testen — dasselbe Prinzip wie der Fake-Provider aus Phase 3.

## Übung

Ergänze `VectorStore::query` (oder eine neue Methode `query_mit_schwelle`) um einen
Mindest-Score: Treffer mit `score` unterhalb eines übergebenen Schwellenwerts
(`min_score: f32`) sollen gar nicht erst zurückgegeben werden, selbst wenn `top_k` noch
nicht erreicht ist. Überlege dir zuerst, warum ein reines "gib mir die besten `top_k`
Treffer" bei einer völlig themenfremden Anfrage ein Problem sein kann — und schreibe
danach einen Test, der genau diesen Fall abdeckt.

[Weiter: Lektion 4 — Retriever und Quellenangaben](04-retriever-quellenangaben.md)
