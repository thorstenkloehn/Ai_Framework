# Lektion 5: REST mit Axum oder eine TUI als Anwendungsschicht

## Problem

`mein_cli` war bisher unsere einzige Anwendungsschicht — ein Programm, ein Prozess, ein
Nutzer. Sobald unser Framework Dokumente durchsucht und Agenten betreibt, wollen wir es
oft **als Dienst** anbieten: mehrere Clients (eine Web-Oberfläche, ein anderes Programm,
ein Kollege im selben Netzwerk) sollen gleichzeitig Anfragen stellen können, ohne dass
jede Anfrage einen neuen Prozess startet. Die naheliegende Lösung: eine REST-API über
HTTP. Wir bauen dafür ein neues Binary-Crate `mein_server` mit dem Web-Framework
**Axum**, als Ergänzung — nicht Ersatz — zu `mein_cli`.

## Code (Zielbild)

```rust
async fn chat(
    State(state): State<AppState>,
    Json(anfrage): Json<ChatRequest>,
) -> Json<ChatResponse> {
    // ...
}

let app = Router::new().route("/chat", post(chat)).with_state(state);
```

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"frage": "Wie beantrage ich Urlaub?"}'
```

## Dekonstruktion

### Warum ein eigenes Crate statt `axum` in `mein_cli`?

`mein_cli` ist ein kurzlebiger Prozess: starten, eine Aufgabe erledigen, beenden. Ein
Axum-Server ist ein **langlaufender** Prozess, der auf einem Port lauscht. Das sind zwei
grundsätzlich verschiedene Betriebsarten mit unterschiedlichen Abhängigkeiten (Axum,
Tower, ein TCP-Listener) — sie in einem Binary zu vermengen würde `mein_cli` unnötig
schwergewichtig machen, selbst für Nutzer:innen, die nie einen Server starten wollen.
Genau wie `mein_agent` in Phase 4 sein eigenes Crate bekam, bekommt der Server-Betrieb
jetzt seins: `mein_server`, mit `mein_core`, `mein_agent` und `mein_rag` als
Abhängigkeiten.

### `AppState` — geteilter Zustand über Anfragen hinweg

Ein einzelner Axum-Server bedient viele Anfragen nebenläufig, aber alle greifen auf
**dieselben** Ports zu — denselben `Retriever`, denselben `LlmProvider`. Axum reicht
diesen geteilten Zustand über den `State`-Extractor an jeden Handler weiter:

```rust
#[derive(Clone)]
struct AppState {
    retriever: Arc<dyn mein_rag::Retriever>,
    llm: Arc<dyn mein_core::port::LlmProvider>,
}
```

`Arc<dyn Retriever>` (statt `Box<dyn Retriever>`): `Box` hat genau einen Besitzer,
`Arc` (*atomically reference counted* — ein Zähler, der threadsicher mitzählt, wie viele
Stellen im Programm gerade eine Referenz auf denselben Wert halten) erlaubt **mehrere**
gleichzeitige Besitzer über Threads hinweg. Genau das brauchen wir: Jeder eingehende
Request bekommt eine eigene Kopie des `AppState` — aber `Arc::clone` kopiert nur den
Zeiger und erhöht den Zähler, nicht den `Retriever` selbst.

### `#[derive(Clone)]` auf `AppState` ist keine Formalie

Axums `State`-Extractor **verlangt**, dass der State-Typ `Clone` implementiert — jeder
Handler-Aufruf bekommt seine eigene, gestackte Kopie. Weil alle Felder von `AppState`
selbst `Arc<...>` sind (billig zu klonen), ist `#[derive(Clone)]` hier korrekt und
günstig. Was passiert, wenn wir es vergessen, zeigt der nächste Abschnitt.

## Schritt-Reveal

**Schritt 1 — Crate anlegen:**

