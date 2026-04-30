// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::pfb::{config, Window};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::action;
use std::cell::Cell;
use std::path::PathBuf;

pub fn app_get_default() -> Application {
    gio::Application::default()
        .unwrap()
        .downcast::<Application>()
        .expect("PfbApplication")
}

mod imp {
    use super::*;

    use gtk::glib;

    #[derive(Default)]
    pub struct PfbApplication {
        pub first_run: Cell<Option<bool>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbApplication {
        const NAME: &'static str = "PfbApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for PfbApplication {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup_actions();
        }
    }

    impl ApplicationImpl for PfbApplication {
        fn activate(&self) {
            self.obj().present_main_window();
        }

        fn startup(&self) {
            self.parent_startup();
        }
    }

    impl GtkApplicationImpl for PfbApplication {}
    impl AdwApplicationImpl for PfbApplication {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::PfbApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder::<Application>()
            .property("application-id", config::APP_ID)
            .property("register-session", true)
            .build()
    }
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    fn setup_actions(&self) {
        action!(
            self,
            "quit",
            glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _| {
                    app.quit();
                }
            )
        );
    }

    fn present_main_window(&self) {
        let window = self.active_window().unwrap_or_else(|| {
            let window = Window::new(self);
            self.inhibit(
                Some(&window),
                // Suspend would be enough but we don't want phrog to "lock" the screen
                gtk::ApplicationInhibitFlags::IDLE,
                Some("First Boot Setup"),
            );
            window.into()
        });

        window.present();
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(config::OUTPUTDIR)
    }
}
