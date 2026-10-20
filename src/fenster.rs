//! Das Suchfenster: „Name enthält“, „Suchen in Ordner“, aufklappbare
//! „Weitere Optionen“ als Liste gewählter Kriterien, Ergebnisliste
//! (GtkColumnView) mit Kontextmenü, Statuszeile mit Fortschritt und die
//! Knöpfe Hilfe · Schließen · Suchen/Stopp – wie gnome-search-tool.
//! Keine eigene Titelleiste: labwc zeichnet die klassische Dekoration.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, gio, glib, graphene};

use crate::i18n::{gettext, gettext_f, pgettext};
use crate::konfig::Konfig;
use crate::kriterien::{Anfrage, Art, Eingabe, Kriterium};
use crate::suche::{self, Abschluss, Lauf, Meldung};
use crate::treffer::Treffer;
use crate::{Aufruf, dialoge, fenstertitel, format, protokoll};

const CSS: &str = include_str!("style.css");

/// Eine Aktion des Kontextmenüs.
type Aktion = fn(&Rc<Fenster>);

thread_local! {
    /// Offene Fenster: hält die `Rc`s am Leben, solange das GTK-Fenster
    /// besteht (alle Signal-Handler halten nur schwache Verweise).
    static OFFEN: RefCell<Vec<Rc<Fenster>>> = const { RefCell::new(Vec::new()) };
}

/// Größeneinheiten für „Größe mindestens/höchstens“.
const EINHEITEN: [(&str, u64); 4] = [
    ("Byte", 1),
    ("KB", 1024),
    ("MB", 1024 * 1024),
    ("GB", 1024 * 1024 * 1024),
];

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Eine gewählte Option unter „Weitere Optionen“.
struct Zeile {
    art: Art,
    zeile: gtk::Box,
    text: Option<gtk::Entry>,
    zahl: Option<gtk::SpinButton>,
    einheit: Option<gtk::DropDown>,
}

impl Zeile {
    fn kriterium(&self) -> Kriterium {
        let text = || {
            self.text
                .as_ref()
                .map(|e| e.text().to_string())
                .unwrap_or_default()
        };
        let zahl = || self.zahl.as_ref().map_or(0.0, |s| s.value());
        match self.art {
            Art::EnthaeltText => Kriterium::EnthaeltText(text()),
            Art::Besitzer => Kriterium::Besitzer(text()),
            Art::Gruppe => Kriterium::Gruppe(text()),
            Art::NameRegex => Kriterium::NameRegex(text()),
            Art::GeaendertWenigerAls => Kriterium::GeaendertWenigerAls(zahl().max(0.0) as u32),
            Art::GeaendertMehrAls => Kriterium::GeaendertMehrAls(zahl().max(0.0) as u32),
            Art::GroesseMindestens | Art::GroesseHoechstens => {
                let faktor = self
                    .einheit
                    .as_ref()
                    .and_then(|d| EINHEITEN.get(d.selected() as usize))
                    .map_or(1, |(_, f)| *f);
                let bytes = (zahl().max(0.0) as u64).saturating_mul(faktor);
                if self.art == Art::GroesseMindestens {
                    Kriterium::GroesseMindestens(bytes)
                } else {
                    Kriterium::GroesseHoechstens(bytes)
                }
            }
            a => a.vorgabe(),
        }
    }
}

/// Größe in Byte → (Zahl, Einheitenindex), größte Einheit ohne Rest.
fn einheit_zerlegen(bytes: u64) -> (f64, u32) {
    for (i, (_, f)) in EINHEITEN.iter().enumerate().rev() {
        if bytes >= *f && bytes.is_multiple_of(*f) {
            return ((bytes / f) as f64, i as u32);
        }
    }
    (bytes as f64, 0)
}

pub struct Fenster {
    window: gtk::ApplicationWindow,
    name: gtk::Entry,
    ordner: RefCell<PathBuf>,
    ordner_label: gtk::Label,
    ordner_knopf: gtk::Button,
    expander: gtk::Expander,
    zeilen_box: gtk::Box,
    zeilen: RefCell<Vec<Zeile>>,
    auswahl: gtk::DropDown,
    store: gio::ListStore,
    modell: gtk::SingleSelection,
    liste: gtk::ColumnView,
    status: gtk::Label,
    fortschritt: gtk::ProgressBar,
    suchen_knopf: gtk::Button,
    suchen_bild: gtk::Image,
    suchen_label: gtk::Label,
    kontextmenue: gtk::PopoverMenu,
    lauf: RefCell<Option<Lauf>>,
    /// Nach „Stopp“: noch wartende Treffer nicht mehr anzeigen.
    verwerfen: Cell<bool>,
    abholer: RefCell<Option<glib::SourceId>>,
    puls: RefCell<Option<glib::SourceId>>,
    text_max_mb: Cell<u32>,
}

