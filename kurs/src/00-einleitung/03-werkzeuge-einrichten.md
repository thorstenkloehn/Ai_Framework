# Werkzeuge einrichten

Bevor wir in Kapitel 0 die ersten Codezeilen anfassen, installieren wir alles, was wir für
den ganzen Kurs brauchen. Das machen wir einmal, gründlich, und dann nie wieder.

## 1. Rust selbst: rustup

Rust wird nicht direkt installiert, sondern über **rustup**, den offiziellen
Versions-Manager. rustup installiert den Compiler (`rustc`), den Paketmanager (`cargo`)
und hält beides aktuell.

**Linux/macOS**, im Terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows**: Lade `rustup-init.exe` von [rustup.rs](https://rustup.rs) herunter und folge
dem Installer (er installiert bei Bedarf auch die "Visual Studio C++ Build Tools", die
Rust unter Windows zum Linken braucht).

Nach der Installation ein neues Terminal öffnen und prüfen:

```bash
rustc --version
cargo --version
```

Beides sollte eine Versionsnummer ausgeben, z. B. `rustc 1.8x.0`. Falls nicht: Terminal neu
starten oder `source "$HOME/.cargo/env"` ausführen (Linux/macOS).

> **💡 Tipp**
>
> `rustc` ist der Compiler — er übersetzt eine `.rs`-Datei in ein ausführbares Programm.
> `cargo` ist das Werkzeug, mit dem wir in der Praxis fast immer arbeiten: Es ruft `rustc`
> für uns auf, verwaltet Abhängigkeiten, führt Tests aus, formatiert Code. Wir werden
> `rustc` direkt so gut wie nie von Hand aufrufen.

## 2. Editor: Visual Studio Code + rust-analyzer

Dieser Kurs geht davon aus, dass du in **VS Code** tippst (kostenlos, für alle
Betriebssysteme: [code.visualstudio.com](https://code.visualstudio.com)).

Nach der Installation von VS Code installierst du die Extension **rust-analyzer**
(Erweiterungen-Icon in der Seitenleiste → nach "rust-analyzer" suchen → Installieren).
rust-analyzer gibt dir:

- Fehler und Warnungen direkt im Editor, noch bevor du `cargo check` aufrufst,
- Autovervollständigung, die Typen kennt,
- "Gehe zu Definition" für Typen und Funktionen — nützlich, um fremden Code (auch unseren
  eigenen aus früheren Lektionen) schnell wiederzufinden.

> **⚠️ Warnung**
>
> rust-analyzer braucht beim ersten Öffnen eines Projekts etwas Zeit (es kompiliert die
> Abhängigkeiten einmal im Hintergrund, um sie zu verstehen). Wenn Autovervollständigung
> zuerst nicht funktioniert: kurz warten, unten rechts in VS Code zeigt ein Symbol den
> Fortschritt.

Optional, aber empfohlen: Extension **Even Better TOML** (für `Cargo.toml`-Dateien) und
**crates** (zeigt aktuelle Versionsnummern direkt in `Cargo.toml` an).

## 3. Versionskontrolle: Git

Falls noch nicht vorhanden: [git-scm.com](https://git-scm.com). Prüfen mit:

```bash
git --version
```

Wir nutzen Git durchgehend für die Release-Struktur des Kurses: Am Ende jeder Phase
committen wir und setzen einen Tag (z. B. `git tag conversation-in-memory`). Falls dir
Git selbst neu ist: Ein knapper Einstieg genügt — `git init`, `git add`, `git commit -m
"..."`, `git tag <name>`. Das ist alles, was wir in diesem Kurs an Git-Befehlen brauchen.

## 4. Das Repository

Klone das Framework-Repository, das wir in diesem Kurs Schritt für Schritt füllen:

```bash
git clone https://github.com/thorstenkloehn/Ai_Framework.git
cd Ai_Framework
code .
```

Der letzte Befehl öffnet den Ordner in VS Code. Wenn `code .` nicht funktioniert: VS Code
öffnen → Command Palette (`Strg+Shift+P` bzw. `Cmd+Shift+P`) → "Shell Command: Install
'code' command in PATH" ausführen, dann Terminal neu starten.

## 5. Kurzer Funktionstest

Im geklonten Ordner:

```bash
cargo build
```

Cargo lädt beim ersten Mal alle Abhängigkeiten herunter und baut den Workspace. Am Ende
solltest du eine Zeile wie `Compiling mein_core v0.1.0 (...)` und zum Schluss `Finished`
sehen. Kein Internetzugang oder ein Fehler an dieser Stelle? Dann erst das lösen, bevor es
weitergeht — [Fehlermeldungen verstehen](../09-anhang/03-fehlermeldungen.md) hilft dir
dabei, eine unbekannte Fehlermeldung systematisch einzuordnen.

Alles installiert und der Build läuft grün? Dann auf zu
[Kapitel 0 — Programmier-Grundlagen](../01-grundlagen/README.md).
