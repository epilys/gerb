/*
 * gerb
 *
 * Copyright 2022 - Manos Pitsidianakis
 *
 * This file is part of gerb.
 *
 * gerb is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * gerb is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with gerb. If not, see <http://www.gnu.org/licenses/>.
 */

#[macro_use]
extern crate glib;

use gtk::prelude::*;

mod app;
pub use app::*;
mod glyph_edit;
pub use glyph_edit::*;
pub mod glyphs;
pub mod project;
pub mod resources;
pub mod ufo;
pub mod unicode;
pub mod utils;
pub(crate) use utils::UI_EDITABLE;
pub mod views;
pub mod window;
use app::Application;
use gtk::subclass::prelude::ObjectSubclassIsExt;
pub use window::Workspace;

pub const APPLICATION_NAME: &str = "gerb";
pub const APPLICATION_ID: &str = "com.epilys.gerb";

fn main() {
    gtk::init().expect("Failed to initialize gtk");

    let app = Application::new();
    app.add_main_option(
        "ufo",
        gtk::glib::Char('u' as i8),
        gtk::glib::OptionFlags::IN_MAIN,
        gtk::glib::OptionArg::Filename,
        "some description",
        Some("some other description"),
    );

    app.connect_handle_local_options(|_self, dict| {
        if let Some(mut ufo_path) = dict
            .lookup_value("ufo", None)
            .and_then(|var| var.get::<Vec<u8>>())
        {
            while ufo_path.ends_with(b"\0") {
                ufo_path.pop();
            }
            if let Ok(s) = String::from_utf8(ufo_path) {
                _self
                    .imp()
                    .env_args
                    .set(vec![s])
                    .expect("Failed to initialize gtk");
            } else {
                _self
                    .imp()
                    .env_args
                    .set(vec![])
                    .expect("Failed to initialize gtk");
            }
        } else {
            _self
                .imp()
                .env_args
                .set(vec![])
                .expect("Failed to initialize gtk");
        }

        -1
    });

    app.run();
}
