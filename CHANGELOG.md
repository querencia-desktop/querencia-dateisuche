# Änderungen

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
- Doku nach ENTSCHEIDUNGEN Nr. 17 (`docs/building.md`, `docs/testing.md`,
  Arbeitsregeln), Lizenzkopf (Nr. 14 Q5).

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
