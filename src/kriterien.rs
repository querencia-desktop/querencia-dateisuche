//! Suchkriterien wie in „Nach Dateien suchen …“ (gnome-search-tool, GNOME 2):
//! Namensmuster mit `*` und `?` (Groß/Klein ignoriert), dazu wählbare
//! „Weitere Optionen“. Hier steht nur die Logik ohne GTK; die Prüfung je
//! Datei arbeitet auf einfachen Werten ([`Werte`]) und ist dadurch testbar.

use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use regex::Regex;
use regex::bytes;

/// Ein gewähltes Kriterium mit seinem Wert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kriterium {
    /// Datei enthält den Text (Groß/Klein ignoriert, nur Textdateien).
    EnthaeltText(String),
    /// Geändert vor weniger als N Tagen.
    GeaendertWenigerAls(u32),
    /// Geändert vor mehr als N Tagen.
    GeaendertMehrAls(u32),
    /// Größe mindestens N Byte.
    GroesseMindestens(u64),
    /// Größe höchstens N Byte.
    GroesseHoechstens(u64),
    DateiIstLeer,
    /// Besitzer (Name oder numerische ID).
    Besitzer(String),
    /// Gruppe (Name oder numerische ID).
    Gruppe(String),
    /// Name entspricht regulärem Ausdruck (Groß/Klein beachtet).
    NameRegex(String),
    VersteckteEinbeziehen,
    VerknuepfungenFolgen,
    AndereDateisystemeFolgen,
    NurOrdner,
}

/// Art eines Kriteriums (für Auswahlliste und Konfiguration).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    EnthaeltText,
    GeaendertWenigerAls,
    GeaendertMehrAls,
    GroesseMindestens,
    GroesseHoechstens,
    DateiIstLeer,
    Besitzer,
    Gruppe,
    NameRegex,
    VersteckteEinbeziehen,
    VerknuepfungenFolgen,
    AndereDateisystemeFolgen,
    NurOrdner,
}

/// Welches Eingabefeld eine Art braucht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eingabe {
    Text,
    Tage,
    Groesse,
    Keine,
}

impl Art {
    /// Reihenfolge wie im Original-Dialog.
    pub const ALLE: [Art; 13] = [
        Art::EnthaeltText,
        Art::GeaendertWenigerAls,
        Art::GeaendertMehrAls,
        Art::GroesseMindestens,
        Art::GroesseHoechstens,
        Art::DateiIstLeer,
        Art::Besitzer,
        Art::Gruppe,
        Art::NameRegex,
        Art::VersteckteEinbeziehen,
        Art::VerknuepfungenFolgen,
        Art::AndereDateisystemeFolgen,
        Art::NurOrdner,
    ];

