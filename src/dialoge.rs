//! Dialoge als gewöhnliche, modale Fenster ohne eigene Titelleiste: labwc
//! dekoriert sie wie jedes andere Fenster. Escape und der Schließen-Knopf
//! brechen ab; eine Antwort wird genau einmal gemeldet. Aufbau und
//! Symbolregel wie im Querencia Editor (`uebergabe-w5-editor-symbolabsturz.md`).

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::i18n::{gettext, gettext_f};
use crate::{APP_NAME, fenstertitel};

/// Noch nicht gemeldete Antwort (wird genau einmal genommen).
type Antwort = Rc<RefCell<Option<Box<dyn FnOnce(bool)>>>>;

/// Schließt `window` erst im nächsten Leerlauf, nicht aus dem gerade
/// laufenden Tasten- oder Klick-Handler heraus.
pub fn close_later(window: &gtk::Window) {
    let window = window.clone();
    glib::idle_add_local_once(move || window.close());
}

fn modal_window(parent: &impl IsA<gtk::Window>, title: &str) -> gtk::Window {
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .resizable(false)
        .destroy_with_parent(true)
        .build();
    window.add_css_class("qd-dialog");
    attach_to_app(&window, parent);
    window
}

/// Dialoge gehören zur laufenden Anwendung (wie das Hauptfenster), sonst
/// meldet GTK den Programmnamen als App-ID und schickt labwc ein eigenes
/// Fenstersymbol aus dem Thema (GTK-4.22-Absturz bei 48×48 + Scalable).
pub fn attach_to_app(window: &gtk::Window, parent: &impl IsA<gtk::Window>) {
    if let Some(app) = parent.as_ref().application() {
        window.set_application(Some(&app));
    }
    set_window_icon(window);
}

/// Name des Fenstersymbols, das GTK labwc schickt (xdg-toplevel-icon):
/// bewusst ein eigener, nur als SVG vorhandener Name statt der App-ID.
const WINDOW_ICON: &str = "org.querencia.Dateisuche-fenster";

/// Bettet das Fenstersymbol (nur SVG) als Ressource ein und macht es dem
/// Symbolthema bekannt; läuft nach dem Start von GTK.
pub fn register_window_icon() {
    let bytes = glib::Bytes::from_static(include_bytes!("../data/resource/dateisuche.gresource"));
    match gio::Resource::from_data(&bytes) {
        Ok(resource) => {
            gio::resources_register(&resource);
            if let Some(display) = gdk::Display::default() {
                gtk::IconTheme::for_display(&display)
                    .add_resource_path("/org/querencia/Dateisuche/icons");
            }
        }
        Err(err) => eprintln!("querencia-dateisuche: Ressourcen nicht lesbar: {err}"),
    }
}

pub fn set_window_icon(window: &impl IsA<gtk::Window>) {
    window.as_ref().set_icon_name(Some(WINDOW_ICON));
}

fn close_on_escape(window: &gtk::Window) {
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let w = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            if let Some(w) = w.upgrade() {
                close_later(&w);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
}

fn button_row(buttons: &[&gtk::Button]) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_halign(gtk::Align::End);
    row.set_margin_top(12);
    for b in buttons {
        row.append(*b);
    }
    row
}

/// Frage im Stil von GNOME 2: Symbol, fette Hauptzeile, Erklärung.
fn message_box(icon: &str, primary: &str, secondary: &str) -> gtk::Box {
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(48);
    image.set_valign(gtk::Align::Start);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let p = gtk::Label::new(None);
    p.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(primary)
    ));
    p.set_wrap(true);
    p.set_max_width_chars(48);
    p.set_xalign(0.0);
    let s = gtk::Label::new(Some(secondary));
    s.set_wrap(true);
    s.set_max_width_chars(48);
    s.set_xalign(0.0);
    s.set_selectable(true);
    text.append(&p);
    text.append(&s);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.append(&image);
    row.append(&text);
    row
}

fn content(child: &impl IsA<gtk::Widget>, buttons: &gtk::Box) -> gtk::Box {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_margin_top(12);
    outer.set_margin_bottom(12);
    outer.set_margin_start(12);
    outer.set_margin_end(12);
    outer.append(child);
    outer.append(buttons);
    outer
}

fn schliess_knopf(window: &gtk::Window) -> gtk::Button {
    let close = gtk::Button::with_mnemonic(&gettext("_Close"));
    let w = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    close
}

