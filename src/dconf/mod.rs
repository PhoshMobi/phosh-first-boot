// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use log::info;

pub struct DconfItem {
    // Donf path
    #[allow(dead_code)]
    pub path: &'static str,
    // Output filename
    #[allow(dead_code)]
    pub filename: &'static str,
}

pub static DCONF_EXPORTS: &[DconfItem] = &[
    // OSK config
    DconfItem {
        path: "/mobi/phosh/osk/",
        filename: "mobi.phosh.osk.dconf",
    },
    // Keyboard layouts
    DconfItem {
        path: "/org/gnome/desktop/input-sources/",
        filename: "org.gnome.desktop.input-sources.dconf",
    },
];

impl DconfItem {
    pub fn import(&self, dir: &Path) -> io::Result<()> {
        let file = dir.join(self.filename);

        let mut file = File::open(file)?;
        let mut child = Command::new("dconf")
            .args(["load", self.path])
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        io::copy(&mut file, &mut stdin)?;
        drop(stdin);
        let status = child.wait()?;

        if !status.success() {
            return Err(io::Error::other(format!(
                "process exited with status {}",
                status
            )));
        }

        Ok(())
    }

    pub fn export(&self, dir: &Path) -> io::Result<()> {
        let file = dir.join(self.filename);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file)?;
        let mut child = Command::new("dconf")
            .args(["dump", self.path])
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdout = child.stdout.take().expect("Failed to get stdout");
        let bytes = io::copy(&mut stdout, &mut file)?;
        info!("copied {} bytes for '{}'", bytes, self.path);
        drop(stdout);
        let status = child.wait()?;

        if !status.success() {
            return Err(io::Error::other(format!(
                "process exited with status {}",
                status
            )));
        }
        // Make sure we see errors in the logs
        file.sync_all()?;

        Ok(())
    }
}