```bash
cargo new mein_server
cd mein_server
cargo add axum
cargo add tokio --features full
cargo add serde --features derive
cargo add mein_core --path ../mein_core
cargo add mein_rag --path ../mein_rag
cargo add mein_agent --path ../mein_agent
```

Trage `mein_server` in die Workspace-`Cargo.toml` ein.

**Schritt 2 — `AppState` bewusst ohne `Clone` anlegen** und eine minimale Route bauen:

```rust
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

struct AppState {
    modell_name: Arc<String>,
}

#[derive(Deserialize)]
struct ChatRequest {
    frage: String,
}

#[derive(Serialize)]
struct ChatResponse {
    antwort: String,
}

async fn chat(State(state): State<AppState>, Json(req): Json<ChatRequest>) -> Json<ChatResponse> {
    Json(ChatResponse { antwort: format!("{} sagt: {}", state.modell_name, req.frage) })
}

#[tokio::main]
async fn main() {
    let state = AppState { modell_name: Arc::new("demo".to_string()) };
    let app: Router = Router::new().route("/chat", post(chat)).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

`cargo check -p mein_server`:

```
error[E0277]: the trait bound `AppState: Clone` is not satisfied
  --> src/main.rs:28:52
   |
28 |     let app: Router = Router::new().route("/chat", post(chat)).with_state(state);
   |                                                    ^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
   |
note: required by a bound in `post`
   |
   |             S: Clone + Send + Sync + 'static,
   |                ^^^^^ required by this bound in `post`
   |
help: consider annotating `AppState` with `#[derive(Clone)]`
```

Axums eigener `post`-Handler-Wrapper verlangt `S: Clone + Send + Sync + 'static` für den
State-Typ — genau das Muster, das dir in Lektion 4 schon bei `Retriever: Send + Sync`
begegnet ist, nur diesmal von einer fremden Bibliothek eingefordert statt von uns selbst
definiert. Der Compiler schlägt die Lösung sogar direkt vor.

**Schritt 3 — Reparieren:**

```rust
#[derive(Clone)]
struct AppState {
    modell_name: Arc<String>,
}
```

`cargo check -p mein_server` — kompiliert.

**Schritt 4 — Echten `AppState` mit Retriever und Provider bauen.** Erinnerung an die
`LlmProvider`-Signatur, wie sie in
[Phase 4, Lektion 4](../05-phase4-agenten/04-agent-loop.md) auf `async` erweitert
wurde: `antworten(&self, verlauf: &Konversation) -> Result<Nachricht, ...>` — sie nimmt
also einen ganzen `Konversation`-Verlauf entgegen, keinen einzelnen String.

```rust
use mein_core::{Konversation, Rolle};

#[derive(Clone)]
struct AppState {
    retriever: Arc<dyn mein_rag::Retriever>,
    llm: Arc<dyn mein_core::port::LlmProvider>,
}

async fn chat(State(state): State<AppState>, Json(req): Json<ChatRequest>) -> Json<ChatResponse> {
    let treffer = state.retriever.retrieve(&req.frage, 3).await.unwrap_or_default();
    let kontext: String = treffer.iter().map(|t| t.content.clone()).collect::<Vec<_>>().join("\n---\n");

    // Wichtig: kontext ist Retrieval-Inhalt, kein Systembefehl -- siehe Lektion 8
    // fuer den korrekten, sicheren Prompt-Aufbau. Dieses Beispiel zeigt nur die
    // Axum-Mechanik, nicht die vollständige Absicherung.
    let mut konversation = Konversation::neu();
    let _ = konversation.hinzufuegen(Rolle::System, format!("Referenzmaterial:\n{kontext}"));
    let _ = konversation.hinzufuegen(Rolle::Benutzer, req.frage.clone());

    let text = match state.llm.antworten(&konversation).await {
        Ok(nachricht) => nachricht.inhalt,
        Err(_) => "Fehler beim Provider-Aufruf".to_string(),
    };
    Json(ChatResponse { antwort: text })
}
```

