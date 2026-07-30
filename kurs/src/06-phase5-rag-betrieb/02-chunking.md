# Lektion 2: Chunking

## Problem

Ein `Document` aus [Lektion 1](01-document-loader.md) kann beliebig lang sein — ein
ganzes Handbuch als eine einzige `content: String`. Zwei Probleme entstehen daraus:
Erstens passt ein langes Dokument oft nicht in das Kontextfenster eines LLM (die maximale
Textmenge, die ein Modell auf einmal "sehen" kann). Zweitens verwässert ein sehr langes
Stück Text die spätere Ähnlichkeitssuche (Lektion 3/4) — wer nach "Wie beantrage ich
Urlaub?" sucht, will genau den Absatz über Urlaubsanträge finden, nicht das gesamte
Handbuch als "am ehesten passendes" Ergebnis. Wir zerlegen Dokumente deshalb in kleinere
**Chunks** (Textstücke).

Die Stellschraube dabei ist ein echter Zielkonflikt: **große Chunks** behalten mehr
Zusammenhang (ein Absatz bleibt vollständig), aber die Suche wird ungenauer. **Kleine
Chunks** treffen präziser, reißen aber Sätze aus ihrem Zusammenhang — ein Chunk, der nur
"Der Antrag muss mindestens zwei Wochen vorher gestellt werden." enthält, ist ohne die
vorangehende Überschrift ("Urlaubsantrag") schwerer einzuordnen.

## Code (Zielbild)

```rust
pub fn chunke_dokument(
    dokument: &Document,
    max_woerter: usize,
    ueberlappung: usize,
) -> Vec<Chunk>
```

```rust
let chunks = chunke_dokument(&dokument, 200, 40);
// chunks[0] und chunks[1] teilen sich die letzten 40 Wörter von chunks[0]
// mit den ersten 40 Wörtern von chunks[1] -- der "Klebstoff" gegen Kontextverlust
// an den Chunk-Grenzen.
```

## Dekonstruktion

### Wortbasiert statt zeichenbasiert

Wir zählen in **Wörtern**, nicht in Zeichen. Ein rein zeichenbasierter Schnitt
("nimm die ersten 500 Zeichen") reißt Wörter mitten durch — aus "Urlaubsantrag" würde
"Urlaubsant" + "rag" in zwei Chunks. `split_whitespace()` liefert uns Wortgrenzen
geschenkt; wir bauen Chunks danach wieder aus ganzen Wörtern zusammen.

> **💡 Tipp**
>
> In echten RAG-Systemen wird oft nach **Tokens** gezählt (die Einheit, in der ein LLM
> tatsächlich abrechnet und sein Kontextfenster misst — meist Teilwörter, siehe
> [Phase 2](../03-phase2-llm-anbindung/README.md)), nicht nach Wörtern. Wortbasiertes
> Chunking ist einfacher zu verstehen und für unser Framework ausreichend; ein
> Tokenizer-basierter Ersatz ließe sich später hinter derselben Funktionssignatur
> austauschen.

### Overlap — warum Chunks sich überlappen

Stell dir vor, du zerschneidest ein Foto in Kacheln für ein Puzzle, aber jede Kachel zeigt
einen kleinen Streifen der Nachbarkachel mit. Genau das macht `ueberlappung`: Die letzten
`ueberlappung` Wörter eines Chunks erscheinen auch am Anfang des nächsten. Ohne Overlap
könnte ein wichtiger Satz genau auf der Schnittkante liegen — die erste Hälfte landet in
Chunk 3, die zweite in Chunk 4, und keiner der beiden Chunks für sich ergibt beim
Retrieval noch vollen Sinn. Der Overlap kostet Speicherplatz (Text wird doppelt
gespeichert), kauft dafür Robustheit gegen genau diesen Fall.

