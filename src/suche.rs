//! Die Suche selbst: ein Hintergrund-Thread läuft mit `walkdir` durch den
//! Ordner, prüft jeden Eintrag gegen den [`Filter`] und schickt Treffer
//! über einen Kanal an das Fenster. Stopp wirkt sofort (Prüfung vor jedem
//! Eintrag), Fehler (keine Berechtigung, Verknüpfungsschleifen) werden
//! gezählt, nicht abgebrochen. Gelöscht oder verschoben wird hier nichts.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::SystemTime;

use crate::i18n::gettext_f;

use walkdir::WalkDir;

use crate::kriterien::{Anfrage, Filter, Inhalt, Werte};

/// Ein gefundener Eintrag (ohne GTK, aus dem Thread).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fund {
    pub pfad: PathBuf,
    pub ist_ordner: bool,
    pub groesse: u64,
    /// Änderungszeit in Unix-Sekunden.
    pub geaendert: i64,
}

/// Nachricht aus dem Suchthread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Meldung {
    Treffer(Fund),
    Fertig(Abschluss),
}

/// Zahlen am Ende einer Suche.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Abschluss {
    pub geprueft: u64,
    pub gefunden: u64,
    /// Nicht lesbare Einträge (Berechtigung, Schleife, verschwunden).
    pub fehler: u64,
    /// Für „Enthält den Text“ nicht gelesen (binär oder zu groß).
    pub uebersprungen: u64,
    pub abgebrochen: bool,
}

/// Eine laufende Suche; fällt sie weg, hält der Thread beim nächsten
/// Eintrag an (Kanal geschlossen oder Stopp-Markierung).
pub struct Lauf {
    stopp: Arc<AtomicBool>,
    empfaenger: Receiver<Meldung>,
}

impl Lauf {
    pub fn stoppen(&self) {
        self.stopp.store(true, Ordering::Relaxed);
    }

    /// Bis zu `hoechstens` eingetroffene Meldungen, ohne zu warten; der
    /// Rest bleibt im Kanal (hält das Fenster bei Hunderttausenden
    /// Treffern bedienbar).
    pub fn abholen(&self, hoechstens: usize) -> Vec<Meldung> {
        let mut aus = Vec::new();
        while aus.len() < hoechstens {
            match self.empfaenger.try_recv() {
                Ok(m) => aus.push(m),
                Err(_) => break,
            }
        }
        aus
    }

    /// Wartet auf das Ende (nur für Tests).
    #[cfg(test)]
    pub fn alles(&self) -> Vec<Meldung> {
        let mut aus = Vec::new();
        while let Ok(m) = self.empfaenger.recv() {
            let fertig = matches!(m, Meldung::Fertig(_));
            aus.push(m);
            if fertig {
                break;
            }
        }
        aus
    }
}

impl Drop for Lauf {
    fn drop(&mut self) {
        self.stoppen();
    }
}

/// Prüft die Anfrage und startet den Thread. Fehler in Muster, regulärem
/// Ausdruck oder Benutzer-/Gruppenname kommen sofort zurück.
pub fn starten(anfrage: Anfrage) -> Result<Lauf, String> {
    let filter = Filter::neu(&anfrage)?;
    if !anfrage.ordner.is_dir() {
        return Err(gettext_f(
            "“{folder}” is not a folder or is not readable.",
            &[("folder", &anfrage.ordner.display().to_string())],
        ));
    }
    let stopp = Arc::new(AtomicBool::new(false));
    let (sender, empfaenger) = mpsc::channel();
    let s = stopp.clone();
    thread::Builder::new()
        .name("dateisuche".into())
        .spawn(move || durchlaufen(anfrage, filter, s, sender))
        .map_err(|e| {
            gettext_f(
                "The search thread cannot be started: {error}",
                &[("error", &e.to_string())],
            )
        })?;
    Ok(Lauf { stopp, empfaenger })
}

