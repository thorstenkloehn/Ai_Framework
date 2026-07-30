# Fehlermeldungen verstehen

Eine unbekannte Rust-Fehlermeldung wirkt anfangs oft länger und einschüchternder, als sie
ist. Dieser Anhang ist eine systematische Anleitung, wie man sie liest — ausführlicher
eingeführt in [Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md).
Danach folgt eine Liste der Fehler, die im Kurs am häufigsten bewusst provoziert werden.

## Systematisch vorgehen

1. **Nur den ersten Fehler lesen.** Rust meldet oft mehrere Fehler gleichzeitig — spätere
   sind häufig nur Folgefehler des ersten. Den ersten beheben, neu kompilieren, erst dann
   weiterschauen.
2. **Den Fehlercode notieren.** `error[E0308]` — der Code in eckigen Klammern lässt sich
   nachschlagen: `rustc --explain E0308` gibt im Terminal eine ausführliche Erklärung mit
   Beispielen aus, oft hilfreicher als die kurze Fehlermeldung selbst.
3. **Den `-->`-Ort lesen.** `--> src/main.rs:2:5` heißt Datei, Zeile, Spalte — die exakte
   Stelle, an der der Compiler das Problem erkannt hat (nicht immer die Stelle, an der der
   eigentliche Denkfehler passiert ist, aber der beste Startpunkt).
4. **Die `^^^`-Markierung ernst nehmen.** Sie zeigt exakt, welcher Ausdruck oder Typ
   gemeint ist — bei mehrzeiligen Ausdrücken lohnt es, genau zu prüfen, ob die Markierung
   den erwarteten Teil trifft oder eine ganz andere Stelle als vermutet.
5. **`help:`-Vorschläge zuerst ausprobieren.** Rusts Fehlermeldungen enthalten sehr häufig
   einen konkreten, meist korrekten Lösungsvorschlag (`help: remove this semicolon`,
   `help: consider borrowing here`). Das ist kein Zufall, sondern bewusste
   Designphilosophie des Compilers — erklären statt nur melden.
6. **`error` vs. `warning` unterscheiden.** Ein `warning` verhindert das Kompilieren nicht,
   sollte aber trotzdem behoben werden — ein sauberer Build ohne Warnungen ist Teil der
   "Definition of Done" jeder Lektion.

> **💡 Tipp**
>
> Bei sehr langen Fehlermeldungen lohnt es, das Terminal-Fenster zu vergrößern oder
> `cargo check` statt `cargo build` zu nutzen — es meldet dieselben Fehler schneller, ohne
> eine Binärdatei zu erzeugen.

## Die häufigsten Fehler in diesem Kurs

### E0308 — mismatched types

```
error[E0308]: mismatched types
 --> mein_cli/src/main.rs:3:20
  |
3 |     println!("{:?}", nachricht);
  |                       ^^^^^^^^^ expected `Nachricht`, found `Result<Nachricht, NachrichtFehler>`
```

**Bedeutung:** An dieser Stelle wird ein anderer Typ erwartet, als tatsächlich vorliegt.
Sehr häufig, wenn sich die Signatur einer Funktion ändert (z. B. `Nachricht::neu` gibt
seit [Phase 1, Lektion 3](../02-phase1-fundament/03-invarianten.md) ein `Result` statt
einer nackten `Nachricht` zurück), aber Aufrufer*innen noch nicht angepasst wurden.

**Typischer Fix:** Den Rückgabewert mit `match`, `if let` oder `?` behandeln, statt ihn
direkt so zu verwenden, als wäre er der entpackte Wert.

### E0502 / E0499 — Borrow-Checker-Konflikte

```
error[E0502]: cannot borrow `konversation` as mutable because it is also borrowed as immutable
```

**Bedeutung:** Zu einem Zeitpunkt existiert gleichzeitig eine lesende (`&`) und eine
schreibende (`&mut`) Referenz auf denselben Wert — oder zwei schreibende Referenzen
gleichzeitig (E0499). Rusts Borrow-Checker verbietet das grundsätzlich, weil daraus
Dateninkonsistenzen entstehen könnten.

**Typischer Fix:** Die lesende Referenz vor der schreibenden beenden (z. B. Ergebnis in
eine Variable zwischenspeichern statt eine Referenz über mehrere Zeilen offenzuhalten),
oder — falls möglich — mit einer Kopie statt einer Referenz arbeiten (`.clone()`).

### E0596 — cannot borrow as mutable

```
error[E0596]: cannot borrow `konversation` as mutable, as it is not declared as mutable
```

**Bedeutung:** Eine Methode wie `hinzufuegen(&mut self, ...)` verlangt veränderbaren
Zugriff, aber die Variable wurde ohne `mut` deklariert. Genau dieser Fehler wird bewusst in
[Phase 1, Lektion 4](../02-phase1-fundament/04-konversation.md) provoziert.

