// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::Object;

use crate::pfb::{Page, PageImpl};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::MainContext;
use gtk::glib;

use std::cell::{Cell, RefCell};

use zbus::{proxy, Connection};

use log::{debug, trace, warn};

// The longest label allowed in a DNS name
const HOSTNAME_MAX_LEN: usize = 63;

#[proxy(
    interface = "org.freedesktop.hostname1",
    default_service = "org.freedesktop.hostname1",
    default_path = "/org/freedesktop/hostname1"
)]
trait Hostname1 {
    #[zbus(allow_interactive_auth)]
    async fn set_static_hostname(&self, hostname: &str, interactive: bool) -> zbus::Result<()>;

    #[zbus(allow_interactive_auth)]
    async fn set_pretty_hostname(&self, hostname: &str, interactive: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn chassis(&self) -> zbus::Result<String>;
}

/// Chassis types known by hostnamed
#[derive(Clone, Copy, Default, PartialEq, Debug)]
enum Chassis {
    Handset,
    Tablet,
    Laptop,
    Desktop,
    Server,
    Watch,
    Vm,
    #[default]
    Other,
}

impl Chassis {
    fn from_hostnamed(chassis: &str) -> Self {
        match chassis {
            "handset" => Chassis::Handset,
            "tablet" => Chassis::Tablet,
            "laptop" | "convertible" => Chassis::Laptop,
            "desktop" => Chassis::Desktop,
            "server" => Chassis::Server,
            "watch" => Chassis::Watch,
            "vm" | "container" => Chassis::Vm,
            _ => Chassis::Other,
        }
    }

    fn icon_resource(&self) -> &'static str {
        match self {
            Chassis::Handset => "/mobi/phosh/FirstBoot/chassis/handset.svg",
            Chassis::Tablet => "/mobi/phosh/FirstBoot/chassis/tablet.svg",
            Chassis::Laptop => "/mobi/phosh/FirstBoot/chassis/laptop.svg",
            Chassis::Desktop => "/mobi/phosh/FirstBoot/chassis/desktop.svg",
            Chassis::Server => "/mobi/phosh/FirstBoot/chassis/server.svg",
            Chassis::Watch => "/mobi/phosh/FirstBoot/chassis/watch.svg",
            Chassis::Vm => "/mobi/phosh/FirstBoot/chassis/vm.svg",
            Chassis::Other => "/mobi/phosh/FirstBoot/chassis/generic.svg",
        }
    }