fn durchlaufen(anfrage: Anfrage, filter: Filter, stopp: Arc<AtomicBool>, sender: Sender<Meldung>) {
    let jetzt = SystemTime::now();
    let mut z = Abschluss::default();
    let laeufer = WalkDir::new(&anfrage.ordner)
        .follow_links(filter.folgen)
        .same_file_system(!filter.andere_dateisysteme)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0 || !e.file_name().to_str().is_some_and(|n| filter.uebergehen(n))
        });
    for eintrag in laeufer {
        if stopp.load(Ordering::Relaxed) {
            z.abgebrochen = true;
            break;
        }
        let eintrag = match eintrag {
            Ok(e) => e,
            Err(_) => {
                z.fehler += 1;
                continue;
            }
        };
        if eintrag.depth() == 0 {
            continue;
        }
        z.geprueft += 1;
        let name = eintrag.file_name().to_string_lossy();
        if !filter.passt_name(&name) {
            continue;
        }
        let md = match eintrag.metadata() {
            Ok(md) => md,
            Err(_) => {
                z.fehler += 1;
                continue;
            }
        };
        let werte = Werte::aus_metadaten(&md, jetzt);
        if !filter.passt_werte(&werte) {
            continue;
        }
        if filter.braucht_inhalt() {
            // Nur reguläre Dateien lesen (auch über eine Verknüpfung); Ordner,
            // Geräte, FIFOs und Sockets würden fehlschlagen oder blockieren.
            let regulaer = md.is_file()
                || (md.file_type().is_symlink()
                    && std::fs::metadata(eintrag.path()).is_ok_and(|m| m.is_file()));
            if !regulaer {
                continue;
            }
            match filter.passt_inhalt(eintrag.path(), werte.groesse) {
                Ok(Inhalt::Treffer) => {}
                Ok(Inhalt::KeinTreffer) => continue,
                Ok(Inhalt::Uebersprungen) => {
                    z.uebersprungen += 1;
                    continue;
                }
                Err(_) => {
                    z.fehler += 1;
                    continue;
                }
            }
        }
        z.gefunden += 1;
        let geaendert = md
            .modified()
            .ok()
            .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs() as i64);
        let fund = Fund {
            pfad: eintrag.into_path(),
            ist_ordner: werte.ist_ordner,
            groesse: werte.groesse,
            geaendert,
        };
        if sender.send(Meldung::Treffer(fund)).is_err() {
            return; // Fenster weg
        }
    }
    let _ = sender.send(Meldung::Fertig(z));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kriterien::Kriterium;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::Path;

    fn baum(name: &str) -> PathBuf {
        let w = std::env::temp_dir().join(format!("qd-suche-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&w);
        fs::create_dir_all(w.join("a/b")).unwrap();
        fs::create_dir_all(w.join(".versteckt")).unwrap();
        fs::write(w.join("a/notiz.txt"), "Hallo Querencia\n").unwrap();
        fs::write(w.join("a/b/tief.md"), "# Tief\n").unwrap();
        fs::write(w.join("a/b/leer.txt"), "").unwrap();
        fs::write(w.join(".versteckt/geheim.txt"), "querencia\n").unwrap();
        fs::write(w.join("sicherung.txt~"), "").unwrap();
        fs::write(
            w.join("bild.png"),
            b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR querencia",
        )
        .unwrap();
        // Verknüpfungsschleife
        symlink(&w, w.join("a/schleife")).unwrap();
        w
    }

    fn anfrage(w: &Path, muster: &str, kriterien: Vec<Kriterium>) -> Anfrage {
        Anfrage {
            muster: muster.into(),
            ordner: w.to_path_buf(),
            kriterien,
            text_max_bytes: 1024 * 1024,
        }
    }

    fn namen(meldungen: &[Meldung]) -> Vec<String> {
        let mut n: Vec<String> = meldungen
            .iter()
            .filter_map(|m| match m {
                Meldung::Treffer(f) => {
                    Some(f.pfad.file_name().unwrap().to_string_lossy().into_owned())
                }
                Meldung::Fertig(_) => None,
            })
            .collect();
        n.sort();
        n
    }

    fn abschluss(meldungen: &[Meldung]) -> Abschluss {
        match meldungen.last() {
            Some(Meldung::Fertig(a)) => *a,
            _ => panic!("kein Abschluss"),
        }
    }

    #[test]
    fn namenssuche_ohne_versteckte() {
        let w = baum("namen");
        let m = starten(anfrage(&w, "*.txt", vec![])).unwrap().alles();
        assert_eq!(namen(&m), ["leer.txt", "notiz.txt"]);
        let a = abschluss(&m);
        assert_eq!(a.gefunden, 2);
        assert!(!a.abgebrochen);
        fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn versteckte_einbeziehen_und_nur_ordner() {
        let w = baum("versteckt");
        let m = starten(anfrage(
            &w,
            "*.txt*",
            vec![Kriterium::VersteckteEinbeziehen],
        ))
        .unwrap()
        .alles();
        assert_eq!(
            namen(&m),
            ["geheim.txt", "leer.txt", "notiz.txt", "sicherung.txt~"]
        );
        let m = starten(anfrage(&w, "", vec![Kriterium::NurOrdner]))
            .unwrap()
            .alles();
        assert_eq!(namen(&m), ["a", "b"]);
        fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn textsuche_ueberspringt_binaer() {
        let w = baum("text");
        let m = starten(anfrage(
            &w,
            "",
            vec![Kriterium::EnthaeltText("QUERENCIA".into())],
        ))
        .unwrap()
        .alles();
        assert_eq!(namen(&m), ["notiz.txt"]);
        assert_eq!(abschluss(&m).uebersprungen, 1, "bild.png ist binär");
        fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn schleife_mit_verknuepfungen_endet() {
        let w = baum("schleife");
        let m = starten(anfrage(&w, "notiz*", vec![Kriterium::VerknuepfungenFolgen]))
            .unwrap()
            .alles();
        let a = abschluss(&m);
        assert!(a.fehler >= 1, "Schleife wird als Fehler gezählt: {a:?}");
        assert_eq!(a.gefunden, 1);
        // ohne Folgen: Verknüpfung ist ein Eintrag, kein Abstieg
        let m = starten(anfrage(&w, "schleife", vec![])).unwrap().alles();
        assert_eq!(namen(&m), ["schleife"]);
        assert_eq!(abschluss(&m).fehler, 0);
        fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn ohne_leserecht_zaehlt_als_fehler() {
        let w = baum("recht");
        fs::create_dir(w.join("gesperrt")).unwrap();
        fs::write(w.join("gesperrt/drin.txt"), "x").unwrap();
        fs::set_permissions(w.join("gesperrt"), fs::Permissions::from_mode(0o000)).unwrap();
        let m = starten(anfrage(&w, "*.txt", vec![])).unwrap().alles();
        let a = abschluss(&m);
        fs::set_permissions(w.join("gesperrt"), fs::Permissions::from_mode(0o755)).unwrap();
        if fs::metadata("/proc/self").is_ok() && std::env::var("USER").as_deref() != Ok("root") {
            assert_eq!(a.fehler, 1, "{a:?}");
        }
        assert_eq!(namen(&m), ["leer.txt", "notiz.txt"]);
        fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn stopp_bricht_ab() {
        let w = baum("stopp");
        let lauf = starten(anfrage(&w, "", vec![])).unwrap();
        lauf.stoppen();
        let m = lauf.alles();
        assert!(matches!(m.last(), Some(Meldung::Fertig(_))));
        fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn kein_ordner_ist_fehler() {
        assert!(starten(anfrage(&PathBuf::from("/gibt/es/nicht"), "", vec![])).is_err());
    }
}