    /// Beschriftung in der Auswahlliste.
    pub fn bezeichnung(self) -> &'static str {
        match self {
            Art::EnthaeltText => "Enthält den Text",
            Art::GeaendertWenigerAls => "Datum geändert vor weniger als",
            Art::GeaendertMehrAls => "Datum geändert vor mehr als",
            Art::GroesseMindestens => "Größe mindestens",
            Art::GroesseHoechstens => "Größe höchstens",
            Art::DateiIstLeer => "Datei ist leer",
            Art::Besitzer => "Besitzer",
            Art::Gruppe => "Gruppe",
            Art::NameRegex => "Name entspricht regulärem Ausdruck",
            Art::VersteckteEinbeziehen => "Versteckte und Sicherungsdateien einbeziehen",
            Art::VerknuepfungenFolgen => "Symbolischen Verknüpfungen folgen",
            Art::AndereDateisystemeFolgen => "Anderen Dateisystemen folgen",
            Art::NurOrdner => "Nur Ordner",
        }
    }

    /// Text hinter dem Eingabefeld („Tagen“).
    pub fn nachsatz(self) -> &'static str {
        match self {
            Art::GeaendertWenigerAls | Art::GeaendertMehrAls => "Tagen",
            _ => "",
        }
    }

    pub fn eingabe(self) -> Eingabe {
        match self {
            Art::EnthaeltText | Art::Besitzer | Art::Gruppe | Art::NameRegex => Eingabe::Text,
            Art::GeaendertWenigerAls | Art::GeaendertMehrAls => Eingabe::Tage,
            Art::GroesseMindestens | Art::GroesseHoechstens => Eingabe::Groesse,
            _ => Eingabe::Keine,
        }
    }

    /// Schlüssel in `dateisuche.ini`.
    pub fn schluessel(self) -> &'static str {
        match self {
            Art::EnthaeltText => "text",
            Art::GeaendertWenigerAls => "geaendert_weniger",
            Art::GeaendertMehrAls => "geaendert_mehr",
            Art::GroesseMindestens => "groesse_min",
            Art::GroesseHoechstens => "groesse_max",
            Art::DateiIstLeer => "leer",
            Art::Besitzer => "besitzer",
            Art::Gruppe => "gruppe",
            Art::NameRegex => "regex",
            Art::VersteckteEinbeziehen => "versteckte",
            Art::VerknuepfungenFolgen => "verknuepfungen",
            Art::AndereDateisystemeFolgen => "dateisysteme",
            Art::NurOrdner => "nur_ordner",
        }
    }

    pub fn aus_schluessel(s: &str) -> Option<Art> {
        Art::ALLE.into_iter().find(|a| a.schluessel() == s)
    }

    /// Kriterium mit dem Vorgabewert dieser Art.
    pub fn vorgabe(self) -> Kriterium {
        match self {
            Art::EnthaeltText => Kriterium::EnthaeltText(String::new()),
            Art::GeaendertWenigerAls => Kriterium::GeaendertWenigerAls(7),
            Art::GeaendertMehrAls => Kriterium::GeaendertMehrAls(30),
            Art::GroesseMindestens => Kriterium::GroesseMindestens(100 * 1024),
            Art::GroesseHoechstens => Kriterium::GroesseHoechstens(100 * 1024),
            Art::DateiIstLeer => Kriterium::DateiIstLeer,
            Art::Besitzer => Kriterium::Besitzer(String::new()),
            Art::Gruppe => Kriterium::Gruppe(String::new()),
            Art::NameRegex => Kriterium::NameRegex(String::new()),
            Art::VersteckteEinbeziehen => Kriterium::VersteckteEinbeziehen,
            Art::VerknuepfungenFolgen => Kriterium::VerknuepfungenFolgen,
            Art::AndereDateisystemeFolgen => Kriterium::AndereDateisystemeFolgen,
            Art::NurOrdner => Kriterium::NurOrdner,
        }
    }
}

impl Kriterium {
    pub fn art(&self) -> Art {
        match self {
            Kriterium::EnthaeltText(_) => Art::EnthaeltText,
            Kriterium::GeaendertWenigerAls(_) => Art::GeaendertWenigerAls,
            Kriterium::GeaendertMehrAls(_) => Art::GeaendertMehrAls,
            Kriterium::GroesseMindestens(_) => Art::GroesseMindestens,
            Kriterium::GroesseHoechstens(_) => Art::GroesseHoechstens,
            Kriterium::DateiIstLeer => Art::DateiIstLeer,
            Kriterium::Besitzer(_) => Art::Besitzer,
            Kriterium::Gruppe(_) => Art::Gruppe,
            Kriterium::NameRegex(_) => Art::NameRegex,
            Kriterium::VersteckteEinbeziehen => Art::VersteckteEinbeziehen,
            Kriterium::VerknuepfungenFolgen => Art::VerknuepfungenFolgen,
            Kriterium::AndereDateisystemeFolgen => Art::AndereDateisystemeFolgen,
            Kriterium::NurOrdner => Art::NurOrdner,
        }
    }

    /// Textform für `dateisuche.ini`: `schluessel` oder `schluessel=wert`.
    pub fn als_text(&self) -> String {
        let k = self.art().schluessel();
        match self {
            Kriterium::EnthaeltText(s)
            | Kriterium::Besitzer(s)
            | Kriterium::Gruppe(s)
            | Kriterium::NameRegex(s) => format!("{k}={s}"),
            Kriterium::GeaendertWenigerAls(n) | Kriterium::GeaendertMehrAls(n) => {
                format!("{k}={n}")
            }
            Kriterium::GroesseMindestens(n) | Kriterium::GroesseHoechstens(n) => {
                format!("{k}={n}")
            }
            _ => k.to_string(),
        }
    }