**Typischer Fix:** `let konversation = ...` zu `let mut konversation = ...` ändern.

### "missing field" bei serde

```
Error("missing field `modell`", line: 1, column: 20)
```

**Bedeutung:** Ein `#[derive(Deserialize)]`-Typ hat ein Pflichtfeld (ohne
`#[serde(default)]`), das im eingelesenen JSON/TOML nicht vorkommt. Siehe
[Phase 1, Lektion 5](../02-phase1-fundament/05-serde-konfiguration.md).

**Typischer Fix:** Entweder das fehlende Feld in der Eingabedatei ergänzen, oder — falls
das Feld wirklich optional sein soll — `#[serde(default)]` bzw.
`#[serde(default = "funktionsname")]` am Feld ergänzen.

### "the trait bound `...` is not satisfied"

```
error[E0277]: the trait bound `Nachricht: PartialEq` is not satisfied
```

**Bedeutung:** Ein Codeausschnitt verlangt, dass ein Typ ein bestimmtes Trait
implementiert (hier: `PartialEq` für einen `==`-Vergleich oder `assert_eq!`), aber der Typ
tut das nicht. Häufig, wenn eine `derive`-Angabe vergessen oder bewusst weggelassen wurde
(siehe [YAGNI im Glossar](01-glossar.md)).

**Typischer Fix:** Das fehlende Trait zur `#[derive(...)]`-Liste des Typs hinzufügen —
sofern das fachlich sinnvoll ist. Falls nicht: den Code so umschreiben, dass er ohne
dieses Trait auskommt.

### E0004 — unvollständiges match (non-exhaustive)

```
error[E0004]: non-exhaustive patterns: `Rolle::Assistent` not covered
```

**Bedeutung:** Ein `match` über ein `enum` deckt nicht alle möglichen Varianten ab. Rust
erzwingt Vollständigkeit bewusst — das ist einer der Fälle, in denen der Compiler einen
Bug verhindert, der in anderen Sprachen erst zur Laufzeit auffiele (siehe
[Der Compiler als Lehrer](../01-grundlagen/05-der-compiler-als-lehrer.md)).

**Typischer Fix:** Den fehlenden Fall explizit ergänzen, oder — falls wirklich gewünscht —
einen Catch-all-Zweig `_ => { ... }` hinzufügen (mit Bedacht: Ein neuer `enum`-Fall bleibt
dann unbemerkt).

### E0382 — use of moved value

```
error[E0382]: use of moved value: `inhalt`
```

**Bedeutung:** Ein Wert wurde bereits per Ownership-Übergabe (*move*) an eine andere
Stelle übergeben (z. B. an `Nachricht::neu(rolle, inhalt)`) und danach erneut benutzt,
obwohl der ursprüngliche Besitzer den Wert nicht mehr besitzt.

**Typischer Fix:** Den Wert vor der Übergabe klonen (`inhalt.clone()`), falls er danach
noch gebraucht wird — oder die Reihenfolge der Verwendung so ändern, dass die letzte
Nutzung zuerst passiert.

### E0433 / "cannot find type/value" bei falschem Import

```
error[E0433]: failed to resolve: use of undeclared type `Konversation`
```

**Bedeutung:** Ein Typ oder eine Funktion wird benutzt, ohne zuvor mit `use` importiert
worden zu sein — häufig direkt nach dem Hinzufügen eines neuen Typs in `mein_core`, wenn
`mein_cli` noch den alten `use`-Block hat.

**Typischer Fix:** Die passende `use`-Zeile ergänzen, z. B.
`use mein_core::{Konversation, Rolle};`.

### "unused variable" / "unused import" (Warning)

```
warning: unused variable: `ergebnis`
```

**Bedeutung:** Kein Fehler, aber ein Hinweis, dass eine Variable oder ein Import nicht
gebraucht wird — oft ein Zeichen für vergessenen Code oder einen Tippfehler beim
Variablennamen.

**Typischer Fix:** Variable tatsächlich nutzen, entfernen, oder bewusst mit einem
führenden Unterstrich markieren (`_ergebnis`), falls sie absichtlich ungenutzt bleibt (z. B.
in einem Testaufbau).

---

> **⚠️ Warnung**
>
> Keine dieser Fehlermeldungen sollte "einfach ignoriert" oder mit `.unwrap()`/`#[allow(...)]`
> weggedrückt werden, nur um wieder kompilieren zu können. Der Compiler zeigt hier fast immer
> auf einen echten Gedankenfehler im Entwurf — genau das macht ihn zum "Lehrer" statt zum
> Hindernis.
