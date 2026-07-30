# Lektion 7: Optionaler MCP-Client

> **Diese Lektion ist optional.** Die Roadmap dieses Kurses nennt sie ausdrücklich als
> Ausblick, nicht als Pflichtstoff für [Release 4](08-release-4.md). Du kannst sie
> überspringen und direkt mit [Lektion 8](08-release-4.md) weitermachen, ohne dass dir
> etwas für die Definition of Done fehlt — komm gern später hierher zurück.

## Problem

Jedes Werkzeug, das wir bisher gebaut haben ([Lektion 3](03-tool-schema-function-calling.md)),
ist Rust-Code, den wir selbst schreiben und in unseren `AgentLoop` einkompilieren. In der
Praxis will man oft Werkzeuge nutzen, die **jemand anders** betreibt — eine
Dateisystem-Anbindung, eine Firmen-Datenbank, ein Kalender —, ohne für jedes einzelne
einen eigenen Rust-`Tool`-Adapter von Hand zu schreiben und ohne dass jeder Anbieter sein
eigenes, inkompatibles Protokoll erfindet. Genau dieses Problem adressiert das **Model
Context Protocol (MCP)**: ein offener Standard, mit dem eine LLM-Anwendung Werkzeuge und
Datenquellen eines *MCP-Servers* einheitlich entdecken und aufrufen kann — unabhängig
davon, in welcher Sprache der Server geschrieben ist.

## Code (Zielbild)

```rust
pub struct McpWerkzeug {
    server_name: String,
    werkzeug_name: String,
    beschreibung: String,
    schema: serde_json::Value,
    // ein Client-Handle zum jeweiligen MCP-Server steckt hier dahinter
}

#[async_trait::async_trait]
impl crate::agent::Tool for McpWerkzeug {
    fn name(&self) -> &str { &self.werkzeug_name }
    fn beschreibung(&self) -> &str { &self.beschreibung }
    fn parameter_schema(&self) -> serde_json::Value { self.schema.clone() }

    async fn ausfuehren(&self, argumente: serde_json::Value) -> Result<String, crate::agent::ToolFehler> {
        // ruft den MCP-Server über dessen Transport auf (siehe unten) und
        // übersetzt das Ergebnis in einen String
        todo!()
    }
}
```

## Dekonstruktion

### Was MCP löst — und was nicht

MCP standardisiert **drei** Dinge zwischen einer Anwendung (uns, dem "Host"/"Client")
und einem Server: **Tools** (aufrufbare Aktionen, genau wie unser `Tool`-Trait), **Resources**
(lesbare Daten, ohne Seiteneffekt) und **Prompts** (wiederverwendbare Prompt-Vorlagen).
Die Kommunikation läuft über **JSON-RPC**, ein einfaches Anfrage-Antwort-Format über
verschiedene mögliche Transportwege — häufig entweder über die Standard-Ein-/Ausgabe
eines lokal gestarteten Server-Prozesses (*stdio*) oder über HTTP mit
Server-Sent-Events, dasselbe Grundprinzip, das du in
[Lektion 2](02-sse-streaming.md) kennengelernt hast. MCP löst **nicht** das
Function-Calling-Format zwischen uns und dem *Sprachmodell* (das bleibt Sache von
[Lektion 3](03-tool-schema-function-calling.md)) — es löst die Verbindung zwischen
**unserer Anwendung** und **fremden Werkzeugservern**.

### `McpWerkzeug implements Tool` — ein Adapter, kein Sonderfall

Der entscheidende Design-Punkt: `AgentLoop` aus [Lektion 4](04-agent-loop.md) muss **kein
einziges Byte** verändern, um MCP-Werkzeuge zu unterstützen. `McpWerkzeug` implementiert
denselben `Tool`-Trait wie `Taschenrechner` — der Agent Loop sieht nur
`Box<dyn Tool>`, egal ob dahinter lokaler Rust-Code oder ein entfernter MCP-Server über
JSON-RPC steckt. Das ist derselbe Nutzen, den ein Port/Adapter-Schnitt
([Phase 3, Lektion 2](../04-phase3-architektur/02-hexagonal-architecture.md)) immer
bringt: Neue Implementierungen andocken, ohne den Kern anzufassen.

### Grober Ablauf eines MCP-Aufrufs (skizziert)

1. Beim Start verbindet sich unser Framework zu einem konfigurierten MCP-Server (z. B.
   startet es einen lokalen Prozess über `tokio::process::Command` und spricht mit ihm
   über dessen Standard-Ein-/Ausgabe).
2. Es fragt per JSON-RPC `tools/list` ab — der Server antwortet mit einer Liste
   verfügbarer Werkzeuge, je mit Name, Beschreibung und JSON-Schema, strukturell sehr
   ähnlich zu dem, was `parameter_schema()` in [Lektion 3](03-tool-schema-function-calling.md)
   liefert.
3. Für jedes gemeldete Werkzeug bauen wir zur Laufzeit ein `McpWerkzeug` und reichen es
   wie jedes andere `Box<dyn Tool>` in die `werkzeuge`-Liste des `AgentLoop` hinein.
4. Ruft der Agent es auf, schickt `McpWerkzeug::ausfuehren(...)` eine `tools/call`-Anfrage
   an denselben Server und übersetzt dessen Antwort zurück in den `String`, den unser
   `Tool`-Trait erwartet.

