# Lektion 6: Golden Set und LLM-as-Judge

## Problem

Der `FakeProvider` aus [Lektion 4](04-fake-provider.md) ist perfekt für alles, was
**deterministisch** ist: Kontrollfluss, Fehlerbehandlung, Timeouts. Aber irgendwann willst
du wissen, ob dein Framework mit einem **echten** LLM tatsächlich brauchbare Antworten
bekommt — und da beginnt ein neues Problem. Ein LLM ist nicht deterministisch: Dieselbe
Frage kann heute und morgen unterschiedlich formuliert beantwortet werden. `assert_eq!(antwort,
"Paris ist die Hauptstadt von Frankreich.")` würde bei der kleinsten Umformulierung
fehlschlagen, obwohl die Antwort inhaltlich richtig ist. Wie testet man also etwas, dessen
korrekte Ausgabe nicht als exakter String feststeht?

Zwei verbreitete Antworten aus der Praxis: ein **Golden Set** (eine feste Sammlung von
Testfällen mit erwarteten *Eigenschaften*, nicht erwarteten *exakten Texten*) und
**LLM-as-Judge** (ein zweites LLM bewertet die Antwort des ersten). Beide sind nützlich —
und beide haben Grenzen, die du kennen musst, bevor du dich darauf verlässt.

## Code (Zielbild)

```rust
pub struct GoldenFall {
    pub eingabe: &'static str,
    pub erwartete_eigenschaften: Vec<Eigenschaft>,
}

pub enum Eigenschaft {
    EnthaeltText(&'static str),
    MindestensLaenge(usize),
    HoechstensLaenge(usize),
    EnthaeltNicht(&'static str),
}

pub fn pruefe(antwort: &str, eigenschaften: &[Eigenschaft]) -> Vec<String> {
    let mut verstoesse = Vec::new();
    for eigenschaft in eigenschaften {
        if let Some(verstoss) = pruefe_eine(antwort, eigenschaft) {
            verstoesse.push(verstoss);
        }
    }
    verstoesse
}
```

## Dekonstruktion

### Ein Golden Set ist keine exakte Antwort, sondern eine Menge von Eigenschaften

Ein **Golden Set** ist eine feste, von Menschen kuratierte Sammlung von Testfällen — jeder
mit einer Eingabe und einer Reihe von *Eigenschaften*, die eine gute Antwort haben sollte,
statt eines einzigen exakten Erwartungswerts. Für die Frage "Was ist die Hauptstadt von
Frankreich?" wäre eine sinnvolle Eigenschaft `EnthaeltText("Paris")` — egal, ob die Antwort
ein vollständiger Satz, eine Stichliste oder eine andere Formulierung ist, solange "Paris"
vorkommt, gilt der Testfall als bestanden. Das ist ein bewusster Kompromiss: Wir verzichten
auf hundertprozentige Präzision (die Antwort könnte "Paris" enthalten und trotzdem falsch
sein, z. B. "Paris ist NICHT die Hauptstadt von Frankreich") zugunsten von Tests, die
überhaupt automatisierbar sind.

### Die `Eigenschaft`-`enum` — geschlossen, aber erweiterbar

```rust
pub enum Eigenschaft {
    EnthaeltText(&'static str),
    MindestensLaenge(usize),
    HoechstensLaenge(usize),
    EnthaeltNicht(&'static str),
}
```

Dieselbe Designentscheidung wie bei `Rolle` in
[Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md): eine feste Menge
bekannter Prüfarten statt eines offenen, freien Textformats. `EnthaeltNicht` ist dabei
genauso wichtig wie `EnthaeltText` — ein Golden-Set-Fall kann prüfen, dass eine Antwort auf
eine sensible Frage **kein** bestimmtes Wort enthält (z. B. eine persönliche Meinung, wo
Neutralität gefordert ist), nicht nur, dass sie etwas Bestimmtes enthält.

### `pruefe` — ein Prüfprogramm, das gegen jeden `LlmProvider` funktioniert

```rust
pub fn pruefe(antwort: &str, eigenschaften: &[Eigenschaft]) -> Vec<String> {
```

Der Rückgabetyp ist bewusst `Vec<String>` (eine Liste der **Verstöße**), nicht `bool`. Ein
einzelnes `bool` sagt dir nur "bestanden oder nicht", eine Liste von Verstößen sagt dir
*welche* Eigenschaften verletzt wurden — entscheidend, um bei einem fehlgeschlagenen
Golden-Set-Lauf schnell zu verstehen, was konkret schiefging, statt bei jedem Fehlschlag von
vorne zu debuggen.

Der eigentliche Clou dieser Lektion ist, was hier **fehlt**: `pruefe` nimmt einen fertigen
`&str` entgegen, keinen `LlmProvider`. Das Golden Set selbst weiß nichts von Anbietern — es
prüft nur Text gegen Eigenschaften. Das eigentliche "Frage stellen" passiert separat, über
genau das `LlmProvider`-Trait aus [Lektion 1](01-llmprovider-port.md).