    /// Umkehrung von [`Kriterium::als_text`]; unbekannte oder unlesbare
    /// Einträge ergeben `None`.
    pub fn aus_text(text: &str) -> Option<Kriterium> {
        let (schluessel, wert) = match text.split_once('=') {
            Some((k, w)) => (k, Some(w)),
            None => (text, None),
        };
        let art = Art::aus_schluessel(schluessel.trim())?;
        Some(match (art, wert) {
            (Art::EnthaeltText, Some(w)) => Kriterium::EnthaeltText(w.to_string()),
            (Art::Besitzer, Some(w)) => Kriterium::Besitzer(w.to_string()),
            (Art::Gruppe, Some(w)) => Kriterium::Gruppe(w.to_string()),
            (Art::NameRegex, Some(w)) => Kriterium::NameRegex(w.to_string()),
            (Art::GeaendertWenigerAls, Some(w)) => Kriterium::GeaendertWenigerAls(w.parse().ok()?),
            (Art::GeaendertMehrAls, Some(w)) => Kriterium::GeaendertMehrAls(w.parse().ok()?),
            (Art::GroesseMindestens, Some(w)) => Kriterium::GroesseMindestens(w.parse().ok()?),
            (Art::GroesseHoechstens, Some(w)) => Kriterium::GroesseHoechstens(w.parse().ok()?),
            (a, None) if a.eingabe() == Eingabe::Keine => a.vorgabe(),
            _ => return None,
        })
    }
}

/// Eine vollständige Suchanfrage.
#[derive(Debug, Clone)]
pub struct Anfrage {
    /// „Name enthält“ (leer = alle Namen).
    pub muster: String,
    pub ordner: PathBuf,
    pub kriterien: Vec<Kriterium>,
    /// Größte Datei, die für „Enthält den Text“ gelesen wird.
    pub text_max_bytes: u64,
}

/// Was die Prüfung je Eintrag braucht (aus den Metadaten).
#[derive(Debug, Clone, Copy)]
pub struct Werte {
    pub ist_ordner: bool,
    pub groesse: u64,
    /// Zeit seit der letzten Änderung.
    pub alter: Duration,
    pub uid: u32,
    pub gid: u32,
}

impl Werte {
    pub fn aus_metadaten(md: &fs::Metadata, jetzt: SystemTime) -> Werte {
        let alter = md
            .modified()
            .ok()
            .and_then(|m| jetzt.duration_since(m).ok())
            .unwrap_or(Duration::ZERO);
        Werte {
            ist_ordner: md.is_dir(),
            groesse: if md.is_dir() { 0 } else { md.len() },
            alter,
            uid: md.uid(),
            gid: md.gid(),
        }
    }
}

/// Ergebnis der Inhaltsprüfung einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inhalt {
    Treffer,
    KeinTreffer,
    /// Binärdatei oder größer als die Schwelle: nicht gelesen.
    Uebersprungen,
}

/// Aus einer [`Anfrage`] übersetzte, schnell prüfbare Form.
#[derive(Debug)]
pub struct Filter {
    name: Option<Regex>,
    name_regex: Option<Regex>,
    text: Option<bytes::Regex>,
    text_max: u64,
    weniger_als: Option<Duration>,
    mehr_als: Option<Duration>,
    mindestens: Option<u64>,
    hoechstens: Option<u64>,
    leer: bool,
    uid: Option<u32>,
    gid: Option<u32>,
    pub versteckte: bool,
    pub folgen: bool,
    pub andere_dateisysteme: bool,
    nur_ordner: bool,
}

const TAG: u64 = 24 * 60 * 60;

