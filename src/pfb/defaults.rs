// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use log::warn;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Storage {
    #[default]
    Luks,
    Directory,
}

#[derive(Debug, Deserialize, Default)]
pub struct Defaults {
    pub user: UserDefaults,
}

#[derive(Debug, Deserialize, Default)]
pub struct UserDefaults {
    // Auxiliary groups we add the user to
    pub aux_groups: Vec<String>,
    // Minimum username length
    #[serde(default = "default_username_min_len")]
    pub username_min_len: usize,
    // Minimum pin/password length
    #[serde(default = "default_pin_min_len")]
    pub pin_min_len: usize,
    // homed storage backend
    pub storage: Storage,
}

const fn default_username_min_len() -> usize {
    2
}

const fn default_pin_min_len() -> usize {
    4
}

impl Defaults {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(err) => {
                warn!(
                    "Failed to load default config '{}': {err}",
                    path.as_ref().display()
                );
                return Self::default();
            }
        };

        toml::from_str(&contents).unwrap_or_else(|err| {
            warn!(
                "Failed to parse default config '{}': {err}",
                path.as_ref().display()
            );
            Self::default()
        })
    }
}

impl std::fmt::Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Storage::Luks => "luks",
            Storage::Directory => "directory",
        };

        f.write_str(value)
    }
}