fn knopf(symbol: &str, text: &str) -> (gtk::Button, gtk::Image, gtk::Label) {
    let bild = gtk::Image::from_icon_name(symbol);
    let label = gtk::Label::with_mnemonic(text);
    let inhalt = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    inhalt.set_halign(gtk::Align::Center);
    inhalt.append(&bild);
    inhalt.append(&label);
    let b = gtk::Button::new();
    b.set_child(Some(&inhalt));
    b.add_css_class("qd-knopf");
    (b, bild, label)
}

fn ausdruck(eigenschaft: &str) -> gtk::PropertyExpression {
    gtk::PropertyExpression::new(Treffer::static_type(), None::<gtk::Expression>, eigenschaft)
}

fn menue_modell() -> gio::Menu {
    fn abschnitt(eintraege: &[(String, &str)]) -> gio::Menu {
        let m = gio::Menu::new();
        for (label, aktion) in eintraege {
            m.append(Some(label), Some(aktion));
        }
        m
    }
    let menu = gio::Menu::new();
    menu.append_section(
        None,
        // TRANSLATORS: Kontextmenü der Ergebnisliste; Zugriffstasten verschieden halten.
        &abschnitt(&[
            (gettext("_Open"), "win.oeffnen"),
            (gettext("Open _With…"), "win.oeffnen-mit"),
        ]),
    );
    menu.append_section(
        None,
        &abschnitt(&[
            (gettext("Open Containing _Folder"), "win.ordner-oeffnen"),
            (gettext("_Copy Path to Clipboard"), "win.pfad-kopieren"),
        ]),
    );
    menu.append_section(
        None,
        &abschnitt(&[(gettext("Move to _Trash…"), "win.papierkorb")]),
    );
    menu.append_section(
        None,
        &abschnitt(&[(gettext("_Save Results As…"), "win.speichern")]),
    );
    menu
}

