# Querencia Dateisuche

„Nach Dateien suchen …“ für alle, die Querencia Desktop benutzen: Dateien
nach Name, Inhalt, Datum, Größe und Besitzer finden, ohne Indexdienst. Teil
von [Querencia Desktop](https://querencialinux.org) –
einer klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust,
GTK 4).
Installiert wird es zusammen mit Querencia Desktop aus der APT-Quelle;
Anleitung auf <https://querencialinux.org>.

*English summary:* A "Search for Files" tool without an indexing service,
modelled on GNOME 2's gnome-search-tool. Part of Querencia Desktop, a classic
GNOME-2-style desktop for Wayland. Alpha software; documentation is in German.

> **Alpha (Prototyp).** Die Suche läuft jedes Mal frisch über das
> Dateisystem. Keine Sicherheits- oder
> Datenverlustgarantie.

## Zweck

- Name enthält (`*`, `?`; Groß/Klein egal), Suchen in Ordner (klassischer
  GTK-Ordnerdialog), weitere Optionen: Enthält den Text, Datum geändert vor
  weniger/mehr als N Tagen, Größe mindestens/höchstens, Datei ist leer,
  Besitzer, Gruppe, regulärer Ausdruck, versteckte und Sicherungsdateien,
  Verknüpfungen folgen, andere Dateisysteme, nur Ordner.
- Suche im Hintergrund (`walkdir`), Stopp/Escape sofort, Fehler werden
  gezählt; „Enthält den Text“ liest nur Textdateien bis 10 MB (einstellbar in
  `~/.config/querencia/dateisuche.ini`, `[Textsuche] MaxMB`).
- Ergebnisliste mit Symbol, Name, Ordner, Größe, Typ, Geändert; Sortierung je
  Spalte; Doppelklick/Eingabetaste öffnet (GIO); Kontextmenü: Öffnen, Öffnen
  mit …, Enthaltenden Ordner öffnen, Pfad kopieren (Strg+C), In den
  Papierkorb verschieben (Entf, mit Rückfrage), Ergebnisse als Textdatei
  speichern.
- Aufruf: `querencia-dateisuche [--ordner PFAD] [--name MUSTER] [--suchen]`.
- Bewusst **nicht** Teil dieses Programms: ein Suchindex oder Hintergrunddienst.

Anwendungskennung (App-ID): `org.querencia.Dateisuche` · Binary:
`querencia-dateisuche` · Debian-Paket: `querencia-dateisuche`

| Datei | Inhalt |
|---|---|
| `src/kriterien.rs`, `src/suche.rs` | Suchkriterien, Filter, Hintergrundsuche; Unit-Tests |
| `src/fenster.rs`, `src/treffer.rs`, `src/dialoge.rs` | Fenster, Ergebnisliste, Rückfragen |
| `src/konfig.rs`, `src/format.rs`, `src/protokoll.rs` | INI-Datei, Anzeigeformate, Prüfprotokoll (`QD_PROTOKOLL`) |
| `data/` | Desktop-Datei, Symbol (nur SVG), Logo, eingebettete Ressource |
| `werkzeug/` | isolierte GUI-Prüfung (siehe docs/testing.md) |

## Stand

| | |
|---|---|
| Version | 0.1.6 (Tag `dateisuche/0.1.6`; Italienisch, Spanisch, Französisch); seit 0.1.5 Info-Dialog mit Alpha-Hinweis, Website, Copyright und Lizenz, Kurzhilfe mit Online-Hilfe und „Fehler melden“); seit 0.1.4 folgt es einem dunklen GTK-Thema |
| Letzte Prüfung | 26.10.2026 (0.1.6): fmt und Clippy sauber, 33 Unit-Tests, `i18n.py pruefen` und `msgfmt --check` für de/es/fr/it; isoliert 1024×768: `szenario-sprache.sh` de/en/it/es/fr je 7 PASS (Statuszeile, Optionen, Kontextmenü, Papierkorb-Rückfrage, Hilfe, Info); Regression `szenario.sh` de 36/37 wie die Live-Binary 0.1.5 (T19 im Prüfprofil). 22.10.2026 (0.1.5): fmt und Clippy sauber, 33 Unit-Tests, `i18n.py pruefen`; isoliert Deutsch/Englisch 1024×768: Kurzhilfe → Online-Hilfe, „Fehler melden“ (je über die Zugriffstaste) und Info-Dialog. 21.10.2026 (0.1.4): isoliert 28 Läufe (de/en, 1024×768 und 1920×1080, hell und dunkel, beim Start und beim Wechsel), Kontrast dunkel WCAG AA |
| Bekannte Lücken | siehe „Alpha“ |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, Rust 1.93 |
| Nicht getestet | andere Distributionen, HiDPI, Mehrmonitor, sehr große Verzeichnisbäume |

