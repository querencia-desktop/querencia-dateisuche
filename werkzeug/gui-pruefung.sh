#!/bin/bash
# Isolierte GUI-Prüfung der Querencia Dateisuche (G2-T). Nur Wegwerfprofil,
# eigener headless labwc, eigener D-Bus, eigenes XDG-Profil (Konfiguration,
# Daten, Papierkorb); die laufende Sitzung bleibt unberührt. Gesucht wird
# nur im künstlichen Testbaum unter gui-test/profil/baum.
#   bash werkzeug/gui-pruefung.sh [szenario.sh] [release|debug]
# QD_BIN=<binary> prüft eine andere Binary (z. B. die Vorlage der Integration
# mit share/locale daneben); QD_LANGUAGE=en die englische Oberfläche
# (Standard wie die Sitzung: LANG=en_US.UTF-8, LANGUAGE=de); QD_GROESSE
# (BREITExHÖHE) setzt die Auflösung für szenario-sprache.sh. Ab 0.1.2 ist das
# Prüfprotokoll (FERTIG-Zeile) übersetzt: szenario.sh nur mit LANGUAGE=de und
# einer Binary mit deutschem Katalog (QD_BIN=<vorlage>).
set -euo pipefail
E=$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)
PROFIL=$E/gui-test/profil
[ -d "$E/gui-test" ] && [ "$(dirname "$PROFIL")" = "$E/gui-test" ] && chmod -R u+rwX "$PROFIL" 2>/dev/null && rm -rf "$PROFIL"
mkdir -p "$PROFIL/data/applications" "$PROFIL/config/querencia" "$E/nachweis/gui" "$E/laufzeit"
cp -r "$E/data/icons" "$PROFIL/data/icons"
# Testöffner: die „Standardanwendung“ für Text und Ordner im Profil
# schreibt nur den übergebenen Pfad in ein Protokoll (beweist GIO-Start).
cat >"$PROFIL/data/applications/qd-testoeffner.desktop" <<X
[Desktop Entry]
Type=Application
Name=Testöffner
Exec=/bin/sh -c 'echo "GEOEFFNET \$1" >> "$PROFIL/oeffner.log"' sh %f
NoDisplay=true
MimeType=text/plain;inode/directory;text/markdown;
X
update-desktop-database "$PROFIL/data/applications" 2>/dev/null || true
cat >"$PROFIL/config/mimeapps.list" <<X
[Default Applications]
text/plain=qd-testoeffner.desktop
text/markdown=qd-testoeffner.desktop
inode/directory=qd-testoeffner.desktop
X
ART=${2:-release}
BIN=${QD_BIN:-$E/target/$ART/querencia-dateisuche}
[ -x "$BIN" ] || { echo "Binary fehlt: $BIN" >&2; exit 2; }
# Starter im Profil mit absolutem Exec/TryExec auf das Testbinary (ein
# relatives TryExec außerhalb des PATH blendet GIO den Eintrag aus).
sed -e "s|^Exec=querencia-dateisuche|Exec=$BIN|" -e "s|^TryExec=querencia-dateisuche|TryExec=$BIN|" \
    "$E/data/org.querencia.Dateisuche.desktop" >"$PROFIL/data/applications/org.querencia.Dateisuche.desktop"
update-desktop-database "$PROFIL/data/applications" 2>/dev/null || true
cat >"$PROFIL/env.sh" <<X
BIN=$BIN
Q=$E/werkzeug/bin/qeingabe
QA=$E/werkzeug/bin/qausgabe
OUT=$E/nachweis/gui
D=$PROFIL/baum
PROFIL=$PROFIL
QD_GROESSE=${QD_GROESSE:-}
X
SZ=${1:-szenario.sh}
LANG=en_US.UTF-8 LANGUAGE=${QD_LANGUAGE:-de} \
    python3 -I "$E/werkzeug/isoliert.py" "$PROFIL" "$E/werkzeug/$SZ" "${QD_ZEITLIMIT:-420}"
cp "$PROFIL"/logs/*.log "$E/nachweis/gui/" 2>/dev/null || true
cat "$E/nachweis/gui/ergebnis.txt" 2>/dev/null || true
