#!/bin/bash
# Läuft IN der isolierten labwc-Sitzung (isoliert.py). Sprache der Querencia
# Dateisuche 0.1.2 (G2-AH) in LANGUAGE (gui-pruefung.sh QD_LANGUAGE):
# eingeblendeter Katalog (/proc/<pid>/maps), Statuszeile im Prüfprotokoll,
# Hauptfenster mit Ergebnissen, weitere Optionen mit Auswahlliste,
# hinzugefügte Option, Kontextmenü, Papierkorb-Rückfrage (abgebrochen),
# Hilfe und Info, Schließen. Nur Tastatur über die Zugriffstasten der
# Sprache, damit die Bedienung nicht von der Breite der Beschriftungen
# abhängt. Auflösung aus QD_GROESSE (Standard 1280x720). Gesucht wird nur in
# einem kleinen Testbaum im Wegwerfprofil.
set -u
. "$HMP04_PROFILE/env.sh"
mkdir -p "$OUT"
R=$OUT/ergebnis.txt; : >"$R"
ok() { echo "PASS $*" | tee -a "$R"; }
nok() { echo "FAIL $*" | tee -a "$R"; }
. "$(dirname "$0")/eingabe.sh"
SPRACHE=${LANGUAGE%%:*}
SPRACHE=${SPRACHE%%_*}
shot() { grim "$OUT/$1-$SPRACHE.png"; }
export QD_PROTOKOLL=$OUT/protokoll-$SPRACHE.txt
APP=""
running() { [ -n "$APP" ] && kill -0 "$APP" 2>/dev/null; }
warte_fertig() {
    local i
    for i in $(seq 1 60); do grep -q '^FERTIG' "$QD_PROTOKOLL" 2>/dev/null && { sleep 0.4; return 0; }; sleep 0.25; done
    return 1
}
status() { grep '^FERTIG' "$QD_PROTOKOLL" | tail -1 | sed 's/^FERTIG //'; }

GROESSE=${QD_GROESSE:-1280x720}
"$QA" "$GROESSE" >"$OUT/qausgabe.log" 2>&1 || true
sleep 1
echo "INFO LANG=$LANG LANGUAGE=$LANGUAGE Ausgang $GROESSE" >>"$R"

# Kleiner Testbaum: zwei Treffer, ein gesperrter Ordner (Zusatz „nicht lesbar“)
chmod -R u+rwX "$D" 2>/dev/null || true
rm -rf "$D"
mkdir -p "$D/unter" "$D/gesperrt"
printf 'Erste Notiz\n' >"$D/notiz.txt"
printf 'notiz zwei\n' >"$D/unter/notiz2.txt"
printf 'drin\n' >"$D/gesperrt/drin.txt"; chmod 000 "$D/gesperrt"
rm -f "$PROFIL/config/querencia/dateisuche.ini"

: >"$QD_PROTOKOLL"
"$BIN" --ordner "$D" --name notiz --suchen >"$OUT/app-$SPRACHE.log" 2>&1 &
APP=$!
sleep 3
warte_fertig || true
running && ok "Start mit --suchen" || nok "Start"
KAT=$(grep -o '[^ ]*querencia-dateisuche.mo' "/proc/$APP/maps" | head -1)
echo "INFO Katalog: ${KAT:-keiner}" >>"$R"
echo "INFO Status: $(status)" >>"$R"
case $SPRACHE in
de)
    [ -n "$KAT" ] && ok "de: Katalog eingeblendet" || nok "de: kein Katalog eingeblendet"
    [ "$(status)" = "2 Dateien gefunden · 1 nicht lesbar (keine Berechtigung)" ] && ok "de: Statuszeile „$(status)“" || nok "de: Statuszeile „$(status)“"
    K_OPT=w; K_HINZU=z; K_LISTE=u; K_INFO=i
    ;;
