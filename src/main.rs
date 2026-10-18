//! Querencia Dateisuche – „Nach Dateien suchen …“ für Querencia Desktop.
//!
//! Vorbild ist gnome-search-tool (GNOME 2): ein Dialog mit „Name enthält“,
//! „Suchen in Ordner“, aufklappbaren weiteren Optionen und der
//! Ergebnisliste; gesucht wird ohne Indexdienst direkt im Dateisystem.
//! GTK 4 ohne libadwaita und ohne eigene Titelleiste, damit labwc die
//! klassische Server-Titelleiste zeichnet.
//!
//! Aufruf: `querencia-dateisuche [--ordner PFAD] [--name MUSTER] [--suchen]`
//! Jeder Aufruf öffnet ein eigenes Fenster (wie das Original).

mod dialoge;
mod fenster;
mod format;
mod konfig;
mod kriterien;
mod protokoll;
mod suche;
mod treffer;

use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{gio, glib};

pub const APP_ID: &str = "org.querencia.Dateisuche";
pub const APP_NAME: &str = "Querencia Dateisuche";
pub const FENSTERTITEL: &str = "Nach Dateien suchen";

/// Ausgewertete Befehlszeile.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Aufruf {
    pub ordner: Option<PathBuf>,
    pub name: Option<String>,
    pub suchen: bool,
    pub version: bool,
    pub hilfe: bool,
    pub fehler: Option<String>,
}

pub fn aufruf_auswerten<S: AsRef<str>>(argumente: &[S]) -> Aufruf {
    let mut a = Aufruf::default();
    let mut i = 1; // 0 = Programmname
    while i < argumente.len() {
        let arg = argumente[i].as_ref();
        let wert = |a: &mut Aufruf, i: &mut usize| -> Option<String> {
            *i += 1;
            match argumente.get(*i) {
                Some(w) => Some(w.as_ref().to_string()),
                None => {
                    a.fehler = Some(format!("Option „{arg}“ braucht einen Wert."));
                    None
                }
            }
        };
        match arg {
            "--ordner" | "--path" => {
                if let Some(w) = wert(&mut a, &mut i) {
                    a.ordner = Some(PathBuf::from(w));
                }
            }
            "--name" | "--named" => {
                if let Some(w) = wert(&mut a, &mut i) {
                    a.name = Some(w);
                }
            }
            "--suchen" | "--start" => a.suchen = true,
            "--version" => a.version = true,
            "--hilfe" | "--help" | "-h" => a.hilfe = true,
            s if s.starts_with("--ordner=") => a.ordner = Some(PathBuf::from(&s[9..])),
            s if s.starts_with("--name=") => a.name = Some(s[7..].to_string()),
            s => {
                a.fehler = Some(format!("Unbekannte Option „{s}“."));
                break;
            }
        }
        i += 1;
    }
    a
}

pub const HILFETEXT: &str = "Aufruf: querencia-dateisuche [OPTIONEN]\n\
  --ordner PFAD   Ordner, in dem gesucht wird (Vorgabe: Persönlicher Ordner)\n\
  --name MUSTER   Vorbelegung für „Name enthält“ (* und ? erlaubt)\n\
  --suchen        Suche sofort starten\n\
  --version       Version ausgeben\n\
  --hilfe         diese Hilfe\n";

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE | gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| {
        dialoge::register_window_icon();
        fenster::load_css();
    });
    app.connect_command_line(|app, cmd| {
        let argumente: Vec<String> = cmd
            .arguments()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let aufruf = aufruf_auswerten(&argumente);
        if let Some(f) = &aufruf.fehler {
            eprint!("querencia-dateisuche: {f}\n{HILFETEXT}");
            return glib::ExitCode::from(2);
        }
        if aufruf.version {
            println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
            return glib::ExitCode::SUCCESS;
        }
        if aufruf.hilfe {
            print!("{HILFETEXT}");
            return glib::ExitCode::SUCCESS;
        }
        let f = fenster::Fenster::neu(app, &aufruf);
        f.present();
        if aufruf.suchen {
            f.suchen();
        }
        glib::ExitCode::SUCCESS
    });
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn befehlszeile() {
        let a = aufruf_auswerten(&["prog", "--ordner", "/tmp", "--name", "*.txt", "--suchen"]);
        assert_eq!(a.ordner, Some(PathBuf::from("/tmp")));
        assert_eq!(a.name.as_deref(), Some("*.txt"));
        assert!(a.suchen && a.fehler.is_none());
        let a = aufruf_auswerten(&["prog", "--ordner=/x", "--name=y"]);
        assert_eq!(a.ordner, Some(PathBuf::from("/x")));
        assert_eq!(a.name.as_deref(), Some("y"));
        assert!(aufruf_auswerten(&["prog", "--ordner"]).fehler.is_some());
        assert!(aufruf_auswerten(&["prog", "--quatsch"]).fehler.is_some());
        assert!(aufruf_auswerten(&["prog", "--version"]).version);
        assert!(aufruf_auswerten(&["prog"]).fehler.is_none());
    }
}