/// Fehlermeldung mit einem Knopf „Schließen“.
pub fn error(parent: &impl IsA<gtk::Window>, primary: &str, secondary: &str) {
    let window = modal_window(parent, &fenstertitel());
    let close = schliess_knopf(&window);
    let body = message_box("dialog-error", primary, secondary);
    window.set_child(Some(&content(&body, &button_row(&[&close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    window.present();
    close.grab_focus();
}

/// Rückfrage mit „Abbrechen“ und einem Bestätigungsknopf; `antwort(true)`
/// nur nach Bestätigung, `antwort(false)` bei Abbrechen, Escape, Schließen.
pub fn frage(
    parent: &impl IsA<gtk::Window>,
    primary: &str,
    secondary: &str,
    bestaetigen: &str,
    gefaehrlich: bool,
    antwort: impl FnOnce(bool) + 'static,
) {
    let window = modal_window(parent, &fenstertitel());
    let cancel = gtk::Button::with_mnemonic(&gettext("_Cancel"));
    let ok = gtk::Button::with_mnemonic(bestaetigen);
    ok.add_css_class(if gefaehrlich {
        "destructive-action"
    } else {
        "suggested-action"
    });
    let body = message_box(
        if gefaehrlich {
            "dialog-warning"
        } else {
            "dialog-question"
        },
        primary,
        secondary,
    );
    window.set_child(Some(&content(&body, &button_row(&[&cancel, &ok]))));
    // Vorgabe ist Abbrechen: Eingabetaste verschiebt nichts versehentlich.
    window.set_default_widget(Some(&cancel));
    close_on_escape(&window);

    let antwort: Antwort = Rc::new(RefCell::new(Some(Box::new(antwort))));
    let melden = {
        let antwort = antwort.clone();
        let window = window.downgrade();
        move |ja: bool| {
            let Some(f) = antwort.borrow_mut().take() else {
                return;
            };
            if let Some(w) = window.upgrade() {
                close_later(&w);
            }
            glib::idle_add_local_once(move || f(ja));
        }
    };
    let m = melden.clone();
    cancel.connect_clicked(move |_| m(false));
    let m = melden.clone();
    ok.connect_clicked(move |_| m(true));
    window.connect_close_request(move |_| {
        melden(false);
        glib::Propagation::Proceed
    });
    window.present();
    cancel.grab_focus();
}

/// Ordner- oder Speicherdialog im eigenen Prozess, klassisch mit Knopfleiste
/// (Titelleiste von labwc, deutsche Knöpfe) – wie Querencia Screenshot
/// 0.2.3 (QREL-04b): `gtk::FileDialog` ginge über das Portal, dessen
/// Fenster unter labwc keinen Tastaturfokus bekommt. GtkFileChooserDialog
/// ist seit GTK 4.10 als veraltet markiert, in GTK 4.22 vorhanden.
#[allow(deprecated)]
pub fn datei_dialog(
    parent: &impl IsA<gtk::Window>,
    titel: &str,
    aktion: gtk::FileChooserAction,
    bestaetigen: &str,
    startordner: &std::path::Path,
    vorgabe_name: Option<&str>,
    gewaehlt: impl Fn(gio::File) + 'static,
) {
    use gtk::prelude::{DialogExt as _, FileChooserExt as _};
    let chooser = gtk::FileChooserDialog::builder()
        .title(titel)
        .transient_for(parent)
        .modal(true)
        .use_header_bar(0)
        .action(aktion)
        .default_width(760)
        .default_height(500)
        .destroy_with_parent(true)
        .build();
    chooser.add_button(&gettext("_Cancel"), gtk::ResponseType::Cancel);
    chooser.add_button(bestaetigen, gtk::ResponseType::Accept);
    chooser.set_default_response(gtk::ResponseType::Accept);
    chooser.add_css_class("qd-dialog");
    attach_to_app(chooser.upcast_ref(), parent);
    let start = startordner
        .ancestors()
        .find(|p| p.is_dir())
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(glib::home_dir);
    if let Err(e) = chooser.set_current_folder(Some(&gio::File::for_path(&start))) {
        eprintln!("querencia-dateisuche: Startordner {}: {e}", start.display());
    }
    if let Some(n) = vorgabe_name {
        chooser.set_current_name(n);
    }
    chooser.connect_response(move |chooser, antwort| {
        let wahl = (antwort == gtk::ResponseType::Accept)
            .then(|| chooser.file())
            .flatten();
        chooser.destroy();
        if let Some(f) = wahl {
            gewaehlt(f);
        }
    });
    chooser.present();
}

const LOGO_64: &[u8] = include_bytes!("../data/logo/querencia-logo-64.png");

/// „Info“: offizielles Querencia-Logo C2, Name, Version, Lizenz.
pub fn about(parent: &impl IsA<gtk::Window>) {
    let window = modal_window(parent, &gettext_f("About {name}", &[("name", APP_NAME)]));
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_halign(gtk::Align::Center);
    if let Ok(texture) = gdk::Texture::from_bytes(&glib::Bytes::from_static(LOGO_64)) {
        let image = gtk::Image::from_paintable(Some(&texture));
        image.set_pixel_size(64);
        body.append(&image);
    }
    let name = gtk::Label::new(None);
    name.set_markup(&format!(
        "<b><big>{APP_NAME} {}</big></b>",
        env!("CARGO_PKG_VERSION")
    ));
    let text = gtk::Label::new(Some(&gettext(
        "“Search for Files…” for Querencia Desktop\n\
         modelled on gnome-search-tool (GNOME 2),\n\
         without an indexing service.\n\n\
         Prototype – not released for everyday use.",
    )));
    text.set_justify(gtk::Justification::Center);
    let tech = gtk::Label::new(Some(&format!(
        "GTK {}.{}.{}",
        gtk::major_version(),
        gtk::minor_version(),
        gtk::micro_version()
    )));
    tech.add_css_class("dim-label");
    let license = gtk::Label::new(Some(&gettext("License: GNU GPL, version 3 or later")));
    license.add_css_class("dim-label");
    body.append(&name);
    body.append(&text);
    body.append(&tech);
    body.append(&license);
    let close = schliess_knopf(&window);
    window.set_child(Some(&content(&body, &button_row(&[&close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    window.present();
    close.grab_focus();
}

/// Kurzhilfe als Pango-Markup, im Ganzen übersetzt (Absätze und Markup
/// bleiben in jeder Sprache gleich aufgebaut).
fn hilfe_text() -> String {
    // TRANSLATORS: Pango-Markup; <b>, <tt> und die Leerzeilen zwischen den
    // Absätzen beibehalten, Befehlszeile und Pfad nicht übersetzen.
    gettext(
        "\
<b>Name contains</b>: part of the file name, upper and lower case do not \
matter. With the wildcards <tt>*</tt> (any number of characters) and \
<tt>?</tt> (one character) the whole name must match, e.g. <tt>*.txt</tt>.

<b>Look in folder</b>: the starting folder; subfolders are always searched. \
Hidden folders and files (<tt>.name</tt>) and backup files \
(<tt>name~</tt>) only with the option of the same name.

<b>More options</b>: choose an option from the list and press “Add”; all \
chosen options must apply at the same time. “Contains the text” reads only \
text files up to the configured size (default 10 MB, \
<tt>~/.config/querencia/dateisuche.ini</tt>), case is ignored. \
“Name matches regular expression” respects case.

<b>Results</b>: double-click or Enter opens with the default application. \
Right mouse button: Open, Open With…, Open Containing Folder, Copy Path \
(Ctrl+C), Move to Trash (Delete, with confirmation), Save Results As a text \
file. Column headers sort.

<b>Command line</b>: <tt>querencia-dateisuche [--ordner PFAD] [--name MUSTER] [--suchen]</tt>",
    )
}

/// Kurzhilfe mit Knopf „Info“.
pub fn hilfe(parent: &impl IsA<gtk::Window>) {
    let window = modal_window(
        parent,
        &gettext_f("Help for “{title}”", &[("title", &fenstertitel())]),
    );
    let text = gtk::Label::new(None);
    text.set_markup(&hilfe_text());
    text.set_wrap(true);
    text.set_max_width_chars(70);
    text.set_xalign(0.0);
    let info = gtk::Button::with_mnemonic(&gettext("_About"));
    let close = schliess_knopf(&window);
    window.set_child(Some(&content(&text, &button_row(&[&info, &close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    let w = window.clone();
    info.connect_clicked(move |_| about(&w));
    window.present();
    close.grab_focus();
}

/// „Öffnen mit …“: Anwendungen für den MIME-Typ (empfohlene zuerst), auf
/// Wunsch alle; Doppelklick oder „Öffnen“ startet die Anwendung mit der Datei.
pub fn oeffnen_mit(parent: &impl IsA<gtk::Window>, datei: &gio::File, mime: &str) {
    let name = datei
        .basename()
        .map(|b| b.to_string_lossy().into_owned())
        .unwrap_or_default();
    let window = modal_window(parent, &gettext_f("Open “{name}” With", &[("name", &name)]));
    window.set_resizable(true);
    window.set_default_size(420, 440);

    let hinweis = gtk::Label::new(Some(&gettext_f(
        "Select an application to open “{name}” ({type}):",
        &[
            ("name", &name),
            ("type", &gio::content_type_get_description(mime)),
        ],
    )));
    hinweis.set_xalign(0.0);
    hinweis.set_wrap(true);
    let liste = gtk::ListBox::new();
    liste.set_selection_mode(gtk::SelectionMode::Single);
    liste.add_css_class("rich-list");
    let rollen = gtk::ScrolledWindow::builder()
        .child(&liste)
        .vexpand(true)
        .has_frame(true)
        .min_content_height(260)
        .build();
    let alle = gtk::CheckButton::with_mnemonic(&gettext("_Show all applications"));
    let apps: Rc<RefCell<Vec<gio::AppInfo>>> = Rc::new(RefCell::new(Vec::new()));

    let fuellen = {
        let liste = liste.clone();
        let apps = apps.clone();
        let mime = mime.to_string();
        move |alle: bool| {
            while let Some(k) = liste.first_child() {
                liste.remove(&k);
            }
            let mut gefunden: Vec<gio::AppInfo> = gio::AppInfo::all_for_type(&mime);
            if alle {
                for a in gio::AppInfo::all() {
                    if a.should_show() && !gefunden.iter().any(|g| g.equal(&a)) {
                        gefunden.push(a);
                    }
                }
            }
            for a in &gefunden {
                let zeile = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                zeile.set_margin_top(3);
                zeile.set_margin_bottom(3);
                zeile.set_margin_start(6);
                let bild = gtk::Image::new();
                bild.set_pixel_size(24);
                if let Some(icon) = a.icon() {
                    bild.set_from_gicon(&icon);
                } else {
                    bild.set_icon_name(Some("application-x-executable"));
                }
                let label = gtk::Label::new(Some(&a.display_name()));
                label.set_xalign(0.0);
                zeile.append(&bild);
                zeile.append(&label);
                liste.append(&zeile);
            }
            *apps.borrow_mut() = gefunden;
            if let Some(erste) = liste.row_at_index(0) {
                liste.select_row(Some(&erste));
            }
        }
    };
    fuellen(false);
    let f = fuellen.clone();
    alle.connect_toggled(move |c| f(c.is_active()));

    let cancel = gtk::Button::with_mnemonic(&gettext("_Cancel"));
    let open = gtk::Button::with_mnemonic(&gettext("_Open"));
    open.add_css_class("suggested-action");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.append(&hinweis);
    body.append(&rollen);
    body.append(&alle);
    window.set_child(Some(&content(&body, &button_row(&[&cancel, &open]))));
    window.set_default_widget(Some(&open));
    close_on_escape(&window);
    let w = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    let starten = {
        let liste = liste.clone();
        let apps = apps.clone();
        let datei = datei.clone();
        let window = window.clone();
        Rc::new(move || {
            let Some(zeile) = liste.selected_row() else {
                return;
            };
            let Some(app) = apps
                .borrow()
                .get(usize::try_from(zeile.index()).unwrap_or(usize::MAX))
                .cloned()
            else {
                return;
            };
            match app.launch(std::slice::from_ref(&datei), None::<&gio::AppLaunchContext>) {
                Ok(()) => close_later(&window),
                Err(e) => error(
                    &window,
                    &gettext_f(
                        "“{name}” could not be started.",
                        &[("name", &app.display_name())],
                    ),
                    e.message(),
                ),
            }
        })
    };
    let s = starten.clone();
    open.connect_clicked(move |_| s());
    liste.connect_row_activated(move |_, _| starten());
    window.present();
    liste.grab_focus();
}
