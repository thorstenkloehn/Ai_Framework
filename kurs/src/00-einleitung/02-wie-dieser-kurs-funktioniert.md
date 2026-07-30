# Wie dieser Kurs funktioniert

## Das Lektionsformat

Jede Lektion in diesem Buch folgt derselben Struktur. Das ist Absicht, nicht Monotonie:
Wenn das Format immer gleich ist, kannst du dich auf den Inhalt konzentrieren statt jedes
Mal neu zu orientieren.

1. **Problem** — Wir formulieren eine konkrete Anforderung. Meist als Frage: "Wie
   verhindern wir, dass eine Nachricht ohne Inhalt entsteht?"
2. **Code (Zielbild)** — Ein kurzer Blick auf das Ergebnis, bevor wir den Weg dorthin
   gehen. So weißt du, worauf die folgenden Schritte hinauslaufen.
3. **Dekonstruktion** — Wir zerlegen das Zielbild in Begriffe: Welche Typen brauchen wir?
   Welches Modul? Welche Abhängigkeit? Warum so und nicht anders?
4. **Schritt-Reveal** — Die eigentliche Lösung, in kleinen Häppchen. Jeder Schritt soll für
   sich kompilieren (oder absichtlich nicht — dann erklären wir, warum).
5. **Ausführung** — Konkrete Befehle für dein Terminal (`cargo check`, `cargo test`,
   `cargo run`), mit der Ausgabe, die du erwarten kannst.
6. **Zusammenfassung** — Was haben wir entschieden, welche Alternativen gab es, welchen
   Kompromiss sind wir eingegangen?
7. **Übung** — Eine Transferaufgabe. Nicht "tippe das Gleiche nochmal ab", sondern "wende
   das Prinzip auf eine neue Anforderung an". Ohne Musterlösung — das ist Absicht.

Dazwischen findest du zwei Arten von Boxen:

> **💡 Tipp**
>
> So sehen Tipps aus — kleine Hinweise, die dir den nächsten Schritt erleichtern.

> **⚠️ Warnung**
>
> So sehen Warnungen aus — typische Stolperfallen, gerade für Umsteiger*innen aus anderen
> Sprachen oder für den allerersten Kontakt mit Rust.

## Die wichtigste Regel: Selbst tippen

Dieses Buch zeigt dir vollständigen Code — anders als in einer reinen Chat-Unterhaltung,
wo Code erst nach und nach besprochen wird. Trotzdem gilt für dich beim Lernen dieselbe
Regel, die auch das Ai_Framework-Projekt selbst für sich festgelegt hat:

> Code wird besprochen, verstanden und **selbst getippt** — nicht kopiert. Erst wenn er
> bei dir kompiliert und die Tests laufen, kommt der Git-Release.

Warum diese Regel? Weil Tippen und Kompilieren-Lassen der Moment ist, in dem du wirklich
lernst. Copy-Paste erzeugt die Illusion von Verständnis: Es sieht aus, als hättest du es
verstanden, weil es läuft. Aber du hast nur den Editor eines anderen kopiert. Sobald du
selbst tippst, passieren dir Tippfehler, vergisst du ein Semikolon, vertauschst zwei
Argumente — und genau diese kleinen Fehler, samt der Compilermeldung, die sie erklärt,
sind der eigentliche Lernstoff.

Praktisch heißt das für jede Lektion:

1. Lies **Problem** und **Zielbild**, ohne schon zu tippen.
2. Lies die **Dekonstruktion** — versuche, dir den Aufbau vorzustellen, bevor du den
   Schritt-Reveal siehst.
3. Tippe **Schritt für Schritt** selbst in VS Code, in der im Text genannten Datei.
   Kompiliere nach jedem Schritt (`cargo check`).
4. Wenn ein Schritt absichtlich einen Fehler zeigt: Lass ihn zu, lies die
   Compilermeldung selbst, bevor du zur Erklärung im Buch weiterliest.
5. Führe den **Ausführung**-Abschnitt wirklich aus, nicht nur gedanklich.
6. Löse die **Übung**, bevor du zur nächsten Lektion springst — auch wenn deine Lösung
   nicht perfekt ist.
7. Am Ende jeder Phase: `git add`, `git commit`, `git tag <release-name>`.

## Der agile Zyklus

Jede Phase (und im Grunde jede Lektion) durchläuft denselben Kreislauf, den wir auch in
professionellen Rust-Projekten verwenden:

**Planung → Analyse → Entwurf → Implementierung → Test → Review → Dokumentation**

Du wirst diesen Rhythmus in jeder Phasen-Übersichtsseite wiedererkennen: Wir formulieren
ein sichtbares Lernziel und eine kleine "Definition of Done", bevor wir anfangen. Eine
Lektion gilt erst als fertig, wenn:

- der Code kompiliert,
- mindestens ein relevanter Test läuft,
- der Fehlerpfad (was passiert bei ungültiger Eingabe?) besprochen wurde,
- die Transferaufgabe eine eigene Lösungsidee bekommen hat.

## Wenn du feststeckst

Rust-Fehlermeldungen sind lang und wirken am Anfang einschüchternd, sind aber meist die
hilfreichsten Fehlermeldungen, die du je gesehen hast. [Der Anhang "Fehlermeldungen
verstehen"](../09-anhang/03-fehlermeldungen.md) zeigt dir, wie man sie liest. Und
[Kapitel 0.5, "Der Compiler als Lehrer"](../01-grundlagen/05-der-compiler-als-lehrer.md),
bereitet dich grundsätzlich darauf vor, bevor der erste echte Fehler auftaucht.

[Weiter: Werkzeuge einrichten](03-werkzeuge-einrichten.md)
