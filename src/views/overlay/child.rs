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

use super::*;
use glib::{ParamFlags, ParamSpecBoolean};
use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub struct ChildInner {
    widget: RefCell<gtk::Widget>,
    moveable: Cell<bool>,
}

impl Default for ChildInner {
    fn default() -> ChildInner {
        ChildInner {
            widget: RefCell::new(
                gtk::builders::FrameBuilder::new()
                    .build()
                    .upcast::<gtk::Widget>(),
            ),
            moveable: Cell::new(true),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ChildInner {
    const NAME: &'static str = "ChildInner";
    type Type = Child;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for ChildInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        Child::WIDGET,
                        Child::WIDGET,
                        Child::WIDGET,
                        gtk::Widget::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Child::MOVEABLE,
                        Child::MOVEABLE,
                        Child::MOVEABLE,
                        true,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Child::WIDGET => self.widget.borrow().to_value(),
            Child::MOVEABLE => self.moveable.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Child::MOVEABLE => self.moveable.set(value.get().unwrap()),
            Child::WIDGET => *self.widget.borrow_mut() = value.get().unwrap(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl ChildInner {}

glib::wrapper! {
    pub struct Child(ObjectSubclass<ChildInner>);
}

impl Child {
    pub const WIDGET: &str = "widget";
    pub const MOVEABLE: &str = "moveable";

    pub fn new<P: IsA<gtk::Widget>>(child: P, moveable: bool) -> Self {
        Self::new_inner(child.upcast(), moveable)
    }

    fn new_inner(child: gtk::Widget, moveable: bool) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Overlay");
        ret.set_property::<gtk::Widget>(Self::WIDGET, child);
        ret.set_property::<bool>(Self::MOVEABLE, moveable);
        ret
    }
}
