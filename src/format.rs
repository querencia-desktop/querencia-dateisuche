//! Anzeigeformate der Ergebnisliste und der Statuszeile: Größen in
//! Byte/KB/MB/GB mit deutschem Dezimalkomma, Datum als „TT.MM.JJJJ HH:MM“,
//! Zählung „N Dateien gefunden“.

use gtk::glib;

/// Größe wie in Nautilus 2: ganze Byte, darüber eine Nachkommastelle
/// (Basis 1024), deutsches Komma.
pub fn groesse(bytes: u64) -> String {
    const STUFEN: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return if bytes == 1 {
            "1 Byte".to_string()
        } else {
            format!("{bytes} Byte")
        };
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

/// „18.10.2026 18:30“ (Ortszeit des Aufrufers).
pub fn datum(zeit: &glib::DateTime) -> String {
    zeit.format("%d.%m.%Y %H:%M")
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
    match anzahl {
        0 => "Keine Dateien gefunden".to_string(),
        1 => "1 Datei gefunden".to_string(),
        n => format!("{} Dateien gefunden", tausender(n)),
    }
}

/// Tausenderpunkte: 1234567 → „1.234.567“.
pub fn tausender(n: usize) -> String {
    let ziffern = n.to_string();
    let mut aus = String::new();
    for (i, z) in ziffern.chars().enumerate() {
        if i > 0 && (ziffern.len() - i) % 3 == 0 {
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
        assert_eq!(groesse(0), "0 Byte");
        assert_eq!(groesse(1), "1 Byte");
        assert_eq!(groesse(1023), "1023 Byte");
        assert_eq!(groesse(1024), "1,0 KB");
        assert_eq!(groesse(1536), "1,5 KB");
        assert_eq!(groesse(10 * 1024 * 1024), "10,0 MB");
        assert_eq!(groesse(250 * 1024 * 1024), "250 MB");
        assert_eq!(groesse(3 * 1024 * 1024 * 1024 + 400 * 1024 * 1024), "3,4 GB");
        assert_eq!(groesse(2_u64.pow(40) * 5), "5,0 TB");
    }

    #[test]
    fn datumsformat() {
        let z = glib::DateTime::from_unix_utc(0).unwrap();
        assert_eq!(datum(&z), "01.01.1970 00:00");
        let z = glib::DateTime::from_utc(2026, 10, 18, 18, 30, 7.0).unwrap();
        assert_eq!(datum(&z), "18.10.2026 18:30");
    }

    #[test]
    fn zaehlung() {
        assert_eq!(gefunden(0), "Keine Dateien gefunden");
        assert_eq!(gefunden(1), "1 Datei gefunden");
        assert_eq!(gefunden(2), "2 Dateien gefunden");
        assert_eq!(gefunden(1234567), "1.234.567 Dateien gefunden");
        assert_eq!(tausender(999), "999");
        assert_eq!(tausender(1000), "1.000");
    }
}
