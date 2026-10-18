#!/bin/sh
# Erzeugt data/resource/dateisuche.gresource aus dateisuche.gresource.xml
# (Bauzeit-Artefakt, eingecheckt, damit der Bau ohne glib-compile-resources
# geht) und die PNG-Programmsymbole aus dem SVG – bewusst OHNE 48x48
# (GTK-4.22-Symbolabsturz bei 48x48 zusammen mit Scalable).
set -eu
cd "$(dirname "$0")/.."
glib-compile-resources --sourcedir=data/resource \
    --target=data/resource/dateisuche.gresource data/resource/dateisuche.gresource.xml
SVG=data/resource/icons/scalable/apps/org.querencia.Dateisuche-fenster.svg
mkdir -p data/icons/hicolor/scalable/apps
cp "$SVG" data/icons/hicolor/scalable/apps/org.querencia.Dateisuche.svg
python3 -I - "$SVG" <<'PY'
import sys, gi
gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf
svg = sys.argv[1]
for s in (16, 22, 24, 32, 64, 128, 256):
    pb = GdkPixbuf.Pixbuf.new_from_file_at_scale(svg, s, s, True)
    d = f"data/icons/hicolor/{s}x{s}/apps"
    import os; os.makedirs(d, exist_ok=True)
    pb.savev(f"{d}/org.querencia.Dateisuche.png", "png", [], [])
    print("ok", s)
PY
