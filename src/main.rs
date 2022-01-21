/*
 * gerb
 *
 * Copyright 2021 - Manos Pitsidianakis
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

use gtk::{AboutDialog, DrawingArea};

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};

mod app;
mod window;
use app::GerbApp;

fn main() {
    gtk::init().expect("Failed to initialize gtk");

    let app = GerbApp::new();

    app.run();
}
