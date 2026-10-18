//! Prüfprotokoll: Ist `QD_PROTOKOLL=<datei>` gesetzt, hängt das Programm
//! je abgeschlossener Suche und je Aktion an Treffern eine Zeile an. Nur
//! für die isolierte GUI-Prüfung; ohne die Variable passiert nichts.

use std::fs::OpenOptions;
use std::io::Write;

pub fn schreiben(zeile: &str) {
    let Ok(pfad) = std::env::var("QD_PROTOKOLL") else {
        return;
    };
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(pfad) {
        let _ = writeln!(f, "{zeile}");
    }
}