it | es | fr)
    case $SPRACHE in
    it) SOLL="2 file trovati · non leggibili: 1 (nessun permesso)"; K_OPT=o; K_VERF=d; K_HINZU=a; K_LISTE=r; K_INFO=i; K_HILFE=i; K_ZU=h ;;
    es) SOLL="2 archivos encontrados · no legibles: 1 (sin permiso)"; K_OPT=o; K_VERF=d; K_HINZU=a; K_LISTE=r; K_INFO=a; K_HILFE=u; K_ZU=e ;;
    fr) SOLL="2 fichiers trouvés · illisibles\u202f: 1 (pas d’autorisation)"; SOLL=$(printf "$SOLL"); K_OPT=o; K_VERF=p; K_HINZU=a; K_LISTE=r; K_INFO=p; K_HILFE=e; K_ZU=f ;;
    esac
    [ -n "$KAT" ] && ok "$SPRACHE: Katalog eingeblendet" || nok "$SPRACHE: kein Katalog eingeblendet"
    [ "$(status)" = "$SOLL" ] && ok "$SPRACHE: Statuszeile „$(status)“" || nok "$SPRACHE: Statuszeile „$(status)“"
    ;;
*)
    [ -z "$KAT" ] && ok "$SPRACHE: kein Katalog eingeblendet" || nok "$SPRACHE: Katalog eingeblendet"
    [ "$(status)" = "2 files found · 1 not readable (no permission)" ] && ok "$SPRACHE: Statuszeile „$(status)“" || nok "$SPRACHE: Statuszeile „$(status)“"
    K_OPT=o; K_HINZU=a; K_LISTE=e; K_INFO=a
    ;;
esac
shot s1-ergebnisse

# Weitere Optionen aufklappen, Auswahlliste öffnen, erste Option hinzufügen
q "chord alt $K_OPT" "wait 800"; shot s2-optionen
q "chord alt ${K_VERF:-v}" "wait 400" "key Return" "wait 900"; shot s3-auswahlliste
q "key Escape" "wait 500"
q "chord alt $K_HINZU" "wait 800"; shot s4-option-hinzugefuegt
running && ok "weitere Optionen und Auswahlliste (Bilder s2–s4)" || nok "Absturz bei den Optionen"

# Ergebnisliste: Kontextmenü (Menütaste), Papierkorb-Rückfrage abbrechen
q "chord alt $K_LISTE" "wait 400" "key Home" "wait 300" "key Menu" "wait 900"; shot s5-kontextmenue
q "key Escape" "wait 500"
q "key Delete" "wait 1200"; shot s6-papierkorb-rueckfrage
q "key Escape" "wait 800"
if [ -f "$D/notiz.txt" ] && [ -f "$D/unter/notiz2.txt" ] && grep -q '^PAPIERKORB-ABGEBROCHEN' "$QD_PROTOKOLL"; then
    ok "Kontextmenü und Papierkorb-Rückfrage abgebrochen, Dateien bleiben (Bilder s5, s6)"
else
    nok "Papierkorb-Rückfrage: $(grep PAPIERKORB "$QD_PROTOKOLL")"
fi

# Hilfe und Info
q "chord alt ${K_HILFE:-h}" "wait 1000"; shot s7-hilfe
q "chord alt $K_INFO" "wait 1000"; shot s8-info
q "key Escape" "wait 700" "key Escape" "wait 700"
running && ok "Hilfe und Info geöffnet und geschlossen (Bilder s7, s8)" || nok "Absturz bei Hilfe/Info"

# Schließen über die Zugriffstaste (de „S_chließen“, en „_Close“, it „C_hiudi“, es „C_errar“, fr „_Fermer“)
q "chord alt ${K_ZU:-c}" "wait 2000"
running && nok "Schließen beendet nicht" || ok "Schließen (Alt+${K_ZU:-c}) beendet das Programm"
running && kill "$APP" 2>/dev/null
chmod -R u+rwX "$D" 2>/dev/null || true
echo "Ergebnis $GROESSE $SPRACHE: $(grep -c '^PASS' "$R") PASS, $(grep -c '^FAIL' "$R") FAIL" | tee -a "$R"
touch "$HMP04_PROFILE/logs/FERTIG"
