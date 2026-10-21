# Änderungen

## 0.1.4 – 2026-10-21

Dunkeltauglich (CSS-Welle dunkel, Alpha 2), hell unverändert:

- Menü (Kontextmenü, Auswahllisten), Spaltenköpfe der Ergebnisliste und
  Statuszeile ohne feste Hellfarben: Grund und Rand der Menüs sowie Verlauf
  und Schrift der Spaltenköpfe kommen aus dem GTK-Thema, die Ränder der
  Spaltenköpfe aus `@borders`, die Statuszeile aus `@theme_fg_color`. Mit
  Querencia-Klassisch hell sind es dieselben Werte wie bisher; mit einem
  dunklen Thema folgt die Dateisuche ohne eigenen Dunkelblock. Keine Bindung
  der Programm-CSS an das Farbschema nötig (kein `@media`), daher keine
  Kopplung an ein bestimmtes Thema.
- Beim Start einmal `gtk-interface-color-scheme` melden: GTK 4.22 lädt bei
  „prefer-dark“ das dunkle Thema-CSS sonst nur bei einem Wechsel im
  laufenden Programm.
- Unit-Test gegen feste Hellfarben in `style.css`. Texte, Katalog, Suche und
  Befehlszeile unverändert. Die orange Auswahl (`#f6a179` → `#f07746`,
  Schrift `#1a1a1a`) bleibt fest.
- Spaltenköpfe zeigen beim Überfahren und Drücken jetzt die Hervorhebung des
  Themas (bisher überdeckte die eigene Regel jeden Zustand mit dem
  Grundverlauf).

## 0.1.3 – 2026-10-20

Dunkle Auswahlschrift (CSS-Welle, passend zum GTK-Thema Querencia-Klassisch,
Entscheidung 20.10.):

- Menüeinträge und die ausgewählte Zeile der Ergebnisliste schreiben
  `#1a1a1a` statt `#ffffff` auf den orangen Verlauf. Kontrast 6,2:1 (unten,
  `#f07746`) bis 8,5:1 (oben, `#f6a179`); mit Weiß waren es 2,8:1 bzw.
  2,0:1. Verläufe und alles Übrige unverändert; Texte und Katalog
  unverändert.

## 0.1.2 – 2026-10-20

Englisch und Deutsch über GNU gettext (G2-AH, Muster Querencia Editor 0.2.0):

- Texte englisch im Code (Wortschatz gnome-search-tool 2.x), Deutsch aus
  `po/de.po` mit dem bisherigen Wortlaut: Fenster, 13 Optionen,
  Kontextmenü, Statuszeile, Rückfragen und Fehlermeldungen, „Öffnen mit“,
  Hilfe, Info, Desktop-Datei (englischer Name jetzt „Search for Files…“).
- Datum der Spalte „Geändert“ je Sprache (`%Y-%m-%d %H:%M`, deutsch
  `%d.%m.%Y %H:%M`); Dezimalkomma und Tausenderpunkt unverändert.
- Suche, Kriterien, Konfiguration und Befehlszeile unverändert.
- `werkzeug/i18n.py` für die po-Pflege; GUI-Prüfung mit `QD_LANGUAGE`,
  `QD_BIN`, `QD_GROESSE` und neuem Szenario `szenario-sprache.sh`; Quelltexte
  von `qeingabe`/`qausgabe` liegen jetzt im Repo (bauen byte-gleich nach).
- Doku unter `docs/` (`docs/building.md`, `docs/testing.md`), Lizenzkopf.

## 0.1.1 (2026-10-18)

Eigenes Programmsymbol nach Abnahmebefund des Besitzers: Lupe über einem
Dokument, flächig in Querencia-Orange (#F07746) und Grau, nur als SVG
(`hicolor/scalable`), zugleich eingebettetes Fenstersymbol. Keine PNG-Größen
mehr. Sonst unverändert.

## 0.1.0 (2026-10-18)

Erster Prototyp (G2-T): Dialog nach gnome-search-tool, 13 Kriterien,
Hintergrundsuche mit Stopp, Ergebnisliste mit Kontextmenü, Papierkorb über
GIO, Konfiguration in `dateisuche.ini`, Befehlszeile `--ordner/--name/--suchen`.
Isoliert geprüft: 27 Unit-Tests, GUI 37/37 (zweimal, Release-Build).
