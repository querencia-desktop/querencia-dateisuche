#!/bin/bash
# Isolierte GUI-Prüfung der Querencia Dateisuche (G2-T). Nur Wegwerfprofil,
# eigener headless labwc, eigener D-Bus, eigenes XDG-Profil (Konfiguration,
# Daten, Papierkorb); die laufende Sitzung bleibt unberührt. Gesucht wird
# nur im künstlichen Testbaum unter gui-test/profil/baum.
#   bash werkzeug/gui-pruefung.sh [szenario.sh] [release|debug]
set -euo pipefail
E=$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)
PROFIL=$E/gui-test/profil
[ -d "$E/gui-test" ] && [ "$(dirname "$PROFIL")" = "$E/gui-test" ] && chmod -R u+rwX "$PROFIL" 2>/dev/null && rm -rf "$PROFIL"
mkdir -p "$PROFIL/data/applications" "$PROFIL/config/querencia" "$E/nachweis/gui" "$E/laufzeit"
cp "$E/data/org.querencia.Dateisuche.desktop" "$PROFIL/data/applications/"
cp -r "$E/data/icons" "$PROFIL/data/icons"
# Testöffner: die „Standardanwendung“ für Text und Ordner im Profil
# schreibt nur den übergebenen Pfad in ein Protokoll (beweist GIO-Start).
cat >"$PROFIL/data/applications/qd-testoeffner.desktop" <<X
[Desktop Entry]
Type=Application
Name=Testöffner
Exec=/bin/sh -c 'printf "GEOEFFNET %s\\n" "\$1" >> "$PROFIL/oeffner.log"' sh %f
NoDisplay=true
MimeType=text/plain;inode/directory;text/markdown;
X
cat >"$PROFIL/config/mimeapps.list" <<X
[Default Applications]
text/plain=qd-testoeffner.desktop
text/markdown=qd-testoeffner.desktop
inode/directory=qd-testoeffner.desktop
X
ART=${2:-release}
BIN=$E/target/$ART/querencia-dateisuche
[ -x "$BIN" ] || { echo "Binary fehlt: $BIN" >&2; exit 2; }
cat >"$PROFIL/env.sh" <<X
BIN=$BIN
Q=$E/werkzeug/bin/qeingabe
QA=$E/werkzeug/bin/qausgabe
OUT=$E/nachweis/gui
D=$PROFIL/baum
PROFIL=$PROFIL
X
SZ=${1:-szenario.sh}
python3 -I "$E/werkzeug/isoliert.py" "$PROFIL" "$E/werkzeug/$SZ" 420
cp "$PROFIL"/logs/*.log "$E/nachweis/gui/" 2>/dev/null || true
cat "$E/nachweis/gui/ergebnis.txt" 2>/dev/null || true
