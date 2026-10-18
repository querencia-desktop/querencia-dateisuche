#!/bin/bash
set -u
. "$HMP04_PROFILE/env.sh"
export QD_PROTOKOLL=$OUT/protokoll.txt; : >"$QD_PROTOKOLL"
"$BIN" --ordner "$D" --name notiz --suchen >"$OUT/erkunden-app.log" 2>&1 &
P=$!
sleep 5
echo "PID $P"; for t in /proc/$P/task/*; do echo "$(cat $t/comm) wchan=$(cat $t/wchan) stat=$(cut -d' ' -f3,14,15 $t/stat)"; done
cat "$QD_PROTOKOLL"
sleep 10
echo "--- nach 15 s"; for t in /proc/$P/task/*; do echo "$(cat $t/comm) wchan=$(cat $t/wchan) stat=$(cut -d' ' -f3,14,15 $t/stat)"; done
cat "$QD_PROTOKOLL"
touch "$HMP04_PROFILE/logs/FERTIG"
