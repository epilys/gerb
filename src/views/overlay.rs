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

use glib::{ParamSpec, Value};

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::rc::Rc;

mod child;

pub use child::Child;

#[derive(Debug, Default)]
pub struct OverlayInner {
    overlay: gtk::Overlay,
    main_child: OnceCell<gtk::Widget>,
    widgets: Rc<RefCell<Vec<Child>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for OverlayInner {
    const NAME: &'static str = "Overlay";
    type Type = Overlay;
    type ParentType = gtk::Box;
}

impl ObjectImpl for OverlayInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_expand(true);
        obj.set_visible(true);
        obj.set_can_focus(true);
        self.overlay.set_expand(true);
        self.overlay.set_visible(true);
        self.overlay.set_can_focus(true);
        obj.add(&self.overlay);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(std::vec::Vec::new);
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl OverlayInner {}

impl WidgetImpl for OverlayInner {}
impl ContainerImpl for OverlayInner {}
impl BoxImpl for OverlayInner {}

glib::wrapper! {
    pub struct Overlay(ObjectSubclass<OverlayInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl Overlay {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Overlay");
        ret
    }

    pub fn set_child<P: IsA<gtk::Widget>>(&self, child: &P) {
        self.imp().overlay.set_child(Some(child));
        self.imp()
            .main_child
            .set(child.upcast_ref::<gtk::Widget>().clone())
            .unwrap();
    }

    pub fn add_overlay(&self, child: Child) {
        self.imp()
            .overlay
            .add_overlay(&child.property::<gtk::Widget>(Child::WIDGET));
        self.imp().widgets.borrow_mut().push(child);
    }
}

impl Default for Overlay {
    fn default() -> Self {
        Self::new()
    }
}
