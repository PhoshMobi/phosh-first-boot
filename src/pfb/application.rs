// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::pfb::{config, defaults::Defaults, Window};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::action;
use log::info;
use std::cell::{OnceCell, RefCell};
use std::path::{Path, PathBuf};

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
        pub defaults_path: RefCell<PathBuf>,
        pub defaults: OnceCell<Defaults>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbApplication {
        const NAME: &'static str = "PfbApplication";
        type Type = super::Application;
        type ParentType = adw::Application;

        fn new() -> Self {
            Self {
                defaults_path: RefCell::new(Path::new(config::DEFAULTSDIR).join("defaults.conf")),
                defaults: OnceCell::new(),
            }
        }
    }

    impl ObjectImpl for PfbApplication {
        fn constructed(&self) {
            self.parent_constructed();

            let defaults_path = &*self.defaults_path.borrow();
            self.obj().add_main_option(
                "defaults",
                glib::Char::from(b'd'),
                glib::OptionFlags::NONE,
                glib::OptionArg::String,
                &format!(
                    "Config file with defaults, (default is {})",
                    defaults_path.display()
                ),
                Some("FILE"),
            );

            self.obj().setup_actions();
        }
    }

    impl ApplicationImpl for PfbApplication {
        fn activate(&self) {
            self.obj().present_main_window();
        }

        fn startup(&self) {
            self.parent_startup();

            let defaults = Defaults::load(&*self.defaults_path.borrow());
            self.defaults
                .set(defaults)
                .expect("Defaults already initialized");
        }

        fn handle_local_options(
            &self,
            options: &glib::VariantDict,
        ) -> std::ops::ControlFlow<glib::ExitCode> {
            if let Ok(Some(filename)) = options.lookup::<String>("defaults") {
                info!("Using default config {}", filename);
                *self.defaults_path.borrow_mut() = PathBuf::from(filename);
            }

            std::ops::ControlFlow::Continue(())
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

    pub fn defaults(&self) -> &Defaults {
        self.imp().defaults.get().expect("Defaults not initialized")
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(config::OUTPUTDIR)
    }
}
