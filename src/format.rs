//! Anzeigeformate der Ergebnisliste und der Statuszeile: Größen in
//! Byte/KB/MB/GB mit Dezimalkomma, Datum je Sprache (deutsch
//! „TT.MM.JJJJ HH:MM“), Zählung „N Dateien gefunden“. Dezimalkomma und
//! Tausenderpunkt bleiben in jeder Sprache (Entscheidung wie im Rechner;
//! Trennzeichen nach Region statt Sprache ist QD-056).

use gtk::glib;

use crate::i18n::{gettext, ngettext_f};

/// Größe wie in Nautilus 2: ganze Byte, darüber eine Nachkommastelle
/// (Basis 1024), deutsches Komma.
pub fn groesse(bytes: u64) -> String {
    const STUFEN: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        // TRANSLATORS: Dateigröße unter 1 KB.
        return ngettext_f("{n} byte", "{n} bytes", bytes as usize, &[]);
    }
    let mut wert = bytes as f64 / 1024.0;
    let mut stufe = 0;
    while wert >= 1024.0 && stufe + 1 < STUFEN.len() {
        wert /= 1024.0;
        stufe += 1;
    }
    let text = if wert >= 100.0 {
        format!("{wert:.0}")
    } else {
        format!("{wert:.1}")
    };
    format!("{} {}", text.replace('.', ","), STUFEN[stufe])
}

/// „2026-10-18 18:30“, deutsch „18.10.2026 18:30“ (Ortszeit des Aufrufers).
pub fn datum(zeit: &glib::DateTime) -> String {
    // TRANSLATORS: Spalte „Date Modified“, Format von g_date_time_format.
    zeit.format(&gettext("%Y-%m-%d %H:%M"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Unix-Sekunden in Ortszeit; unlesbare Zeiten ergeben einen leeren Text.
pub fn datum_unix(sekunden: i64) -> String {
    glib::DateTime::from_unix_local(sekunden)
        .map(|z| datum(&z))
        .unwrap_or_default()
}

/// Statuszeile: „Keine Dateien gefunden“, „1 Datei gefunden“, „12 Dateien gefunden“.
pub fn gefunden(anzahl: usize) -> String {
    if anzahl == 0 {
        return gettext("No files found");
    }
    // TRANSLATORS: {count} mit Tausenderpunkten, z. B. „1.234“.
    ngettext_f(
        "{count} file found",
        "{count} files found",
        anzahl,
        &[("count", &tausender(anzahl))],
    )
}

/// Tausenderpunkte: 1234567 → „1.234.567“.
pub fn tausender(n: usize) -> String {
    let ziffern = n.to_string();
    let mut aus = String::new();
    for (i, z) in ziffern.chars().enumerate() {
        if i > 0 && (ziffern.len() - i).is_multiple_of(3) {
            aus.push('.');
        }
        aus.push(z);
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groessen() {
        // Ohne Katalog englisch (msgid), deutsch siehe po/de.po.
        assert_eq!(groesse(0), "0 bytes");
        assert_eq!(groesse(1), "1 byte");
        assert_eq!(groesse(1023), "1023 bytes");
        assert_eq!(groesse(1024), "1,0 KB");
        assert_eq!(groesse(1536), "1,5 KB");
        assert_eq!(groesse(10 * 1024 * 1024), "10,0 MB");
        assert_eq!(groesse(250 * 1024 * 1024), "250 MB");
        assert_eq!(
            groesse(3 * 1024 * 1024 * 1024 + 400 * 1024 * 1024),
            "3,4 GB"
        );
        assert_eq!(groesse(2_u64.pow(40) * 5), "5,0 TB");
    }

    #[test]
    fn datumsformat() {
        let z = glib::DateTime::from_unix_utc(0).unwrap();
        assert_eq!(datum(&z), "1970-01-01 00:00");
        let z = glib::DateTime::from_utc(2026, 10, 18, 18, 30, 7.0).unwrap();
        assert_eq!(datum(&z), "2026-10-18 18:30");
    }

    #[test]
    fn zaehlung() {
        assert_eq!(gefunden(0), "No files found");
        assert_eq!(gefunden(1), "1 file found");
        assert_eq!(gefunden(2), "2 files found");
        assert_eq!(gefunden(1234567), "1.234.567 files found");
        assert_eq!(tausender(999), "999");
        assert_eq!(tausender(1000), "1.000");
    }
}
