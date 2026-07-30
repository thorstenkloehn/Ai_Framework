# Glossar

Alphabetische Liste aller Fachbegriffe, die im Kurs vorkommen. Domänenbegriffe wie
`Rolle`, `Nachricht` oder `Konversation` sind deutsch benannt, weil der echte
Ai_Framework-Code das so vormacht. Architektur- und Rust-Fachbegriffe (`Trait`, `Port`,
`Retriever` ...) bleiben englisch, weil sie in dieser Form in jeder Rust- und KI-Literatur
wiederzufinden sind. Jeder Eintrag verweist auf die Lektion, in der der Begriff zuerst
eingeführt wird — dort steht die ausführliche Erklärung mit Codebeispiel.

---

### Adapter

In der Hexagonal Architecture die konkrete Implementierung eines `Port`, also die Anbindung
an ein reales Außensystem — zum Beispiel ein Adapter, der das `LlmProvider`-Trait für einen
bestimmten API-Anbieter implementiert. Mehrere Adapter können denselben Port erfüllen, ohne
dass der Domänencode etwas davon merkt. Ein Test-Adapter (`fake`) ersetzt im Unit-Test das
echte Netzwerk.

→ [Phase 3, Lektion 2](../04-phase3-architektur/02-hexagonal-architecture.md)

### Agent

Eine Komponente, die selbstständig entscheidet, welche `Tool`-Aufrufe nötig sind, um ein
Ziel zu erreichen, und die Ergebnisse in ihre nächste Entscheidung einbezieht — im
Unterschied zu einem einzelnen, direkten LLM-Aufruf. Der Agent im Kurs lebt im eigenen
Crate `mein_agent` und kapselt Zustand, Werkzeuge und Abbruchbedingungen.

→ [Phase 4, Lektion 4](../05-phase4-agenten/04-agent-loop.md)

### Agent Loop

Die Kernschleife eines Agenten: Modell befragen, Antwort auf Werkzeugaufrufe prüfen,
Werkzeug ausführen, Ergebnis zurück in die Konversation einspeisen, von vorn — bis das
Modell eine finale Antwort liefert oder eine Abbruchbedingung greift. Der Agent Loop ist
das Herzstück von Phase 4.

→ [Phase 4, Lektion 4](../05-phase4-agenten/04-agent-loop.md)

### anyhow

Eine Rust-Bibliothek für pragmatische Fehlerbehandlung in Anwendungscode (im Gegensatz zu
Bibliothekscode): Statt für jeden möglichen Fehler einen eigenen Typ zu pflegen, bündelt
`anyhow::Error` beliebige Fehler mit Kontextinformation. Im Kurs nutzt `mein_cli`
`anyhow`, während `mein_core` eigene Fehlertypen mit `thiserror` definiert.

→ [Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md)

### async/await

Rusts Syntax für asynchronen, nicht-blockierenden Code: Eine `async fn` gibt sofort ein
`Future` zurück, statt zu warten; `.await` an einer Aufrufstelle übergibt die Kontrolle so
lange an den Executor (z. B. Tokio), bis das `Future` fertig ist, ohne den ganzen Thread zu
blockieren. Für ein KI-Framework zentral, weil Netzwerkaufrufe an LLM-APIs Sekunden dauern
können und währenddessen niemand den Prozess blockieren soll.

→ [Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md)

### Axum

Ein Rust-Webframework, das auf `tokio` aufbaut und im Kurs als REST-Schnittstelle für
`mein_server` dient — die Alternative bzw. Ergänzung zur reinen Kommandozeilen-Anwendung
`mein_cli`, sobald das Framework als Dienst betrieben werden soll.

→ [Phase 5, Lektion 5](../06-phase5-rag-betrieb/05-rest-axum-oder-tui.md)

### Backoff

Eine Strategie, nach einem fehlgeschlagenen Aufruf (z. B. wegen Rate Limit) nicht sofort,
sondern nach einer ansteigenden Wartezeit erneut zu versuchen — typischerweise
exponentiell (1s, 2s, 4s, ...), um ein bereits überlastetes System nicht weiter zu
belasten. Backoff wird fast immer zusammen mit `Retry` implementiert.

→ [Phase 5, Lektion 6](../06-phase5-rag-betrieb/06-retry-rate-limit-backoff.md)

### Benchmark (criterion)

