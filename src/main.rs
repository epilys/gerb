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

#![deny(clippy::dbg_macro)]

use gerb::prelude::*;
use gtk::glib::{OptionArg, OptionFlags};

fn main() {
    gtk::init().expect("Failed to initialize gtk");

    let app = Application::new();
    app.add_main_option(
        "ufo",
        glib::Char('u' as i8),
        OptionFlags::IN_MAIN,
        OptionArg::Filename,
        "UFO project path to load on launch",
        Some("Specify a UFO directory in your filesystem to load on launch"),
    );
    app.add_main_option(
        "version",
        glib::Char('v' as i8),
        OptionFlags::IN_MAIN,
        OptionArg::None,
        "show version",
        None,
    );
    app.add_main_option(
        "info",
        glib::Char(0),
        OptionFlags::IN_MAIN,
        OptionArg::None,
        "show program and build info",
        None,
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
        if dict
            .lookup_value("version", Some(glib::VariantTy::BOOLEAN))
            .is_some()
        {
            println!("{}", crate::VERSION_INFO);
            // Exit with success code.
            return 0;
        }
        if dict
            .lookup_value("info", Some(glib::VariantTy::BOOLEAN))
            .is_some()
        {
            println!("{}", crate::INFO);
            // Exit with success code.
            return 0;
        }

        -1
    });

    app.run();
}
