// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::Object;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use log::trace;

use crate::pfb::{Application, Page};
use crate::pfb::{DonePage, LangPage, UserPage};

mod imp {

    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use glib_macros::Properties;
    use gtk::{glib, CompositeTemplate};

    use super::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/mobi/phosh/FirstBoot/window.ui")]
    #[properties(wrapper_type = super::Window)]
    pub struct PfbWindow {
        #[template_child]
        pub start_page: TemplateChild<Page>,

        #[template_child]
        pub lang_page: TemplateChild<LangPage>,

        #[template_child]
        pub user_page: TemplateChild<UserPage>,

        #[template_child]
        pub done_page: TemplateChild<DonePage>,

        #[property(get, set)]
        pub locale: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbWindow {
        const NAME: &'static str = "PfbWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PfbWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup();
        }

        fn dispose(&self) {}
    }

    impl WidgetImpl for PfbWindow {}
    impl WindowImpl for PfbWindow {}
    impl ApplicationWindowImpl for PfbWindow {}
    impl AdwApplicationWindowImpl for PfbWindow {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::PfbWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl Window {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", Some(app)).build()
    }

    #[template_callback]
    fn on_page_changed(&self, _n: u32, _carousel: adw::Carousel) {}

    fn setup(&self) {
        trace!("Setting up main window");
    }
}