Eine wiederholte, statistisch ausgewertete Zeitmessung eines Codeausschnitts, um
Performance-Regressionen sichtbar zu machen. `criterion` ist das im Kurs verwendete
Rust-Crate für Benchmarks; sie leben in einem eigenen `benches/`-Ordner, getrennt von
regulären Tests.

→ [Phase 6, Lektion 1](../07-phase6-performance/01-benchmarks-criterion.md)

### Borrowing

Rusts Mechanismus, einen Wert **auszuleihen**, statt ihn zu übernehmen (siehe `Ownership`):
Eine Referenz (`&T` für lesenden, `&mut T` für veränderenden Zugriff) gewährt temporären
Zugriff, ohne den Besitz zu übertragen. Zu jedem Zeitpunkt gilt: entweder beliebig viele
lesende Referenzen oder genau eine veränderende — nie beides gleichzeitig. Der
Borrow-Checker erzwingt diese Regel beim Kompilieren, nicht zur Laufzeit.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### Builder Pattern

Ein Entwurfsmuster, das die Konstruktion eines komplexen Werts über eine Kette von
Methodenaufrufen erlaubt (`Config::builder().modell("...").temperatur(0.7).build()`), statt
über einen einzigen, unübersichtlichen Konstruktor mit vielen Parametern. Im Kurs kommt das
Muster in Phase 7 zum Einsatz, wenn `mein_core` eine stabile, öffentliche API bekommt.

→ [Phase 7, Lektion 1](../08-phase7-release/01-builder-pattern.md)

### Chain Pattern (Runnable)

Ein Architekturmuster (bekannt aus LangChain), bei dem Verarbeitungsschritte als
einheitliche, verkettbare Bausteine modelliert werden — jeder implementiert ein
`Runnable`-Trait und kann mit anderen zu einer Pipeline zusammengesetzt werden, ohne dass
die Schritte sich gegenseitig kennen müssen.

→ [Phase 3, Lektion 7](../04-phase3-architektur/07-chain-pattern-runnable.md)

### Chunking

Das Zerlegen langer Dokumente in kleinere, in sich sinnvolle Textabschnitte (*Chunks*),
bevor sie eingebettet und in einem Vector Store abgelegt werden. Die Chunk-Größe und
-Überlappung beeinflussen direkt, wie gut ein `Retriever` später relevante Abschnitte
findet.

→ [Phase 5, Lektion 2](../06-phase5-rag-betrieb/02-chunking.md)

### CI/CD

*Continuous Integration / Continuous Deployment* — automatisierte Pipelines, die bei jedem
Push Tests, Formatierung (`cargo fmt --check`) und Linting (`cargo clippy`) laufen lassen
und optional automatisch ausliefern. Im Kurs wird eine einfache CI-Pipeline für das
Ai_Framework eingerichtet.

→ [Phase 5, Lektion 8](../06-phase5-rag-betrieb/08-security-docker-ci.md)

### clap

Das im Kurs verwendete Rust-Crate für Kommandozeilen-Parsing. Über `#[derive(Parser)]` und
`#[derive(Subcommand)]` wird aus einer Typdefinition automatisch ein vollständiges
CLI-Programm mit Validierung, `--help`-Ausgabe und Fehlermeldungen bei Tippfehlern.

→ [Phase 1, Lektion 6](../02-phase1-fundament/06-cli-mit-clap.md)

### Closure

Eine anonyme Funktion, die Variablen aus ihrer Umgebung "einfangen" kann (`|x| x + eins`).
Closures sind zentral für Muster, bei denen Verhalten als Wert übergeben wird — etwa
einzelne Schritte einer `Runnable`-Kette oder Callback-artiger Code.

→ [Phase 3, Lektion 7](../04-phase3-architektur/07-chain-pattern-runnable.md)

### Crate

Die Grundeinheit kompilierbaren Rust-Codes: entweder eine Bibliothek (*library*, erkennbar
an `src/lib.rs`) oder ein ausführbares Programm (*binary*, `src/main.rs`). Das
Ai_Framework-Repository ist ein Workspace aus mehreren Crates wie `mein_core` und
`mein_cli`.

→ [Phase 1, Lektion 1](../02-phase1-fundament/01-workspace-lesen.md)

### crates.io

Das offizielle, öffentliche Registry für Rust-Crates — vergleichbar mit npm für
JavaScript oder PyPI für Python. `cargo publish` veröffentlicht ein Crate dorthin; `cargo
add <crate>` lädt von dort herunter.