Der Kommentar ist bewusst gesetzt: Diese Lektion konzentriert sich auf die
Axum-Mechanik (Routing, State, Extractoren); die korrekte, sichere Trennung von
System-, Retrieval- und Nutzeranteil im Prompt bauen wir erst vollständig in
[Lektion 8](08-security-docker-ci.md) aus — `Konversation` kennt aktuell nur die Rollen
`System`/`Benutzer`/`Assistent` ([Phase 1](../02-phase1-fundament/02-rolle-und-nachricht.md)),
keine eigene Rolle für Retrieval-Inhalte, was genau der Grund ist, warum eine explizite,
strukturelle Trennung nötig ist statt sich auf `Rolle::System` allein zu verlassen.

## Ausführung

```bash
cargo run -p mein_server
```

```
   Compiling mein_server v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.1s
     Running `target/debug/mein_server`
```

In einem zweiten Terminal:

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"frage": "Wie beantrage ich Urlaub?"}'
```

```json
{"antwort":"..."}
```

Provoziere den Fehlerpfad bewusst: Schicke ungültiges JSON (`-d 'kein json'`) — Axum
antwortet automatisch mit `400 Bad Request`, bevor dein Handler-Code überhaupt läuft, weil
der `Json`-Extractor die Deserialisierung schon vor dem Handler übernimmt.

> **💡 Tipp**
>
> Bevorzugst du ein Terminal-Interface ohne HTTP? Statt `mein_server` mit Axum kannst du
> stattdessen (oder zusätzlich) eine **TUI** (*Terminal User Interface*) bauen, z. B. mit
> dem Crate `ratatui`. Der Vorteil: keine Netzwerkschicht, kein separater Client nötig —
> alles läuft interaktiv im selben Terminal. Der Nachteil: keine gleichzeitigen externen
> Clients. Für unser Framework verbindet sich Axum sauberer mit dem Hexagonal-Muster aus
> [Phase 3](../04-phase3-architektur/02-hexagonal-architecture.md), weil die
> HTTP-Schicht genau wie `mein_cli` nur ein weiterer **Adapter** um dieselben Ports ist —
> deshalb ist sie hier unser Hauptbeispiel. Wer lieber mit einer TUI experimentiert, kann
> dieselben Ports (`Retriever`, `LlmProvider`) genauso dahinter verdrahten.

## Zusammenfassung

- `mein_server` ist ein eigenes Binary-Crate für den langlaufenden Betrieb — getrennt von
  `mein_cli`, aus demselben Grund, aus dem `mein_agent` ein eigenes Crate wurde.
- `AppState` bündelt geteilten Zustand (`Arc<dyn Retriever>`, `Arc<dyn LlmProvider>`) für
  alle Handler.
- Axums `State`-Extractor verlangt `Clone` auf dem State-Typ — der Compiler benennt das
  über `E0277` präzise und schlägt die Lösung vor.
- `Arc` statt `Box` erlaubt geteilten Besitz über gleichzeitige Anfragen hinweg, ohne den
  zugrunde liegenden Port zu kopieren.
- Eine TUI ist eine plausible Alternative zu REST für rein interaktive,
  einzelbenutzerorientierte Anwendungsfälle — die Ports dahinter bleiben identisch.

## Übung

Ergänze eine zweite Route `GET /health`, die ohne `AppState`-Zugriff sofort
`{"status": "ok"}` mit HTTP-Status 200 zurückgibt (in Axum: `routing::get`). Nutze das,
um zu überprüfen, dass ein Betriebssystem oder Load Balancer den Server-Zustand abfragen
kann, ohne einen echten `Retriever`- oder `LlmProvider`-Aufruf auszulösen. Teste die neue
Route mit `curl`.

[Weiter: Lektion 6 — Retry, Rate Limit und Backoff](06-retry-rate-limit-backoff.md)
