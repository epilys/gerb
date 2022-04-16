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

mod transformation;
use transformation::*;

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, Value};
use gtk::cairo::{Context, FontSlant, FontWeight, Matrix};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
pub struct CanvasInner {
    pub show_grid: Cell<bool>,
    pub show_guidelines: Cell<bool>,
    pub show_handles: Cell<bool>,
    pub inner_fill: Cell<bool>,
    pub transformation: Transformation,
}

#[glib::object_subclass]
impl ObjectSubclass for CanvasInner {
    const NAME: &'static str = "CanvasInner";
    type Type = Canvas;
    type ParentType = gtk::DrawingArea;
}

impl ObjectImpl for CanvasInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_tooltip_text(None);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK
                | gtk::gdk::EventMask::BUTTON_RELEASE_MASK
                | gtk::gdk::EventMask::BUTTON_MOTION_MASK
                | gtk::gdk::EventMask::SCROLL_MASK
                | gtk::gdk::EventMask::SMOOTH_SCROLL_MASK
                | gtk::gdk::EventMask::POINTER_MOTION_MASK,
        );
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
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecObject::new(
                        "transformation",
                        "transformation",
                        "transformation",
                        Transformation::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            "show-grid" => self.show_grid.get().to_value(),
            "show-guidelines" => self.show_guidelines.get().to_value(),
            "show-handles" => self.show_handles.get().to_value(),
            "inner-fill" => self.inner_fill.get().to_value(),
            "transformation" => self.transformation.to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "show-grid" => {
                self.show_grid.set(value.get().unwrap());
            }
            "show-guidelines" => {
                self.show_guidelines.set(value.get().unwrap());
            }
            "show-handles" => {
                self.show_handles.set(value.get().unwrap());
            }
            "inner-fill" => {
                self.inner_fill.set(value.get().unwrap());
            }
            "transformation" => {
                let new_val: Transformation = value.get().unwrap();
                self.transformation
                    .imp()
                    .matrix
                    .set(new_val.imp().matrix.get());
            }
            _ => unimplemented!(),
        }
    }
}

impl CanvasInner {}

impl DrawingAreaImpl for CanvasInner {}
impl WidgetImpl for CanvasInner {}

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<CanvasInner>)
        @extends gtk::DrawingArea, gtk::Widget;
}

impl Canvas {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Canvas");
        ret
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}
