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
use std::cell::Cell;

glib::wrapper! {
    pub struct Contour(ObjectSubclass<imp::Contour>);
}

impl Default for Contour {
    fn default() -> Self {
        Self::new()
    }
}

impl Contour {
    pub const OPEN: &str = "open";
    pub const CONTINUITIES: &str = "continuities";
    pub const CONTINUITY: &str = "continuity";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn curves(&self) -> &RefCell<Vec<Bezier>> {
        &self.imp().curves
    }

    pub fn push_curve(&self, curve: Bezier) {
        let mut curves = self.imp().curves.borrow_mut();
        let mut continuities = self.imp().continuities.borrow_mut();
        if curve.points().borrow().is_empty() {
            return;
        }
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
            (&[_, _, p2, p3_1], &[p3_2, p4, _, _]) if p3_1 == p3_2 && p2.collinear(&p3_1, &p4) => {
                let beta = (p4 - p3_1) / (p3_1 - p2);
                //assert_eq!(beta.x, beta.y);
                let beta = beta.y;
                continuities.push(Continuity::Tangent { beta });
            }
            (&[_, _, p2, p3_1], &[p3_2, p4, _, _])
            | (&[_, p2, p3_1], &[p3_2, p4, _, _])
            | (&[_, p2, p3_1], &[p3_2, p4, _])
            | (&[_, _, p2, p3_1], &[p3_2, p4, _])
                if p3_1 == p3_2 && p4 == 2.0 * p3_1 - p2 =>
            {
                continuities.push(Continuity::Velocity);
            }
            _ => {
                continuities.push(Continuity::Positional);
            }
        }
        drop(curr);
        drop(prev);
        curves.push(curve);
    }

    pub fn reverse_direction(&self) {
        let mut curves = self.imp().curves.borrow_mut();
        curves.reverse();
        let mut continuities = self.imp().continuities.borrow_mut();
        continuities.reverse();
        for c in curves.iter_mut() {
            c.points().borrow_mut().reverse();
        }
    }
}

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct Contour {
        pub open: Cell<bool>,
        pub curves: RefCell<Vec<Bezier>>,
        pub continuities: RefCell<Vec<Continuity>>,
    }

    impl std::fmt::Debug for Contour {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.debug_struct("Contour")
                .field("open", &self.open.get())
                .field(
                    "curves",
                    &self
                        .curves
                        .borrow()
                        .iter()
                        .map(Bezier::imp)
                        .collect::<Vec<_>>(),
                )
                .field("continuities", &self.continuities.borrow())
                .finish()
        }
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
                    vec![
                        glib::ParamSpecValueArray::new(
                            super::Contour::CONTINUITIES,
                            super::Contour::CONTINUITIES,
                            super::Contour::CONTINUITIES,
                            &glib::ParamSpecBoxed::new(
                                super::Contour::CONTINUITY,
                                super::Contour::CONTINUITY,
                                super::Contour::CONTINUITY,
                                Continuity::static_type(),
                                glib::ParamFlags::READWRITE,
                            ),
                            glib::ParamFlags::READWRITE,
                        ),
                        glib::ParamSpecBoolean::new(
                            super::Contour::OPEN,
                            super::Contour::OPEN,
                            super::Contour::OPEN,
                            true,
                            glib::ParamFlags::READWRITE,
                        ),
                    ]
                });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                super::Contour::CONTINUITIES => {
                    let continuities = self.continuities.borrow();
                    let mut ret = glib::ValueArray::new(continuities.len() as u32);
                    for c in continuities.iter() {
                        ret.append(&c.to_value());
                    }
                    ret.to_value()
                }
                super::Contour::OPEN => self.open.get().to_value(),
                _ => unimplemented!("{}", pspec.name()),
            }
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                super::Contour::CONTINUITIES => {
                    let arr: glib::ValueArray = value.get().unwrap();
                    let mut continuities = self.continuities.borrow_mut();
                    continuities.clear();
                    for c in arr.iter() {
                        continuities.push(c.get().unwrap());
                    }
                }
                super::Contour::OPEN => {
                    self.open.set(value.get().unwrap());
                }
                _ => unimplemented!("{}", pspec.name()),
            }
        }
    }
}
