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
use gtk::prelude::*;

glib::wrapper! {
    pub struct Contour(ObjectSubclass<imp::Contour>);
}

impl Default for Contour {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn push_curve(&self, curve: Bezier) {
        let mut curves = self.imp().curves.borrow_mut();
        let mut continuities = self.imp().continuities.borrow_mut();
        if curves.is_empty() {
            curves.push(curve);
            return;
        }
        let prev = curves[curves.len() - 1].points().borrow();
        let curr = curve.points().borrow();
        match (
            <Vec<Point> as AsRef<[Point]>>::as_ref(&prev),
            <Vec<Point> as AsRef<[Point]>>::as_ref(&curr),
        ) {
            (&[_p0, _p1, p2, p3_1], &[p3_2, p4, _p5, _p6])
                if p3_1 == p3_2 && p2.collinear(&p3_1, &p4) =>
            {
                let beta = (p4 - p3_1) / (p3_1 - p2);
                //assert_eq!(beta.x, beta.y);
                let beta = beta.y;
                continuities.push(Continuity::Tangent { beta });
            }
            (&[_p0, _p1, p2, p3_1], &[p3_2, p4, _p5, _p6])
                if p3_1 == p3_2 && p4 == 2.0 * p3_1 - p2 =>
            {
                continuities.push(Continuity::Velocity);
            }
            (&[_p0, _p1, _p2, p3_1], &[p3_2, _p4, _p5, _p6]) if p3_1 == p3_2 => {
                continuities.push(Continuity::Positional);
            }
            (&[_p0, p1_1], &[p1_2, _p_3]) if p1_1 == p1_2 => {
                continuities.push(Continuity::Positional);
            }
            (&[_p0, _p1, _p2, p3_1], &[p3_2, _p_4]) if p3_1 == p3_2 => {
                continuities.push(Continuity::Positional);
            }
            (&[_p0, p1_1], &[p1_2, _p2, _p3, _p4]) if p1_1 == p1_2 => {
                continuities.push(Continuity::Positional);
            }
            _ => panic!("prev {:?} curr {:?}", prev, curr),
        }
        drop(curr);
        drop(prev);
        curves.push(curve);
    }
}

mod imp {
    use super::*;
    #[derive(Debug, Default)]
    pub struct Contour {
        pub open: RefCell<bool>,
        pub curves: RefCell<Vec<Bezier>>,
        pub continuities: RefCell<Vec<Continuity>>,
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
                once_cell::sync::Lazy::new(|| {
                    vec![glib::ParamSpecValueArray::new(
                        "continuities",
                        "continuities",
                        "continuities",
                        &glib::ParamSpecBoxed::new(
                            "continuities",
                            "continuities",
                            "continuities",
                            Continuity::static_type(),
                            glib::ParamFlags::READWRITE,
                        ),
                        glib::ParamFlags::READWRITE,
                    )]
                });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "continuities" => {
                    let continuities = self.continuities.borrow();
                    let mut ret = glib::ValueArray::new(continuities.len() as u32);
                    for c in continuities.iter() {
                        ret.append(&c.to_value());
                    }
                    ret.to_value()
                }
                _ => unimplemented!("{}", pspec.name()),
            }
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "continuities" => {
                    let arr: glib::ValueArray = value.get().unwrap();
                    let mut continuities = self.continuities.borrow_mut();
                    continuities.clear();
                    for c in arr.iter() {
                        continuities.push(c.get().unwrap());
                    }
                }
                _ => unimplemented!("{}", pspec.name()),
            }
        }
    }
}
