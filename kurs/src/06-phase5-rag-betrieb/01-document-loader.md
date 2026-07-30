# Lektion 1: Document Loader

## Problem

Wir wollen unserem Framework beibringen, aus eigenen Dokumenten zu antworten — zunächst
aus einfachen Textdateien, später vielleicht aus Markdown, PDFs oder einer Webseite. Jede
dieser Quellen hat ein anderes Format, aber alles, was danach passiert (Zerlegen in
Chunks in [Lektion 2](02-chunking.md), Einbetten in Vektoren in
[Lektion 3](03-embeddings-vector-store.md)), soll **nicht** wissen müssen, ob ein Stück
Text ursprünglich aus einer `.txt`-Datei oder einem Webcrawler kam. Wir brauchen also
zwei Dinge: ein einheitliches Dokumentmodell und einen austauschbaren Weg, es zu füllen.

Das ist dieselbe Idee wie `LlmProvider` als Port in
[Phase 3](../04-phase3-architektur/01-llmprovider-port.md): Innen bleibt alles gleich,
außen tauschen wir die Anbindung.

## Code (Zielbild)

```rust
#[async_trait]
pub trait DocumentLoader: Send + Sync {
    async fn load(&self) -> Result<Vec<Document>, RagFehler>;
}
```

```rust
let loader = DateisystemLoader::neu("./wissensbasis");
let dokumente = loader.load().await?;
```

## Dekonstruktion

### Ein neues Crate: `mein_rag`

Bisher hatten wir `mein_core` (Domäne, Ports), `mein_cli`, `mein_agent`. RAG-spezifischer
Code — Laden, Chunking, Embeddings, Retrieval — bekommt ein eigenes Crate `mein_rag`, aus
demselben Grund wie `mein_agent` in Phase 4: Es hat eigene Abhängigkeiten (später z. B.
einen Vector-Store-Client), die nicht jedes andere Crate mitschleppen muss. `mein_rag`
hängt von `mein_core` ab (für gemeinsame Fehlerkonventionen), aber nicht umgekehrt —
Abhängigkeiten zeigen weiterhin nach innen.

```bash
cargo new --lib mein_rag
```

Trage `mein_rag` in die `members`-Liste der Workspace-`Cargo.toml` im Projekt-Root ein,
genau wie du es für `mein_agent` in Phase 4 schon gemacht hast.

### `Document` — das gemeinsame Dokumentmodell

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}
```

`Document` ist bewusst **englisch** benannt — anders als `Nachricht` oder `Konversation`
ist das kein Begriff, den wir Nicht-Programmierer:innen "eindeutschen" müssen; er ist in
jeder RAG-Bibliothek (egal in welcher Sprache) exakt dieser Fachbegriff, und er gehört
unmittelbar zu `DocumentLoader`, das laut `roadmap.md` schon englisch feststeht — ein
`DocumentLoader`, der ein `Dokument` zurückgibt, würde unnötig zwei Sprachen im selben
Typ mischen (Regel, die wir schon in [Phase 1](../02-phase1-fundament/README.md)
festgelegt haben und aus Konsistenzgründen weiter befolgen).

`metadata: HashMap<String, String>` statt fester Felder wie `quelle: String,
erstellt_am: String, ...`: Wir wissen heute noch nicht, welche Zusatzinformation eine
künftige Quelle mitbringt (Dateiname, URL, Autor, Seitenzahl bei PDFs). Eine `HashMap`
hält die Tür offen, ohne dass wir `Document` bei jeder neuen Quelle anfassen müssen. Der
Preis: Zugriffe wie `metadata.get("source")` liefern `Option<&String>` statt eines
garantierten Felds — ein bewusster Kompromiss zwischen Flexibilität und Typsicherheit.

### `DocumentLoader` als `async_trait`

```rust
#[async_trait]
pub trait DocumentLoader: Send + Sync {
    async fn load(&self) -> Result<Vec<Document>, RagFehler>;
}
```

Warum `async`? Dokumente zu laden bedeutet meist Datei- oder Netzwerk-I/O — genau die
Art von Wartezeit, die wir seit
[Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md) mit `tokio` nicht
blockierend behandeln. Warum `#[async_trait]` und nicht einfach `async fn` direkt im
Trait? Rust erlaubt `async fn` in Traits inzwischen nativ, aber ein Trait mit `async fn`
ist **nicht objektsicher** (*object safe* — der Compiler kann kein `dyn Trait` daraus
bauen, weil er nicht weiß, wie groß der von der Methode erzeugte Future-Typ ist). Wir
wollen aber später verschiedene Loader (Dateisystem, künftig vielleicht eine Webseite)
hinter demselben `Box<dyn DocumentLoader>` austauschen können — genau wie `LlmProvider`
in Phase 3 hinter `dyn LlmProvider` steckt. Das Crate `async-trait` löst das, indem es die
Methode zur Kompilierzeit so umschreibt, dass sie einen boxten Future zurückgibt.

