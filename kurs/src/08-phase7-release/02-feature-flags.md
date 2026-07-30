# Lektion 2: Feature Flags

## Problem

Jemand, der unser Crate nur für einfache Chat-Anfragen ohne RAG und ohne Agenten nutzen
möchte, muss beim `cargo build` trotzdem `mein_rag` (mit seinen Vector-Store- und
Embedding-Abhängigkeiten) und `mein_agent` (mit `tokio` und Tool-Infrastruktur)
mitkompilieren, wenn unser Haupt-Crate sie unbedingt referenziert. Das kostet
Compile-Zeit, vergrößert die Binary und zieht Abhängigkeiten in Projekte, die sie nie
benutzen. Wir wollen: Nutzer*innen zahlen (an Compile-Zeit, an Binärgröße, an
Sicherheits-Audit-Aufwand für Abhängigkeiten) nur für das, was sie tatsächlich einbinden.

## Code (Zielbild)

```toml
# mein_core/Cargo.toml
[features]
default = []
rag = ["dep:mein_rag"]
agent = ["dep:mein_agent"]

[dependencies]
mein_rag = { path = "../mein_rag", optional = true }
mein_agent = { path = "../mein_agent", optional = true }
```

```rust
// mein_core/src/lib.rs
#[cfg(feature = "rag")]
pub use mein_rag::Retriever;

#[cfg(feature = "agent")]
pub use mein_agent::Agent;
```

## Dekonstruktion

### `[features]` und optionale Dependencies

Ein **Feature Flag** in Cargo ist ein benannter Schalter, den Nutzer*innen beim Bauen
an- oder ausschalten (`cargo build --features rag`). `optional = true` bei einer
Dependency sagt: "Diese Abhängigkeit wird nur mitkompiliert, wenn ein Feature sie
tatsächlich aktiviert" — ohne `optional = true` würde `mein_rag` immer mitgebaut, egal
welche Features aktiv sind, und das `[features]`-Flag `rag` hätte keine Wirkung.

### `dep:mein_rag` — die "weak"-Syntax

Bis vor einigen Cargo-Versionen erzeugte jede optionale Dependency automatisch *auch* ein
gleichnamiges Feature (`mein_rag = ["mein_rag"]` implizit) — verwirrend, wenn man ein
eigenes, anders benanntes Feature dafür bauen wollte. Die `dep:`-Präfix-Syntax
(`["dep:mein_rag"]`) trennt das sauber: `dep:mein_rag` bedeutet "aktiviere die
Abhängigkeit `mein_rag`", ohne automatisch ein öffentliches Feature namens `mein_rag`
freizugeben. Unser Feature heißt bewusst `rag`, nicht `mein_rag` — der fachliche Begriff,
nicht der interne Crate-Name.

### `#[cfg(feature = "...")]` — bedingte Kompilierung

Genau wie `#[cfg(test)]` aus [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md)
nur beim Testen mitkompiliert, kompiliert `#[cfg(feature = "rag")]` eine Codezeile nur,
wenn das Feature `rag` aktiv ist. Ohne aktives Feature existiert `mein_core::Retriever`
für den Compiler schlicht nicht — kein totes Codeschnipsel, das nur zur Laufzeit
ignoriert wird, sondern komplett aus der Kompilierung entfernter Code.

### Warum `default = []`?

Ein leeres `default`-Feature bedeutet: Wer `mein_core` ohne weitere Angaben einbindet
(`mein_core = "0.1"`), bekommt den schlanken Kern ohne RAG, ohne Agenten. Das ist eine
bewusste Design-Entscheidung — die Alternative (`default = ["rag", "agent"]`) würde
allen Erstnutzer*innen automatisch alle Abhängigkeiten aufbürden und genau das Problem
zurückbringen, das wir mit Feature Flags lösen wollten.

## Schritt-Reveal

**Schritt 1 — Feature-Definitionen in `mein_core/Cargo.toml` ergänzen** (siehe Zielbild
oben).

**Schritt 2 — Provoziere den Cargo-Fehler bewusst.** Lösche testweise `optional = true`
bei `mein_rag`, behalte aber `rag = ["dep:mein_rag"]` im `[features]`-Block:

```bash
cargo check -p mein_core --features rag
```

