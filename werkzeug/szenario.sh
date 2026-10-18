#!/bin/bash
# Läuft IN der isolierten labwc-Sitzung (isoliert.py). Prüft die Querencia
# Dateisuche mit virtuellen Eingaben, Bildschirmfotos, dem Prüfprotokoll
# (QD_PROTOKOLL) und Dateivergleichen – nur im künstlichen Testbaum.
set -u
. "$HMP04_PROFILE/env.sh"
mkdir -p "$OUT"
R=$OUT/ergebnis.txt; : >"$R"
ok() { echo "PASS $*" | tee -a "$R"; }
nok() { echo "FAIL $*" | tee -a "$R"; }
. "$(dirname "$0")/eingabe.sh"
shot() { grim "$OUT/$1.png"; }
export QD_PROTOKOLL=$OUT/protokoll.txt
INI=$PROFIL/config/querencia/dateisuche.ini
OEFFNER=$PROFIL/oeffner.log
APP=""
running() { [ -n "$APP" ] && kill -0 "$APP" 2>/dev/null; }
# Start mit Argumenten; erste Zeigerbewegung „wärmt“ den virtuellen Zeiger
# (der erste Klick nach einem Start geht sonst verloren).
start() {
    : >"$QD_PROTOKOLL"
    "$BIN" "$@" >>"$OUT/app.log" 2>&1 &
    APP=$!
    sleep 3
    q "move 640 420" "wait 300" "move 700 450" "wait 200" "click" "wait 300"
}
stop_app() { if running; then kill "$APP" 2>/dev/null; sleep 1; fi; APP=""; }
# Suche über das Namensfeld auslösen und auf FERTIG warten.
suche() {
    : >"$QD_PROTOKOLL"
    q "move 712 122" "click" "wait 200" "chord ctrl a" "key BackSpace" "type $1" "key Return"
    warte_fertig
}
warte_fertig() {
    local i
    for i in $(seq 1 60); do grep -q '^FERTIG' "$QD_PROTOKOLL" 2>/dev/null && { sleep 0.4; return 0; }; sleep 0.25; done
    return 1
}
# Kontextmenü an Zeile (y) öffnen und Eintrag anklicken; Abstände vom
# Klickpunkt: Öffnen 21, Öffnen mit 57, Ordner 106, Pfad 142, Papierkorb 191, Speichern 240.
kontext() { q "move 350 $1" "rclick" "wait 1000" "move 500 $(($1 + $2))" "click"; }
treffer() { grep '^TREFFER ' "$QD_PROTOKOLL" | sed 's/^TREFFER //' | xargs -r -n1 basename | LC_ALL=C sort | tr '\n' ' ' | sed 's/ $//'; }
status() { grep '^FERTIG' "$QD_PROTOKOLL" | tail -1 | sed 's/^FERTIG //'; }

bash "$(dirname "$0")/testbaum.sh" "$D" >"$OUT/testbaum.log" 2>&1
: >"$OEFFNER"; : >"$OUT/app.log"
rm -f "$INI"

# T01 Start
start --ordner "$D"
running && ok "T01 Start mit --ordner, Fenster offen" || nok "T01 Start"
shot 01-start

# T02 Namenssuche ohne Platzhalter: enthält, Groß/Klein egal, keine versteckten/Sicherungen
suche NOTIZ
shot 02-namenssuche
[ "$(treffer)" = "notiz.txt notiz2.txt" ] && ok "T02 Namenssuche „NOTIZ“: notiz.txt notiz2.txt (ohne notiz.txt~, .versteckt)" || nok "T02 Namenssuche: $(treffer)"
echo "$(status)" | grep -q "1 nicht lesbar" && ok "T02b gesperrter Ordner gezählt: „$(status)“" || nok "T02b Status: $(status)"

# T03 Platzhalter
suche '*.md'
[ "$(treffer)" = "bericht.md" ] && ok "T03 Platzhalter *.md: bericht.md" || nok "T03 Platzhalter: $(treffer)"
suche 'd0?-00?.txt'
[ "$(treffer | wc -w)" = 100 ] && ok "T03b Platzhalter d0?-00?.txt: 100 Treffer" || nok "T03b: $(treffer | wc -w)"

