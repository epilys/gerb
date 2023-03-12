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
    movable: Cell<bool>,
    expandable: Cell<bool>,
    expanded: Cell<bool>,
}

impl Default for ChildInner {
    fn default() -> Self {
        Self {
            widget: RefCell::new(
                gtk::builders::FrameBuilder::new()
                    .build()
                    .upcast::<gtk::Widget>(),
            ),
            movable: Cell::new(true),
            expandable: Cell::new(true),
            expanded: Cell::new(true),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ChildInner {
    const NAME: &'static str = "Child";
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
                        Child::MOVABLE,
                        Child::MOVABLE,
                        Child::MOVABLE,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Child::EXPANDABLE,
                        Child::EXPANDABLE,
                        Child::EXPANDABLE,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Child::EXPANDED,
                        Child::EXPANDED,
                        Child::EXPANDED,
                        true,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Child::WIDGET => {
                if self.expandable.get() {
                    let widget = self.widget.borrow();
                    let expander = gtk::Expander::builder()
                        .child(&*widget)
                        .expanded(self.expanded.get())
                        .visible(true)
                        .can_focus(true)
                        .halign(widget.halign())
                        .valign(widget.valign())
                        .margin_start(10)
                        .build();
                    for prop in ["halign", "valign", "visible", "tooltip-text"] {
                        widget
                            .bind_property(prop, &expander, prop)
                            .flags(glib::BindingFlags::SYNC_CREATE)
                            .build();
                    }
                    expander.to_value()
                } else {
                    self.widget.borrow().to_value()
                }
            }
            Child::MOVABLE => self.movable.get().to_value(),
            Child::EXPANDABLE => self.expandable.get().to_value(),
            Child::EXPANDED => self.expanded.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Child::EXPANDABLE => self.expandable.set(value.get().unwrap()),
            Child::EXPANDED => self.expanded.set(value.get().unwrap()),
            Child::MOVABLE => self.movable.set(value.get().unwrap()),
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
    pub const MOVABLE: &str = "movable";
    pub const EXPANDABLE: &str = "expandable";
    pub const EXPANDED: &str = "expanded";

    pub fn new<P: IsA<gtk::Widget>>(child: P) -> Self {
        Self::new_inner(child.upcast())
    }

    fn new_inner(child: gtk::Widget) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Overlay");
        ret.set_property::<gtk::Widget>(Self::WIDGET, child);
        ret
    }

    pub fn movable(self, movable: bool) -> Self {
        self.set_property::<bool>(Self::MOVABLE, movable);
        self
    }

    pub fn expandable(self, expandable: bool) -> Self {
        self.set_property::<bool>(Self::EXPANDABLE, expandable);
        self
    }

    pub fn expanded(self, expanded: bool) -> Self {
        self.set_property::<bool>(Self::EXPANDED, expanded);
        self
    }
}