→ [Phase 7, Lektion 5](../08-phase7-release/05-crates-io-checkliste.md)

### Dependency Injection

Ein Entwurfsprinzip, bei dem eine Komponente ihre Abhängigkeiten (z. B. einen konkreten
`LlmProvider`) von außen übergeben bekommt, statt sie selbst zu erzeugen. Das macht die
Komponente testbar: In Tests wird statt des echten Adapters ein Fake-Adapter injiziert.

→ [Phase 3, Lektion 4](../04-phase3-architektur/04-fake-provider.md)

### derive-Attribut

Ein Compiler-Attribut der Form `#[derive(Debug, Clone, ...)]`, das für einen Typ
automatisch eine Standardimplementierung bestimmter Traits generiert, ohne dass diese von
Hand geschrieben werden müssen. `derive` ist der mit Abstand am häufigsten genutzte
Attributtyp im gesamten Kurs.

→ [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)

### Docker

Ein Werkzeug, um eine Anwendung samt ihrer Laufzeitumgebung in einem portablen, isolierten
"Container" zu verpacken. Im Kurs bekommt `mein_server` ein `Dockerfile`, damit der
RAG-Dienst reproduzierbar auf beliebigen Maschinen laufen kann.

→ [Phase 5, Lektion 8](../06-phase5-rag-betrieb/08-security-docker-ci.md)

### Domain Type

Ein Typ, der ein Konzept aus der fachlichen Domäne modelliert — im Kurs zum Beispiel
`Rolle`, `Nachricht` und `Konversation` — und der so entworfen ist, dass ungültige
fachliche Zustände nach Möglichkeit gar nicht erst darstellbar sind.

→ [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)

### dyn Trait

Eine Schreibweise für dynamischen Polymorphismus zur Laufzeit: `Box<dyn LlmProvider>` hält
einen Wert eines beliebigen Typs, der `LlmProvider` implementiert, ohne dass der konkrete
Typ zur Kompilierzeit bekannt sein muss. Der Preis dafür ist ein kleiner Laufzeit-Overhead
(dynamischer Funktionsaufruf über eine sogenannte vtable) gegenüber statischem
Polymorphismus (`impl Trait`/Generics).

→ [Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md)

### Edition

Rusts Mechanismus, Sprachänderungen einzuführen, ohne bestehenden Code zu brechen: Jedes
Crate wählt in seiner `Cargo.toml` eine Edition (z. B. `edition = "2024"`), die festlegt,
welche Sprachvariante gilt. Crates unterschiedlicher Editionen können im selben Workspace
zusammenarbeiten.

→ [Phase 1, Lektion 1](../02-phase1-fundament/01-workspace-lesen.md)

### Embedding

Die numerische Darstellung eines Textabschnitts als Vektor reeller Zahlen, so berechnet,
dass inhaltlich ähnliche Texte auch im Vektorraum nahe beieinanderliegen. Embeddings sind
die Grundlage jeder semantischen Ähnlichkeitssuche in einem `VectorStore`.

→ [Phase 5, Lektion 3](../06-phase5-rag-betrieb/03-embeddings-vector-store.md)

### Enum

Kurzform für *enumeration*: ein Typ, dessen Wert genau eine von mehreren fest benannten
Möglichkeiten ist, nie mehrere gleichzeitig. Im Kurs das zentrale Werkzeug, um ungültige
Zustände (wie eine unbekannte `Rolle`) zur Kompilierzeit auszuschließen.

→ [Kapitel 0](../01-grundlagen/04-daten-buendeln.md)

### Feature Flag

Ein Eintrag in `Cargo.toml` unter `[features]`, der optionale Funktionalität eines Crates
ein- oder ausschaltbar macht (z. B. `rag = [...]`, `agent = [...]`), ohne dass Nutzer*innen,
die diese Funktionalität nicht brauchen, unnötige Abhängigkeiten mitkompilieren müssen.

→ [Phase 7, Lektion 2](../08-phase7-release/02-feature-flags.md)

### Fehler-Weiterreicher (?-Operator)

Das `?`-Zeichen direkt nach einem Ausdruck, der ein `Result` (oder `Option`) liefert:
Ist der Wert `Ok`/`Some`, wird er entpackt und die Ausführung geht weiter; ist er
`Err`/`None`, wird sofort aus der aktuellen Funktion mit genau diesem Fehlerwert
zurückgekehrt. Eine der am häufigsten genutzten Kurzschreibweisen in echtem Rust-Code.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### Function Calling (Tool Use)

