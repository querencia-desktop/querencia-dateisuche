#!/bin/sh
# Erzeugt data/resource/dateisuche.gresource aus dateisuche.gresource.xml
# (Bauzeit-Artefakt, eingecheckt, damit der Bau ohne glib-compile-resources
# geht). Das Programmsymbol ist bewusst NUR ein SVG (hicolor/scalable):
# Vorgabe des Besitzers (0.1.1) und Symbolregel (kein 48x48 neben Scalable,
# GTK-4.22-Symbolabsturz). Fenstersymbol = dasselbe SVG als Ressource.
set -eu
cd "$(dirname "$0")/.."
cp data/icons/hicolor/scalable/apps/org.querencia.Dateisuche.svg \
   data/resource/icons/scalable/apps/org.querencia.Dateisuche-fenster.svg
glib-compile-resources --sourcedir=data/resource \
    --target=data/resource/dateisuche.gresource data/resource/dateisuche.gresource.xml
