# Querencia Dateisuche

„Nach Dateien suchen …“ für Querencia Desktop, nach dem Vorbild von
gnome-search-tool (GNOME 2), ohne Indexdienst. Rust + GTK 4 (gtk-rs 0.11),
kein libadwaita, keine eigene Titelleiste (labwc-Serverdekoration).
App-ID `org.querencia.Dateisuche`.

> **Alpha.** Sucht ohne Indexdienst direkt im Dateisystem. Keine Sicherheits-
> oder Datenverlustgarantie.

## Stand

| | |
|---|---|
| Version | 0.1.4 (Tag `dateisuche/0.1.4`, Integration `dateisuche/0.1.4-int`; folgt einem dunklen GTK-Thema, hell wie 0.1.3) |
| Auf der Entwicklungs-VM | live 0.1.3 seit 20.10.2026; 0.1.4 wartet auf Freigabe und die CSS-Welle dunkel |
| Letzte Prüfung | 21.10.2026: fmt und Clippy sauber, 32/32 Unit-Tests; isoliert 28 Läufe (de/en, 1024×768/1920×1080, hell/dunkel, Start und Wechsel), hell pixelgleich mit 0.1.3, Kontrast dunkel AA |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22 |

Änderungen je Version: [CHANGELOG.md](CHANGELOG.md).

## Funktionen

- Name enthält (`*`, `?`; Groß/Klein egal), Suchen in Ordner (klassischer
  GTK-Ordnerdialog), weitere Optionen: Enthält den Text, Datum geändert vor
  weniger/mehr als N Tagen, Größe mindestens/höchstens, Datei ist leer,
  Besitzer, Gruppe, regulärer Ausdruck, versteckte und Sicherungsdateien,
  Verknüpfungen folgen, andere Dateisysteme, nur Ordner.
- Suche im Hintergrund-Thread (`walkdir`), Stopp/Escape sofort, Fehler werden
  gezählt; „Enthält den Text“ liest nur Textdateien bis 10 MB (einstellbar in
  `~/.config/querencia/dateisuche.ini`, `[Textsuche] MaxMB`).
- Ergebnisliste mit Symbol, Name, Ordner, Größe, Typ, Geändert; Sortierung je
  Spalte; Doppelklick/Eingabetaste öffnet (GIO); Kontextmenü: Öffnen, Öffnen
  mit …, Enthaltenden Ordner öffnen, Pfad kopieren (Strg+C), In den
  Papierkorb verschieben (Entf, mit Rückfrage, `g_file_trash`), Ergebnisse
  speichern als Textdatei.
- Aufruf: `querencia-dateisuche [--ordner PFAD] [--name MUSTER] [--suchen]`.

Programmsymbol nur als SVG (`data/icons/hicolor/scalable`), kein 48×48
(GTK-4.22-Symbolabsturz); Fenstersymbol = dasselbe SVG als GResource.

## Bauen und Prüfen

`cargo build --release --locked` (offline mit dem lokalen Cargo-Cache), danach `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` und `python3 werkzeug/i18n.py pruefen`.
Voraussetzungen, Katalog, Integration und die isolierte GUI-Prüfung: [docs/building.md](docs/building.md), [docs/testing.md](docs/testing.md).

## Sprachen

Seit 0.1.2 sind die Texte englisch im Code (msgid) und deutsch in
`po/de.po` (GNU gettext, Muster Querencia Editor 0.2.0); der deutsche
Wortlaut ist der bisherige. Der englische Wortschatz folgt
gnome-search-tool 2.x („Search for Files“, „Name contains“, „Look in
folder“, „Select more options“). Übersetzt sind Fenster, Optionen,
Kontextmenü, Statuszeile, Rückfragen, Hilfe, Info und die Desktop-Datei
(`data/org.querencia.Dateisuche.desktop.in` → `…desktop`). Die Sprache folgt
`LANGUAGE`/`LANG`; ohne deutschen Katalog erscheint Englisch. Dezimalkomma
und Tausenderpunkt bleiben in beiden Sprachen; das Datum ist englisch
`2026-10-20 18:30`, deutsch `20.10.2026 18:30`. Befehlszeile (`--ordner`,
`--name`), `--hilfe` und Meldungen auf stderr bleiben deutsch.

## Lizenz

GNU GPL, Version 3 oder neuer, siehe [LICENSE](LICENSE) (reiner Lizenztext).
Copyright 2026 Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT). Eigener Code; aus gnome-search-tool wurde
kein Code übernommen. Änderungen: [CHANGELOG.md](CHANGELOG.md).

Eigene Vermerke: `werkzeug/virtual-keyboard-unstable-v1.xml` (gekürzte Kopie
des Wayland-Protokolls, Copyright Kristian Høgsberg u. a., MIT-artige Lizenz
im Dateikopf) und die mit `wayland-scanner` erzeugten Dateien unter
`werkzeug/bin/` (Lizenz der jeweiligen Protokolle); nur für das Prüfwerkzeug.

