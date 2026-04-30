// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod application;
pub mod config;
pub mod crypt;
pub mod page;
pub mod window;

pub use application::{app_get_default, Application};
pub use page::{Page, PageImpl};
pub use window::Window;

// Individual pages

pub mod done_page;
pub mod lang_page;
pub mod user_page;

pub use done_page::DonePage;
pub use lang_page::LangPage;
pub use user_page::UserPage;