Die Fähigkeit eines LLM, statt einer reinen Textantwort strukturiert anzuzeigen, dass es
eine bestimmte Funktion mit bestimmten Parametern aufrufen möchte — die eigentliche
Ausführung übernimmt die Anwendung, nicht das Modell. Grundlage jedes werkzeugnutzenden
Agenten.

→ [Phase 4, Lektion 3](../05-phase4-agenten/03-tool-schema-function-calling.md)

### Future

Ein Rust-Wert, der eine noch nicht abgeschlossene asynchrone Berechnung repräsentiert.
Ein `Future` tut von sich aus nichts — erst ein Executor (wie Tokio), der es per `.await`
oder `spawn` antreibt, bringt es zur Fertigstellung.

→ [Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md)

### Generics

Rusts Mechanismus, Typen und Funktionen über einen Platzhalter-Typparameter (z. B. `T` in
`Vec<T>`) für viele konkrete Typen wiederverwendbar zu machen, ohne Laufzeit-Overhead —
der Compiler erzeugt für jede genutzte konkrete Instanz spezialisierten Code.

→ [Kapitel 0](../01-grundlagen/04-daten-buendeln.md)

### Golden Set

Eine feste, kuratierte Sammlung von Testfällen (Eingabe plus erwartetes bzw. akzeptables
Ergebnis), gegen die die Qualität von LLM-Antworten regelmäßig geprüft wird — die
KI-Entsprechung einer Testsuite, bei der die "richtige" Antwort nicht immer exakt, sondern
oft nur bewertbar ist.

→ [Phase 3, Lektion 6](../04-phase3-architektur/06-golden-set-llm-judge.md)

### Hexagonal Architecture

Ein Architekturmuster (auch *Ports and Adapters* genannt), das die Domänenlogik strikt von
technischen Details (Netzwerk, Datenbank, CLI) trennt: Die Domäne definiert `Port`-Traits,
die von austauschbaren `Adapter`n implementiert werden. Das Fundament dafür wird schon in
Phase 1 mit der Trennung `mein_core`/`mein_cli` gelegt, explizit benannt und vertieft wird
es in Phase 3.

→ [Phase 3, Lektion 2](../04-phase3-architektur/02-hexagonal-architecture.md)

### impl Trait

Eine Schreibweise für statischen Polymorphismus: `impl Into<String>` als Parametertyp
bedeutet "irgendein konkreter Typ, der `Into<String>` implementiert" — welcher Typ das
konkret ist, entscheidet der Compiler pro Aufrufstelle, ohne Laufzeit-Overhead (im
Unterschied zu `dyn Trait`).

→ [Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md)

### Invariante

Eine Regel, die für jeden gültigen Wert eines Typs immer gelten muss — im Kurs zum
Beispiel "eine `Nachricht` hat nie leeren Inhalt". Invarianten werden entweder durch die
Typstruktur selbst unmöglich gemacht (z. B. `enum` statt `String`) oder zur
Konstruktionszeit geprüft und mit `Result` durchgesetzt.

→ [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md)

### Iterator

Ein Wert, der eine Folge von Elementen nacheinander liefert, etwa über eine `for`-Schleife
(`for nachricht in konversation.verlauf()`) oder Iterator-Methoden wie `.map()`/`.filter()`.
Iteratoren sind in Rust "lazy": Sie berechnen ein Element erst, wenn es tatsächlich
gebraucht wird.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### Kapselung

Das Prinzip, den internen Zustand eines Typs (z. B. das private Feld `verlauf` in
`Konversation`) vor direktem Zugriff von außen zu schützen und stattdessen nur über
geprüfte, öffentliche Methoden zu erlauben. So bleiben Invarianten über die gesamte
Lebensdauer eines Werts garantiert, nicht nur bei der Konstruktion.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### Konfiguration

Ein serde-basiertes Struct, das Laufzeiteinstellungen wie API-Key, Modellname und
Temperatur aus einer Datei (JSON/TOML) statt aus hartkodiertem Quellcode lädt. Bewusst
englisch als `Config`/`Konfiguration`-Mischform benannt, weil der Typ mit externen
JSON-Feldern und späteren Web-Framework-Strukturen interagiert.

→ [Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md)

### Konversation