impl Filter {
    pub fn neu(anfrage: &Anfrage) -> Result<Filter, String> {
        let mut f = Filter {
            name: None,
            name_regex: None,
            text: None,
            text_max: anfrage.text_max_bytes,
            weniger_als: None,
            mehr_als: None,
            mindestens: None,
            hoechstens: None,
            leer: false,
            uid: None,
            gid: None,
            versteckte: false,
            folgen: false,
            andere_dateisysteme: false,
            nur_ordner: false,
        };
        let muster = anfrage.muster.trim();
        if !muster.is_empty() {
            f.name = Some(
                Regex::new(&glob_zu_regex(muster))
                    .map_err(|e| format!("Namensmuster unbrauchbar: {e}"))?,
            );
        }
        for k in &anfrage.kriterien {
            match k {
                Kriterium::EnthaeltText(t) => {
                    if !t.is_empty() {
                        let re = bytes::RegexBuilder::new(&regex::escape(t))
                            .case_insensitive(true)
                            .build()
                            .map_err(|e| format!("Suchtext unbrauchbar: {e}"))?;
                        f.text = Some(re);
                    }
                }
                Kriterium::GeaendertWenigerAls(n) => {
                    f.weniger_als = Some(Duration::from_secs(u64::from(*n) * TAG));
                }
                Kriterium::GeaendertMehrAls(n) => {
                    f.mehr_als = Some(Duration::from_secs(u64::from(*n) * TAG));
                }
                Kriterium::GroesseMindestens(n) => f.mindestens = Some(*n),
                Kriterium::GroesseHoechstens(n) => f.hoechstens = Some(*n),
                Kriterium::DateiIstLeer => f.leer = true,
                Kriterium::Besitzer(b) => {
                    if !b.trim().is_empty() {
                        f.uid = Some(
                            uid_fuer(b.trim())
                                .ok_or_else(|| format!("Unbekannter Benutzer „{}“", b.trim()))?,
                        );
                    }
                }
                Kriterium::Gruppe(g) => {
                    if !g.trim().is_empty() {
                        f.gid = Some(
                            gid_fuer(g.trim())
                                .ok_or_else(|| format!("Unbekannte Gruppe „{}“", g.trim()))?,
                        );
                    }
                }
                Kriterium::NameRegex(r) => {
                    if !r.is_empty() {
                        f.name_regex = Some(
                            Regex::new(r)
                                .map_err(|e| format!("Regulärer Ausdruck unbrauchbar: {e}"))?,
                        );
                    }
                }
                Kriterium::VersteckteEinbeziehen => f.versteckte = true,
                Kriterium::VerknuepfungenFolgen => f.folgen = true,
                Kriterium::AndereDateisystemeFolgen => f.andere_dateisysteme = true,
                Kriterium::NurOrdner => f.nur_ordner = true,
            }
        }
        Ok(f)
    }

    /// Versteckte (`.name`) und Sicherungsdateien (`name~`, `#name#`)
    /// werden ohne die Option übergangen, Ordner samt Inhalt.
    pub fn uebergehen(&self, name: &str) -> bool {
        !self.versteckte && ist_versteckt_oder_sicherung(name)
    }

    pub fn passt_name(&self, name: &str) -> bool {
        self.name.as_ref().is_none_or(|re| re.is_match(name))
            && self.name_regex.as_ref().is_none_or(|re| re.is_match(name))
    }

    pub fn passt_werte(&self, w: &Werte) -> bool {
        if self.nur_ordner && !w.ist_ordner {
            return false;
        }
        if self.leer && (w.ist_ordner || w.groesse != 0) {
            return false;
        }
        if self.mindestens.is_some_and(|m| w.groesse < m) {
            return false;
        }
        if self.hoechstens.is_some_and(|m| w.groesse > m) {
            return false;
        }
        if self.weniger_als.is_some_and(|d| w.alter >= d) {
            return false;
        }
        if self.mehr_als.is_some_and(|d| w.alter <= d) {
            return false;
        }
        if self.uid.is_some_and(|u| w.uid != u) {
            return false;
        }
        if self.gid.is_some_and(|g| w.gid != g) {
            return false;
        }
        true
    }

    pub fn braucht_inhalt(&self) -> bool {
        self.text.is_some()
    }

    /// Liest die Datei (bis zur Schwelle) und sucht den Text. Binärdateien
    /// (Nullbyte oder viele Steuerzeichen im Kopf) werden übersprungen.
    pub fn passt_inhalt(&self, pfad: &Path, groesse: u64) -> io::Result<Inhalt> {
        let Some(re) = &self.text else {
            return Ok(Inhalt::Treffer);
        };
        if groesse > self.text_max {
            return Ok(Inhalt::Uebersprungen);
        }
        let mut datei = fs::File::open(pfad)?;
        let mut kopf = vec![0u8; 8192];
        let n = lesen_voll(&mut datei, &mut kopf)?;
        kopf.truncate(n);
        if ist_binaer(&kopf) {
            return Ok(Inhalt::Uebersprungen);
        }
        let mut daten = kopf;
        if (n as u64) < groesse {
            datei.by_ref().take(self.text_max).read_to_end(&mut daten)?;
        }
        Ok(if re.is_match(&daten) {
            Inhalt::Treffer
        } else {
            Inhalt::KeinTreffer
        })
    }
}