### Warum `Vec<Chunk>` mit eigenem Typ statt `Vec<String>`?

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub document_id: String,
    pub index: usize,
    pub content: String,
}
```

Ein nackter `String` würde die Herkunft verlieren. `document_id` verbindet den Chunk
zurück zu seinem `Document` (und damit zu dessen `metadata`, inklusive der Quellenangabe
aus Lektion 1); `index` hält die Reihenfolge fest — beides brauchen wir für die
Quellenangaben in [Lektion 4](04-retriever-quellenangaben.md).

## Schritt-Reveal

**Schritt 1 — Naiver erster Versuch mit geliehenen Strings.** Es liegt nahe, Chunks
zunächst als Textausschnitte (`&str`) *in* das Originaldokument zu modellieren, um keine
Kopien anzulegen:

```rust
pub fn chunke_woerter(woerter: &[&str], max_woerter: usize) -> Vec<&str> {
    let mut chunks = Vec::new();
    for gruppe in woerter.chunks(max_woerter) {
        let text = gruppe.join(" ");
        chunks.push(text.as_str());
    }
    chunks
}
```

`cargo check -p mein_rag`:

```
error[E0106]: missing lifetime specifier
 --> src/chunking.rs:1:68
  |
1 | pub fn chunke_woerter(woerter: &[&str], max_woerter: usize) -> Vec<&str> {
  |                                -------                             ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does
    not say which one of `woerter`'s 2 lifetimes it is borrowed from
```

Der Compiler will wissen, **wie lange** die zurückgegebenen `&str` gültig bleiben — dazu
später mehr in [Kapitel 0](../01-grundlagen/05-der-compiler-als-lehrer.md), falls dir
Lifetimes noch nicht begegnet sind: Eine Lifetime ist keine Laufzeitgröße, sondern eine
Zusicherung an den Compiler, wie lange eine Referenz höchstens gültig ist.

**Schritt 2 — Lifetime ergänzen, wie vom Compiler vorgeschlagen.**

```rust
pub fn chunke_woerter<'a>(woerter: &'a [&'a str], max_woerter: usize) -> Vec<&'a str> {
    let mut chunks = Vec::new();
    for gruppe in woerter.chunks(max_woerter) {
        let text = gruppe.join(" ");
        chunks.push(text.as_str());
    }
    chunks
}
```

`cargo check -p mein_rag` — ein **neuer**, tieferliegender Fehler:

```
error[E0515]: cannot return value referencing local variable `text`
 --> src/chunking.rs:5:9
  |
4 |         chunks.push(text.as_str());
  |                     ---- `text` is borrowed here
5 |     }
6 |     chunks
  |     ^^^^^^ returns a value referencing data owned by the current function
```

Das ist der eigentliche Lehrmoment: `text` (das Ergebnis von `gruppe.join(" ")`) ist ein
**neuer, eigener** `String` — er existiert nur innerhalb dieser Schleifeniteration und
wird danach verworfen (*dropped*). `text.as_str()` leiht sich etwas, das gleich nicht
mehr da ist. Keine noch so geschickte Lifetime-Annotation kann das reparieren, weil das
Problem kein Notationsproblem ist, sondern ein Designfehler: Wir wollten geliehene Daten
zurückgeben, obwohl wir gerade neue, eigene Daten erzeugt haben.

**Schritt 3 — Design korrigieren: eigene `String`s statt geliehener `&str`.**

```rust
pub fn chunke_woerter(woerter: &[&str], max_woerter: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    for gruppe in woerter.chunks(max_woerter) {
        chunks.push(gruppe.join(" "));
    }
    chunks
}
```

`cargo check -p mein_rag` — kompiliert. Kein Lifetime-Parameter mehr nötig, weil
`Vec<String>` seine Daten selbst besitzt (*owned*) statt sie zu leihen.

**Schritt 4 — Die eigentliche `chunke_dokument`-Funktion mit Overlap.**

```rust
pub fn chunke_dokument(
    dokument: &Document,
    max_woerter: usize,
    ueberlappung: usize,
) -> Vec<Chunk> {
    assert!(
        ueberlappung < max_woerter,
        "Überlappung muss kleiner als die Chunk-Größe sein"
    );

    let woerter: Vec<&str> = dokument.content.split_whitespace().collect();
    if woerter.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let schritt = max_woerter - ueberlappung;

    while start < woerter.len() {
        let ende = (start + max_woerter).min(woerter.len());
        let content = woerter[start..ende].join(" ");

        chunks.push(Chunk { document_id: dokument.id.clone(), index, content });
        index += 1;

        if ende == woerter.len() {
            break;
        }
        start += schritt;
    }

    chunks
}
```

`schritt = max_woerter - ueberlappung` ist der Kern des Overlaps: Ohne Overlap
(`ueberlappung == 0`) wäre `schritt == max_woerter`, und jeder Chunk beginnt genau dort,
wo der vorige endete. Mit Overlap rückt das Fenster weniger weit vor, als es breit ist —
die letzten `ueberlappung` Wörter des vorigen Chunks tauchen erneut auf. Das `assert!` am
Anfang verhindert eine Endlosschleife: Wäre `ueberlappung >= max_woerter`, würde
`schritt` null oder negativ (bei `usize` sogar zu einem Overflow-Panic führen) — wir
brechen lieber sofort mit einer sprechenden Meldung ab, statt das Programm hängen zu
lassen.

## Ausführung

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn testdokument(content: &str) -> Document {
        Document { id: "doc-1".to_string(), content: content.to_string(), metadata: HashMap::new() }
    }

    #[test]
    fn zerlegt_langen_text_in_mehrere_chunks() {
        let doc = testdokument(&"wort ".repeat(200));
        let chunks = chunke_dokument(&doc, 50, 10);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.content.split_whitespace().count() <= 50);
        }
    }

    #[test]
    fn kurzer_text_ergibt_genau_einen_chunk() {
        let doc = testdokument("Kurzer Satz hier.");
        let chunks = chunke_dokument(&doc, 50, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Kurzer Satz hier.");
    }
}
```

```bash
cargo test -p mein_rag
```

```
running 2 tests
test chunking::tests::kurzer_text_ergibt_genau_einen_chunk ... ok
test chunking::tests::zerlegt_langen_text_in_mehrere_chunks ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Provoziere den Fehlerpfad bewusst: Rufe `chunke_dokument(&doc, 10, 10)` auf (Overlap
gleich Chunk-Größe) — das `assert!` bricht mit `panicked at ... Überlappung muss kleiner
als die Chunk-Größe sein` ab, statt in eine Endlosschleife zu laufen.

## Zusammenfassung

- Chunking zerlegt lange Dokumente in kleinere, durchsuchbare Stücke — der Kompromiss ist
  Zusammenhang gegen Suchgranularität.
- Wortbasierte Zerlegung vermeidet abgeschnittene Wörter; produktive Systeme zählen oft
  in Tokens statt Wörtern.
- Overlap lässt benachbarte Chunks sich teilweise überschneiden, damit Sätze an
  Chunk-Grenzen nicht ihren Kontext verlieren.
- Ein naiver Versuch mit geliehenen `&str` führt über zwei verschiedene Compilerfehler
  (`E0106`, dann `E0515`) zur Erkenntnis: Neu erzeugte Daten müssen als eigene Werte
  (`String`, nicht `&str`) zurückgegeben werden.
- `assert!` schützt vor einer Konfiguration (`ueberlappung >= max_woerter`), die das
  Programm in eine Endlosschleife oder einen Overflow laufen ließe.

## Übung

Die aktuelle Funktion zerschneidet mitten in Sätzen, wenn die Wortgrenze zufällig genau
in einem Satz liegt. Schreibe eine Variante `chunke_dokument_absatzweise`, die zuerst am
doppelten Zeilenumbruch (`\n\n`, typische Absatztrennung) aufteilt und **nur** Absätze,
die für sich genommen länger als `max_woerter` sind, mit der bestehenden Logik weiter
zerlegt. Kurze Absätze bleiben unverändert als eigener Chunk erhalten, auch wenn sie
deutlich kürzer als `max_woerter` sind. Überlege dir zuerst an einem Beispieltext, wie
viele Chunks du danach erwartest, bevor du den Test schreibst.

[Weiter: Lektion 3 — Embeddings und Vector Store](03-embeddings-vector-store.md)