`Send + Sync` als Supertraits: Ein `DocumentLoader`, den wir später aus einer Axum-Route
heraus aufrufen (Lektion 5), muss zwischen Threads wandern dürfen — Voraussetzung dafür
ist `Send`, für gemeinsame Referenzen zusätzlich `Sync`. Wir ergänzen das jetzt schon,
weil ein nachträgliches Hinzufügen an einem bereits verbreiteten Trait alle Implementierer
zwingt, ihre Typen zu prüfen.

### `RagFehler` — ein Fehlertyp für das ganze Crate

```rust
#[derive(Debug, thiserror::Error)]
pub enum RagFehler {
    #[error("Dokument konnte nicht gelesen werden: {0}")]
    Laden(#[from] std::io::Error),
    #[error("Chunking fehlgeschlagen: {0}")]
    Chunking(String),
    #[error("Embedding fehlgeschlagen: {0}")]
    Embedding(String),
    #[error("Vector-Store-Fehler: {0}")]
    VectorStore(String),
}
```

`thiserror` kennst du bereits aus
[Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md). Wir legen den
Fehlertyp schon jetzt vollständig an (auch die Varianten für Lektion 2–4), damit jede
Lektion dieselbe Fehler-API benutzt, statt sie schrittweise zu erweitern.

> **💡 Tipp**
>
> `#[from] std::io::Error` erlaubt `?` direkt auf `tokio::fs`-Aufrufen — der Compiler
> wandelt einen `std::io::Error` automatisch in `RagFehler::Laden` um, ganz ohne
> `.map_err(...)`.

## Schritt-Reveal

**Schritt 1 — Crate anlegen und Abhängigkeiten ergänzen.**

```bash
cargo new --lib mein_rag
cd mein_rag
cargo add async-trait thiserror
cargo add serde --features derive
cargo add tokio --features rt,rt-multi-thread,macros,fs,sync,time
cargo add mein_core --path ../mein_core
```

`cargo add` löst dabei jeweils die aktuelle stabile Version auf — wir schreiben hier
bewusst keine Versionsnummer vor, die morgen schon veraltet wäre.

**Schritt 2 — `Document`, `RagFehler` und `DocumentLoader` in `src/lib.rs` bzw.
`src/loader.rs` anlegen** (Modulaufteilung wie im Zielbild oben: `loader.rs`,
`chunking.rs`, `embedding.rs`, `retriever.rs`, jeweils über `pub mod` in `lib.rs`
eingebunden). Tippe zunächst das Trait **ohne** `#[async_trait]`, wie es intuitiv
naheliegt:

```rust
pub trait DocumentLoader {
    async fn load(&self) -> Result<Vec<Document>, RagFehler>;
}
```

Nutze es probeweise dyn-basiert:

```rust
pub struct Sammlung {
    pub loaders: Vec<Box<dyn DocumentLoader>>,
}
```

`cargo check -p mein_rag`:

```
error[E0038]: the trait `DocumentLoader` is not dyn compatible
  --> src/lib.rs:23:26
   |
23 |     pub loaders: Vec<Box<dyn DocumentLoader>>,
   |                          ^^^^^^^^^^^^^^^^^^ `DocumentLoader` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
  --> src/lib.rs:19:14
   |
18 | pub trait DocumentLoader {
   |           -------------- this trait is not dyn compatible...
19 |     async fn load(&self) -> Result<Vec<Document>, RagFehler>;
   |              ^^^^ ...because method `load` is `async`
   = help: consider moving `load` to another trait
```

Genau das Problem, das oben in der Dekonstruktion beschrieben ist: Der Compiler kann
keine Vtable (Sprungtabelle für `dyn Trait`-Aufrufe) für eine Methode bauen, deren
Rückgabetyp (ein anonymer Future) unbekannte Größe hat.

**Schritt 3 — Mit `async_trait` reparieren.**

```rust
use async_trait::async_trait;

#[async_trait]
pub trait DocumentLoader: Send + Sync {
    async fn load(&self) -> Result<Vec<Document>, RagFehler>;
}
```

`cargo check -p mein_rag` — kompiliert jetzt sauber, auch mit
`Vec<Box<dyn DocumentLoader>>`.

**Schritt 4 — `DateisystemLoader` implementieren.** Er liest alle `.txt`-Dateien aus
einem Verzeichnis:

