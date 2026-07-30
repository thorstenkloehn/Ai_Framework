# Was ist Programmieren überhaupt?

## Ein Computer tut nur, was du ihm sagst — nicht mehr, nicht weniger

Ein Computer ist, ganz nüchtern betrachtet, eine Maschine, die extrem schnell extrem
simple Anweisungen abarbeitet: Zahlen addieren, Werte vergleichen, Daten von einer Stelle
im Speicher zu einer anderen kopieren, und abhängig von einem Ergebnis an einer anderen
Stelle weitermachen. Alles, was ein Computer "kann" — Videos abspielen, mit einem
Sprachmodell chatten, dieses Buch anzeigen — ist am Ende auf solche simplen Schritte
heruntergebrochen.

**Programmieren** heißt: Diese Schritte so aufschreiben, dass der Computer genau das tut,
was du willst. Die Herausforderung dabei ist selten die Technik — sie ist, präzise genug
zu denken. Menschen kommunizieren mit Lücken, Kontext und gutem Willen ("mach das Licht
aus" versteht jeder Mensch richtig, auch ohne zu sagen, welches Licht). Computer haben
keinen gesunden Menschenverstand. Jede Lücke in deiner Anweisung ist entweder ein Fehler
oder — schlimmer — ein Programm, das etwas anderes tut, als du dachtest.

## Quellcode, Compiler, Programm

Wir schreiben unsere Anweisungen nicht in reinen Nullen und Einsen, sondern in einer
**Programmiersprache** — einer für Menschen lesbaren Notation mit festen Regeln
(Syntax). Diese Datei mit unseren Anweisungen heißt **Quellcode** (englisch *source
code*).

Ein Computer versteht Quellcode nicht direkt. Bei Rust übersetzt ein Programm namens
**Compiler** (`rustc`) den Quellcode in Maschinencode — die Nullen und Einsen, die der
Prozessor tatsächlich ausführen kann. Diesen Vorgang nennen wir **kompilieren**. Erst das
Ergebnis, eine ausführbare Datei, kannst du starten.

```
Quellcode (.rs-Datei)  --[Compiler: rustc]-->  ausführbares Programm  --[Ausführen]--> Wirkung
```

Das unterscheidet Rust von Sprachen wie Python oder JavaScript, wo ein **Interpreter** den
Quellcode Zeile für Zeile liest und direkt ausführt, ohne vorherigen Übersetzungsschritt.
Der Kompilier-Schritt kostet Zeit vor der Ausführung — dafür findet der Compiler viele
Fehler, bevor das Programm überhaupt läuft, statt dass sie erst bei einem Nutzer auffallen.
Genau das macht Rust so streng, und genau das machen wir uns in diesem Kurs zunutze (mehr
dazu in [Der Compiler als Lehrer](05-der-compiler-als-lehrer.md)).

## Ein Programm ist eine Abfolge von Anweisungen

Im Kern besteht jedes Programm aus vier Grundbausteinen, die in jeder Programmiersprache
in irgendeiner Form wiederkehren:

- **Daten halten** — etwas merken, z. B. den Namen eines Nutzers oder den Inhalt einer
  Chat-Nachricht. Dafür gibt es [Variablen und Typen](01-variablen-und-typen.md).
- **Handlungen bündeln** — einen benannten, wiederverwendbaren Block von Anweisungen
  definieren. Dafür gibt es [Funktionen](02-funktionen.md).
- **Entscheiden und wiederholen** — abhängig von einer Bedingung unterschiedlich
  reagieren, oder etwas mehrfach tun. Dafür gibt es
  [Kontrollfluss](03-kontrollfluss.md).
- **Zusammengehörige Daten bündeln** — mehrere Werte, die logisch zusammengehören, als
  eine Einheit behandeln. Dafür gibt es
  [Structs und Enums](04-daten-buendeln.md).

Diese vier Bausteine reichen aus, um jedes Programm zu bauen, das je geschrieben wurde —
von einem Taschenrechner bis zu einem KI-Framework. Der Rest ist, wie man sie sinnvoll
kombiniert. Genau das üben wir ab jetzt, zuerst isoliert in diesem Kapitel, dann am echten
Framework-Code ab [Phase 1](../02-phase1-fundament/README.md).

[Weiter: Variablen, Werte und Typen](01-variablen-und-typen.md)