Der Domain Type, der einen Gesprächsverlauf als geordnete Folge von `Nachricht`-Werten
kapselt (`Vec<Nachricht>` intern, aber privat) und nur über geprüfte Methoden wie
`hinzufuegen` verändert werden kann.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### Lifetime

Rusts Mechanismus, dem Compiler mitzuteilen, wie lange eine Referenz gültig bleiben muss,
damit sie nie auf bereits freigegebenen Speicher zeigt. Lifetimes werden meist automatisch
abgeleitet; explizite Lifetime-Annotationen (`'a`) werden nötig, sobald Referenzen über
Funktions- oder Trait-Grenzen hinweg gehalten werden — etwa bei `dyn Trait`-Objekten.

→ [Phase 3, Lektion 3](../04-phase3-architektur/03-dyn-trait-ownership.md)

### LLM (Large Language Model)

Ein Sprachmodell, das auf riesigen Textmengen trainiert wurde und darauf spezialisiert
ist, auf einen Eingabetext (`Prompt`) einen plausiblen Fortsetzungstext zu erzeugen. Das
gesamte Ai_Framework ist letztlich eine typisierte Rust-Schicht um Anfragen an ein solches
Modell.

→ [Phase 2, Lektion 1](../03-phase2-llm-anbindung/01-http-grenze-reqwest.md)

### LLM-as-Judge

Ein Muster, bei dem ein (meist stärkeres) LLM eingesetzt wird, um die Antwortqualität eines
anderen LLM-Aufrufs automatisiert zu bewerten — als Ergänzung zu exakten Tests, gerade dort,
wo es keine einzige "richtige" Textantwort gibt.

→ [Phase 3, Lektion 6](../04-phase3-architektur/06-golden-set-llm-judge.md)

### Macro

Rust-Code, der zur Kompilierzeit anderen Code erzeugt, erkennbar am `!` (`println!`,
`vec!`, `assert_eq!`). Makros sind mächtiger als Funktionen, weil sie mit Code selbst statt
nur mit Werten arbeiten, werden im Kurs aber überwiegend genutzt (nicht selbst geschrieben).

→ [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md)

### MCP (Model Context Protocol)

Ein offenes Protokoll, über das ein Agent standardisiert auf externe Werkzeuge und
Datenquellen zugreifen kann, ohne für jede Quelle eine eigene Integration zu schreiben. Im
Kurs als optionaler MCP-Client in Phase 4 behandelt.

→ [Phase 4, Lektion 7](../05-phase4-agenten/07-mcp-client.md)

### Model Routing

Die Entscheidung, welche konkrete Modellvariante (z. B. ein schnelles, günstiges Modell
oder ein langsameres, leistungsfähigeres) für eine gegebene Anfrage genutzt wird — samt
Fallback-Logik, falls das bevorzugte Modell nicht erreichbar ist oder ein Limit
überschreitet.

→ [Phase 6, Lektion 3](../07-phase6-performance/03-model-routing-fallback.md)

### Modul

Rusts Werkzeug, Code innerhalb eines Crates in benannte, hierarchisch organisierte Einheiten
zu gliedern (z. B. `mein_core::domain`, `mein_core::port`), unabhängig von
Sichtbarkeitsregeln, die pro Modul über `pub` gesteuert werden.

→ [Phase 3, Lektion 2](../04-phase3-architektur/02-hexagonal-architecture.md)

### Multi-Agent-Orchestrierung

Die Koordination mehrerer spezialisierter Agenten, die gemeinsam an einer Aufgabe
arbeiten (z. B. ein planender und ein ausführender Agent), inklusive der Frage, wie
Zwischenergebnisse zwischen ihnen weitergereicht werden.

→ [Phase 6, Lektion 4](../07-phase6-performance/04-multi-agent-orchestrierung.md)

### Nachricht

Der zentrale Domain Type des Kurses: ein Struct mit den Feldern `rolle: Rolle` und
`inhalt: String`, das einen einzelnen Beitrag in einer `Konversation` repräsentiert.

→ [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)

### Option

Ein eingebautes Enum (`Some(T)` / `None`), das Rusts Antwort auf `null` ist: Der Compiler
zwingt dazu, den Fall "kein Wert vorhanden" explizit zu behandeln, bevor auf den
eigentlichen Wert zugegriffen werden kann.

→ [Kapitel 0](../01-grundlagen/04-daten-buendeln.md)

### Ownership

