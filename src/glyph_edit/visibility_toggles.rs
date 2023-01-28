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
use std::cell::Cell;

use super::{Canvas, GlyphEditView};

#[derive(Debug, Default)]
pub struct ViewHideBoxInner {
    show_grid: Cell<bool>,
    show_guidelines: Cell<bool>,
    lock_guidelines: Cell<bool>,
    show_handles: Cell<bool>,
    inner_fill: Cell<bool>,
    show_total_area: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for ViewHideBoxInner {
    const NAME: &'static str = "ViewHideBox";
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
        self.lock_guidelines.set(false);
        obj.set_expand(false);
        obj.set_halign(gtk::Align::End);
        obj.set_valign(gtk::Align::End);
        obj.set_spacing(5);
        obj.set_visible(true);
        obj.set_can_focus(true);

        for (property, label) in [
            (ViewHideBox::SHOW_GRID, "Show grid"),
            (ViewHideBox::SHOW_GUIDELINES, "Show guidelines"),
            (ViewHideBox::LOCK_GUIDELINES, "Lock guidelines"),
            (ViewHideBox::SHOW_HANDLES, "Show handles"),
            (ViewHideBox::INNER_FILL, "Inner fill"),
            (ViewHideBox::SHOW_TOTAL_AREA, "Show total area"),
        ] {
            let btn = gtk::CheckButton::with_label(label);
            btn.set_visible(true);
            btn.set_active(false);
            obj.pack_start(&btn, false, false, 0);
            obj.bind_property(property, &btn, "active")
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();
        }
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        ViewHideBox::SHOW_GRID,
                        ViewHideBox::SHOW_GRID,
                        ViewHideBox::SHOW_GRID,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        ViewHideBox::SHOW_GUIDELINES,
                        ViewHideBox::SHOW_GUIDELINES,
                        ViewHideBox::SHOW_GUIDELINES,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        ViewHideBox::SHOW_HANDLES,
                        ViewHideBox::SHOW_HANDLES,
                        ViewHideBox::SHOW_HANDLES,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        ViewHideBox::INNER_FILL,
                        ViewHideBox::INNER_FILL,
                        ViewHideBox::INNER_FILL,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        ViewHideBox::SHOW_TOTAL_AREA,
                        ViewHideBox::SHOW_TOTAL_AREA,
                        ViewHideBox::SHOW_TOTAL_AREA,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        ViewHideBox::LOCK_GUIDELINES,
                        ViewHideBox::LOCK_GUIDELINES,
                        ViewHideBox::LOCK_GUIDELINES,
                        true,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            ViewHideBox::SHOW_GRID => self.show_grid.get().to_value(),
            ViewHideBox::SHOW_GUIDELINES => self.show_guidelines.get().to_value(),
            ViewHideBox::SHOW_HANDLES => self.show_handles.get().to_value(),
            ViewHideBox::INNER_FILL => self.inner_fill.get().to_value(),
            ViewHideBox::SHOW_TOTAL_AREA => self.show_total_area.get().to_value(),
            ViewHideBox::LOCK_GUIDELINES => self.lock_guidelines.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        let val = value.get().expect("The value needs to be of type `bool`.");
        match pspec.name() {
            ViewHideBox::SHOW_GRID => {
                self.show_grid.set(val);
            }
            ViewHideBox::SHOW_GUIDELINES => {
                self.show_guidelines.set(val);
            }
            ViewHideBox::SHOW_HANDLES => {
                self.show_handles.set(val);
            }
            ViewHideBox::INNER_FILL => {
                self.inner_fill.set(val);
            }
            ViewHideBox::SHOW_TOTAL_AREA => {
                self.show_total_area.set(val);
            }
            ViewHideBox::LOCK_GUIDELINES => {
                self.lock_guidelines.set(val);
            }
            _ => unimplemented!("{}", pspec.name()),
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
    pub const INNER_FILL: &str = Canvas::INNER_FILL;
    pub const SHOW_GRID: &str = Canvas::SHOW_GRID;
    pub const SHOW_GUIDELINES: &str = Canvas::SHOW_GUIDELINES;
    pub const SHOW_HANDLES: &str = Canvas::SHOW_HANDLES;
    pub const SHOW_TOTAL_AREA: &str = Canvas::SHOW_TOTAL_AREA;
    pub const LOCK_GUIDELINES: &str = GlyphEditView::LOCK_GUIDELINES;

    pub fn new(canvas: &Canvas) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create ViewHideBox");
        for property in [
            Canvas::INNER_FILL,
            Canvas::SHOW_GRID,
            Canvas::SHOW_GUIDELINES,
            Canvas::SHOW_HANDLES,
            Canvas::SHOW_TOTAL_AREA,
        ] {
            ret.set_property(property, canvas.property::<bool>(property));
            ret.bind_property(property, canvas, property)
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();
            ret.connect_notify_local(
                Some(property),
                clone!(@weak canvas => move |_self, _| {
                    canvas.queue_draw();
                }),
            );
        }
        ret
    }
}
