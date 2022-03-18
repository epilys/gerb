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
use glib::{ParamFlags, ParamSpec, ParamSpecDouble, ParamSpecInt64, ParamSpecString, Value};
use gtk::glib;
use gtk::prelude::*;

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct Guideline {
        pub name: RefCell<Option<String>>,
        pub identifier: RefCell<Option<String>>,
        pub color: RefCell<Option<String>>,
        pub angle: RefCell<f64>,
        pub x: RefCell<i64>,
        pub y: RefCell<i64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Guideline {
        const NAME: &'static str = "GlyphGuideline";
        type Type = super::Guideline;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    // Trait shared by all GObjects
    impl ObjectImpl for Guideline {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
                once_cell::sync::Lazy::new(|| {
                    vec![
                        ParamSpecString::new("name", "name", "name", None, ParamFlags::READWRITE),
                        ParamSpecString::new(
                            "identifier",
                            "identifier",
                            "identifier",
                            None,
                            ParamFlags::READWRITE,
                        ),
                        ParamSpecDouble::new(
                            "angle",
                            "angle",
                            "angle",
                            -360.0,
                            360.0,
                            0.,
                            ParamFlags::READWRITE,
                        ),
                        ParamSpecInt64::new(
                            "x",
                            "x",
                            "x",
                            std::i64::MIN,
                            std::i64::MAX,
                            0,
                            ParamFlags::READWRITE,
                        ),
                        ParamSpecInt64::new(
                            "y",
                            "y",
                            "y",
                            std::i64::MIN,
                            std::i64::MAX,
                            0,
                            ParamFlags::READWRITE,
                        ),
                    ]
                });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                "identifier" => self.identifier.borrow().to_value(),
                "angle" => self.angle.borrow().to_value(),
                "x" => self.x.borrow().to_value(),
                "y" => self.y.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "name" => {
                    *self.name.borrow_mut() = value.get().unwrap();
                }
                "identifier" => {
                    *self.identifier.borrow_mut() = value.get().unwrap();
                }
                "angle" => {
                    *self.angle.borrow_mut() = value.get().unwrap();
                }
                "x" => {
                    *self.x.borrow_mut() = value.get().unwrap();
                }
                "y" => {
                    *self.y.borrow_mut() = value.get().unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl Guideline {
        pub fn draw(
            &self,
            cr: &Context,
            matrix: Matrix,
            (_width, height): (f64, f64),
            highlight: bool,
        ) {
            fn move_point(p: (f64, f64), d: f64, r: f64) -> (f64, f64) {
                let (x, y) = p;
                (x + (d * f64::cos(r)), y + (d * f64::sin(r)))
            }
            cr.save().unwrap();
            if highlight {
                cr.set_source_rgba(1., 0., 0., 0.8);
                cr.set_line_width(2.0);
            } else {
                cr.set_source_rgba(0., 0., 1., 0.8);
                cr.set_line_width(1.5);
            }
            let p = matrix.transform_point(*self.x.borrow() as f64, *self.y.borrow() as f64);
            let r = *self.angle.borrow() * 0.01745;
            let top = move_point(p, height * 10., r);
            cr.move_to(top.0, top.1);
            let bottom = move_point(p, -height * 10., r);
            cr.line_to(bottom.0, bottom.1);
            cr.stroke().unwrap();
            cr.restore().unwrap();
        }

        pub fn distance_from_point(&self, (xp, yp): Point) -> f64 {
            // Using an 𝐿 defined by a point 𝑃𝑙 and angle 𝜃
            //𝑑 = ∣cos(𝜃)(𝑃𝑙𝑦 − 𝑦𝑝) − sin(𝜃)(𝑃𝑙𝑥 − 𝑃𝑥)∣
            let r = -*self.angle.borrow() * 0.01745;
            let sin = f64::sin(r);
            let cos = f64::cos(r);
            (cos * (*self.y.borrow() - yp) as f64 - sin * (*self.x.borrow() - xp) as f64).abs()
        }

        pub fn on_line_query(&self, point: Point, error: Option<f64>) -> bool {
            let error = error.unwrap_or(5.0);
            self.distance_from_point(point) <= error
        }
    }
}

glib::wrapper! {
    pub struct Guideline(ObjectSubclass<imp::Guideline>);
}

impl Guideline {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }
    pub fn name(&self) -> Option<String> {
        self.property("name")
    }
    pub fn identifier(&self) -> Option<String> {
        self.property("identifier")
    }
    pub fn color(&self) -> Option<String> {
        self.property("color")
    }
    pub fn angle(&self) -> f64 {
        self.property("angle")
    }
    pub fn x(&self) -> i64 {
        self.property("x")
    }
    pub fn y(&self) -> i64 {
        self.property("y")
    }

    pub fn builder() -> GuidelineBuilder {
        GuidelineBuilder::new()
    }
}

pub struct GuidelineBuilder(Guideline);

impl GuidelineBuilder {
    pub fn new() -> Self {
        GuidelineBuilder(Guideline::new())
    }

    pub fn name(self, name: Option<String>) -> Self {
        *self.0.imp().name.borrow_mut() = name;
        self
    }

    pub fn identifier(self, identifier: Option<String>) -> Self {
        *self.0.imp().identifier.borrow_mut() = identifier;
        self
    }

    pub fn color(self, color: Option<String>) -> Self {
        *self.0.imp().color.borrow_mut() = color;
        self
    }

    pub fn angle(self, angle: f64) -> Self {
        *self.0.imp().angle.borrow_mut() = angle;
        self
    }

    pub fn x(self, x: i64) -> Self {
        *self.0.imp().x.borrow_mut() = x;
        self
    }

    pub fn y(self, y: i64) -> Self {
        *self.0.imp().y.borrow_mut() = y;
        self
    }

    pub fn build(self) -> Guideline {
        self.0
    }
}