## Schritt-Reveal

**Schritt 1 — `pruefe_eine` implementieren.** `pruefe` ruft für jede `Eigenschaft` eine
kleine Hilfsfunktion auf, die eine einzelne Prüfung durchführt und bei Verstoß eine
Beschreibung zurückgibt. Tippe sie in `mein_core/src/eval.rs`, bevor du weiterliest:

```rust
fn pruefe_eine(antwort: &str, eigenschaft: &Eigenschaft) -> Option<String> {
    match eigenschaft {
        Eigenschaft::EnthaeltText(text) if !antwort.contains(text) => {
            Some(format!("erwartet Text '{text}', nicht gefunden"))
        }
        Eigenschaft::EnthaeltNicht(text) if antwort.contains(text) => {
            Some(format!("sollte '{text}' nicht enthalten, aber tut es"))
        }
        Eigenschaft::MindestensLaenge(min) if antwort.chars().count() < *min => {
            Some(format!("kürzer als erwartete Mindestlänge {min}"))
        }
        Eigenschaft::HoechstensLaenge(max) if antwort.chars().count() > *max => {
            Some(format!("länger als erlaubte Höchstlänge {max}"))
        }
        _ => None,
    }
}
```

Der `match` mit Guard-Bedingungen (`if !antwort.contains(text)`) prüft nur den
Verstoßfall — trifft keine Bedingung zu, liefert der `_`-Zweig `None` ("kein Verstoß").
`cargo check -p mein_core` sollte jetzt sauber durchlaufen.

**Schritt 2 — Golden-Set-Runner.** Eine Funktion, die ein ganzes Golden Set gegen einen
echten oder gefälschten Provider laufen lässt:

```rust
pub fn fuehre_golden_set_aus(
    provider: &dyn LlmProvider,
    faelle: &[GoldenFall],
) -> Vec<(String, Vec<String>)> {
    let mut ergebnisse = Vec::new();
    for fall in faelle {
        let anfrage = ChatAnfrage {
            nachrichten: vec![Nachricht::neu(Rolle::Benutzer, fall.eingabe).unwrap()],
            modell: "irgendein-modell".into(),
        };
        let antwort = provider.chat(anfrage).unwrap();
        let verstoesse = pruefe(&antwort.inhalt, &fall.erwartete_eigenschaften);
        if !verstoesse.is_empty() {
            ergebnisse.push((fall.eingabe.to_string(), verstoesse));
        }
    }
    ergebnisse
}
```

Weil diese Funktion nur gegen `&dyn LlmProvider` programmiert ist (siehe
[Lektion 3](03-dyn-trait-ownership.md)), funktioniert **dasselbe** Prüfprogramm sowohl mit
einem `FakeProvider` (schnelle, deterministische Tests deiner Golden-Set-Logik selbst) als
auch mit dem echten `OpenAiKompatiblerClient` (der eigentliche, aussagekräftige Lauf gegen
ein echtes Modell). Das ist der Lohn der ganzen bisherigen Phase, an einer neuen Stelle
angewendet.

**Schritt 3 — LLM-as-Judge als Erweiterung.** Manche Eigenschaften lassen sich nicht mit
einfachen Textregeln wie `EnthaeltText` prüfen — zum Beispiel "ist die Antwort höflich
formuliert?" oder "fasst die Antwort den Kerninhalt korrekt zusammen?". Ein verbreiteter
Ansatz: ein zweites LLM ("der Richter") bewertet die Antwort des ersten anhand eines
Bewertungs-Prompts:

```rust
pub struct RichterUrteil {
    pub bestanden: bool,
    pub begruendung: String,
}

pub fn richte_antwort(
    richter: &dyn LlmProvider,
    frage: &str,
    antwort: &str,
) -> Result<RichterUrteil, ProviderFehler> {
    let bewertungs_prompt = format!(
        "Beurteile, ob die folgende Antwort die Frage sinnvoll und korrekt beantwortet. \
         Antworte ausschließlich mit JA oder NEIN, gefolgt von einer kurzen Begründung.\n\
         Frage: {frage}\nAntwort: {antwort}"
    );
    let anfrage = ChatAnfrage {
        nachrichten: vec![Nachricht::neu(Rolle::Benutzer, bewertungs_prompt).unwrap()],
        modell: "irgendein-modell".into(),
    };
    let urteil = richter.chat(anfrage)?;
    Ok(RichterUrteil {
        bestanden: urteil.inhalt.trim_start().to_uppercase().starts_with("JA"),
        begruendung: urteil.inhalt,
    })
}
```

Das kompiliert und funktioniert — aber bevor du diesen Ansatz für bare Münze nimmst, drei
Grenzen, die du kennen musst:

