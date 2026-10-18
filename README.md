# Querencia Dateisuche

„Nach Dateien suchen …“ für Querencia Desktop, nach dem Vorbild von
gnome-search-tool (GNOME 2), ohne Indexdienst. Rust + GTK 4 (gtk-rs 0.11),
kein libadwaita, keine eigene Titelleiste (labwc-Serverdekoration).
App-ID `org.querencia.Dateisuche`, Prototyp 0.1.0.

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

Bauen: `cargo build --release --locked` (Ubuntu 26.04: cargo, rustc ≥ 1.92,
pkgconf, libgtk-4-dev; offline mit dem lokalen Cargo-Cache). Prüfen: siehe
`TESTING.md`. Symbole bewusst **ohne 48×48** (GTK-4.22-Symbolabsturz).
Lizenz: GPL-3.0-or-later.
