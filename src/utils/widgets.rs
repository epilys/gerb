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

use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;

thread_local! {
static ICONS: once_cell::unsync::Lazy<(Option<Pixbuf>, Option<Pixbuf>)> =
    once_cell::unsync::Lazy::new(|| {
        (
            crate::resources::icons::CHECKBOX_ICON.to_pixbuf(),
            crate::resources::icons::CHECKBOX_CHECKED_ICON.to_pixbuf(),
        )
    });
 }
#[derive(Debug, Default)]
pub struct ToggleButtonInner {
    active: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for ToggleButtonInner {
    const NAME: &'static str = "ToggleButton";
    type Type = ToggleButton;
    type ParentType = gtk::Button;
}

impl ObjectImpl for ToggleButtonInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_property(ToggleButton::ACTIVE, false);
        obj.connect_clicked(|obj| {
            obj.set_property(
                ToggleButton::ACTIVE,
                !obj.property::<bool>(ToggleButton::ACTIVE),
            );
        });
        obj.style_context().add_class("toggle-button");
        obj.set_always_show_image(true);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    ToggleButton::ACTIVE,
                    ToggleButton::ACTIVE,
                    ToggleButton::ACTIVE,
                    false,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            ToggleButton::ACTIVE => self.active.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(
        &self,
        obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            ToggleButton::ACTIVE => {
                let val = value.get::<bool>().unwrap();
                ICONS.with(|f| {
                    let img = if val {
                        gtk::Image::from_pixbuf(f.1.as_ref())
                    } else {
                        gtk::Image::from_pixbuf(f.0.as_ref())
                    };
                    crate::resources::UIIcon::image_into_surface(
                        &img,
                        obj.scale_factor(),
                        obj.window(),
                    );
                    obj.set_image(Some(&img));
                });

                self.active.set(val);
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl ButtonImpl for ToggleButtonInner {}
impl BinImpl for ToggleButtonInner {}
impl ContainerImpl for ToggleButtonInner {}
impl WidgetImpl for ToggleButtonInner {}

// [ref:needs_dev_doc]
glib::wrapper! {
    pub struct ToggleButton(ObjectSubclass<ToggleButtonInner>)
        @extends gtk::Button, gtk::Widget;
}

impl Default for ToggleButton {
    fn default() -> Self {
        Self::new()
    }
}

impl ToggleButton {
    pub const ACTIVE: &str = "active";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create ToggleButton");
        ret
    }

    pub fn set_active(&self, val: bool) {
        self.set_property(Self::ACTIVE, val);
    }
}

pub fn new_simple_error_dialog(
    title: Option<&str>,
    text: &str,
    secondary_text: Option<&str>,
    window: &gtk::Window,
) -> gtk::MessageDialog {
    let dialog = gtk::MessageDialog::new(
        Some(window),
        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        text,
    );
    dialog.set_secondary_text(secondary_text);
    dialog.set_secondary_use_markup(true);
    dialog.set_title(title.unwrap_or("Error"));
    dialog.set_use_markup(true);
    dialog
}

pub fn new_simple_info_dialog(
    title: Option<&str>,
    text: &str,
    secondary_text: Option<&str>,
    window: &gtk::Window,
) -> gtk::MessageDialog {
    let dialog = gtk::MessageDialog::new(
        Some(window),
        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::Close,
        text,
    );
    dialog.set_secondary_text(secondary_text);
    dialog.set_secondary_use_markup(true);
    dialog.set_title(title.unwrap_or("Information"));
    dialog.set_use_markup(true);
    dialog
}
