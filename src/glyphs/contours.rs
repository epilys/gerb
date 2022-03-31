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
use glib::{ParamSpec, Value};
use gtk::glib;

glib::wrapper! {
    pub struct Contour(ObjectSubclass<imp::Contour>);
}

impl Contour {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn open(&self) -> &RefCell<bool> {
        &self.imp().open
    }

    pub fn curves(&self) -> &RefCell<Vec<Bezier>> {
        &self.imp().curves
    }
}

mod imp {
    use super::*;
    #[derive(Debug, Default)]
    pub struct Contour {
        pub open: RefCell<bool>,
        pub curves: RefCell<Vec<Bezier>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Contour {
        const NAME: &'static str = "Contour";
        type Type = super::Contour;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    impl ObjectImpl for Contour {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
                once_cell::sync::Lazy::new(|| vec![]);
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }
    }
}
