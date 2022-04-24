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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;

glib::wrapper! {
    pub struct Settings(ObjectSubclass<imp::Settings>);
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Settings {
        pub handle_size: Cell<f64>,
        pub line_width: Cell<f64>,
        pub warp_cursor: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Settings {
        const NAME: &'static str = "Settings";
        type Type = super::Settings;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    impl ObjectImpl for Settings {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
                once_cell::sync::Lazy::new(|| {
                    vec![
                        ParamSpecDouble::new(
                            "handle-size",
                            "handle-size",
                            "handle-size",
                            2.0,
                            10.0,
                            5.0,
                            ParamFlags::READWRITE,
                        ),
                        ParamSpecDouble::new(
                            "line-width",
                            "line-width",
                            "line-width",
                            2.0,
                            10.0,
                            2.0,
                            ParamFlags::READWRITE,
                        ),
                        ParamSpecBoolean::new(
                            "warp-cursor",
                            "warp-cursor",
                            "warp-cursor",
                            true,
                            ParamFlags::READWRITE,
                        ),
                    ]
                });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "handle-size" => self.handle_size.get().to_value(),
                "line-width" => self.line_width.get().to_value(),
                "warp-cursor" => self.warp_cursor.get().to_value(),
                _ => unimplemented!("{}", pspec.name()),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "handle-size" => {
                    self.handle_size.set(value.get().unwrap());
                }
                "line-width" => {
                    self.line_width.set(value.get().unwrap());
                }
                "warp-cursor" => {
                    self.warp_cursor.set(value.get().unwrap());
                }
                _ => unimplemented!("{}", pspec.name()),
            }
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret.imp().handle_size.set(5.0);
        ret.imp().line_width.set(8.0);
        ret.imp().warp_cursor.set(true);
        ret
    }
}
