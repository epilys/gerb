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

use super::Point;
use glib::prelude::*;
use gtk::glib;
use gtk::prelude::ToValue;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::cell::{Cell, Ref};
use std::rc::Rc;

glib::wrapper! {
    pub struct Bezier(ObjectSubclass<BezierInner>);
}

impl Bezier {
    pub fn points(&self) -> &RefCell<Vec<Point>> {
        &self.imp().points
    }
}

/// Given two cubic Bézier curves with control points [P0, P1, P2, P3] and [P3, P4, P5, P6]
/// respectively, the constraints for ensuring continuity at P3 can be defined as follows:
#[derive(Clone, Debug, Default, Copy, glib::Boxed)]
#[boxed_type(name = "Continuity", nullable)]
pub enum Continuity {
    /// C0 / G0 (positional continuity) requires that they meet at the same point, which all
    /// Bézier splines do by definition. In this example, the shared point is P3
    #[default]
    Positional,
    /// C1 (velocity continuity) requires the neighboring control points around the join to be
    /// mirrors of each other. In other words, they must follow the constraint of P4 = 2P3 − P2
    Velocity,
    /// G1 (tangent continuity) requires the neighboring control points to be collinear with
    /// the join. This is less strict than C1 continuity, leaving an extra degree of freedom
    /// which can be parameterized using a scalar β. The constraint can then be expressed by P4
    /// = P3 + (P3 − P2)β
    Tangent { beta: f64 },
}

#[derive(Default)]
pub struct BezierInner {
    pub smooth: Cell<bool>,
    pub points: Rc<RefCell<Vec<Point>>>,
    pub lut: Rc<RefCell<Vec<Point>>>,
}

impl std::fmt::Debug for BezierInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Bezier")
            .field("degree", &{
                let points = self.points.borrow();
                if points.is_empty() {
                    None
                } else {
                    Some(points.len() - 1)
                }
            })
            .field("smooth", &self.smooth.get())
            .field("points", &self.points)
            .field("lut entries", &self.lut.borrow().len())
            .finish()
    }
}

