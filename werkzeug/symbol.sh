#!/bin/bash
# Symbolprüfung 0.1.1 in der isolierten Sitzung: Fenstersymbol in der
# labwc-Titelleiste (SSD) und Eintrag im Panel-Menü (Zubehör) mit dem
# Live-Panel-Binary, nur im Wegwerfprofil.
set -u
. "$HMP04_PROFILE/env.sh"
. "$(dirname "$0")/eingabe.sh"
PANEL=${QUERENCIA_PANEL:-$HOME/opt/querencia/bin/querencia-panel}
"$PANEL" >"$OUT/symbol-panel.log" 2>&1 & PP=$!
sleep 3
"$BIN" --ordner "$D" >"$OUT/symbol-app.log" 2>&1 & AP=$!
sleep 3
q "move 640 420" "wait 300" "move 700 450" "wait 200" "click" "wait 300"
grim "$OUT/s01-fenster-und-panel.png"
q "move ${QD_MENU_X:-60} ${QD_MENU_Y:-14}" "wait 300" "click" "wait 1500"; grim "$OUT/s02-anwendungen-menue.png"
if [ -n "${QD_ZUB_X:-}" ]; then q "move $QD_ZUB_X $QD_ZUB_Y" "wait 800" "click" "wait 1500"; grim "$OUT/s03-zubehoer.png"; fi
q "key Escape" "wait 300"
kill $AP $PP 2>/dev/null
touch "$HMP04_PROFILE/logs/FERTIG"
