# Testen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                       # 31 Unit-Tests, ohne Display
python3 werkzeug/i18n.py pruefen          # .pot aktuell, de.po vollständig, Platzhalter, Zugriffstasten
```

Die Unit-Tests laufen ohne Katalog, erwarten also die englischen Texte
(msgid); `german_catalog_is_complete` prüft, dass jede msgid in `po/de.po`
übersetzt ist.

GUI nur isoliert (eigener headless labwc 0.9.8, eigener D-Bus, eigenes
XDG-Profil, künstlicher Testbaum unter `gui-test/profil/baum`, virtuelle
Eingaben mit `qeingabe`; nie die laufende Sitzung). Die Prüfwerkzeuge
`qeingabe` und `qausgabe` liegen gebaut unter `werkzeug/bin/`; neu bauen:

```bash
cd werkzeug && gcc -O2 -o bin/qeingabe qeingabe.c bin/virtual-keyboard-unstable-v1-protocol.c \
    bin/wlr-virtual-pointer-unstable-v1-protocol.c -Ibin $(pkg-config --cflags --libs wayland-client xkbcommon) && \
    gcc -O2 -o bin/qausgabe qausgabe.c bin/wlr-output-management-unstable-v1-protocol.c -Ibin \
    $(pkg-config --cflags --libs wayland-client) && cd ..
```

`qeingabe`/`qausgabe` verweigern außerhalb eines `qed-runtime-*`-Ordners.

**Regression** (37 Fälle, ca. 4 min): Ab 0.1.2 ist die FERTIG-Zeile des
Prüfprotokolls übersetzt und das Szenario bedient deutsche Zugriffstasten,
also nur mit `LANGUAGE=de` (Vorgabe) und einer Binary mit deutschem Katalog
daneben (`<ordner>/share/locale/de/LC_MESSAGES/querencia-dateisuche.mo`):

```bash
QD_BIN=<ordner>/querencia-dateisuche bash werkzeug/gui-pruefung.sh szenario.sh
```

**Sprache** (`szenario-sprache.sh`, je Lauf ca. 1 min): Katalog eingeblendet
(`/proc/<pid>/maps`), Statuszeile, weitere Optionen, Auswahlliste,
Kontextmenü, Papierkorb-Rückfrage, Hilfe, Info, Schließen – nur über die
Zugriffstasten der Sprache:

```bash
QD_BIN=<ordner>/querencia-dateisuche QD_LANGUAGE=de QD_GROESSE=1024x768 \
    bash werkzeug/gui-pruefung.sh szenario-sprache.sh
QD_BIN=<ordner>/querencia-dateisuche QD_LANGUAGE=en QD_GROESSE=1920x1080 \
    bash werkzeug/gui-pruefung.sh szenario-sprache.sh
```

Ergebnis in `nachweis/gui/ergebnis.txt`, Bilder und Prüfprotokoll
(`QD_PROTOKOLL`) daneben. Das Programm schreibt mit `QD_PROTOKOLL=<datei>`
je Suche und Aktion eine Zeile – nur für die Prüfung. Die Läufe
überschreiben die eingecheckten Belege unter `nachweis/gui/`; danach
`git checkout -- nachweis/ && git clean -fq nachweis/gui/`.
