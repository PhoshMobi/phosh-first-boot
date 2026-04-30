// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::Object;

use crate::pfb::{app_get_default, Page, PageImpl};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use shared::dconf::DCONF_EXPORTS;

use log::{debug, warn};

const FIRST_RUN_SCHEMA: &str = "mobi.phosh.phrog";
const FIRST_RUN_KEY: &str = "first-run";

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::{glib, CompositeTemplate};

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/mobi/phosh/FirstBoot/pages/done_page.ui")]
    pub struct PfbDonePage {}

    #[glib::object_subclass]
    impl ObjectSubclass for PfbDonePage {
        const NAME: &'static str = "PfbDonePage";
        type Type = super::DonePage;
        type ParentType = Page;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PfbDonePage {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for PfbDonePage {}
    impl NavigationPageImpl for PfbDonePage {}
    impl PageImpl for PfbDonePage {}
}

glib::wrapper! {
    pub struct DonePage(ObjectSubclass<imp::PfbDonePage>)
    @extends Page, adw::NavigationPage, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl DonePage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn disable_first_boot(&self) {
        let source = gio::SettingsSchemaSource::default().unwrap();

        let Some(schema) = source.lookup(FIRST_RUN_SCHEMA, true) else {
            warn!(
                "Can't disable first run: '{}' schema not found",
                FIRST_RUN_SCHEMA
            );
            return;
        };

        if !schema.has_key(FIRST_RUN_KEY) {
            warn!(
                "Can't disable first run: '{}' schema not found",
                FIRST_RUN_KEY
            );
            return;
        }

        let settings = gio::Settings::new(FIRST_RUN_SCHEMA);

        settings.set_string(FIRST_RUN_KEY, "").unwrap();
    }

    fn export_settings(&self) {
        let app = app_get_default();
        let dir = app.output_dir();
        let mut errors = Vec::new();

        debug!("Exporting to '{}'", dir.display());
        for item in DCONF_EXPORTS {
            debug!("Exporting '{}'", item.path);
            if let Err(e) = item.export(&dir) {
                errors.push((item.filename, e));
            }
        }

        if !errors.is_empty() {
            for (path, err) in errors {
                warn!("Failed to export '{path}': {err}");
            }
        }
    }

    #[template_callback]
    fn on_get_started_clicked(&self, _button: gtk::Button) {
        self.disable_first_boot();

        self.export_settings();

        self.activate_action("app.quit", None)
            .expect("Action does not exit");
    }
}
