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

use gtk::gdk::EventButton;
use gtk::prelude::GtkMenuExtManual;
use gtk::prelude::MenuShellExt;
use gtk::prelude::WidgetExt;

use gtk::Inhibit;

pub struct Menu {
    inner: gtk::Menu,
    buttons: Vec<gtk::MenuItem>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            inner: gtk::Menu::new(),
            buttons: vec![],
        }
    }

    pub fn add_button(mut self, label: &str) -> Self {
        let mut button: gtk::MenuItem = gtk::MenuItem::with_label(label);
        self.inner.append(&button);
        self.buttons.push(button);
        self
    }

    pub fn add_button_cb<F>(mut self, label: &str, callback: F) -> Self
    where
        F: Fn(&gtk::MenuItem, &EventButton) -> Inhibit + 'static,
    {
        let mut button: gtk::MenuItem = gtk::MenuItem::with_label(label);
        button.connect_button_press_event(callback);
        self.inner.append(&button);
        self.buttons.push(button);
        self
    }

    pub fn popup(&self) {
        self.inner.show_all();
        self.inner.popup_easy(gtk::gdk::BUTTON_SECONDARY, 0);
    }
}
