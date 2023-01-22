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

use gtk::glib::Cast;
use gtk::prelude::{GtkMenuExt, GtkMenuExtManual, GtkMenuItemExt, IsA, MenuShellExt, WidgetExt};

use std::borrow::Cow;

pub struct Menu {
    inner: gtk::Menu,
    buttons: Vec<gtk::MenuItem>,
    title: Option<Cow<'static, str>>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            inner: gtk::Menu::builder().take_focus(true).visible(true).build(),
            buttons: vec![],
            title: None,
        }
    }

    pub fn attach_to_widget(self, widget: &impl IsA<gtk::Widget>) -> Self {
        self.inner.set_attach_widget(Some(widget.upcast_ref()));
        self
    }

    pub fn title(mut self, title: Option<Cow<'static, str>>) -> Self {
        self.title = title;
        if let Some(title) = self.title.as_ref() {
            let name = gtk::MenuItem::builder()
                .label(title.as_ref())
                .sensitive(false)
                .visible(true)
                .build();
            self.inner.append(&name);
        }
        self
    }

    pub fn separator(self) -> Self {
        self.inner
            .append(&gtk::SeparatorMenuItem::builder().visible(true).build());
        self
    }

    pub fn add_button(mut self, label: &str) -> Self {
        let button: gtk::MenuItem = gtk::MenuItem::with_label(label);
        self.inner.append(&button);
        self.buttons.push(button);
        self
    }

    pub fn add_button_cb<F>(mut self, label: &str, callback: F) -> Self
    where
        F: Fn(&gtk::MenuItem) + 'static,
    {
        let button: gtk::MenuItem = gtk::MenuItem::builder()
            .label(label)
            .sensitive(true)
            .visible(true)
            .build();
        button.connect_activate(callback);
        self.inner.append(&button);
        self.buttons.push(button);
        self
    }

    pub fn popup(&self) {
        self.inner.show_all();
        self.inner.popup_easy(gtk::gdk::BUTTON_SECONDARY, 0);
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