impl Fenster {
    pub fn neu(app: &gtk::Application, aufruf: &Aufruf) -> Rc<Fenster> {
        let konfig = Konfig::laden();
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(fenstertitel())
            .default_width(konfig.breite)
            .default_height(konfig.hoehe)
            .build();
        window.add_css_class("qd-window");
        dialoge::set_window_icon(&window);

        // Name und Ordner
        let gitter = gtk::Grid::new();
        gitter.set_row_spacing(6);
        gitter.set_column_spacing(12);
        // TRANSLATORS: Hauptfenster wie gnome-search-tool 2.x; Zugriffstasten
        // im Fenster verschieden halten.
        let name_label = gtk::Label::with_mnemonic(&gettext("_Name contains:"));
        name_label.set_xalign(1.0);
        let name = gtk::Entry::new();
        name.set_hexpand(true);
        name.set_placeholder_text(Some(&gettext("e.g. report or *.txt")));
        name_label.set_mnemonic_widget(Some(&name));
        let ordner_text = gtk::Label::with_mnemonic(&gettext("_Look in folder:"));
        ordner_text.set_xalign(1.0);
        let ordner_knopf = gtk::Button::new();
        ordner_knopf.set_hexpand(true);
        let ordner_inhalt = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let ordner_bild = gtk::Image::from_icon_name("folder");
        let ordner_label = gtk::Label::new(None);
        ordner_label.set_xalign(0.0);
        ordner_label.set_hexpand(true);
        ordner_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        let ordner_pfeil = gtk::Image::from_icon_name("pan-down-symbolic");
        ordner_inhalt.append(&ordner_bild);
        ordner_inhalt.append(&ordner_label);
        ordner_inhalt.append(&ordner_pfeil);
        ordner_knopf.set_child(Some(&ordner_inhalt));
        ordner_knopf.set_tooltip_text(Some(&gettext("Choose a folder…")));
        ordner_text.set_mnemonic_widget(Some(&ordner_knopf));
        gitter.attach(&name_label, 0, 0, 1, 1);
        gitter.attach(&name, 1, 0, 1, 1);
        gitter.attach(&ordner_text, 0, 1, 1, 1);
        gitter.attach(&ordner_knopf, 1, 1, 1, 1);

        // Weitere Optionen
        let expander = gtk::Expander::with_mnemonic(&gettext("Select more _options"));
        expander.set_expanded(konfig.weitere_sichtbar);
        let optionen = gtk::Box::new(gtk::Orientation::Vertical, 6);
        optionen.set_margin_start(18);
        optionen.set_margin_top(6);
        let zeilen_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let auswahl_zeile = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let auswahl_text = gtk::Label::with_mnemonic(&gettext("A_vailable options:"));
        let namen: Vec<String> = Art::ALLE.iter().map(|a| a.bezeichnung()).collect();
        let namen: Vec<&str> = namen.iter().map(String::as_str).collect();
        let auswahl = gtk::DropDown::from_strings(&namen);
        auswahl.set_hexpand(true);
        auswahl_text.set_mnemonic_widget(Some(&auswahl));
        let (hinzufuegen, _, _) = knopf("list-add", &gettext("_Add"));
        auswahl_zeile.append(&auswahl_text);
        auswahl_zeile.append(&auswahl);
        auswahl_zeile.append(&hinzufuegen);
        optionen.append(&zeilen_box);
        optionen.append(&auswahl_zeile);
        expander.set_child(Some(&optionen));

        // Ergebnisse
        let ergebnis_text = gtk::Label::with_mnemonic(&gettext("S_earch results:"));
        ergebnis_text.set_xalign(0.0);
        let liste = gtk::ColumnView::new(None::<gtk::SingleSelection>);
        liste.set_show_row_separators(false);
        liste.set_show_column_separators(true);
        liste.set_reorderable(false);
        ergebnis_text.set_mnemonic_widget(Some(&liste));
        let store = gio::ListStore::new::<Treffer>();
        let sortiert = gtk::SortListModel::new(Some(store.clone()), liste.sorter());
        let modell = gtk::SingleSelection::new(Some(sortiert));
        modell.set_autoselect(false);
        modell.set_can_unselect(true);
        liste.set_model(Some(&modell));
        let rollen = gtk::ScrolledWindow::builder()
            .child(&liste)
            .vexpand(true)
            .has_frame(true)
            .min_content_height(140)
            .build();

        // Statuszeile
        let status = gtk::Label::new(Some(""));
        status.set_xalign(0.0);
        status.set_hexpand(true);
        status.set_ellipsize(gtk::pango::EllipsizeMode::End);
        status.add_css_class("qd-status");
        let fortschritt = gtk::ProgressBar::new();
        fortschritt.set_size_request(160, -1);
        fortschritt.set_valign(gtk::Align::Center);
        fortschritt.set_visible(false);
        let statuszeile = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        statuszeile.append(&status);
        statuszeile.append(&fortschritt);

        // Knöpfe
        let (hilfe, _, _) = knopf("help-browser", &gettext("_Help"));
        // TRANSLATORS: im Deutschen „S_chließen“ (S gehört „_Suchen“).
        let (schliessen, _, _) = knopf("window-close", &pgettext("main window", "_Close"));
        let (suchen_knopf, suchen_bild, suchen_label) = knopf("edit-find", &gettext("_Find"));
        suchen_knopf.add_css_class("suggested-action");
        let knoepfe = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        knoepfe.set_halign(gtk::Align::End);
        knoepfe.append(&hilfe);
        knoepfe.append(&schliessen);
        knoepfe.append(&suchen_knopf);

        let aussen = gtk::Box::new(gtk::Orientation::Vertical, 12);
        aussen.set_margin_top(12);
        aussen.set_margin_bottom(12);
        aussen.set_margin_start(12);
        aussen.set_margin_end(12);
        aussen.append(&gitter);
        aussen.append(&expander);
        aussen.append(&ergebnis_text);
        aussen.append(&rollen);
        aussen.append(&statuszeile);
        aussen.append(&knoepfe);
        window.set_child(Some(&aussen));
        window.set_default_widget(Some(&suchen_knopf));

        let kontextmenue = gtk::PopoverMenu::from_model(Some(&menue_modell()));
        kontextmenue.set_parent(&liste);
        kontextmenue.set_has_arrow(false);
        kontextmenue.set_halign(gtk::Align::Start);

        let f = Rc::new(Fenster {
            window,
            name,
            ordner: RefCell::new(glib::home_dir()),
            ordner_label,
            ordner_knopf,
            expander,
            zeilen_box,
            zeilen: RefCell::new(Vec::new()),
            auswahl,
            store,
            modell,
            liste,
            status,
            fortschritt,
            suchen_knopf,
            suchen_bild,
            suchen_label,
            kontextmenue,
            lauf: RefCell::new(None),
            verwerfen: Cell::new(false),
            abholer: RefCell::new(None),
            puls: RefCell::new(None),
            text_max_mb: Cell::new(konfig.text_max_mb),
        });
        f.spalten_anlegen();
        f.aktionen_anlegen();

        // Vorbelegung: Konfiguration, dann Befehlszeile
        for k in &konfig.kriterien {
            f.zeile_hinzufuegen(k.art(), Some(k));
        }
        f.ordner_setzen(aufruf.ordner.clone().unwrap_or_else(glib::home_dir));
        if let Some(n) = &aufruf.name {
            f.name.set_text(n);
        }
        f.status_setzen("");

        // Signale
        let w = Rc::downgrade(&f);
        hinzufuegen.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                let art = Art::ALLE[f.auswahl.selected() as usize];
                f.zeile_hinzufuegen(art, None);
            }
        });
        let w = Rc::downgrade(&f);
        f.ordner_knopf.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                f.ordner_waehlen();
            }
        });
        let w = Rc::downgrade(&f);
        f.name.connect_activate(move |_| {
            if let Some(f) = w.upgrade() {
                f.suchen();
            }
        });
        let w = Rc::downgrade(&f);
        f.suchen_knopf.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                f.suchen();
            }
        });
        let w = Rc::downgrade(&f);
        hilfe.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                dialoge::hilfe(&f.window);
            }
        });
        let w = Rc::downgrade(&f);
        schliessen.connect_clicked(move |_| {
            if let Some(f) = w.upgrade() {
                f.window.close();
            }
        });
        let w = Rc::downgrade(&f);
        f.liste.connect_activate(move |_, pos| {
            if let Some(f) = w.upgrade() {
                f.modell.set_selected(pos);
                f.oeffnen();
            }
        });
        let tasten = gtk::EventControllerKey::new();
        let w = Rc::downgrade(&f);
        tasten.connect_key_pressed(move |_, key, _, zustand| {
            let Some(f) = w.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let strg = zustand.contains(gdk::ModifierType::CONTROL_MASK);
            let umschalt = zustand.contains(gdk::ModifierType::SHIFT_MASK);
            match key {
                gdk::Key::Delete | gdk::Key::KP_Delete if !strg => f.papierkorb(),
                gdk::Key::c | gdk::Key::C if strg => f.pfad_kopieren(),
                gdk::Key::Menu => f.kontextmenue_zeigen(None),
                gdk::Key::F10 if umschalt => f.kontextmenue_zeigen(None),
                _ => return glib::Propagation::Proceed,
            }
            glib::Propagation::Stop
        });
        f.liste.add_controller(tasten);
        // Escape im Fenster hält eine laufende Suche an.
        let escape = gtk::EventControllerKey::new();
        let w = Rc::downgrade(&f);
        escape.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape
                && let Some(f) = w.upgrade()
                && f.lauf.borrow().is_some()
            {
                f.stoppen();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        f.window.add_controller(escape);
        let w = Rc::downgrade(&f);
        f.window.connect_close_request(move |_| {
            if let Some(f) = w.upgrade() {
                f.aufraeumen();
            }
            glib::Propagation::Proceed
        });
        f.name.grab_focus();
        OFFEN.with(|o| o.borrow_mut().push(f.clone()));
        f
    }

    pub fn present(&self) {
        self.window.present();
    }

    // ------------------------------------------------------------ Aufbau

    fn fabrik(
        self: &Rc<Self>,
        mit_symbol: bool,
        text: fn(&Treffer) -> String,
    ) -> gtk::SignalListItemFactory {
        let fabrik = gtk::SignalListItemFactory::new();
        let w = Rc::downgrade(self);
        fabrik.connect_setup(move |_, obj| {
            let Some(item) = obj.downcast_ref::<gtk::ListItem>() else {
                return;
            };
            let zeile = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            if mit_symbol {
                let bild = gtk::Image::new();
                bild.set_pixel_size(16);
                zeile.append(&bild);
            }
            let label = gtk::Label::new(None);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            zeile.append(&label);
            item.set_child(Some(&zeile));
            // Rechtsklick: Zeile wählen, Kontextmenü am Zeiger
            let geste = gtk::GestureClick::new();
            geste.set_button(3);
            // Vor der Zeilenauswahl der Liste (Capture), sonst kommt der
            // Rechtsklick nie bei der Zelle an.
            geste.set_propagation_phase(gtk::PropagationPhase::Capture);
            let item_w = item.downgrade();
            let w = w.clone();
            geste.connect_pressed(move |g, _, x, y| {
                if let (Some(f), Some(item)) = (w.upgrade(), item_w.upgrade()) {
                    g.set_state(gtk::EventSequenceState::Claimed);
                    f.modell.set_selected(item.position());
                    let punkt = g
                        .widget()
                        .and_then(|z| {
                            z.compute_point(&f.liste, &graphene::Point::new(x as f32, y as f32))
                        })
                        .map(|p| (p.x() as i32, p.y() as i32));
                    f.kontextmenue_zeigen(punkt);
                }
            });
            zeile.add_controller(geste);
        });
        fabrik.connect_bind(move |_, obj| {
            let Some(item) = obj.downcast_ref::<gtk::ListItem>() else {
                return;
            };
            let Some(t) = item.item().and_downcast::<Treffer>() else {
                return;
            };
            let Some(zeile) = item.child().and_downcast::<gtk::Box>() else {
                return;
            };
            let mut kind = zeile.first_child();
            if mit_symbol && let Some(bild) = kind.clone().and_downcast::<gtk::Image>() {
                bild.set_from_gicon(&t.symbol());
                kind = bild.next_sibling();
            }
            if let Some(label) = kind.and_downcast::<gtk::Label>() {
                label.set_text(&text(&t));
                label.set_tooltip_text(Some(&t.pfad()));
            }
        });
        fabrik
    }

    fn spalten_anlegen(self: &Rc<Self>) {
        // TRANSLATORS: Spaltenköpfe der Ergebnisliste.
        let name = gtk::ColumnViewColumn::new(
            Some(&gettext("Name")),
            Some(self.fabrik(true, |t| t.name())),
        );
        let s = gtk::StringSorter::new(Some(ausdruck("name")));
        s.set_ignore_case(true);
        name.set_sorter(Some(&s));
        name.set_expand(true);
        name.set_resizable(true);

        let ordner = gtk::ColumnViewColumn::new(
            Some(&gettext("Folder")),
            Some(self.fabrik(false, |t| t.ordner())),
        );
        let s = gtk::StringSorter::new(Some(ausdruck("ordner")));
        s.set_ignore_case(true);
        ordner.set_sorter(Some(&s));
        ordner.set_expand(true);
        ordner.set_resizable(true);

        let groesse = gtk::ColumnViewColumn::new(
            Some(&gettext("Size")),
            Some(self.fabrik(false, |t| {
                if t.ist_ordner() {
                    "–".to_string()
                } else {
                    format::groesse(t.groesse())
                }
            })),
        );
        groesse.set_sorter(Some(&gtk::NumericSorter::new(Some(ausdruck("groesse")))));
        groesse.set_fixed_width(90);
        groesse.set_resizable(true);

        let typ = gtk::ColumnViewColumn::new(
            Some(&gettext("Type")),
            Some(self.fabrik(false, |t| t.typ())),
        );
        let s = gtk::StringSorter::new(Some(ausdruck("typ")));
        s.set_ignore_case(true);
        typ.set_sorter(Some(&s));
        typ.set_fixed_width(130);
        typ.set_resizable(true);

        let geaendert = gtk::ColumnViewColumn::new(
            Some(&gettext("Date Modified")),
            Some(self.fabrik(false, |t| format::datum_unix(t.geaendert()))),
        );
        geaendert.set_sorter(Some(&gtk::NumericSorter::new(Some(ausdruck("geaendert")))));
        geaendert.set_fixed_width(130);
        geaendert.set_resizable(true);

        for c in [&name, &ordner, &groesse, &typ, &geaendert] {
            self.liste.append_column(c);
        }
    }

    fn aktionen_anlegen(self: &Rc<Self>) {
        let aktionen: [(&str, Aktion); 6] = [
            ("oeffnen", |f| f.oeffnen()),
            ("oeffnen-mit", |f| f.oeffnen_mit()),
            ("ordner-oeffnen", |f| f.ordner_oeffnen()),
            ("pfad-kopieren", |f| f.pfad_kopieren()),
            ("papierkorb", |f| f.papierkorb()),
            ("speichern", |f| f.speichern()),
        ];
        for (name, tun) in aktionen {
            let a = gio::SimpleAction::new(name, None);
            let w = Rc::downgrade(self);
            a.connect_activate(move |_, _| {
                if let Some(f) = w.upgrade() {
                    tun(&f);
                }
            });
            self.window.add_action(&a);
        }
    }

    /// Fügt eine Option an; eine schon vorhandene Option ohne Wert wird
    /// nicht doppelt angelegt.
    fn zeile_hinzufuegen(&self, art: Art, wert: Option<&Kriterium>) {
        if art.eingabe() == Eingabe::Keine && self.zeilen.borrow().iter().any(|z| z.art == art) {
            return;
        }
        let zeile = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let label = gtk::Label::new(Some(&gettext_f(
            // TRANSLATORS: Beschriftung einer gewählten Option mit Eingabefeld.
            "{option}:",
            &[("option", &art.bezeichnung())],
        )));
        if art.eingabe() == Eingabe::Keine {
            label.set_text(&art.bezeichnung());
        }
        label.set_xalign(0.0);
        zeile.append(&label);
        let mut text = None;
        let mut zahl = None;
        let mut einheit = None;
        match art.eingabe() {
            Eingabe::Text => {
                let e = gtk::Entry::new();
                e.set_hexpand(true);
                if let Some(
                    Kriterium::EnthaeltText(s)
                    | Kriterium::Besitzer(s)
                    | Kriterium::Gruppe(s)
                    | Kriterium::NameRegex(s),
                ) = wert
                {
                    e.set_text(s);
                }
                let w = self.suchen_knopf.clone();
                e.connect_activate(move |_| w.emit_clicked());
                zeile.append(&e);
                text = Some(e);
            }
            Eingabe::Tage => {
                let s = gtk::SpinButton::with_range(0.0, 36500.0, 1.0);
                let tage = match wert {
                    Some(Kriterium::GeaendertWenigerAls(n) | Kriterium::GeaendertMehrAls(n)) => *n,
                    _ => match art.vorgabe() {
                        Kriterium::GeaendertWenigerAls(n) | Kriterium::GeaendertMehrAls(n) => n,
                        _ => 7,
                    },
                };
                s.set_value(f64::from(tage));
                zeile.append(&s);
                let nach = gtk::Label::new(Some(&art.nachsatz()));
                zeile.append(&nach);
                zahl = Some(s);
            }
            Eingabe::Groesse => {
                let s = gtk::SpinButton::with_range(0.0, 1.0e9, 1.0);
                // TRANSLATORS: kleinste Größeneinheit in der Auswahl (danach KB, MB, GB).
                let byte = gettext("bytes");
                let namen: Vec<&str> = EINHEITEN
                    .iter()
                    .map(|(n, _)| if *n == "Byte" { byte.as_str() } else { n })
                    .collect();
                let d = gtk::DropDown::from_strings(&namen);
                let bytes = match wert {
                    Some(Kriterium::GroesseMindestens(n) | Kriterium::GroesseHoechstens(n)) => *n,
                    _ => match art.vorgabe() {
                        Kriterium::GroesseMindestens(n) | Kriterium::GroesseHoechstens(n) => n,
                        _ => 1024,
                    },
                };
                let (z, e) = einheit_zerlegen(bytes);
                s.set_value(z);
                d.set_selected(e);
                zeile.append(&s);
                zeile.append(&d);
                zahl = Some(s);
                einheit = Some(d);
            }
            Eingabe::Keine => {
                label.set_hexpand(true);
            }
        }
        let fueller = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        fueller.set_hexpand(true);
        zeile.append(&fueller);
        let entfernen = gtk::Button::from_icon_name("list-remove");
        entfernen.set_tooltip_text(Some(&gettext("Remove this option")));
        entfernen.add_css_class("flat");
        zeile.append(&entfernen);
        self.zeilen_box.append(&zeile);
        let z = Zeile {
            art,
            zeile: zeile.clone(),
            text: text.clone(),
            zahl: zahl.clone(),
            einheit,
        };
        self.zeilen.borrow_mut().push(z);
        let box_ = self.zeilen_box.clone();
        // Entfernen über den Zeilen-Container (Fenster kann längst weg sein).
        entfernen.connect_clicked(move |_| {
            box_.remove(&zeile);
        });
        if let Some(e) = &text {
            e.grab_focus();
        } else if let Some(s) = &zahl {
            s.grab_focus();
        }
    }

    /// Gewählte Kriterien (Zeilen, die noch im Fenster hängen).
    fn kriterien(&self) -> Vec<Kriterium> {
        // Entfernte Zeilen (ohne Elternelement) aus der Liste nehmen.
        self.zeilen
            .borrow_mut()
            .retain(|z| z.zeile.parent().is_some());
        self.zeilen.borrow().iter().map(Zeile::kriterium).collect()
    }

    fn ordner_setzen(&self, pfad: PathBuf) {
        let anzeige = if pfad == glib::home_dir() {
            gettext("Home Folder")
        } else {
            pfad.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| pfad.display().to_string())
        };
        self.ordner_label.set_text(&anzeige);
        self.ordner_knopf
            .set_tooltip_text(Some(&pfad.display().to_string()));
        *self.ordner.borrow_mut() = pfad;
    }

    fn ordner_waehlen(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        let start = self.ordner.borrow().clone();
        dialoge::datei_dialog(
            &self.window,
            &gettext("Choose a Folder"),
            gtk::FileChooserAction::SelectFolder,
            &gettext("_Select"),
            &start,
            None,
            move |datei| {
                if let (Some(f), Some(pfad)) = (w.upgrade(), datei.path()) {
                    protokoll::schreiben(&format!("ORDNERWAHL {}", pfad.display()));
                    f.ordner_setzen(pfad);
                }
            },
        );
    }

    fn status_setzen(&self, text: &str) {
        self.status.set_text(text);
    }

    // ------------------------------------------------------------- Suche

    /// Startet die Suche oder hält die laufende an (Knopf „Stopp“).
    pub fn suchen(self: &Rc<Self>) {
        if self.lauf.borrow().is_some() {
            self.stoppen();
            return;
        }
        let anfrage = Anfrage {
            muster: self.name.text().to_string(),
            ordner: self.ordner.borrow().clone(),
            kriterien: self.kriterien(),
            text_max_bytes: Konfig {
                text_max_mb: self.text_max_mb.get(),
                ..Konfig::default()
            }
            .text_max_bytes(),
        };
        match suche::starten(anfrage) {
            Err(e) => dialoge::error(&self.window, &gettext("The search cannot be started."), &e),
            Ok(lauf) => {
                self.store.remove_all();
                self.verwerfen.set(false);
                *self.lauf.borrow_mut() = Some(lauf);
                self.suchen_bild.set_icon_name(Some("process-stop"));
                // TRANSLATORS: Suchknopf während der Suche (im Deutschen „S_topp“).
                self.suchen_label.set_text_with_mnemonic(&gettext("_Stop"));
                self.suchen_knopf.remove_css_class("suggested-action");
                self.fortschritt.set_visible(true);
                self.status_setzen(&gettext("Searching…"));
                let fb = self.fortschritt.clone();
                *self.puls.borrow_mut() = Some(glib::timeout_add_local(
                    Duration::from_millis(80),
                    move || {
                        fb.pulse();
                        glib::ControlFlow::Continue
                    },
                ));
                let w = Rc::downgrade(self);
                *self.abholer.borrow_mut() = Some(glib::timeout_add_local(
                    Duration::from_millis(100),
                    move || match w.upgrade() {
                        Some(f) => f.abholen(),
                        None => glib::ControlFlow::Break,
                    },
                ));
            }
        }
    }

    fn stoppen(&self) {
        if let Some(l) = &*self.lauf.borrow() {
            l.stoppen();
            self.verwerfen.set(true);
            self.status_setzen(&gettext("Stopping the search…"));
        }
    }

    /// Je Takt höchstens so viele Treffer in die Liste (ein `splice`, ein
    /// Signal), damit Eingaben und Stopp nicht hinter der Liste warten.
    const JE_TAKT: usize = 2000;

    fn abholen(self: &Rc<Self>) -> glib::ControlFlow {
        let verwerfen = self.verwerfen.get();
        let meldungen = match &*self.lauf.borrow() {
            Some(l) => l.abholen(if verwerfen { usize::MAX } else { Self::JE_TAKT }),
            None => return glib::ControlFlow::Break,
        };
        let mut fertig = None;
        let mut neue: Vec<Treffer> = Vec::new();
        for m in meldungen {
            match m {
                Meldung::Treffer(fund) => {
                    if !verwerfen {
                        neue.push(Treffer::neu(&fund));
                    }
                }
                Meldung::Fertig(a) => fertig = Some(a),
            }
        }
        if !neue.is_empty() {
            self.store.splice(self.store.n_items(), 0, &neue);
        }
        match fertig {
            Some(a) => {
                self.beenden(a);
                glib::ControlFlow::Break
            }
            None => {
                if !verwerfen {
                    self.status_setzen(&gettext_f(
                        "Searching… {status}",
                        &[("status", &format::gefunden(self.store.n_items() as usize))],
                    ));
                }
                glib::ControlFlow::Continue
            }
        }
    }

    fn beenden(&self, a: Abschluss) {
        self.lauf.borrow_mut().take();
        self.abholer.borrow_mut().take(); // Quelle endet über Break
        if let Some(p) = self.puls.borrow_mut().take() {
            p.remove();
        }
        self.fortschritt.set_visible(false);
        self.fortschritt.set_fraction(0.0);
        self.suchen_bild.set_icon_name(Some("edit-find"));
        self.suchen_label.set_text_with_mnemonic(&gettext("_Find"));
        self.suchen_knopf.add_css_class("suggested-action");
        let mut text = format::gefunden(self.store.n_items() as usize);
        // TRANSLATORS: Zusätze der Statuszeile nach der Suche; {status} ist
        // z. B. „12 files found“.
        if a.abgebrochen {
            text = gettext_f("{status} (search stopped)", &[("status", &text)]);
        }
        if a.fehler > 0 {
            text = gettext_f(
                "{status} · {count} not readable (no permission)",
                &[
                    ("status", &text),
                    ("count", &format::tausender(a.fehler as usize)),
                ],
            );
        }
        if a.uebersprungen > 0 {
            text = gettext_f(
                "{status} · {count} skipped (binary or too large)",
                &[
                    ("status", &text),
                    ("count", &format::tausender(a.uebersprungen as usize)),
                ],
            );
        }
        self.status_setzen(&text);
        protokoll::schreiben(&format!("FERTIG {text}"));
        for i in 0..self.store.n_items() {
            if let Some(t) = self.store.item(i).and_downcast::<Treffer>() {
                protokoll::schreiben(&format!("TREFFER {}", t.pfad()));
            }
        }
        if self.store.n_items() > 0 && self.modell.selected() == gtk::INVALID_LIST_POSITION {
            self.modell.set_selected(0);
        }
    }

    fn aufraeumen(&self) {
        if let Some(l) = self.lauf.borrow_mut().take() {
            l.stoppen();
        }
        if let Some(a) = self.abholer.borrow_mut().take() {
            a.remove();
        }
        if let Some(p) = self.puls.borrow_mut().take() {
            p.remove();
        }
        self.kontextmenue.unparent();
        let (breite, hoehe) = self.window.default_size();
        let k = Konfig {
            breite,
            hoehe,
            weitere_sichtbar: self.expander.is_expanded(),
            kriterien: self.kriterien(),
            text_max_mb: self.text_max_mb.get(),
            letzter_ordner: Some(self.ordner.borrow().display().to_string()),
        };
        if let Err(e) = k.speichern() {
            eprintln!("querencia-dateisuche: Einstellungen nicht gespeichert: {e}");
        }
        let w = self.window.clone();
        // Erst nach dem Schließen freigeben, nicht aus dem Handler heraus.
        glib::idle_add_local_once(move || {
            OFFEN.with(|o| o.borrow_mut().retain(|f| f.window != w));
        });
    }

    // --------------------------------------------------------- Ergebnisse

    fn gewaehlt(&self) -> Option<Treffer> {
        self.modell.selected_item().and_downcast::<Treffer>()
    }

    fn kontextmenue_zeigen(&self, punkt: Option<(i32, i32)>) {
        if self.gewaehlt().is_none() {
            return;
        }
        let (x, y) = punkt.unwrap_or((24, 24));
        self.kontextmenue
            .set_pointing_to(Some(&gdk::Rectangle::new(x, y, 1, 1)));
        self.kontextmenue.popup();
    }

    fn oeffnen(&self) {
        let Some(t) = self.gewaehlt() else {
            return;
        };
        match gio::AppInfo::launch_default_for_uri(&t.datei().uri(), None::<&gio::AppLaunchContext>)
        {
            Ok(()) => protokoll::schreiben(&format!("OEFFNEN {}", t.pfad())),
            Err(e) => dialoge::error(
                &self.window,
                &gettext_f("“{name}” could not be opened.", &[("name", &t.name())]),
                e.message(),
            ),
        }
    }

    fn oeffnen_mit(&self) {
        if let Some(t) = self.gewaehlt() {
            dialoge::oeffnen_mit(&self.window, &t.datei(), &t.mime());
        }
    }

    /// Öffnet den Ordner des Treffers mit dem Ordnerprogramm (Querencia Files
    /// über `inode/directory`).
    fn ordner_oeffnen(&self) {
        let Some(t) = self.gewaehlt() else {
            return;
        };
        let ordner = gio::File::for_path(t.ordner());
        match gio::AppInfo::launch_default_for_uri(&ordner.uri(), None::<&gio::AppLaunchContext>) {
            Ok(()) => protokoll::schreiben(&format!("ORDNER {}", t.ordner())),
            Err(e) => dialoge::error(
                &self.window,
                &gettext_f(
                    "The folder “{folder}” could not be opened.",
                    &[("folder", &t.ordner())],
                ),
                e.message(),
            ),
        }
    }

    fn pfad_kopieren(&self) {
        if let Some(t) = self.gewaehlt() {
            self.window.clipboard().set_text(&t.pfad());
            self.status_setzen(&gettext_f("Path copied: {path}", &[("path", &t.pfad())]));
            protokoll::schreiben(&format!("KOPIERT {}", t.pfad()));
        }
    }

    /// In den Papierkorb verschieben – nur nach Rückfrage, über GIO
    /// (`g_file_trash`), nie Löschen.
    fn papierkorb(self: &Rc<Self>) {
        let Some(t) = self.gewaehlt() else {
            return;
        };
        let w = Rc::downgrade(self);
        let erklaerung = if t.ist_ordner() {
            gettext_f(
                "The folder will be moved from “{folder}” to the trash and can be restored from there.",
                &[("folder", &t.ordner())],
            )
        } else {
            gettext_f(
                "The file will be moved from “{folder}” to the trash and can be restored from there.",
                &[("folder", &t.ordner())],
            )
        };
        dialoge::frage(
            &self.window,
            &gettext_f("Move “{name}” to the trash?", &[("name", &t.name())]),
            &erklaerung,
            &gettext("Move to _Trash"),
            true,
            move |ja| {
                let Some(f) = w.upgrade() else {
                    return;
                };
                if !ja {
                    protokoll::schreiben(&format!("PAPIERKORB-ABGEBROCHEN {}", t.pfad()));
                    return;
                }
                match t.datei().trash(None::<&gio::Cancellable>) {
                    Ok(()) => {
                        protokoll::schreiben(&format!("PAPIERKORB {}", t.pfad()));
                        f.entfernen(&t);
                        f.status_setzen(&gettext_f(
                            "“{name}” moved to the trash.",
                            &[("name", &t.name())],
                        ));
                    }
                    Err(e) => dialoge::error(
                        &f.window,
                        &gettext_f(
                            "“{name}” could not be moved to the trash.",
                            &[("name", &t.name())],
                        ),
                        e.message(),
                    ),
                }
            },
        );
    }

    fn entfernen(&self, t: &Treffer) {
        let n = self.store.n_items();
        for i in 0..n {
            if self
                .store
                .item(i)
                .and_downcast::<Treffer>()
                .is_some_and(|x| x.pfad() == t.pfad())
            {
                self.store.remove(i);
                break;
            }
        }
    }

    /// Alle Pfade in Listenreihenfolge als Textdatei.
    fn speichern(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        let start = self.ordner.borrow().clone();
        dialoge::datei_dialog(
            &self.window,
            &gettext("Save Search Results As"),
            gtk::FileChooserAction::Save,
            &gettext("_Save"),
            &start,
            Some("suchergebnisse.txt"),
            move |datei| {
                let (Some(f), Some(pfad)) = (w.upgrade(), datei.path()) else {
                    return;
                };
                let mut text = String::new();
                for i in 0..f.modell.n_items() {
                    if let Some(t) = f.modell.item(i).and_downcast::<Treffer>() {
                        text.push_str(&t.pfad());
                        text.push('\n');
                    }
                }
                match std::fs::write(&pfad, text) {
                    Ok(()) => {
                        f.status_setzen(&gettext_f(
                            "Results saved: {path}",
                            &[("path", &pfad.display().to_string())],
                        ));
                        protokoll::schreiben(&format!("GESPEICHERT {}", pfad.display()));
                    }
                    Err(e) => dialoge::error(
                        &f.window,
                        &gettext_f(
                            "“{path}” could not be written.",
                            &[("path", &pfad.display().to_string())],
                        ),
                        &e.to_string(),
                    ),
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn einheiten_zerlegen() {
        assert_eq!(einheit_zerlegen(0), (0.0, 0));
        assert_eq!(einheit_zerlegen(500), (500.0, 0));
        assert_eq!(einheit_zerlegen(1024), (1.0, 1));
        assert_eq!(einheit_zerlegen(1536), (1536.0, 0));
        assert_eq!(einheit_zerlegen(100 * 1024), (100.0, 1));
        assert_eq!(einheit_zerlegen(3 * 1024 * 1024), (3.0, 2));
        assert_eq!(einheit_zerlegen(2 * 1024 * 1024 * 1024), (2.0, 3));
    }
}