> **💡 Tipp**
>
> Wir zeigen hier bewusst **keinen** vollständig lauffähigen JSON-RPC-Client — das wäre
> ein eigenes, umfangreiches Kapitel für sich (Transport, Fehlerprotokoll, Handshake).
> Wenn du das ausprobieren willst: Suche nach einem aktuellen Rust-SDK für MCP auf
> crates.io (die Landschaft entwickelt sich schnell, ein konkreter Crate-Name und eine
> Versionsnummer wären hier schnell veraltet) und beginne mit dem `stdio`-Transport
> gegen einen der offiziellen Beispiel-Server — das Grundprinzip (Werkzeuge auflisten,
> als `Tool` verpacken, aufrufen) bleibt so, wie hier beschrieben.

## Schritt-Reveal

Weil diese Lektion optional und bewusst skizzenhaft bleibt, "reveal"en wir hier keine
vollständige Implementierung, sondern die Denkschritte, mit denen du selbst weiterbauen
könntest, falls du magst:

**Schritt 1 — Werkzeuge als Daten, nicht als Code.** Anders als `Taschenrechner` aus
[Lektion 3](03-tool-schema-function-calling.md), den wir als festen Rust-`struct`
schreiben, entsteht `McpWerkzeug` **zur Laufzeit**, aus der Antwort eines Servers
(Name, Beschreibung, Schema kommen als JSON, nicht aus deinem Quelltext). Der
`Tool`-Trait selbst ändert sich dafür nicht — nur, *woher* die Werte in seinen Feldern
stammen.

**Schritt 2 — `#[async_trait]` auch auf dem `impl`-Block.**

```rust
#[async_trait::async_trait]
impl crate::agent::Tool for McpWerkzeug {
    // ...
}
```

Auch `impl Tool for McpWerkzeug` braucht das `#[async_trait]`-Attribut **auf dem
`impl`-Block**, nicht nur auf der Trait-Definition selbst — beide Seiten (Definition und
jede Implementierung) müssen das Makro kennen, damit die Übersetzung in
`Pin<Box<dyn Future<...>>>` konsistent bleibt (siehe die Erklärung in
[Lektion 3](03-tool-schema-function-calling.md)). Vergisst du es an einer der beiden
Stellen, meldet der Compiler einen Typfehler zwischen "normalem" `async fn`-Rückgabetyp
und dem, was der Trait erwartet — derselbe Fehler in der Kategorie, die du in
[Lektion 3](03-tool-schema-function-calling.md) und
[Lektion 1](01-async-und-tokio.md) schon einmal gesehen hast: ein asynchroner Wert, der
nicht dort ankommt, wo ein synchroner erwartet wird.

**Schritt 3 — Einhängen, nicht umbauen.** Die fertigen `McpWerkzeug`-Werte reihst du in
dieselbe `Vec<Box<dyn Tool>>` ein, die du auch `AgentLoop::neu(...)` übergibst
([Lektion 4](04-agent-loop.md)) — gemischt mit lokalen Werkzeugen wie `Taschenrechner`,
falls du beide gleichzeitig anbieten willst.

## Ausführung

Da diese Lektion bewusst skizzenhaft bleibt, gibt es keinen vollständigen
`cargo test`-Durchlauf. Stattdessen eine Selbstkontrolle: Lies dir noch einmal den
`Tool`-Trait aus [Lektion 3](03-tool-schema-function-calling.md) durch und beantworte
für dich: Welche der vier Methoden (`name`, `beschreibung`, `parameter_schema`,
`ausfuehren`) ließen sich bei einem `McpWerkzeug` **automatisch** aus der Antwort von
`tools/list` befüllen, und welche brauchen bei jedem Aufruf eine echte Netzwerk- oder
Prozesskommunikation?

## Zusammenfassung

- MCP ist ein offener Standard für die Verbindung zwischen einer Anwendung und externen
  Werkzeug-/Datenservern — JSON-RPC als Protokoll, Tools/Resources/Prompts als
  Kernkonzepte.
- Ein MCP-Client lässt sich sauber als weitere `Tool`-Implementierung einbauen — der
  `AgentLoop` selbst bleibt unverändert, dieselbe Stärke, die Ports/Adapter immer bieten.
- Diese Lektion ist ein bewusst optionaler Exkurs: Für [Release 4](08-release-4.md)
  brauchst du keinen lauffähigen MCP-Client, nur das Verständnis, wo er andocken würde.

## Übung (optional)

Falls du diese Lektion vertiefen willst: Skizziere (auf Papier oder als Kommentare in
Rust-Code, ohne Anspruch auf Kompilierbarkeit) eine Funktion
`lade_werkzeuge_von_mcp_server(pfad_zum_server: &str) -> Vec<Box<dyn Tool>>`, die einen
lokalen MCP-Server-Prozess startet, `tools/list` abfragt und für jedes gemeldete
Werkzeug ein `McpWerkzeug` erzeugt. Was müsste diese Funktion tun, wenn der Server beim
Start nicht antwortet — sollte das den ganzen `AgentLoop`-Aufbau zum Scheitern bringen,
oder reicht es, dieses eine Werkzeug wegzulassen und mit den übrigen weiterzumachen?
Es gibt hier keine einzig richtige Antwort — wäge beide Seiten gegeneinander ab.

[Weiter: Lektion 8 · Release 4: tool-using-agent](08-release-4.md)
