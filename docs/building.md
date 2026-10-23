# Bauen

Ubuntu 26.04: `cargo`, `rustc` (≥ 1.92, Ubuntu 1.93.1), `pkgconf`,
`libgtk-4-dev`. Das `Cargo.lock` stammt vom Querencia Editor (gtk4 0.11.5,
glib 0.22.10) plus walkdir, regex und gettext-rs, damit der Bau aus dem
lokalen Crate-Cache gelingt:

```bash
cargo build --release --locked --offline
```

Das Release-Binary liegt unter `target/release/querencia-dateisuche`. Für
Pakete und Anhänge mit `RUSTFLAGS="--remap-path-prefix=$HOME=~"` bauen,
damit keine Pfade des Bauordners in der Binary stehen.

**Übersetzungen (ab 0.1.2):** Die Binary sucht
`<lang>/LC_MESSAGES/querencia-dateisuche.mo` in `<Ordner der Binary>/share/locale`,
dann `<Ordner der Binary>/../share/locale`, zuletzt `/usr/share/locale`
(`QUERENCIA_DATEISUCHE_LOCALEDIR` setzt den Ordner ausdrücklich). Den
Katalog erzeugt GNU gettext oder das Werkzeug des Repos:

```bash
msgfmt -c -o share/locale/de/LC_MESSAGES/querencia-dateisuche.mo po/de.po
python3 werkzeug/i18n.py mo <ziel>          # <ziel>/de/LC_MESSAGES/querencia-dateisuche.mo
```

Ohne Katalog ist die Oberfläche englisch. Die Desktop-Datei entsteht aus der
Vorlage: `python3 werkzeug/i18n.py desktop data/org.querencia.Dateisuche.desktop.in data/org.querencia.Dateisuche.desktop`
(entspricht `msgfmt --desktop`).

**Symbol:** nur `data/icons/hicolor/scalable/apps/org.querencia.Dateisuche.svg`,
bewusst keine PNG-Stufen (GTK-4.22-Symbolabsturz bei 48×48 neben Scalable).
Das Fenstersymbol ist zusätzlich unter `org.querencia.Dateisuche-fenster`
als Ressource eingebettet (`werkzeug/ressourcen.sh` erzeugt
`data/resource/dateisuche.gresource`; `werkzeug/symbol.sh` prüft Titelleiste
und Panel-Menü isoliert).

**Integration (nur in der Entwicklungsumgebung, nicht Teil der Veröffentlichung):**
Die Live-Integration in die Querencia-Sitzung der Entwicklungsumgebung erledigen
Skripte unter `integration/` (`pruefen|aktivieren|zurueck`, unveränderliche
Ablage `~/opt/querencia/dateisuche/<sha16>/` mit `share/locale` daneben,
Link `~/opt/querencia/bin/querencia-dateisuche`, Starter). Der Ordner enthält
feste Pfade der VM und wird vor der Veröffentlichung herausgefiltert.
