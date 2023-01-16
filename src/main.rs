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
pub mod glyphs;
pub mod project;
pub mod resources;
pub mod unicode;
pub mod utils;
pub mod views;
mod window;
use app::GerbApp;
use gtk::subclass::prelude::ObjectSubclassIsExt;

fn main() {
    //let mut env_args = std::env::args().skip(1).collect::<Vec<String>>();
    //if env_args.len() > 1 {
    //    eprintln!("Usage: gerb [/path/to/font.ufo]");
    //    drop(env_args);
    //    std::process::exit(-1);
    //}

    gtk::init().expect("Failed to initialize gtk");

    //let app = GerbApp::new(env_args);
    let app = GerbApp::new();
    app.add_main_option(
        "ufo",
        gtk::glib::Char('u' as i8),
        gtk::glib::OptionFlags::IN_MAIN,
        gtk::glib::OptionArg::Filename,
        "some description",
        Some("some other description"),
    );

    app.connect_handle_local_options(|_self, dict| {
        //std::dbg!(&_self);
        if let Some(mut ufo_path) = dict
            .lookup_value("ufo", None)
            .and_then(|var| var.get::<Vec<u8>>())
        {
            while ufo_path.ends_with(b"\0") {
                ufo_path.pop();
            }
            //std::dbg!(&ufo_path);
            //_self.imp().window.get().unwrap().emit_by_name::<()>("open-project", &[&ufo_path]);
            if let Ok(s) = String::from_utf8(ufo_path) {
                std::dbg!(&s);
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
