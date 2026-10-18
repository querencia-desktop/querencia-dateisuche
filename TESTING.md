# Testen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                       # 27 Unit-Tests, ohne Display
```

GUI nur isoliert (eigener headless labwc 0.9.8, eigener D-Bus, eigenes
XDG-Profil, künstlicher Testbaum unter `gui-test/profil/baum`, virtuelle
Eingaben; nie die laufende Sitzung):

```bash
bash werkzeug/gui-pruefung.sh szenario.sh release   # oder debug
```

Ergebnis in `nachweis/gui/ergebnis.txt` (erwartet `37 PASS, 0 FAIL`), Bilder
und Prüfprotokoll (`QD_PROTOKOLL`) daneben. Das Programm schreibt mit
`QD_PROTOKOLL=<datei>` je Suche und Aktion eine Zeile – nur für die Prüfung.
