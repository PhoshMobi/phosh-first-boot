// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::Object;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use glib::subclass::InitializingObject;
    use glib_macros::Properties;
    use gtk::{glib, CompositeTemplate};
    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/mobi/phosh/FirstBoot/page.ui")]
    #[properties(wrapper_type = super::Page)]
    pub struct PfbPage {
        // Page content
        #[property(get, set)]
        pub content: RefCell<Option<gtk::Widget>>,

        // All data setup? Only then is the `Next` button
        // made visible.
        #[property(get, set, construct, default = false)]
        pub can_go_next: Cell<bool>,

        // The tag of the next page
        #[property(get, set)]
        pub next: RefCell<String>,

        // The label on the go-next button
        #[property(get, set)]
        pub next_label: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbPage {
        const NAME: &'static str = "PfbPage";
        type Type = super::Page;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PfbPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup();
        }
    }

    impl WidgetImpl for PfbPage {}
    impl NavigationPageImpl for PfbPage {}
}

glib::wrapper! {
    pub struct Page(ObjectSubclass<imp::PfbPage>)
    @extends adw::NavigationPage, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl Page {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup(&self) {
        // Translators: The label on the button to go to the next page
        self.set_next_label(gettextrs::gettext("Next"));
    }

    #[template_callback]
    fn string_to_variant(&self, string: Option<String>) -> glib::Variant {
        let Some(string) = string else {
            return "".to_variant();
        };

        string.to_variant()
    }

    pub fn go_next(&self) {
        let tag = self.next();
        self.activate_action("navigation.push", Some(&tag.to_variant()))
            .expect("Action does not exit");
    }
}

impl Default for Page {
    fn default() -> Self {
        glib::Object::new()
    }
}

pub trait PageImpl: NavigationPageImpl + ObjectSubclass<Type: IsA<Page>> {}
unsafe impl<Obj: PageImpl + NavigationPageImpl> IsSubclassable<Obj> for Page {}
