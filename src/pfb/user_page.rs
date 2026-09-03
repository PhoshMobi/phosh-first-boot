// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::Object;

use crate::pfb::{application, crypt, Page, PageImpl};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::MainContext;
use gtk::glib;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use zbus::{proxy, Connection};

use log::{debug, trace, warn};

#[proxy(
    interface = "org.freedesktop.home1.Manager",
    default_service = "org.freedesktop.home1",
    default_path = "/org/freedesktop/home1"
)]

trait HomeManager {
    #[zbus(allow_interactive_auth)]
    async fn create_home(&self, user_record: &str) -> zbus::Result<()>;
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct SecretSection {
    password: Vec<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct PrivilegedSection {
    hashed_password: Vec<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct UserRecord {
    user_name: String,
    real_name: String,
    home_directory: String,
    shell: String,
    storage: String,
    disposition: String,
    enforce_password_policy: bool,
    last_change_u_sec: u128,
    last_password_change_u_sec: u128,
    secret: SecretSection,
    privileged: PrivilegedSection,
    member_of: Vec<String>,
}

mod imp {
    use glib::subclass::InitializingObject;
    use glib_macros::Properties;
    use gtk::{glib, CompositeTemplate};
    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/mobi/phosh/FirstBoot/pages/user_page.ui")]
    #[properties(wrapper_type = super::UserPage)]
    pub struct PfbUserPage {
        #[template_child]
        pub user_name_entry_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub full_name_entry_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub pass_entry_row: TemplateChild<adw::PasswordEntryRow>,

        #[template_child]
        pub error_banner: TemplateChild<adw::Banner>,

        // Currently creating the user record
        #[property(get, set)]
        pub active: Cell<bool>,

        // Username derived from full name
        pub auto_user_name: Cell<bool>,
        pub username_min_len: Cell<usize>,
        pub pin_min_len: Cell<usize>,

        pub(super) connection: RefCell<Option<Connection>>,
        pub(super) home1: RefCell<Option<HomeManagerProxy<'static>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbUserPage {
        const NAME: &'static str = "PfbUserPage";
        type Type = super::UserPage;
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
    impl ObjectImpl for PfbUserPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup();
        }
    }

    impl WidgetImpl for PfbUserPage {}
    impl NavigationPageImpl for PfbUserPage {}
    impl PageImpl for PfbUserPage {}
}

glib::wrapper! {
    pub struct UserPage(ObjectSubclass<imp::PfbUserPage>)
    @extends Page, adw::NavigationPage, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl UserPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup(&self) {
        let app = application::app_get_default();
        let defaults = app.defaults();

        self.imp().auto_user_name.set(true);
        self.imp()
            .username_min_len
            .set(defaults.user.username_min_len);
        self.imp().pin_min_len.set(defaults.user.pin_min_len);

        self.upcast_ref::<Page>().set_can_go_next(false);

        MainContext::default().spawn_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                if let Err(e) = this.ensure_home1().await {
                    warn!("Failed to init home1 proxy: {e}");
                }
            }
        ));
    }

    fn is_allowed_username_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
    }

    fn is_valid_username(&self, s: &str) -> bool {
        s.len() >= self.imp().username_min_len.get()
            && s.chars().all(|c| self.is_allowed_username_char(c))
    }

    fn sanitize_username(&self, full_name: &str) -> String {
        // Could use deunicode crate but since we have glib…
        let ascii = glib::convert_with_fallback(
            full_name.as_bytes(),
            "ASCII//TRANSLIT",
            "UTF-8",
            Some("_"),
        )
        .unwrap_or_default();
        let mut name = str::from_utf8(&ascii.0).unwrap_or_default();

        // If conversion failed just use the full_name and let our ascii filter do the job
        if name.is_empty() {
            name = full_name;
        }

        name.chars()
            .filter_map(|c| {
                if self.is_allowed_username_char(c) {
                    Some(c.to_ascii_lowercase())
                } else {
                    None
                }
            })
            .collect()

        // TODO: check if username already exists
    }

    #[template_callback]
    fn on_full_name_row_text_changed(&self) {
        let imp = self.imp();

        if !imp.auto_user_name.get() {
            return;
        }

        let username = self.sanitize_username(&imp.full_name_entry_row.text());
        if username.is_empty() {
            return;
        }

        imp.user_name_entry_row.set_text(&username);
    }

    #[template_callback]
    fn on_user_name_entry_row_text_changed(&self) {
        let imp = self.imp();

        // Username empty, always switch back to automatic
        let username = imp.user_name_entry_row.text();

        if username.is_empty() {
            trace!("Re-enabling auto-username");
            imp.auto_user_name.set(true);
            return;
        }

        // No focus -> this is an automatic update
        if !imp.user_name_entry_row.has_focus() && imp.user_name_entry_row.focus_child().is_none() {
            return;
        }

        // Auto updates already off
        if !imp.auto_user_name.get() {
            return;
        }

        debug!("Using custom username, disabling automatic updates");
        imp.auto_user_name.set(false);
    }

    #[template_callback]
    fn state_to_ready(&self, user_name: String, pin1: String, pin2: String, active: bool) -> bool {
        // Async operation in progress
        if active {
            return false;
        }

        if user_name.len() >= self.imp().username_min_len.get()
            && pin1.len() >= self.imp().pin_min_len.get()
            && pin1 == pin2
        {
            return self.is_valid_username(&user_name);
        }

        false
    }

    async fn ensure_home1(&self) -> zbus::Result<()> {
        if self.imp().home1.borrow().is_some() {
            return Ok(());
        }

        if self.imp().connection.borrow().is_none() {
            trace!("Connecting to system bus");
            let conn = Connection::system().await?;
            self.imp().connection.replace(Some(conn));
        }

        // We clone to not hold the `RefCell` ref across the `await` point
        let conn = self.imp().connection.borrow().as_ref().unwrap().clone();
        trace!("Creating home1 proxy");
        let proxy = HomeManagerProxy::new(&conn).await?;

        self.imp().home1.replace(Some(proxy));

        Ok(())
    }

    async fn create_homed_user(
        &self,
        user_name: String,
        real_name: String,
        password: String,
        aux_groups: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Example record from homectl:
        // {
        //   "enforcePasswordPolicy": false,
        //   "userName": "asdf",
        //   "disposition": "regular",
        //   "lastChangeUSec": 1771451009859149,
        //   "lastPasswordChangeUSec": 1771451009859149,
        //   "secret": {
        //     "password": ["1234"]},
        //     "privileged": {
        //       "hashedPassword": ["$y$j9T$CbejJNQCo5/LHB1rgL6FV0$zddBhz9ayMcWNj0EOgUcBXKhvIj6gEFBw0e51zFwyz2"]
        //   }
        // }

        let now = if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
            now.as_micros()
        } else {
            warn!("Failed to get system time");
            0
        };
        let hash = crypt::Crypt::hash(&password)?;
        let record = UserRecord {
            home_directory: format!("/home/{user_name}"),
            user_name: user_name.clone(),
            real_name,
            disposition: "regular".into(),
            shell: "/bin/bash".into(),
            storage: "directory".into(),
            enforce_password_policy: false,
            last_change_u_sec: now,
            last_password_change_u_sec: now,
            member_of: aux_groups,
            secret: SecretSection {
                password: vec![password],
            },
            privileged: PrivilegedSection {
                hashed_password: vec![hash],
            },
        };

        let json_record = serde_json::to_string(&record)?;

        let proxy = self.imp().home1.borrow().as_ref().unwrap().clone();
        trace!("Creating user '{}'", user_name);
        proxy.create_home(&json_record).await?;

        Ok(())
    }

    #[template_callback]
    fn on_create_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        let username = imp.user_name_entry_row.text().to_string();
        let fullname = imp.full_name_entry_row.text().to_string();
        let pin = imp.pass_entry_row.text().to_string();

        // Get defaults from config
        let app = application::app_get_default();
        let defaults = app.defaults();
        let aux_groups = defaults.user.aux_groups.clone();

        debug!(
            "Creating user {}, groups: {}",
            username,
            aux_groups.join(", ")
        );

        imp.error_banner.set_revealed(false);
        self.set_active(true);
        MainContext::default().spawn_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                match this
                    .create_homed_user(username.clone(), fullname, pin, aux_groups)
                    .await
                {
                    Ok(_) => {
                        debug!("User {} created.", username);
                        this.upcast_ref::<Page>().set_can_go_next(true);
                        this.upcast_ref::<Page>().go_next();
                    }
                    Err(e) => {
                        let imp = this.imp();

                        imp.error_banner.set_title("Failed to create user");
                        imp.error_banner.set_revealed(true);
                        warn!("Failed to create user: {e}");
                    }
                }
                this.set_active(false);
            }
        ));
    }
}