Rusts Kernprinzip: Jeder Wert im Speicher hat genau einen Besitzer; wird der Besitzer aus
dem Gültigkeitsbereich entfernt, wird der Speicher automatisch freigegeben. Wert-Übergaben
(z. B. an eine Funktion) übertragen standardmäßig den Besitz (*move*), sofern der Typ nicht
`Copy` ist. Ownership ersetzt einen Garbage Collector durch Regeln, die der Compiler beim
Kompilieren prüft.

→ [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)

### Panic

Der sofortige, nicht abgefangene Abbruch eines Rust-Programms (`panic!`), reserviert für
Zustände, die laut Programmlogik nie eintreten dürfen. Für erwartbare Fehlerfälle (z. B.
eine ungültige Nutzereingabe) wird stattdessen `Result` verwendet.

→ [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md)

### Pattern Matching

Der `match`-Ausdruck (und verwandte Kurzformen wie `if let`) prüft einen Wert gegen mehrere
mögliche Formen und muss bei einem `enum` **alle** Möglichkeiten abdecken — der Compiler
verweigert sonst die Kompilierung. Das macht vergessene Fälle (z. B. eine neue `Rolle`) zu
einem Compilerfehler statt zu einem stillen Laufzeit-Bug.

→ [Kapitel 0](../01-grundlagen/03-kontrollfluss.md)

### Port

In der Hexagonal Architecture ein Trait, das aus Sicht der Domäne beschreibt, *was*
gebraucht wird (z. B. "irgendetwas, das eine Chat-Anfrage beantworten kann"), ohne
festzulegen, *wie* — das übernimmt der `Adapter`. `LlmProvider` ist der zentrale Port des
Kurses.

→ [Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md)

### Prompt

Der Eingabetext, der an ein LLM geschickt wird, um eine Antwort zu erzeugen — im Kurs
typischerweise aus einer `Konversation` sowie optionalen Vorlagen (`Prompt-Templating`)
zusammengesetzt.

→ [Phase 2, Lektion 5](../03-phase2-llm-anbindung/05-prompt-templating.md)

### Prompt Injection

Ein Angriffsmuster, bei dem eingeschleuster Text (z. B. in einem geladenen Dokument oder
einer Nutzereingabe) das LLM dazu bringen soll, seine eigentlichen Anweisungen zu ignorieren
und stattdessen die Anweisungen des Angreifers zu befolgen. Besonders relevant für
RAG-Systeme, die fremden Text automatisch in den Prompt einbetten.

→ [Phase 5, Lektion 8](../06-phase5-rag-betrieb/08-security-docker-ci.md)

### Prompt-Templating

Das Zusammensetzen eines konkreten Prompts aus einer wiederverwendbaren Vorlage mit
Platzhaltern, statt Prompt-Texte an jeder Aufrufstelle neu und inkonsistent
zusammenzubauen.

→ [Phase 2, Lektion 5](../03-phase2-llm-anbindung/05-prompt-templating.md)

### Property-Testing (proptest)

Eine Teststrategie, bei der nicht einzelne feste Beispiele, sondern eine große Zahl
zufällig generierter Eingaben gegen eine allgemeine Eigenschaft (*property*) geprüft
werden, z. B. "Chunking erzeugt nie einen leeren Chunk". `proptest` ist das dafür
verwendete Rust-Crate.

→ [Phase 6, Lektion 2](../07-phase6-performance/02-fuzzing-proptest.md)

### RAG (Retrieval-Augmented Generation)

Ein Muster, bei dem ein LLM vor der Antwortgenerierung relevante Textabschnitte aus einer
eigenen Wissensbasis (per `Retriever`) erhält, statt sich allein auf sein Trainingswissen
zu verlassen — Grundlage aller Dokumenten-gestützten Antworten des Frameworks.

→ [Phase 5](../06-phase5-rag-betrieb/README.md)

### Rate Limit

Eine vom API-Anbieter durchgesetzte Obergrenze, wie viele Anfragen (oder Tokens) in einem
Zeitraum erlaubt sind. Wird das Limit überschritten, antwortet die API mit einem
Fehlerstatus, auf den typischerweise mit `Backoff` und `Retry` reagiert wird.

→ [Phase 5, Lektion 6](../06-phase5-rag-betrieb/06-retry-rate-limit-backoff.md)

### Result