# T04 Textsuche über die GUI: Option hinzufügen, Text eingeben, Return
q "move 410 200" "click" "wait 600"; shot 04a-optionen-offen
q "move 924 238" "click" "wait 600"; shot 04b-option-text
: >"$QD_PROTOKOLL"
q "move 712 122" "click" "wait 200" "chord ctrl a" "key BackSpace" "move 606 232" "click" "wait 200" "type QUERENCIA" "key Return"
warte_fertig; shot 04c-textsuche
[ "$(treffer)" = "alt.txt bericht.md notiz.txt verknuepfung.txt" ] && ok "T04 Textsuche „QUERENCIA“: alt.txt bericht.md notiz.txt verknuepfung.txt (bild.png/gross.bin binär, unlesbar.txt ohne Recht)" || nok "T04 Textsuche: $(treffer)"
echo "$(status)" | grep -q "2 übersprungen" && echo "$(status)" | grep -q "2 nicht lesbar" && ok "T04b Status zählt übersprungen und nicht lesbar: „$(status)“" || nok "T04b Status: $(status)"
q "move 969 232" "click" "wait 500"; shot 04d-option-entfernt
suche notiz
[ "$(treffer)" = "notiz.txt notiz2.txt" ] && ok "T04c Option entfernt, Namenssuche wieder ohne Text" || nok "T04c: $(treffer)"
q "move 410 200" "click" "wait 400"

# T05 Konfiguration speichern beim Schließen
q "move 824 624" "click" "wait 1500"
if ! running && [ -f "$INI" ] && grep -q '^\[Fenster\]' "$INI"; then ok "T05 Schließen schreibt dateisuche.ini ($(grep -c . "$INI") Zeilen)"; else nok "T05 Konfiguration"; fi
cp "$INI" "$OUT/dateisuche-nach-schliessen.ini" 2>/dev/null; stop_app

# T06 Kriterien aus der Konfiguration: Größe, Datum, leer, nur Ordner
preset() { printf '[Fenster]\nBreite=720\nHoehe=560\n[Optionen]\nWeitereSichtbar=%s\n[Kriterien]\n%s\n[Textsuche]\nMaxMB=10\n' "$1" "$2" >"$INI"; }
preset true '1=groesse_min=1048576'
start --ordner "$D" --suchen; warte_fertig; shot 06a-groesse-mindestens
[ "$(treffer)" = "gross.bin" ] && ok "T06a Größe mindestens 1 MB (aus ini): gross.bin" || nok "T06a: $(treffer)"
stop_app
preset false '1=geaendert_mehr=30'
start --ordner "$D" --suchen; warte_fertig
[ "$(treffer)" = "alt.txt" ] && ok "T06b Geändert vor mehr als 30 Tagen: alt.txt" || nok "T06b: $(treffer)"
stop_app
preset false '1=geaendert_weniger=1
2=groesse_max=100'
start --ordner "$D" --suchen; warte_fertig
t=" $(treffer) "
case "$t" in *" leer.txt "*" notiz2.txt "*" unter "*) case "$t" in *alt.txt*|*gross.bin*|*d00-000.txt*) nok "T06c: $t" ;; *) ok "T06c Weniger als 1 Tag und höchstens 100 Byte: $(echo "$t" | wc -w) Treffer, ohne alt.txt, gross.bin, d*-*.txt" ;; esac ;; *) nok "T06c: $t" ;; esac
stop_app
preset false '1=leer'
start --ordner "$D" --suchen; warte_fertig
[ "$(treffer)" = "leer.txt" ] && ok "T06d Datei ist leer: leer.txt" || nok "T06d: $(treffer)"
stop_app
preset false '1=nur_ordner
2=versteckte'
start --ordner "$D" --name "" --suchen; warte_fertig
t=$(treffer); case "$t" in *".versteckt"*unter*viele*) ok "T06e Nur Ordner + versteckte: $(echo "$t" | wc -w) Ordner inkl. .versteckt" ;; *) nok "T06e: $t" ;; esac
stop_app
preset false '1=regex=^d1[0-9]-00[0-4]\\.txt$'
start --ordner "$D" --suchen; warte_fertig
[ "$(treffer | wc -w)" = 50 ] && ok "T06f Regulärer Ausdruck: 50 Treffer" || nok "T06f: $(treffer | wc -w)"
stop_app
preset false '1=besitzer=root'
start --ordner "$D" --suchen; warte_fertig
[ -z "$(treffer)" ] && ok "T06g Besitzer root: keine Treffer im Testbaum" || nok "T06g: $(treffer)"
stop_app
preset false '1=besitzer=gibt-es-nicht-xyz'
start --ordner "$D" --suchen; sleep 1; shot 06h-unbekannter-benutzer
running && ! grep -q '^FERTIG' "$QD_PROTOKOLL" && ok "T06h Unbekannter Benutzer: Fehlerdialog, keine Suche (Bild 06h)" || nok "T06h"
q "key Escape" "wait 500"; stop_app