Änderungen: [CHANGELOG.md](CHANGELOG.md).

## Bauen

Voraussetzungen (Ubuntu 26.04):

```bash
sudo apt install build-essential pkg-config rustc cargo libgtk-4-dev
```

```bash
cargo build --release --locked
```

Das Ergebnis liegt unter `target/release/querencia-dateisuche`. Programmsymbol
nur als SVG (`data/icons/hicolor/scalable`), kein 48×48 (GTK-4.22-
Symbolabsturz); das Fenstersymbol ist dasselbe SVG als GResource.

## Prüfen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 werkzeug/i18n.py pruefen
```

Diese Befehle laufen ohne Bildschirm; die ersten drei sind auch die CI
(`.github/workflows/ci.yml`), die po-Prüfung braucht gettext und läuft lokal. Die **GUI-Prüfung** läuft in einer
abgeschotteten Sitzung (eigener labwc ohne Bildschirm, eigener D-Bus, eigenes
`XDG_RUNTIME_DIR` und XDG-Profil, künstlicher Testbaum), niemals in der
laufenden Desktopsitzung. Sie braucht die Eingabehilfen `qeingabe`/`qausgabe`
in `werkzeug/bin/`. Sie liegen nicht in diesem Repo; Quelltext und
Bauanleitung gleichnamiger Werkzeuge stehen in `querencia-systemmonitor`
(`werkzeug/qeingabe.c`, `werkzeug/qausgabe.c`, dort docs/testing.md):

```bash
bash werkzeug/gui-pruefung.sh szenario.sh release
```

Fälle, Ablauf und Belege: [docs/testing.md](docs/testing.md). Arbeitsweise insgesamt:
[CONTRIBUTING.md](CONTRIBUTING.md).

## Sprachen

Seit 0.1.2 sind die Texte englisch im Code (msgid) und deutsch in
`po/de.po` (GNU gettext, Muster Querencia Editor 0.2.0); der deutsche
Wortlaut ist der bisherige. Seit 0.1.6 gibt es dazu Italienisch, Spanisch
und Französisch (`po/it.po`, `po/es.po`, `po/fr.po`). Der englische Wortschatz folgt
gnome-search-tool 2.x („Search for Files“, „Name contains“, „Look in
folder“, „Select more options“). Übersetzt sind Fenster, Optionen,
Kontextmenü, Statuszeile, Rückfragen, Hilfe, Info und die Desktop-Datei
(`data/org.querencia.Dateisuche.desktop.in` → `…desktop`). Die Sprache folgt
`LANGUAGE`/`LANG`; ohne passenden Katalog erscheint Englisch. Dezimalkomma
und Tausenderpunkt bleiben in allen Sprachen; das Datum ist englisch
`2026-10-20 18:30`, deutsch `20.10.2026 18:30`. Befehlszeile (`--ordner`,
`--name`), `--hilfe` und Meldungen auf stderr bleiben deutsch.

## Vorbild und Abgrenzung

Vorbild für Bedienung und Aussehen ist **gnome-search-tool aus GNOME 2**.
Querencia übernimmt davon die Bedienideen (Dialogaufbau, Kriterien, Menüs,
Kürzel), **keinen Quellcode, keine Texte, keine Grafiken**. Das Projekt ist
mit gnome-search-tool, GNOME und deren Autoren weder verbunden noch von ihnen
unterstützt.

Abweichungen vom Vorbild und Dinge, die bewusst fehlen:

- GTK 4 statt GTK 2, Fensterrahmen vom Compositor (labwc), kein libadwaita.
- Kein `locate`-Index; „Enthält den Text“ mit Größengrenze.
- Der Eintrag steht im Querencia-Panel unter „Zubehör“ (GNOME 2: „Orte“).

Fremdbestandteile: Rust-Abhängigkeiten (gtk4-rs und Umfeld, `walkdir`,
`regex`), Liste mit `cargo metadata --locked`; dynamisch gelinkt: GTK 4 und
seine Systembibliotheken.

## Lizenz

Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE). Copyright 2026
Querencia Desktop contributors; Urheberhinweis in [COPYRIGHT](COPYRIGHT).

Name und Logo „Querencia“ sind keine freien Inhalte: [MARKE.md](MARKE.md).

## Fehler melden

Fehler, Wünsche und Fragen zu Querencia gehen an die gemeinsame Meldestelle:
<https://github.com/querencia-desktop/querencia/issues/new/choose>.
Sicherheitslücken bitte nicht öffentlich melden, siehe [SECURITY.md](SECURITY.md).
