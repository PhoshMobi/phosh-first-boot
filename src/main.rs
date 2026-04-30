// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

mod pfb;

use crate::pfb::{
    config::{GETTEXT_PACKAGE, LOCALEDIR},
    Application,
};

use gtk::prelude::*;
use gtk::{gio, glib};

use log::LevelFilter;

fn main() -> glib::ExitCode {
    pretty_env_logger::formatted_builder()
        .filter(None, LevelFilter::Warn)
        .parse_default_env()
        .init();

    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");

    gio::resources_register_include!("mobi.phosh.FirstBoot.gresource")
        .expect("Failed to register resources.");

    let app = Application::new();
    app.run()
}