Ein eingebautes Enum (`Ok(T)` / `Err(E)`), das einen Vorgang darstellt, der entweder
erfolgreich einen Wert liefert oder mit einem typisierten Fehler fehlschlägt. Der Compiler
zwingt dazu, beide Fälle zu behandeln, bevor der Erfolgswert genutzt werden kann.

→ [Kapitel 0](../01-grundlagen/04-daten-buendeln.md)

### Retriever

Ein Trait (`Retriever`), das aus einer Anfrage die relevantesten Textabschnitte aus einem
`VectorStore` zurückliefert — die Such-Komponente eines RAG-Systems.

→ [Phase 5, Lektion 4](../06-phase5-rag-betrieb/04-retriever-quellenangaben.md)

### Retry

Der erneute Versuch eines fehlgeschlagenen Vorgangs (z. B. eines API-Aufrufs), meist
begrenzt auf eine feste Anzahl Versuche und kombiniert mit `Backoff`, um nicht sofort erneut
gegen dasselbe Problem zu laufen.

→ [Phase 5, Lektion 6](../06-phase5-rag-betrieb/06-retry-rate-limit-backoff.md)

### Rolle

Das Enum `Rolle { System, Benutzer, Assistent }` — modelliert, wer eine `Nachricht`
gesendet hat, und macht dabei jeden ungültigen Rollenwert zur Kompilierzeit unmöglich.

→ [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)

### Secret

Ein sensibler Wert wie ein API-Key, der niemals im Quellcode oder in Logs auftauchen darf.
Secrets werden aus Umgebungsvariablen oder Konfigurationsdateien geladen, die nicht ins
Git-Repository gehören, und im Speicher nach Möglichkeit mit `Zeroize` behandelt.

→ [Phase 5, Lektion 7](../06-phase5-rag-betrieb/07-tracing-kosten-secrets.md)

### SemVer

*Semantic Versioning*: die Konvention `MAJOR.MINOR.PATCH` für Versionsnummern, bei der
`MAJOR` für Breaking Changes, `MINOR` für abwärtskompatible neue Funktionen und `PATCH` für
Bugfixes steht. Cargo und crates.io setzen SemVer als Grundannahme für
Abhängigkeitsauflösung voraus.

→ [Phase 7, Lektion 4](../08-phase7-release/04-semver-breaking-changes.md)

### Send/Sync

Zwei Marker-Traits, die Rusts Compiler nutzt, um Nebenläufigkeits-Sicherheit zu
garantieren: `Send` heißt, ein Wert darf sicher an einen anderen Thread übergeben werden;
`Sync` heißt, ein Wert darf sicher von mehreren Threads gleichzeitig referenziert werden.
Für asynchronen Code mit Tokio (z. B. im Agent Loop) müssen viele Typen beide Traits
erfüllen.

→ [Phase 4, Lektion 1](../05-phase4-agenten/01-async-und-tokio.md)

### serde

Rusts Standardbibliothek für die Umwandlung zwischen Rust-Werten und Datenformaten wie
JSON oder TOML. `serde` definiert nur die Traits `Serialize`/`Deserialize`; das konkrete
Format kommt aus einem separaten Crate wie `serde_json`.

→ [Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md)

### Slice

Eine Sicht auf zusammenhängende Elemente im Speicher (`&[T]`), ohne Aussage darüber, ob sie
aus einem `Vec`, einem Array oder etwas anderem stammen. `Konversation::verlauf()` gibt
bewusst `&[Nachricht]` statt `&Vec<Nachricht>` zurück, um interne Speicherdetails nicht
nach außen zu verraten.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### SSE (Server-Sent Events) / Streaming

Ein HTTP-basiertes Protokoll, mit dem ein Server fortlaufend einzelne Ereignisse an einen
Client sendet, ohne die Verbindung zu schließen. LLM-Antworten werden häufig per SSE Wort
für Wort (*gestreamt*) statt als ein einziger, langer Block geliefert.

→ [Phase 4, Lektion 2](../05-phase4-agenten/02-sse-streaming.md)

### Struct

Kurzform für *structure*: ein Typ, der mehrere benannte Felder unterschiedlichen Typs zu
einer Einheit bündelt — im Kurs zum Beispiel `Nachricht` mit den Feldern `rolle` und
`inhalt`.

→ [Kapitel 0](../01-grundlagen/04-daten-buendeln.md)

### Structured Output