fn lesen_voll(r: &mut impl Read, puffer: &mut [u8]) -> io::Result<usize> {
    let mut gelesen = 0;
    while gelesen < puffer.len() {
        match r.read(&mut puffer[gelesen..]) {
            Ok(0) => break,
            Ok(n) => gelesen += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(gelesen)
}

/// Namensmuster → regulärer Ausdruck. Ohne `*`/`?` gilt „Name enthält“
/// (wie `find -iname "*muster*"`), sonst muss der ganze Name passen.
pub fn glob_zu_regex(muster: &str) -> String {
    let platzhalter = muster.contains(['*', '?']);
    let mut s = String::from("(?i)^");
    if !platzhalter {
        s.push_str(".*");
    }
    for c in muster.chars() {
        match c {
            '*' => s.push_str(".*"),
            '?' => s.push('.'),
            c => s.push_str(&regex::escape(&c.to_string())),
        }
    }
    if !platzhalter {
        s.push_str(".*");
    }
    s.push('$');
    s
}

/// `.name`, `name~`, `#name#` wie in gnome-search-tool.
pub fn ist_versteckt_oder_sicherung(name: &str) -> bool {
    name.starts_with('.')
        || name.ends_with('~')
        || (name.len() > 2 && name.starts_with('#') && name.ends_with('#'))
}

/// Binärdatei: ein Nullbyte, oder mehr als ein Zehntel der Bytes sind
/// Steuerzeichen außer Tab, Zeilen- und Seitenvorschub.
pub fn ist_binaer(kopf: &[u8]) -> bool {
    if kopf.is_empty() {
        return false;
    }
    if kopf.contains(&0) {
        return true;
    }
    let steuer = kopf
        .iter()
        .filter(|&&b| b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r' | 0x0c | 0x1b))
        .count();
    steuer * 10 > kopf.len()
}

/// Benutzer-ID zu Name oder Zahl (aus `/etc/passwd`).
pub fn uid_fuer(name: &str) -> Option<u32> {
    if let Ok(n) = name.parse() {
        return Some(n);
    }
    let inhalt = fs::read_to_string("/etc/passwd").ok()?;
    id_aus_tabelle(&inhalt, name)
}

/// Gruppen-ID zu Name oder Zahl (aus `/etc/group`).
pub fn gid_fuer(name: &str) -> Option<u32> {
    if let Ok(n) = name.parse() {
        return Some(n);
    }
    let inhalt = fs::read_to_string("/etc/group").ok()?;
    id_aus_tabelle(&inhalt, name)
}

/// `name:x:ID:…` – gleiche Form in passwd und group.
pub fn id_aus_tabelle(inhalt: &str, name: &str) -> Option<u32> {
    inhalt.lines().find_map(|zeile| {
        let mut felder = zeile.split(':');
        (felder.next()? == name)
            .then(|| felder.nth(1)?.parse().ok())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anfrage(muster: &str, kriterien: Vec<Kriterium>) -> Anfrage {
        Anfrage {
            muster: muster.to_string(),
            ordner: PathBuf::from("/"),
            kriterien,
            text_max_bytes: 1024 * 1024,
        }
    }

    fn werte() -> Werte {
        Werte {
            ist_ordner: false,
            groesse: 2048,
            alter: Duration::from_secs(3 * TAG),
            uid: 1001,
            gid: 1001,
        }
    }

    #[test]
    fn muster_enthaelt_ohne_platzhalter() {
        let f = Filter::neu(&anfrage("bericht", vec![])).unwrap();
        assert!(f.passt_name("Bericht.md"));
        assert!(f.passt_name("alter-BERICHT-2026.txt"));
        assert!(!f.passt_name("berich.txt"));
    }

    #[test]
    fn muster_mit_platzhaltern_ganzer_name() {
        let f = Filter::neu(&anfrage("*.txt", vec![])).unwrap();
        assert!(f.passt_name("notiz.TXT"));
        assert!(!f.passt_name("notiz.txt.bak"));
        let f = Filter::neu(&anfrage("bild?.png", vec![])).unwrap();
        assert!(f.passt_name("Bild1.png"));
        assert!(!f.passt_name("bild12.png"));
        // Sonderzeichen sind keine Regex-Zeichen
        let f = Filter::neu(&anfrage("a+b(c).d", vec![])).unwrap();
        assert!(f.passt_name("xa+b(c).dy"));
        assert!(!f.passt_name("xaab(c)ed"));
    }

    #[test]
    fn leeres_muster_passt_immer() {
        let f = Filter::neu(&anfrage("  ", vec![])).unwrap();
        assert!(f.passt_name("irgendwas"));
    }

    #[test]
    fn regulaerer_ausdruck() {
        let f = Filter::neu(&anfrage(
            "",
            vec![Kriterium::NameRegex("^[a-z]+\\.rs$".into())],
        ))
        .unwrap();
        assert!(f.passt_name("main.rs"));
        assert!(!f.passt_name("Main.rs"));
        assert!(Filter::neu(&anfrage("", vec![Kriterium::NameRegex("(".into())])).is_err());
    }

    #[test]
    fn versteckte_und_sicherungen() {
        assert!(ist_versteckt_oder_sicherung(".config"));
        assert!(ist_versteckt_oder_sicherung("notiz.txt~"));
        assert!(ist_versteckt_oder_sicherung("#notiz.txt#"));
        assert!(!ist_versteckt_oder_sicherung("notiz.txt"));
        assert!(!ist_versteckt_oder_sicherung("#"));
        let ohne = Filter::neu(&anfrage("", vec![])).unwrap();
        assert!(ohne.uebergehen(".cache"));
        let mit = Filter::neu(&anfrage("", vec![Kriterium::VersteckteEinbeziehen])).unwrap();
        assert!(!mit.uebergehen(".cache"));
    }

    #[test]
    fn datum_weniger_und_mehr() {
        let f = Filter::neu(&anfrage("", vec![Kriterium::GeaendertWenigerAls(7)])).unwrap();
        assert!(f.passt_werte(&werte()));
        assert!(!f.passt_werte(&Werte {
            alter: Duration::from_secs(8 * TAG),
            ..werte()
        }));
        let f = Filter::neu(&anfrage("", vec![Kriterium::GeaendertMehrAls(2)])).unwrap();
        assert!(f.passt_werte(&werte()));
        assert!(!f.passt_werte(&Werte {
            alter: Duration::from_secs(TAG),
            ..werte()
        }));
    }

    #[test]
    fn groesse_und_leer() {
        let f = Filter::neu(&anfrage("", vec![Kriterium::GroesseMindestens(2048)])).unwrap();
        assert!(f.passt_werte(&werte()));
        assert!(!f.passt_werte(&Werte {
            groesse: 2047,
            ..werte()
        }));
        let f = Filter::neu(&anfrage("", vec![Kriterium::GroesseHoechstens(1024)])).unwrap();
        assert!(!f.passt_werte(&werte()));
        assert!(f.passt_werte(&Werte {
            groesse: 1024,
            ..werte()
        }));
        let f = Filter::neu(&anfrage("", vec![Kriterium::DateiIstLeer])).unwrap();
        assert!(f.passt_werte(&Werte {
            groesse: 0,
            ..werte()
        }));
        assert!(!f.passt_werte(&werte()));
        assert!(!f.passt_werte(&Werte {
            groesse: 0,
            ist_ordner: true,
            ..werte()
        }));
    }

    #[test]
    fn nur_ordner_besitzer_gruppe() {
        let f = Filter::neu(&anfrage("", vec![Kriterium::NurOrdner])).unwrap();
        assert!(!f.passt_werte(&werte()));
        assert!(f.passt_werte(&Werte {
            ist_ordner: true,
            ..werte()
        }));
        let f = Filter::neu(&anfrage("", vec![Kriterium::Besitzer("1001".into())])).unwrap();
        assert!(f.passt_werte(&werte()));
        assert!(!f.passt_werte(&Werte { uid: 0, ..werte() }));
        let f = Filter::neu(&anfrage("", vec![Kriterium::Gruppe("0".into())])).unwrap();
        assert!(!f.passt_werte(&werte()));
        assert!(
            Filter::neu(&anfrage(
                "",
                vec![Kriterium::Besitzer("gibt-es-nicht-xyz".into())]
            ))
            .is_err()
        );
        assert_eq!(uid_fuer("root"), Some(0));
    }

    #[test]
    fn id_tabelle() {
        let tabelle = "root:x:0:0:root:/root:/bin/bash\nuser:x:1000:1000::/home/user:/bin/bash\n";
        assert_eq!(id_aus_tabelle(tabelle, "user"), Some(1000));
        assert_eq!(id_aus_tabelle(tabelle, "root"), Some(0));
        assert_eq!(id_aus_tabelle(tabelle, "use"), None);
    }

    #[test]
    fn binaererkennung() {
        assert!(!ist_binaer(b""));
        assert!(!ist_binaer("Grüße aus Köln\n\tTab und \r\n".as_bytes()));
        assert!(ist_binaer(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"));
        assert!(ist_binaer(b"\x1f\x8b\x08\x00\x01\x02\x03\x04"));
        assert!(ist_binaer(b"\x01\x02\x03\x04abc"));
        assert!(!ist_binaer(b"\x1b[31mFarbe\x1b[0m\n"));
    }

    #[test]
    fn inhalt_suche_in_dateien() {
        let ordner = std::env::temp_dir().join(format!("qd-test-{}", std::process::id()));
        fs::create_dir_all(&ordner).unwrap();
        let text = ordner.join("text.txt");
        fs::write(&text, "Erste Zeile\nQUERENCIA Desktop\n").unwrap();
        let binaer = ordner.join("bin.dat");
        fs::write(&binaer, b"\x00\x01\x02querencia").unwrap();
        let gross = ordner.join("gross.txt");
        fs::write(&gross, "querencia ".repeat(200)).unwrap();

        let mut a = anfrage("", vec![Kriterium::EnthaeltText("querencia".into())]);
        a.text_max_bytes = 1000;
        let f = Filter::neu(&a).unwrap();
        assert!(f.braucht_inhalt());
        let laenge = |p: &Path| fs::metadata(p).unwrap().len();
        assert_eq!(
            f.passt_inhalt(&text, laenge(&text)).unwrap(),
            Inhalt::Treffer
        );
        assert_eq!(
            f.passt_inhalt(&binaer, laenge(&binaer)).unwrap(),
            Inhalt::Uebersprungen
        );
        assert_eq!(
            f.passt_inhalt(&gross, laenge(&gross)).unwrap(),
            Inhalt::Uebersprungen
        );
        let f = Filter::neu(&anfrage("", vec![Kriterium::EnthaeltText("fehlt".into())])).unwrap();
        assert_eq!(
            f.passt_inhalt(&text, laenge(&text)).unwrap(),
            Inhalt::KeinTreffer
        );
        assert!(f.passt_inhalt(&ordner.join("nicht-da"), 1).is_err());
        let ohne = Filter::neu(&anfrage("", vec![])).unwrap();
        assert!(!ohne.braucht_inhalt());
        fs::remove_dir_all(&ordner).unwrap();
    }

    #[test]
    fn kriterium_textform_hin_und_zurueck() {
        let alle = vec![
            Kriterium::EnthaeltText("a=b;c".into()),
            Kriterium::GeaendertWenigerAls(7),
            Kriterium::GeaendertMehrAls(30),
            Kriterium::GroesseMindestens(1024),
            Kriterium::GroesseHoechstens(2048),
            Kriterium::DateiIstLeer,
            Kriterium::Besitzer("user".into()),
            Kriterium::Gruppe("users".into()),
            Kriterium::NameRegex("^x=y$".into()),
            Kriterium::VersteckteEinbeziehen,
            Kriterium::VerknuepfungenFolgen,
            Kriterium::AndereDateisystemeFolgen,
            Kriterium::NurOrdner,
        ];
        assert_eq!(alle.len(), Art::ALLE.len());
        for k in alle {
            assert_eq!(
                Kriterium::aus_text(&k.als_text()),
                Some(k.clone()),
                "{}",
                k.als_text()
            );
        }
        assert_eq!(Kriterium::aus_text("geaendert_weniger=abc"), None);
        assert_eq!(Kriterium::aus_text("unbekannt"), None);
        assert_eq!(Kriterium::aus_text("leer=1"), None);
        assert_eq!(Kriterium::aus_text("text"), None);
    }
}