# T07 Stopp während der Suche (Textsuche über 13 × 30 000 Dateien, Verknüpfungen folgen)
preset false '1=text=inhalt
2=verknuepfungen'
start --ordner "$D"
: >"$QD_PROTOKOLL"
q "chord alt s" "wait 250" "move 937 624" "click" "wait 150"; shot 07-stopp
warte_fertig
echo "$(status)" | grep -q "abgebrochen" && ok "T07 Stopp bricht ab: „$(status)“" || nok "T07 Stopp: $(status)"
# T18 Escape hält Suche an
: >"$QD_PROTOKOLL"
q "move 937 624" "click" "wait 300"; shot 18a-suche-laeuft
q "key Escape" "wait 300"; shot 18b-nach-escape
warte_fertig
echo "$(status)" | grep -q "abgebrochen" && ok "T18 Escape hält laufende Suche an: „$(status)“" || nok "T18: $(status)"
stop_app
rm -f "$INI"

# T08 Doppelklick, Eingabetaste, Kontextmenü, Strg+C
start --ordner "$D"
suche notiz
: >"$OEFFNER"
q "move 350 297" "dclick" "wait 1500"
grep -q "GEOEFFNET $D/notiz.txt" "$OEFFNER" && grep -q "^OEFFNEN $D/notiz.txt" "$QD_PROTOKOLL" && ok "T08 Doppelklick öffnet mit Standardanwendung (GIO): notiz.txt" || nok "T08 Doppelklick: $(cat "$OEFFNER")"
: >"$OEFFNER"
q "move 350 331" "click" "wait 300" "key Return" "wait 1500"
grep -q "GEOEFFNET $D/unter/notiz2.txt" "$OEFFNER" && ok "T09 Eingabetaste öffnet: notiz2.txt" || nok "T09 Eingabetaste: $(cat "$OEFFNER")"
: >"$OEFFNER"
q "move 350 331" "rclick" "wait 1200"; shot 10a-kontextmenue
q "key Escape" "wait 400"; kontext 331 106; q "wait 1500"
grep -q "GEOEFFNET $D/unter$" "$OEFFNER" && grep -q "^ORDNER $D/unter$" "$QD_PROTOKOLL" && ok "T10 Kontextmenü „Enthaltenden Ordner öffnen“: $D/unter" || nok "T10 Ordner öffnen: $(cat "$OEFFNER")"
q "move 350 297" "click" "wait 300" "chord ctrl c" "wait 800"
[ "$(wl-paste 2>/dev/null)" = "$D/notiz.txt" ] && ok "T11 Strg+C kopiert Pfad in die Zwischenablage" || nok "T11 Strg+C: $(wl-paste 2>/dev/null | head -c 100)"
kontext 331 142; q "wait 800"
[ "$(wl-paste 2>/dev/null)" = "$D/unter/notiz2.txt" ] && ok "T11b Kontextmenü „Pfad kopieren“" || nok "T11b: $(wl-paste 2>/dev/null | head -c 100)"
# Öffnen mit …
: >"$OEFFNER"
kontext 297 57; q "wait 1500"; shot 12a-oeffnen-mit
q "key Return" "wait 1500"
grep -q "GEOEFFNET $D/notiz.txt" "$OEFFNER" && ok "T12 „Öffnen mit …“ startet die gewählte Anwendung (Bild 12a)" || nok "T12 Öffnen mit: $(cat "$OEFFNER")"
running && ok "T12b läuft nach „Öffnen mit“" || nok "T12b abgestürzt"

# T13 Papierkorb mit Rückfrage: Abbrechen, dann Bestätigen (GIO trash, kein rm)
q "move 350 297" "click" "wait 300" "key Delete" "wait 1200"; shot 13a-papierkorb-rueckfrage
q "key Escape" "wait 800"
[ -f "$D/notiz.txt" ] && grep -q "^PAPIERKORB-ABGEBROCHEN $D/notiz.txt" "$QD_PROTOKOLL" && ok "T13 Papierkorb-Rückfrage mit Escape abgebrochen, Datei bleibt" || nok "T13 Abbruch"
q "key Delete" "wait 1200" "chord alt p" "wait 1500"; shot 13b-nach-papierkorb
if [ ! -e "$D/notiz.txt" ] && [ -f "$PROFIL/data/Trash/files/notiz.txt" ] && grep -q "^PAPIERKORB $D/notiz.txt" "$QD_PROTOKOLL"; then ok "T13b In den Papierkorb verschoben (liegt in Trash/files, nicht gelöscht)"; else nok "T13b Papierkorb: $(ls "$PROFIL/data/Trash/files" 2>&1)"; fi
mv "$PROFIL/data/Trash/files/notiz.txt" "$D/notiz.txt" 2>/dev/null; rm -f "$PROFIL/data/Trash/info/notiz.txt.trashinfo"

