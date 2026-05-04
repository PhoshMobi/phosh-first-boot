// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use shared::dconf;

use clap::Parser;
use log::LevelFilter;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

use log::{info, warn};

const DONE_FILE: &str = "phosh-first-boot-done";

#[derive(Parser)]
struct Cli {
    /// The directory to import from
    #[arg(short,long,default_value=PathBuf::from(r"/run/phosh-first-boot/").into_os_string())]
    dir: std::path::PathBuf,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn touch_done() -> io::Result<()> {
    let done = glib::user_config_dir().join(DONE_FILE);

    OpenOptions::new().create(true).write(true).open(done)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let level = if args.verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    };

    pretty_env_logger::formatted_builder()
        .filter(None, level)
        .parse_default_env()
        .init();

    info!("Importing from {:?}", args.dir);

    let mut errors = Vec::new();
    for item in dconf::DCONF_EXPORTS {
        info!("Importing {}", item.path);
        if let Err(e) = item.import(&args.dir) {
            warn!("Failed to import '{}': {e}", item.path);
            errors.push((item.filename, e));
        }
    }

    if !errors.is_empty() {
        return Err(io::Error::other("Failed to import"));
    }

    if let Err(e) = touch_done() {
        return Err(io::Error::other(std::format!(
            "Failed to touch '{}' file: {}",
            DONE_FILE,
            e
        )));
    }

    Ok(())
}