```
error: feature `rag` includes `dep:mein_rag`, but `mein_rag` is not an optional dependency
  --> mein_core/Cargo.toml
   |
   = help: correct the name or mark the dependency as `optional = true`
```

Cargo prüft zur Kompilierzeit, dass jedes über `dep:` referenzierte Crate auch tatsächlich
optional ist — konsequent, denn eine *nicht*-optionale Abhängigkeit wird ohnehin immer
mitgebaut, ein Feature dafür wäre bedeutungslos. Setze `optional = true` zurück.

**Schritt 3 — Re-Exports hinter `#[cfg(feature = "...")]` ergänzen** (siehe Zielbild).

**Schritt 4 — Ohne Feature bauen und den fehlenden Pfad beobachten.** Versuche in
`mein_cli`, `mein_core::Retriever` zu benutzen, ohne das Feature zu aktivieren:

```bash
cargo check -p mein_cli
```

```
error[E0432]: unresolved import `mein_core::Retriever`
 --> mein_cli/src/main.rs:3:19
  |
3 | use mein_core::Retriever;
  |                 ^^^^^^^^ no `Retriever` in the root
```

Genau das gewünschte Verhalten: Ohne aktiviertes Feature existiert `Retriever` für
`mein_cli` schlicht nicht. Aktiviere das Feature in `mein_cli/Cargo.toml`:

```toml
[dependencies]
mein_core = { path = "../mein_core", features = ["rag"] }
```

`cargo check -p mein_cli` — kompiliert jetzt.

## Ausführung

```bash
cargo build -p mein_core                      # schlanker Kern, ohne rag/agent
cargo build -p mein_core --features rag        # + mein_rag
cargo build -p mein_core --all-features        # rag + agent
```

Vergleiche `cargo build -p mein_core` und `cargo build -p mein_core --all-features` in der
Kompilierzeit (`time cargo build ...` nach `cargo clean`) — der Unterschied macht den
Sinn der Feature Flags spürbar, nicht nur behauptet.

> **⚠️ Warnung**
>
> Cargo **vereinigt** Features additiv über einen ganzen Dependency-Baum: Baut irgendein
> anderes Crate im selben Workspace `mein_core` mit `--features rag`, ist `rag` für *alle*
> Nutzungen von `mein_core` im selben Build aktiv, nicht nur lokal für dieses eine Crate.
> Das nennt sich Feature Unification. Für additive, rein erweiternde Features (wie unsere)
> ist das unproblematisch — vermeide aber sich gegenseitig ausschließende Features (z. B.
> zwei alternative Backends), Cargo kennt kein "entweder-oder" auf Feature-Ebene.

## Zusammenfassung

- `optional = true` + `dep:name` in `[features]` macht eine Abhängigkeit nur bei aktivem
  Feature Teil des Builds.
- `#[cfg(feature = "...")]` entfernt Code vollständig aus der Kompilierung, wenn das
  Feature nicht aktiv ist — kein Laufzeit-Overhead, kein totes Codeschnipsel.
- `default = []` hält den Einstieg schlank; Nutzer*innen aktivieren gezielt, was sie
  brauchen.
- Cargo vereinigt Features additiv über den gesamten Dependency-Baum (Feature
  Unification) — ein wichtiger Grund, Features rein additiv zu gestalten.

## Übung — Transferaufgabe der Phase

Eine neue Provider-Integration wird hinzugefügt, ohne bestehende Nutzer-Codebeispiele zu
ändern. Füge testweise einen zweiten Adapter für den `LlmProvider`-Port aus
[Phase 3, Lektion 1](../04-phase3-architektur/01-llmprovider-port.md) hinzu (z. B. einen
Adapter für einen anderen Anbieter als den bisherigen) — hinter einem eigenen Feature
(z. B. `anbieter_zwei`). Prüfe danach explizit: Kompiliert der `ClientBuilder`-Aufruf aus
[Lektion 1](01-builder-pattern.md) unverändert, ohne dass du eine einzige Zeile des dort
gezeigten Beispiels anfassen musst? Zwei Leitfragen dafür: Muss der `LlmProvider`-Trait
selbst sich ändern, damit ein neuer Adapter dazukommt — und wo genau (Adapter-Ordner aus
der Hexagonal Architecture in Phase 3) landet neuer Code, ohne bestehende Module zu
berühren?

[Weiter: Lektion 3 — Rustdoc und Beispiele](03-rustdoc-beispiele.md)