# T14 Sortierung + Ergebnisse speichern (gespeicherte Reihenfolge = Listenreihenfolge)
suche notiz
q "move 682 266" "click" "wait 600"; shot 14a-sortiert-groesse
kontext 297 240; q "wait 1500"; shot 14b-speichern-dialog
q "key Return" "wait 1500"
if [ -f "$D/suchergebnisse.txt" ] && [ "$(head -1 "$D/suchergebnisse.txt")" = "$D/unter/notiz2.txt" ] && [ "$(wc -l <"$D/suchergebnisse.txt")" = 2 ]; then ok "T14 Sortierung nach Größe und „Ergebnisse speichern“: notiz2.txt (11 Byte) zuerst"; else nok "T14 Speichern: $(cat "$D/suchergebnisse.txt" 2>&1)"; fi
q "move 682 266" "click" "wait 400"; kontext 297 240; q "wait 1500" "key Return" "wait 1200"
shot 14c-ueberschreiben
q "chord alt r" "wait 1000"
[ "$(head -1 "$D/suchergebnisse.txt")" = "$D/notiz.txt" ] && ok "T14b absteigend sortiert, Überschreiben bestätigt (GTK-Dialog, englisch: _Replace)" || nok "T14b: $(head -1 "$D/suchergebnisse.txt" 2>&1)"

# T15 Ordnerdialog: Abbrechen, dann Ordner „unter“ wählen
q "move 712 162" "click" "wait 2000"; shot 15a-ordnerdialog
q "key Escape" "wait 1000"
grep -q '^ORDNERWAHL' "$QD_PROTOKOLL" && nok "T15 Escape wählte" || ok "T15 Ordnerdialog mit Escape abgebrochen"
# Ordner über die Pfadzeile (Strg+L) wählen; Dialoglage ist nicht fest.
# (Return in der Pfadzeile wählt den Ordner; danach hat der Ordnerknopf den
# Fokus – ein weiteres Return würde den Dialog erneut öffnen.)
q "move 712 162" "click" "wait 2000" "chord ctrl l" "wait 400" "type $D/unter/" "wait 600" "key Return" "wait 1500"; shot 15b-ordner-unter
grep -q "^ORDNERWAHL $D/unter$" "$QD_PROTOKOLL" && ok "T15b Ordner „unter“ gewählt" || nok "T15b: $(grep ORDNERWAHL "$QD_PROTOKOLL")"
suche notiz
[ "$(treffer)" = "notiz2.txt" ] && ok "T15c Suche im gewählten Ordner: nur notiz2.txt" || nok "T15c: $(treffer)"

# T16 Hilfe und Info: Info mit Tastaturfokus schließen, danach Taste (Symbolabsturz-Muster), dreimal
for i in 1 2 3; do
    q "move 711 624" "click" "wait 1000" "move 774 571" "click" "wait 1000"
    [ $i = 1 ] && shot 16a-info
    q "key Return" "wait 700" "key Escape" "wait 700" "key Down" "key Up" "wait 400"
done
shot 16b-nach-info
running && ok "T16 Info-Dialog 3× geöffnet, mit Eingabetaste geschlossen, danach Tasten: läuft" || nok "T16 Absturz nach Info"
# Hilfe mit Schließen-Knopf
q "move 711 624" "click" "wait 1000"; shot 16c-hilfe; q "move 894 571" "click" "wait 700" "key Down" "wait 300"
running && ok "T16b Hilfe geschlossen, läuft" || nok "T16b"

# T17 Auflösungen 1024×768 und 1920×1080
"$QA" 1024x768 2>>"$OUT/app.log"; sleep 1.5; shot 17a-1024x768
"$QA" 1920x1080 2>>"$OUT/app.log"; sleep 1.5; shot 17b-1920x1080
"$QA" 1280x720 2>>"$OUT/app.log"; sleep 1
running && ok "T17 Auflösungswechsel 1024×768 / 1920×1080 überstanden (Bilder 17a/17b)" || nok "T17"

# T19 Schließen beendet das Programm
q "move 824 624" "click" "wait 1500"
running && nok "T19 Schließen beendet nicht" || ok "T19 Schließen beendet das Programm"
stop_app

grep -i -E 'critical|warning|panicked' "$OUT/app.log" | grep -v "libEGL\|Vulkan\|vkGet\|Gdk-WARNING.*vk" | head -20 >"$OUT/gtk-meldungen.txt"
[ -s "$OUT/gtk-meldungen.txt" ] && echo "HINW GTK-Meldungen: $(wc -l <"$OUT/gtk-meldungen.txt")" | tee -a "$R"
echo "Ergebnis: $(grep -c '^PASS' "$R") PASS, $(grep -c '^FAIL' "$R") FAIL" | tee -a "$R"
touch "$HMP04_PROFILE/logs/FERTIG"