Eine LLM-Antwort, die einem vorgegebenen Schema (statt freiem Fließtext) folgt, sodass sie
direkt in einen typisierten Rust-Wert eingelesen werden kann. Das Rust-Crate `schemars`
erzeugt im Kurs das nötige JSON-Schema aus vorhandenen Typen.

→ [Phase 2, Lektion 6](../03-phase2-llm-anbindung/06-structured-output.md)

### thiserror

Ein Rust-Crate, das per `derive`-Attribut eigene, präzise Fehlertypen mit wenig
Schreibarbeit erzeugt — die professionelle Weiterentwicklung des von Hand geschriebenen
`NachrichtFehler`-Enums aus Phase 1.

→ [Phase 2, Lektion 4](../03-phase2-llm-anbindung/04-fehlerbehandlung.md)

### Token

Die kleinste Texteinheit, in die ein LLM Ein- und Ausgabe zerlegt (grob: ein Wortteil).
API-Anbieter berechnen Kosten und Limits fast immer pro Token, nicht pro Zeichen oder Wort.

→ [Phase 2](../03-phase2-llm-anbindung/README.md)

### Tool

Eine Fähigkeit, die ein Agent per `Function Calling` aufrufen kann (z. B. "Websuche" oder
"Datei lesen"), beschrieben durch ein Schema, das das LLM versteht, und eine tatsächliche
Ausführungslogik auf der Rust-Seite.

→ [Phase 4, Lektion 3](../05-phase4-agenten/03-tool-schema-function-calling.md)

### Trait

Ein Vertrag: eine Menge von Fähigkeiten (Methoden), die ein Typ hat oder eben nicht hat.
Traits sind Rusts Antwort auf Interfaces/abstrakte Klassen aus anderen Sprachen und die
Grundlage von `Port`s wie `LlmProvider`.

→ [Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md)

### Tracing

Strukturiertes, kontextreiches Logging, das einzelne Vorgänge (z. B. einen kompletten
Agent-Durchlauf über mehrere Funktionsaufrufe hinweg) nachvollziehbar macht. Das
`tracing`-Crate ist der De-facto-Standard dafür im Rust-Ökosystem.

→ [Phase 5, Lektion 7](../06-phase5-rag-betrieb/07-tracing-kosten-secrets.md)

### Unwrap

Die Methode `.unwrap()` auf `Option`/`Result`: entpackt den Erfolgswert oder löst einen
`panic!` aus, falls keiner vorhanden ist. In Tests üblich und akzeptabel, in
Produktionscode bewusst vermieden zugunsten von `?` oder explizitem Fehlerhandling.

→ [Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md)

### Vector Store

Ein Port (`VectorStore`-Trait) für einen Speicher, der `Embedding`-Vektoren ablegt und
effiziente Ähnlichkeitssuchen darauf erlaubt — die Datengrundlage, auf der ein `Retriever`
arbeitet.

→ [Phase 5, Lektion 3](../06-phase5-rag-betrieb/03-embeddings-vector-store.md)

### Workspace

Ein Cargo-Konzept: mehrere Crates, die eine gemeinsame Root-`Cargo.toml`, dieselbe
`target/`-Ausgabe und dieselbe `Cargo.lock` (feste Abhängigkeitsversionen) teilen. Das
Ai_Framework ist von Anfang an als Workspace aus `mein_core` und `mein_cli` (später weiteren
Crates) aufgebaut.

→ [Phase 1, Lektion 1](../02-phase1-fundament/01-workspace-lesen.md)

### YAGNI

*You Aren't Gonna Need It* — die Faustregel, keine Fähigkeit (z. B. ein abgeleitetes
Trait oder eine Konfigurationsoption) auf Vorrat einzubauen, solange sie nicht tatsächlich
gebraucht wird. Im Kurs sichtbar etwa daran, dass `Nachricht` bewusst kein `PartialEq`
ableitet, solange Nachrichten nicht verglichen werden müssen.

→ [Phase 1, Lektion 2](../02-phase1-fundament/02-rolle-und-nachricht.md)

### Zeroize

Eine Technik (und ein gleichnamiges Rust-Crate), sensible Daten wie API-Keys beim
Verlassen des Gültigkeitsbereichs aktiv im Speicher zu überschreiben, statt sie dem
normalen, potenziell verzögerten Freigabemechanismus zu überlassen.

→ [Phase 5, Lektion 7](../06-phase5-rag-betrieb/07-tracing-kosten-secrets.md)
