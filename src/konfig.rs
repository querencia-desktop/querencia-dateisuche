//! `~/.config/querencia/dateisuche.ini`: Fenstergröße, Sichtbarkeit der
//! weiteren Optionen, zuletzt gewählte Kriterien, Größenschwelle der
//! Textsuche. Gelesen über `glib::user_config_dir()` (also `XDG_CONFIG_HOME`),
//! geschrieben nur beim Schließen des Fensters.

use std::path::PathBuf;

use gtk::glib;

use crate::kriterien::Kriterium;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Konfig {
    pub breite: i32,
    pub hoehe: i32,
    pub weitere_sichtbar: bool,
    pub kriterien: Vec<Kriterium>,
    /// Schwelle für „Enthält den Text“ in MB.
    pub text_max_mb: u32,
    pub letzter_ordner: Option<String>,
}

impl Default for Konfig {
    fn default() -> Self {
        Konfig {
            breite: 720,
            hoehe: 560,
            weitere_sichtbar: false,
            kriterien: Vec::new(),
            text_max_mb: 10,
            letzter_ordner: None,
        }
    }
}

pub fn pfad() -> PathBuf {
    glib::user_config_dir()
        .join("querencia")
        .join("dateisuche.ini")
}

impl Konfig {
    pub fn laden() -> Konfig {
        let kf = glib::KeyFile::new();
        match kf.load_from_file(pfad(), glib::KeyFileFlags::NONE) {
            Ok(()) => Konfig::aus_keyfile(&kf),
            Err(_) => Konfig::default(),
        }
    }

    pub fn aus_keyfile(kf: &glib::KeyFile) -> Konfig {
        let v = Konfig::default();
        let i32_oder = |gruppe: &str, k: &str, vorgabe: i32| {
            kf.integer(gruppe, k)
                .ok()
                .filter(|n| *n > 0)
                .unwrap_or(vorgabe)
        };
        Konfig {
            breite: i32_oder("Fenster", "Breite", v.breite),
            hoehe: i32_oder("Fenster", "Hoehe", v.hoehe),
            weitere_sichtbar: kf
                .boolean("Optionen", "WeitereSichtbar")
                .unwrap_or(v.weitere_sichtbar),
            kriterien: (1..=200)
                .map_while(|i| kf.string("Kriterien", &i.to_string()).ok())
                .filter_map(|s| Kriterium::aus_text(&s))
                .collect(),
            text_max_mb: kf
                .integer("Textsuche", "MaxMB")
                .ok()
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n > 0)
                .unwrap_or(v.text_max_mb),
            letzter_ordner: kf
                .string("Suche", "Ordner")
                .ok()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty()),
        }
    }

    pub fn in_keyfile(&self, kf: &glib::KeyFile) {
        kf.set_comment(
            None,
            None,
            " Querencia Dateisuche – wird beim Schließen des Fensters geschrieben",
        )
        .ok();
        kf.set_integer("Fenster", "Breite", self.breite);
        kf.set_integer("Fenster", "Hoehe", self.hoehe);
        kf.set_boolean("Optionen", "WeitereSichtbar", self.weitere_sichtbar);
        // Ein Schlüssel je Kriterium (1, 2, …), weil Werte ';' enthalten dürfen.
        for (i, k) in self.kriterien.iter().enumerate() {
            kf.set_string("Kriterien", &(i + 1).to_string(), &k.als_text());
        }
        kf.set_integer(
            "Textsuche",
            "MaxMB",
            i32::try_from(self.text_max_mb).unwrap_or(10),
        );
        kf.set_string(
            "Suche",
            "Ordner",
            self.letzter_ordner.as_deref().unwrap_or(""),
        );
    }

    pub fn speichern(&self) -> Result<(), String> {
        let ziel = pfad();
        if let Some(ordner) = ziel.parent() {
            std::fs::create_dir_all(ordner).map_err(|e| format!("{}: {e}", ordner.display()))?;
        }
        let kf = glib::KeyFile::new();
        self.in_keyfile(&kf);
        kf.save_to_file(&ziel)
            .map_err(|e| format!("{}: {e}", ziel.display()))
    }

    pub fn text_max_bytes(&self) -> u64 {
        u64::from(self.text_max_mb) * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hin_und_zurueck() {
        let k = Konfig {
            breite: 800,
            hoehe: 600,
            weitere_sichtbar: true,
            kriterien: vec![
                Kriterium::EnthaeltText("a;b=c".into()),
                Kriterium::GeaendertWenigerAls(3),
                Kriterium::NurOrdner,
            ],
            text_max_mb: 25,
            letzter_ordner: Some("/tmp".into()),
        };
        let kf = glib::KeyFile::new();
        k.in_keyfile(&kf);
        assert_eq!(Konfig::aus_keyfile(&kf), k);
        let text = kf.to_data();
        assert!(text.contains("[Fenster]"));
        assert!(text.contains("[Kriterien]"));
        assert!(text.contains("1=text=a;b=c"));
    }

    #[test]
    fn leere_und_kaputte_datei_ergeben_vorgaben() {
        let kf = glib::KeyFile::new();
        assert_eq!(Konfig::aus_keyfile(&kf), Konfig::default());
        kf.load_from_data(
            "[Fenster]\nBreite=-5\n[Kriterien]\n1=unbekannt\n2=leer\n[Textsuche]\nMaxMB=0\n",
            glib::KeyFileFlags::NONE,
        )
        .unwrap();
        let k = Konfig::aus_keyfile(&kf);
        assert_eq!(k.breite, Konfig::default().breite);
        assert_eq!(k.kriterien, vec![Kriterium::DateiIstLeer]);
        assert_eq!(k.text_max_mb, 10);
    }
}
