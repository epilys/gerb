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

use glib::{clone, ParamFlags, ParamSpec, ParamSpecBoolean, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct ViewHideBoxInner {
    show_grid_btn: OnceCell<gtk::CheckButton>,
    show_guidelines_btn: OnceCell<gtk::CheckButton>,
    show_handles_btn: OnceCell<gtk::CheckButton>,
    inner_fill_btn: OnceCell<gtk::CheckButton>,
    show_total_area_btn: OnceCell<gtk::CheckButton>,
    show_grid: Cell<bool>,
    show_guidelines: Cell<bool>,
    show_handles: Cell<bool>,
    inner_fill: Cell<bool>,
    show_total_area: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for ViewHideBoxInner {
    const NAME: &'static str = "ViewHideBoxInner";
    type Type = ViewHideBox;
    type ParentType = gtk::Box;
}

impl ObjectImpl for ViewHideBoxInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.show_grid.set(true);
        self.show_guidelines.set(true);
        self.show_handles.set(true);
        self.inner_fill.set(false);
        self.show_total_area.set(true);
        //obj.set_orientation(gtk::Orientation::Vertical);
        //obj.set_orientation(gtk::Orientation::Horizontal);
        obj.set_expand(false);
        obj.set_halign(gtk::Align::End);
        obj.set_valign(gtk::Align::End);
        obj.set_spacing(5);
        obj.set_visible(true);
        obj.set_can_focus(true);

        for (property, label, field) in [
            ("show-grid", "Show grid", &self.show_grid_btn),
            (
                "show-guidelines",
                "Show guidelines",
                &self.show_guidelines_btn,
            ),
            ("show-handles", "Show handles", &self.show_handles_btn),
            ("inner-fill", "Inner fill", &self.inner_fill_btn),
            (
                "show-total-area",
                "Show total area",
                &self.show_total_area_btn,
            ),
        ] {
            let btn = gtk::CheckButton::with_label(label);
            btn.set_visible(true);
            btn.set_active(false);
            obj.pack_start(&btn, false, false, 0);
            obj.bind_property(property, &btn, "active")
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();
            field.set(btn).expect("Failed to create ViewHideBox");
        }
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        "show-grid",
                        "show-grid",
                        "show-grid",
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "show-guidelines",
                        "show-guidelines",
                        "show-guidelines",
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "show-handles",
                        "show-handles",
                        "show-handles",
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "inner-fill",
                        "inner-fill",
                        "inner-fill",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "show-total-area",
                        "show-total-area",
                        "show-total-area",
                        true,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "show-grid" => self.show_grid.get().to_value(),
            "show-guidelines" => self.show_guidelines.get().to_value(),
            "show-handles" => self.show_handles.get().to_value(),
            "inner-fill" => self.inner_fill.get().to_value(),
            "show-total-area" => self.show_total_area.get().to_value(),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "show-grid" => {
                let val = value.get().expect("The value needs to be of type `bool`.");
                self.show_grid.set(val);
            }
            "show-guidelines" => {
                let val = value.get().expect("The value needs to be of type `bool`.");
                self.show_guidelines.set(val);
            }
            "show-handles" => {
                let val = value.get().expect("The value needs to be of type `bool`.");
                self.show_handles.set(val);
            }
            "inner-fill" => {
                let val = value.get().expect("The value needs to be of type `bool`.");
                self.inner_fill.set(val);
            }
            "show-total-area" => {
                let val = value.get().expect("The value needs to be of type `bool`.");
                self.show_total_area.set(val);
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for ViewHideBoxInner {}
impl ContainerImpl for ViewHideBoxInner {}
impl BoxImpl for ViewHideBoxInner {}

glib::wrapper! {
    pub struct ViewHideBox(ObjectSubclass<ViewHideBoxInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl ViewHideBox {
    pub fn new(canvas: &super::Canvas) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create ViewHideBox");
        ret.connect_notify_local(
            Some("show-grid"),
            clone!(@weak canvas => move |_self, _| {
                canvas.queue_draw();
            }),
        );
        for property in [
            "show-grid",
            "show-guidelines",
            "show-handles",
            "inner-fill",
            "show-total-area",
        ] {
            ret.bind_property(property, canvas, property)
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        ret
    }
}
