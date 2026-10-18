//! Ein Treffer als GObject für die Ergebnisliste (`GtkColumnView`): Pfad,
//! Name, Ordner, Größe, Typ (aus dem MIME-Typ), Änderungszeit. Die
//! Spaltensortierer greifen über Eigenschaftsausdrücke darauf zu.

use std::cell::{Cell, RefCell};

use gtk::glib;
use gtk::glib::prelude::*;
use gtk::glib::subclass::prelude::*;
use gtk::gio;

use crate::suche::Fund;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Treffer)]
    pub struct Treffer {
        #[property(get, set)]
        pub pfad: RefCell<String>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub ordner: RefCell<String>,
        #[property(get, set)]
        pub groesse: Cell<u64>,
        #[property(get, set)]
        pub typ: RefCell<String>,
        #[property(get, set)]
        pub mime: RefCell<String>,
        #[property(get, set)]
        pub geaendert: Cell<i64>,
        #[property(get, set)]
        pub ist_ordner: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Treffer {
        const NAME: &'static str = "QuerenciaDateisucheTreffer";
        type Type = super::Treffer;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Treffer {}
}

glib::wrapper! {
    pub struct Treffer(ObjectSubclass<imp::Treffer>);
}

impl Treffer {
    pub fn neu(fund: &Fund) -> Treffer {
        let name = fund
            .pfad
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| fund.pfad.display().to_string());
        let ordner = fund
            .pfad
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let (mime, typ) = if fund.ist_ordner {
            ("inode/directory".to_string(), "Ordner".to_string())
        } else {
            let (mime, _unsicher) = gio::content_type_guess(Some(&fund.pfad), &[]);
            let typ = gio::content_type_get_description(&mime).to_string();
            (mime.to_string(), typ)
        };
        glib::Object::builder()
            .property("pfad", fund.pfad.display().to_string())
            .property("name", name)
            .property("ordner", ordner)
            .property("groesse", fund.groesse)
            .property("typ", typ)
            .property("mime", mime)
            .property("geaendert", fund.geaendert)
            .property("ist-ordner", fund.ist_ordner)
            .build()
    }

    pub fn datei(&self) -> gio::File {
        gio::File::for_path(self.pfad())
    }

    pub fn symbol(&self) -> gio::Icon {
        gio::content_type_get_icon(&self.mime())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn eigenschaften_aus_fund() {
        let t = Treffer::neu(&Fund {
            pfad: PathBuf::from("/tmp/ordner/notiz.txt"),
            ist_ordner: false,
            groesse: 12,
            geaendert: 86400,
        });
        assert_eq!(t.name(), "notiz.txt");
        assert_eq!(t.ordner(), "/tmp/ordner");
        assert_eq!(t.groesse(), 12);
        assert_eq!(t.mime(), "text/plain");
        assert!(!t.ist_ordner());
        let o = Treffer::neu(&Fund {
            pfad: PathBuf::from("/tmp/ordner"),
            ist_ordner: true,
            groesse: 0,
            geaendert: 0,
        });
        assert_eq!(o.typ(), "Ordner");
        assert_eq!(o.mime(), "inode/directory");
    }
}
