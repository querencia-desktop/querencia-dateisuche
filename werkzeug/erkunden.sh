#!/bin/bash
set -u
. "$HMP04_PROFILE/env.sh"
. "$(dirname "$0")/eingabe.sh"
bash "$(dirname "$0")/testbaum.sh" "$D" >"$OUT/testbaum.log" 2>&1
export QD_PROTOKOLL=$OUT/protokoll.txt; : >"$QD_PROTOKOLL"
( "$BIN" --ordner "$D" --name notiz --suchen; echo "EXIT $?" ) >"$OUT/erkunden-app.log" 2>&1 &
sleep 4
q "move 400 297" "rclick" "wait 1200"; grim "$OUT/e02-kontext.png"
q "key Escape" "wait 300" "move 350 331" "rclick" "wait 1200"; grim "$OUT/e03-kontext2.png"
echo "--- protokoll:"; cat "$QD_PROTOKOLL"
touch "$HMP04_PROFILE/logs/FERTIG"