- **Bias (Verzerrung).** Ein LLM-Richter bewertet tendenziell Antworten positiver, die im
  Stil dem eigenen Trainingsmaterial ähneln — ausführliche, gut strukturierte, selbstsichere
  Antworten schneiden oft besser ab als kurze, korrekte. Und wenn Richter und geprüftes
  Modell aus derselben Modellfamilie stammen, neigen manche Richter dazu, "ihre eigenen"
  Formulierungsmuster zu bevorzugen (*self-preference bias*) — ein bekanntes, in der
  Forschung dokumentiertes Phänomen.
- **Kosten.** Jede Golden-Set-Auswertung mit LLM-as-Judge kostet **zwei** API-Aufrufe statt
  einem (einen für die eigentliche Antwort, einen für die Bewertung) — bei größeren
  Golden Sets oder häufigen CI-Läufen ein nicht zu vernachlässigender Faktor.
- **Nicht-Determinismus im Quadrat.** Nicht nur die geprüfte Antwort ist nicht-deterministisch
  — das Urteil selbst ist es auch. Derselbe `richte_antwort`-Aufruf kann bei zwei Läufen zu
  unterschiedlichen Ergebnissen kommen, was das Debuggen eines "roten" Golden-Set-Laufs
  erschwert: Lag es an der geprüften Antwort, oder war nur der Richter diesmal launisch?

> **⚠️ Warnung**
>
> LLM-as-Judge ersetzt **nicht** die deterministischen Tests aus
> [Lektion 4](04-fake-provider.md) und [Lektion 5](05-tests-und-clippy.md) — es ergänzt sie
> um ein grobes, zusätzliches Signal für Fälle, die sich nicht in einfache Textregeln
> fassen lassen. Baue deine Test-Pyramide von unten nach oben: viele schnelle, deterministische
> `FakeProvider`-Tests, wenige Golden-Set-Läufe mit Eigenschafts-Prüfungen gegen ein echtes
> Modell, noch weniger (und mit Vorsicht interpretierte) LLM-as-Judge-Auswertungen.

> **💡 Tipp**
>
> Golden-Set-Läufe gegen ein echtes Modell kosten Geld und sind langsam — sie gehören
> deshalb meist **nicht** in den Standard-`cargo test`-Lauf. Markiere sie mit
> `#[ignore]` (ein echtes, eingebautes Rust-Attribut): `#[test] #[ignore] fn
> golden_set_laeuft_gegen_echtes_modell() { ... }`. Ein normaler `cargo test` überspringt
> sie dann, `cargo test -- --ignored` führt gezielt nur die ignorierten Tests aus — praktisch
> für einen separaten, selteneren CI-Job.

## Ausführung

```bash
cargo test -p mein_core
```

Ein Golden-Set-Test gegen den `FakeProvider` (Erfolgsfall der Prüflogik selbst) läuft ganz
normal mit. Ein echter Lauf gegen ein reales Modell:

```bash
cargo test -p mein_core -- --ignored
```

## Zusammenfassung

- Ein **Golden Set** prüft LLM-Antworten gegen *Eigenschaften* (enthält Text X, ist kürzer
  als Y Zeichen, enthält Wort Z nicht), nicht gegen exakte Strings — notwendig, weil
  LLM-Ausgaben nicht deterministisch sind.
- Golden-Set-Prüfcode, der nur gegen `&dyn LlmProvider` programmiert, funktioniert
  unverändert mit `FakeProvider` und einem echten Adapter.
- **LLM-as-Judge** (ein LLM bewertet ein anderes) ist nützlich für Eigenschaften, die sich
  nicht als einfache Textregel ausdrücken lassen — aber anfällig für Bias, teuer im
  Doppel-Aufruf, und selbst nicht-deterministisch.
- `#[ignore]` trennt teure, gegen echte Modelle laufende Tests vom schnellen
  Standard-Testlauf.

## Übung

Baue ein kleines Golden Set mit drei `GoldenFall`-Einträgen für eine Aufgabe deiner Wahl
(z. B. "Fasse einen Satz in maximal 10 Wörtern zusammen" oder "Beantworte eine
Ja/Nein-Frage"). Formuliere für jeden Fall mindestens zwei `Eigenschaft`-Prüfungen. Führe das
Golden Set einmal gegen einen `FakeProvider` mit einer bewusst *fehlerhaften* Antwort aus
und prüfe, dass `pruefe` die erwarteten Verstöße meldet — das ist ein Test des Testwerkzeugs
selbst, bevor du es je gegen ein echtes Modell laufen lässt. Überlege abschließend: Welche
deiner drei Eigenschaften könntest du **nicht** mit einer einfachen `Eigenschaft`-Variante
ausdrücken und würdest stattdessen einen LLM-Richter brauchen? Was würde dich zögern lassen,
diesem Urteil vollständig zu vertrauen?

[Weiter: Lektion 7 — Chain Pattern mit Runnable](07-chain-pattern-runnable.md)
