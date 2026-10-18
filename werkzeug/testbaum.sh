#!/bin/bash
# Künstlicher Testbaum für die GUI-Prüfung (nie das echte Home):
# verschachtelt, versteckte Dateien, Sicherungsdatei, Verknüpfungsschleife,
# Datei und Ordner ohne Leserecht, große Datei, alte Datei, Binärdatei,
# viele kleine Dateien für den Stopp-Test.
#   bash testbaum.sh ZIEL
set -eu
D=$1
chmod -R u+rwX "$D" 2>/dev/null || true
rm -rf "$D"
mkdir -p "$D/unter/tiefer/noch" "$D/.versteckt" "$D/gesperrt" "$D/bilder" "$D/viele"
printf 'Erste Notiz\nQuerencia Desktop\n' >"$D/notiz.txt"
printf '# Bericht\n\nquerencia dateisuche\n' >"$D/unter/bericht.md"
printf 'tief\n' >"$D/unter/tiefer/noch/tief.txt"
printf 'notiz zwei\n' >"$D/unter/notiz2.txt"
: >"$D/unter/leer.txt"
printf 'geheim\n' >"$D/.versteckt/geheim.txt"
printf 'alt\n' >"$D/notiz.txt~"
printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR querencia' >"$D/bilder/bild.png"
head -c 3145728 /dev/urandom >"$D/gross.bin"
printf 'querencia alt\n' >"$D/alt.txt"; touch -d '45 days ago' "$D/alt.txt"
printf 'ohne leserecht querencia\n' >"$D/unlesbar.txt"; chmod 000 "$D/unlesbar.txt"
printf 'drin\n' >"$D/gesperrt/drin.txt"; chmod 000 "$D/gesperrt"
ln -s "$D" "$D/unter/schleife"
ln -s "$D/notiz.txt" "$D/unter/verknuepfung.txt"
python3 -I - "$D/viele" <<'PY'
import os, sys
w = sys.argv[1]
for i in range(60):
    o = os.path.join(w, f"o{i:02d}")
    os.makedirs(o, exist_ok=True)
    for j in range(500):
        with open(os.path.join(o, f"d{i:02d}-{j:03d}.txt"), "w") as f:
            f.write(f"datei {i} {j} inhalt\n" * 20)
PY
echo "Testbaum: $(find "$D" 2>/dev/null | wc -l) Einträge in $D"