```rust
pub struct DateisystemLoader {
    pub verzeichnis: PathBuf,
}

impl DateisystemLoader {
    pub fn neu(verzeichnis: impl Into<PathBuf>) -> Self {
        DateisystemLoader { verzeichnis: verzeichnis.into() }
    }
}

#[async_trait]
impl DocumentLoader for DateisystemLoader {
    async fn load(&self) -> Result<Vec<Document>, RagFehler> {
        let mut dokumente = Vec::new();
        let mut eintraege = tokio::fs::read_dir(&self.verzeichnis).await?;

        while let Some(eintrag) = eintraege.next_entry().await? {
            let pfad = eintrag.path();
            if pfad.extension().and_then(|e| e.to_str()) != Some("txt") {
                continue;
            }
            let content = tokio::fs::read_to_string(&pfad).await?;
            let dateiname = pfad
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unbekannt")
                .to_string();

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), dateiname.clone());

            dokumente.push(Document { id: dateiname, content, metadata });
        }

        Ok(dokumente)
    }
}
```

Beachte `metadata.insert("source", dateiname)`: Der Dateiname wandert als Quellenangabe
mit — sie taucht in [Lektion 4](04-retriever-quellenangaben.md) als Zitat wieder auf.

## Ausführung

Schreibe den Test aus dem Zielbild dieser Lektion — er legt in einem temporären
Verzeichnis zwei `.txt`-Dateien und eine `.md`-Datei an und prüft, dass nur die
`.txt`-Dateien geladen werden:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn laedt_alle_txt_dateien_aus_einem_verzeichnis() {
        let verzeichnis = std::env::temp_dir().join("mein_rag_test_fixtures");
        tokio::fs::create_dir_all(&verzeichnis).await.unwrap();
        tokio::fs::write(verzeichnis.join("a.txt"), "Inhalt A").await.unwrap();
        tokio::fs::write(verzeichnis.join("b.txt"), "Inhalt B").await.unwrap();
        tokio::fs::write(verzeichnis.join("c.md"), "Wird ignoriert").await.unwrap();

        let loader = DateisystemLoader::neu(&verzeichnis);
        let mut dokumente = loader.load().await.unwrap();
        dokumente.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(dokumente.len(), 2);
        assert_eq!(dokumente[0].content, "Inhalt A");

        tokio::fs::remove_dir_all(&verzeichnis).await.unwrap();
    }
}
```

```bash
cargo test -p mein_rag
```

```
running 1 test
test loader::tests::laedt_alle_txt_dateien_aus_einem_verzeichnis ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Provoziere den Fehlerpfad bewusst: Zeige testweise auf ein Verzeichnis, das nicht
existiert (`DateisystemLoader::neu("/pfad/den/es/nicht/gibt")`) und rufe `.load().await`
auf — du bekommst `Err(RagFehler::Laden(...))` statt eines Absturzes, weil `?` den
`std::io::Error` sauber durchreicht.

> **⚠️ Warnung**
>
> `tokio::fs::read_dir` liest nur die **oberste Ebene** eines Verzeichnisses, nicht
> rekursiv Unterordner. Für eine Wissensbasis mit Unterordnern (z. B. nach Thema
> sortiert) müsstest du selbst rekursiv absteigen — das ist die Übung dieser Lektion.

## Zusammenfassung

- `Document` ist ein einheitliches Modell für Inhalte aus beliebigen Quellen —
  `metadata: HashMap<String, String>` hält die Struktur erweiterbar.
- `DocumentLoader` ist ein Port im Sinne von Phase 3: austauschbar, ohne dass
  nachgelagerter Code (Chunking, Embedding) sich ändert.
- `async fn` in einem Trait, das `dyn`-fähig sein soll, braucht `#[async_trait]` — der
  Compiler sagt das über `E0038` sehr präzise.
- `Send + Sync` als Supertraits sind eine Vorleistung für Lektion 5, in der Loader aus
  einer Axum-Route heraus aufgerufen werden.
- `RagFehler` bündelt alle Fehlerfälle des neuen Crates von Anfang an.

## Übung

Erweitere `DateisystemLoader` (oder schreibe einen zweiten Loader,
`RekursiverDateisystemLoader`) so, dass er auch `.txt`-Dateien in Unterverzeichnissen
findet. Nutze dafür entweder eine selbstgeschriebene rekursive Hilfsfunktion oder das
Crate `walkdir` (`cargo add walkdir`). Schreibe einen Test mit einer verschachtelten
Verzeichnisstruktur (mindestens eine Ebene tiefer als in diesem Kapitel).

[Weiter: Lektion 2 — Chunking](02-chunking.md)