#[glib::object_subclass]
impl ObjectSubclass for BezierInner {
    const NAME: &'static str = "Bezier";
    type Type = Bezier;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for BezierInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.smooth.set(true);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        Bezier::SMOOTH,
                        Bezier::SMOOTH,
                        Bezier::SMOOTH,
                        true,
                        glib::ParamFlags::READWRITE | crate::UI_EDITABLE,
                    ),
                    glib::ParamSpecValueArray::new(
                        Bezier::POINTS,
                        Bezier::POINTS,
                        Bezier::POINTS,
                        &glib::ParamSpecBoxed::new(
                            Bezier::POINT,
                            Bezier::POINT,
                            Bezier::POINT,
                            Point::static_type(),
                            glib::ParamFlags::READWRITE,
                        ),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Bezier::SMOOTH => self.smooth.get().to_value(),
            Bezier::POINTS => {
                let points = self.points.borrow();
                let mut ret = glib::ValueArray::new(points.len() as u32);
                for p in points.iter() {
                    ret.append(&p.to_value());
                }
                ret.to_value()
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            Bezier::SMOOTH => {
                self.smooth.set(value.get().unwrap());
            }
            Bezier::POINTS => {
                let arr: glib::ValueArray = value.get().unwrap();
                let mut points = self.points.borrow_mut();
                points.clear();
                for p in arr.iter() {
                    points.push(p.get().unwrap());
                }
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl Default for Bezier {
    fn default() -> Self {
        Bezier::new(true, vec![])
    }
}

impl Bezier {
    pub const SMOOTH: &str = "smooth";
    pub const POINTS: &str = "points";
    pub const POINT: &str = "point";

    pub fn new(smooth: bool, points: Vec<Point>) -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret.imp().smooth.set(smooth);
        *ret.imp().points.borrow_mut() = points;
        ret
    }

    pub fn degree(&self) -> Option<usize> {
        let points = self.points().borrow();
        if points.is_empty() {
            None
        } else {
            Some(points.len() - 1)
        }
    }

    /* https://github.com/Pomax/bezierinfo/blob/adc3ad6397ca9d98339b89183a74cb52fad8f43a/js/graphics-element/lib/bezierjs/bezier.js#L88*/
    pub fn get_lut(&'_ self, steps: Option<usize>) -> Ref<'_, Vec<Point>> {
        let mut steps = steps.unwrap_or(100);
        let mut lut = self.imp().lut.borrow_mut();
        if lut.len() == steps {
            drop(lut);
            return self.imp().lut.borrow();
        }

        lut.clear();
        // We want a range from 0 to 1 inclusive, so
        // we decrement and then use <= rather than <:
        steps -= 1;
        let mut p;
        let mut t;
        for i in 0..steps {
            t = i as f64 / (steps - 1) as f64;
            p = self.compute(t);
            lut.push(p);
        }
        drop(lut);
        self.imp().lut.borrow()
    }

    pub fn compute(&self, t: f64) -> Point {
        let points = self.points().borrow();
        // shortcuts
        if t == 0.0 {
            return points[0];
        }

        let order = self.degree().unwrap();

        if t == 1.0 {
            return points[order];
        }

        let mt = 1.0 - t;
        let mut p = points.as_slice();

        // constant?
        if order == 0 {
            return p[0];
        }

        // linear?
        if order == 1 {
            let ret = ((mt * p[0].x + t * p[1].x), (mt * p[0].y + t * p[1].y));
            return ret.into();
        }

        // quadratic/cubic curve?
        if order < 4 {
            let p2 = &[p[0], p[1], p[2], (0.0, 0.0).into()];
            let mt2 = mt * mt;
            let t2 = t * t;
            let mut a = 0.0;
            let mut b = 0.0;
            let mut c = 0.0;
            let mut d = 0.0;
            if order == 2 {
                p = p2;
                a = mt2;
                b = mt * t * 2.0;
                c = t2;
            } else if order == 3 {
                a = mt2 * mt;
                b = mt2 * t * 3.0;
                c = mt * t2 * 3.0;
                d = t * t2;
            }
            let ret = (
                (a * p[0].x + b * p[1].x + c * p[2].x + d * p[3].x),
                (a * p[0].y + b * p[1].y + c * p[2].y + d * p[3].y),
            )
                .into();
            return ret;
        }
        todo!()

        /*
        // higher order curves: use de Casteljau's computation
        const dCpts = JSON.parse(JSON.stringify(points));
        while (dCpts.length > 1) {
          for (let i = 0; i < dCpts.length - 1; i++) {
            dCpts[i] = {
              x: dCpts[i].x + (dCpts[i + 1].x - dCpts[i].x) * t,
              y: dCpts[i].y + (dCpts[i + 1].y - dCpts[i].y) * t,
            };
            if (typeof dCpts[i].z !== "undefined") {
              dCpts[i] = dCpts[i].z + (dCpts[i + 1].z - dCpts[i].z) * t;
            }
          }
          dCpts.splice(dCpts.length - 1, 1);
        }
        dCpts[0].t = t;
        return dCpts[0];
            */
    }

    pub fn on_curve_query(&self, point: Point, error: Option<f64>) -> bool {
        let error = error.unwrap_or(5.0);
        let lut = self.get_lut(None);
        let mut hits = vec![];
        let mut c;
        let mut t = 0.0;
        for i in 0..lut.len() {
            c = lut[i];
            if super::distance_between_two_points(c, point) < error {
                hits.push(c);
                t += i as f64 / lut.len() as f64;
            }
        }
        if hits.is_empty() {
            return false;
        }
        (t / hits.len() as f64) != 0.0
    }

    pub fn clean_up(&self) {
        match self.degree() {
            Some(3) => {
                let mut pts = self.points().borrow_mut();
                if pts[0] == pts[1] && pts[2] == pts[3] {
                    self.imp().lut.borrow_mut().clear();
                    // Make quadratic
                    let a = pts[0];
                    let b = pts[3];
                    pts.clear();
                    pts.push(a);
                    pts.push(b);
                }
            }
            Some(2) => {
                let mut pts = self.points().borrow_mut();
                if pts[0] == pts[1] || pts[1] == pts[2] {
                    self.imp().lut.borrow_mut().clear();
                    // Make quadratic
                    let a = pts[0];
                    let b = pts[2];
                    pts.clear();
                    pts.push(a);
                    pts.push(b);
                }
            }
            _ => {}
        }
    }
}