    /// The suggested device name for a device owned by `owner`
    fn suggested_name(&self, owner: &str) -> String {
        // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Phone"
        let template = match self {
            Chassis::Handset => gettextrs::gettext("{}'s Phone"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Tablet"
            Chassis::Tablet => gettextrs::gettext("{}'s Tablet"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Laptop"
            Chassis::Laptop => gettextrs::gettext("{}'s Laptop"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Desktop"
            Chassis::Desktop => gettextrs::gettext("{}'s Desktop"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Server"
            Chassis::Server => gettextrs::gettext("{}'s Server"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Watch"
            Chassis::Watch => gettextrs::gettext("{}'s Watch"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Virtual Machine"
            Chassis::Vm => gettextrs::gettext("{}'s Virtual Machine"),
            // Translators: The {} is replaced by the owner's name, e.g. "John Doe's Device"
            Chassis::Other => gettextrs::gettext("{}'s Device"),
        };

        template.replace("{}", owner)
    }
}

/// Turn a free form device name like "John Doe's Phone" into a
/// hostname like "john-does-phone".
fn pretty_to_static(pretty: &str) -> String {
    let ascii =
        glib::convert_with_fallback(pretty.as_bytes(), "ASCII//TRANSLIT", "UTF-8", Some("_"))
            .unwrap_or_default();
    let mut name = str::from_utf8(&ascii.0).unwrap_or_default();

    // If conversion failed just use the pretty name and let our ascii filter do the job
    if name.is_empty() {
        name = pretty;
    }

    let mut hostname = String::new();
    for c in name.chars() {
        if c == '\'' || c == '’' {
            // "John's" -> "johns"
            continue;
        }

        if c.is_ascii_alphanumeric() {
            hostname.push(c.to_ascii_lowercase());
        } else if !hostname.is_empty() && !hostname.ends_with('-') {
            hostname.push('-');
        }
    }

    // Only ASCII so truncating can't split a character
    hostname.truncate(HOSTNAME_MAX_LEN);
    hostname.trim_end_matches('-').to_string()
}

/// A single label hostname
fn is_valid_hostname(hostname: &str) -> bool {
    !hostname.is_empty()
        && hostname.len() <= HOSTNAME_MAX_LEN
        && !hostname.starts_with('-')
        && !hostname.ends_with('-')
        && hostname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

mod imp {
    use glib::subclass::InitializingObject;
    use glib_macros::Properties;
    use gtk::{glib, CompositeTemplate};

    use super::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/mobi/phosh/FirstBoot/pages/hostname_page.ui")]
    #[properties(wrapper_type = super::HostnamePage)]
    pub struct PfbHostnamePage {
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub name_group: TemplateChild<adw::PreferencesGroup>,

        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub error_banner: TemplateChild<adw::Banner>,

        // Currently setting the hostname
        #[property(get, set)]
        pub active: Cell<bool>,

        // The name of the new user, used to suggest a device name
        #[property(get, set)]
        pub full_name: RefCell<String>,
        #[property(get, set)]
        pub user_name: RefCell<String>,

        // Only suggest a name the first time the page is shown
        pub suggested: Cell<bool>,
        pub(super) chassis: Cell<Chassis>,

        pub(super) connection: RefCell<Option<Connection>>,
        pub(super) hostname1: RefCell<Option<Hostname1Proxy<'static>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfbHostnamePage {
        const NAME: &'static str = "PfbHostnamePage";
        type Type = super::HostnamePage;
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
    impl ObjectImpl for PfbHostnamePage {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup();
        }
    }

    impl WidgetImpl for PfbHostnamePage {}
    impl NavigationPageImpl for PfbHostnamePage {}
    impl PageImpl for PfbHostnamePage {}
}

glib::wrapper! {
    pub struct HostnamePage(ObjectSubclass<imp::PfbHostnamePage>)
    @extends Page, adw::NavigationPage, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl HostnamePage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup(&self) {
        trace!("Setting up hostname page");

        self.upcast_ref::<Page>().set_can_go_next(false);
        self.set_chassis(Chassis::default());
    }

    fn set_chassis(&self, chassis: Chassis) {
        let imp = self.imp();

        debug!("Chassis: {chassis:?}");
        imp.chassis.set(chassis);
        imp.status_page
            .set_paintable(Some(&gtk::Svg::from_resource(chassis.icon_resource())));
    }

    fn owner_name(&self) -> String {
        let full_name = self.full_name();
        if full_name.is_empty() {
            self.user_name()
        } else {
            full_name
        }
    }

    async fn ensure_hostname1(&self) -> zbus::Result<()> {
        let imp = self.imp();

        if imp.hostname1.borrow().is_some() {
            return Ok(());
        }

        if imp.connection.borrow().is_none() {
            trace!("Connecting to system bus");
            let conn = Connection::system().await?;
            imp.connection.replace(Some(conn));
        }

        // We clone to not hold the `RefCell` ref across the `await` point
        let conn = imp.connection.borrow().as_ref().unwrap().clone();
        trace!("Creating hostname1 proxy");
        let proxy = Hostname1Proxy::new(&conn).await?;

        imp.hostname1.replace(Some(proxy));

        Ok(())
    }

    async fn get_chassis(&self) -> zbus::Result<Chassis> {
        self.ensure_hostname1().await?;

        // We clone to not hold the `RefCell` ref across the `await` point
        let proxy = self.imp().hostname1.borrow().as_ref().unwrap().clone();
        let chassis = proxy.chassis().await?;

        Ok(Chassis::from_hostnamed(&chassis))
    }

    async fn set_hostnamed_hostname(&self, pretty: &str) -> zbus::Result<()> {
        let hostname = pretty_to_static(pretty);

        self.ensure_hostname1().await?;

        let proxy = self.imp().hostname1.borrow().as_ref().unwrap().clone();
        debug!("Setting hostname to '{hostname}', pretty hostname to '{pretty}'");
        proxy.set_static_hostname(&hostname, true).await?;
        proxy.set_pretty_hostname(pretty, true).await?;

        Ok(())
    }

    #[template_callback]
    fn on_showing(&self) {
        if self.imp().suggested.replace(true) {
            return;
        }

        MainContext::default().spawn_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                let imp = this.imp();

                match this.get_chassis().await {
                    Ok(chassis) => this.set_chassis(chassis),
                    Err(e) => warn!("Failed to get chassis: {e}"),
                }

                let name = imp.chassis.get().suggested_name(&this.owner_name());
                debug!("Suggesting device name '{name}'");
                imp.name_entry_row.set_text(&name);
            }
        ));
    }

    #[template_callback]
    fn on_name_entry_row_text_changed(&self) {
        let imp = self.imp();
        let hostname = pretty_to_static(imp.name_entry_row.text().trim());

        if is_valid_hostname(&hostname) {
            imp.name_entry_row.remove_css_class("error");
            // Translators: The {} is replaced by the hostname derived from the device name
            let description = gettextrs::gettext("Hostname: {}").replace("{}", &hostname);
            imp.name_group.set_description(Some(&description));
        } else {
            imp.name_entry_row.add_css_class("error");
            imp.name_group.set_description(None);
        }
    }

    #[template_callback]
    fn state_to_ready(&self, name: String, active: bool) -> bool {
        // Async operation in progress
        if active {
            return false;
        }

        is_valid_hostname(&pretty_to_static(name.trim()))
    }

    #[template_callback]
    fn on_next_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        let name = imp.name_entry_row.text().trim().to_string();

        imp.error_banner.set_revealed(false);
        self.set_active(true);
        MainContext::default().spawn_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                match this.set_hostnamed_hostname(&name).await {
                    Ok(_) => {
                        this.upcast_ref::<Page>().set_can_go_next(true);
                        this.upcast_ref::<Page>().go_next();
                    }
                    Err(e) => {
                        let imp = this.imp();

                        imp.error_banner
                            .set_title(&gettextrs::gettext("Failed to set device name"));
                        imp.error_banner.set_revealed(true);
                        warn!("Failed to set hostname: {e}");
                    }
                }
                this.set_active(false);
            }
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_to_static_typpical() {
        for (pretty, expected) in [
            ("John Doe's Phone", "john-does-phone"),
            ("John Doe\u{2019}s Tablet", "john-does-tablet"),
            ("my-phone", "my-phone"),
            ("MyPhone", "myphone"),
            ("Pixel 9 Pro", "pixel-9-pro"),
        ] {
            assert_eq!(pretty_to_static(pretty), expected, "{pretty:?}");
        }
    }

    #[test]
    fn pretty_to_static_trim() {
        for (pretty, expected) in [
            ("  My   Phone  ", "my-phone"),
            ("My !! Phone", "my-phone"),
            ("Jane_Doe.1 Laptop", "jane-doe-1-laptop"),
            ("---x---", "x"),
            ("a - b", "a-b"),
            ("(Phone)", "phone"),
        ] {
            assert_eq!(pretty_to_static(pretty), expected, "{pretty:?}");
        }
    }

    #[test]
    fn pretty_to_static_nothing_usable() {
        for pretty in ["", " ", "---", "'", "!?*"] {
            assert_eq!(pretty_to_static(pretty), "", "{pretty:?}");
        }
    }

    #[test]
    fn pretty_to_static_truncates() {
        let pretty = "a".repeat(HOSTNAME_MAX_LEN + 20);
        assert_eq!(pretty_to_static(&pretty).len(), HOSTNAME_MAX_LEN);

        // Separator at the end must not leave a trailing hyphen
        let pretty = format!("{} b", "a".repeat(HOSTNAME_MAX_LEN - 1));
        let hostname = pretty_to_static(&pretty);
        assert!(!hostname.ends_with('-'), "{hostname:?}");
        assert!(is_valid_hostname(&hostname), "{hostname:?}");
    }

    #[test]
    fn pretty_to_static_is_valid_or_empty() {
        for pretty in ["  ", "-", "a.b.c", "a_b", "\u{1F4F1}", "\t\n"] {
            let hostname = pretty_to_static(pretty);
            assert!(
                hostname.is_empty() || is_valid_hostname(&hostname),
                "{pretty:?} -> {hostname:?}"
            );
        }
    }

    #[test]
    fn valid_hostnames() {
        for hostname in [
            "a",
            "A1",
            "john-does-phone",
            "0",
            "a-b-c",
            &"a".repeat(HOSTNAME_MAX_LEN),
        ] {
            assert!(is_valid_hostname(hostname), "{hostname:?}");
        }
    }

    #[test]
    fn invalid_hostnames() {
        for hostname in [
            "",
            "-a",
            "a-",
            "-",
            "a b",
            "a_b",
            "a.b",
            "a.",
            "a'b",
            "\u{e9}",
            "a\n",
            &"a".repeat(HOSTNAME_MAX_LEN + 1),
        ] {
            assert!(!is_valid_hostname(hostname), "{hostname:?}");
        }
    }

    #[test]
    fn chassis_from_hostnamed() {
        for (chassis, expected) in [
            ("handset", Chassis::Handset),
            ("tablet", Chassis::Tablet),
            ("laptop", Chassis::Laptop),
            ("convertible", Chassis::Laptop),
            ("desktop", Chassis::Desktop),
            ("server", Chassis::Server),
            ("watch", Chassis::Watch),
            ("vm", Chassis::Vm),
            ("container", Chassis::Vm),
            ("embedded", Chassis::Other),
            ("", Chassis::Other),
            ("doesnotexist", Chassis::Other),
        ] {
            assert_eq!(Chassis::from_hostnamed(chassis), expected, "{chassis:?}");
        }
    }

    #[test]
    fn suggested_name() {
        assert_eq!(
            Chassis::Handset.suggested_name("John Doe"),
            "John Doe's Phone"
        );
        assert_eq!(
            Chassis::Tablet.suggested_name("John Doe"),
            "John Doe's Tablet"
        );
    }
}
