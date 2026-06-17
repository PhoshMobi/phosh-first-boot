// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::Object;

use crate::pfb::{Page, PageImpl};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::MainContext;
use glib_macros::clone;
use gtk::glib;
use libpms::*;

use std::cell::RefCell;

use zbus::{proxy, Connection};

use log::{debug, trace, warn};

#[proxy(
    interface = "org.freedesktop.locale1",
    default_service = "org.freedesktop.locale1",
    default_path = "/org/freedesktop/locale1"
)]

trait Locale1 {
    #[zbus(allow_interactive_auth)]
    async fn set_locale(&self, locale: &[&str], interactive: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn locale(&self) -> zbus::Result<Vec<String>>;
}

mod imp {
    use glib::subclass::InitializingObject;
    use glib_macros::Properties;
    use gtk::{glib, CompositeTemplate};

    use super::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/mobi/phosh/FirstBoot/pages/lang_page.ui")]
    #[properties(wrapper_type = super::LangPage)]
    pub struct PfbLangPage {
        #[template_child]
        pub lang_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub osk_layout_prefs: TemplateChild<OskLayoutPrefs>,

        #[property(get, set)]
        pub region: RefCell<String>,

        pub(super) connection: RefCell<Option<Connection>>,
        pub(super) locale1: RefCell<Option<Locale1Proxy<'static>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbLangPage {
        const NAME: &'static str = "PfbLangPage";
        type Type = super::LangPage;
        type ParentType = Page;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PfbLangPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup();
        }
    }

    impl WidgetImpl for PfbLangPage {}
    impl NavigationPageImpl for PfbLangPage {}
    impl PageImpl for PfbLangPage {}
}

glib::wrapper! {
    pub struct LangPage(ObjectSubclass<imp::PfbLangPage>)
    @extends Page, adw::NavigationPage, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl LangPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup(&self) {
        trace!("Setting up lang page");

        let imp = self.imp();
        imp.osk_layout_prefs.load_osk_layouts();
        imp.osk_layout_prefs
            .set_title(&gettextrs::gettext("Keyboard Layout"));

        MainContext::default().spawn_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                if let Err(e) = this.ensure_locale1().await {
                    warn!("Failed to init locale1 proxy: {e}");
                } else {
                    this.get_current_locale().await;
                }
            }
        ));

        self.connect_region_notify(clone!(
            #[weak(rename_to = this)]
            self,
            move |_| {
                this.update_lang_row(&this.region().clone());
            }
        ));
    }

    async fn set_localed_locale(&self, region: &str) -> Result<(), Box<dyn std::error::Error>> {
        let categories: Vec<String> = [
            "LANG",
            "LC_TIME",
            "LC_NUMERIC",
            "LC_MONETARY",
            "LC_MEASUREMENT",
            "LC_PAPER",
        ]
        .into_iter()
        .map(|k| format!("{k}={region}"))
        .collect();
        let refs: Vec<&str> = categories.iter().map(String::as_str).collect();

        let proxy = self.imp().locale1.borrow().as_ref().unwrap().clone();
        debug!("Setting localed locale to {region}");
        proxy.set_locale(&refs, true).await?;

        debug!("Updating region");
        self.set_region(region);

        Ok(())
    }

    fn update_lang_row(&self, region: &str) {
        let name = functions::lang_get_language_from_locale(region, region.into());
        self.imp().lang_row.set_subtitle(&name);
    }

    fn update_lang(&self, region: String) {
        self.imp().osk_layout_prefs.add_for_locale(&region, None);

        MainContext::default().spawn_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                if let Err(err) = this.set_localed_locale(&region).await {
                    warn!("Failed to set locale to {region}: {err}");
                }
            }
        ));
    }

    #[template_callback]
    fn on_lang_button_clicked(&self) {
        let lang_chooser = LanguageChooser::new();

        lang_chooser.connect_language_selected(clone!(
            #[weak(rename_to = this)]
            self,
            move |chooser| {
                let locale = chooser.language();
                debug!("locale '{locale}' selected");
                this.update_lang(locale.to_string());
                chooser.close();
            },
        ));

        lang_chooser.present(Some(self));
    }

    async fn ensure_locale1(&self) -> zbus::Result<()> {
        if self.imp().locale1.borrow().is_some() {
            return Ok(());
        }

        trace!("Connecting to system bus");
        if self.imp().connection.borrow().is_none() {
            let conn = Connection::system().await?;
            self.imp().connection.replace(Some(conn));
        }

        // We clone to not hold the `RefCell` ref across the `await` point
        let conn = self.imp().connection.borrow().as_ref().unwrap().clone();
        trace!("Creating locale1 proxy");
        let proxy = Locale1Proxy::new(&conn).await?;

        self.imp().locale1.replace(Some(proxy));

        Ok(())
    }

    async fn get_current_locale(&self) {
        // We clone to not hold the `RefCell` ref across the `await` point
        let proxy = self.imp().locale1.borrow().as_ref().unwrap().clone();
        let locales = proxy.locale().await.expect("Can't get locale");
        let region = locales
            .iter()
            .find_map(|l| l.strip_prefix("LANG=").map(|v| v.to_string()));

        if let Some(region) = region {
            debug!("Got region {region}");
            self.set_region(region);
        } else {
            debug!("Falling back to default locale");
            self.set_region("C.UTF-8");
        }
    }
}
