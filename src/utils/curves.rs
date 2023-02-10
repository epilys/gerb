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

use super::{CurvePoint, Point};
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
    pub fn points(&self) -> &RefCell<Vec<CurvePoint>> {
        &self.imp().points
    }
}

#[derive(Default)]
pub struct BezierInner {
    pub smooth: Cell<bool>,
    pub points: Rc<RefCell<Vec<CurvePoint>>>,
    pub lut: Rc<RefCell<Vec<Point>>>,
    pub emptiest_t: Cell<Option<(f64, Point, bool)>>,
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
        self.smooth.set(false);
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
                            CurvePoint::static_type(),
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
        Bezier::new(vec![])
    }
}

impl Bezier {
    pub const SMOOTH: &str = "smooth";
    pub const POINTS: &str = "points";
    pub const POINT: &str = "point";

    pub fn new(points: Vec<Point>) -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        *ret.imp().points.borrow_mut() = points
            .into_iter()
            .map(|position| CurvePoint {
                position,
                ..CurvePoint::default()
            })
            .collect::<Vec<CurvePoint>>();
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
            return points[0].position;
        }

        let order = self.degree().unwrap();

        if t == 1.0 {
            return points[order].position;
        }

        let mt = 1.0 - t;
        // constant?
        if order == 0 {
            return points[0].position;
        }

        // linear?
        if order == 1 {
            let ret = (
                (mt * points[0].x + t * points[1].x),
                (mt * points[0].y + t * points[1].y),
            );
            return ret.into();
        }

        let positions = points.iter().map(|cp| cp.position).collect::<Vec<Point>>();
        let mut p = positions.as_slice();

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

    pub fn tangent(&self, t: f64) -> Point {
        let points = self.points().borrow();
        // shortcuts
        if t == 0.0 {
            return points[0].position;
        }

        let order = self.degree().unwrap();

        if t == 1.0 {
            return points[order].position;
        }

        if order == 0 {
            return points[0].position;
        }

        // linear
        if order == 1 {
            return points[1].position - points[0].position;
        }

        let mt = 1.0 - t;

        if order == 2 {
            let d: [Point; 2] = [
                (
                    2.0 * (points[1].x - points[0].x),
                    2.0 * (points[1].y - points[0].y),
                )
                    .into(),
                (
                    2.0 * (points[2].x - points[1].x),
                    2.0 * (points[2].y - points[1].y),
                )
                    .into(),
            ];

            return (mt * d[0].x + t * d[1].x, mt * d[0].y + t * d[1].y).into();
        } else if order == 3 {
            let a = mt * mt;
            let b = 2.0 * mt * t;
            let c = t * t;
            let d: [Point; 3] = [
                (
                    3.0 * (points[1].x - points[0].x),
                    3.0 * (points[1].y - points[0].y),
                )
                    .into(),
                (
                    3.0 * (points[2].x - points[1].x),
                    3.0 * (points[2].y - points[1].y),
                )
                    .into(),
                (
                    3.0 * (points[3].x - points[2].x),
                    3.0 * (points[3].y - points[2].y),
                )
                    .into(),
            ];

            return (
                a * d[0].x + b * d[1].x + c * d[2].x,
                a * d[0].y + b * d[1].y + c * d[2].y,
            )
                .into();
        }
        todo!()
    }

    pub fn approx_length(&self) -> f64 {
        let lut = self.get_lut(None);
        let mut ret = 0.0;
        for pair in lut.windows(2).skip(1) {
            let p1 = pair[0];
            let p2 = pair[1];
            ret += p1.distance(p2);
        }
        ret
    }

    pub fn on_curve_query(&self, point: Point, error: Option<f64>) -> bool {
        let error = error.unwrap_or(15.0);
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
                if pts[0].position == pts[1].position && pts[2].position == pts[3].position {
                    self.imp().lut.borrow_mut().clear();
                    // Make quadratic
                    let a = pts[0].clone();
                    let b = pts[3].clone();
                    pts.clear();
                    pts.push(a);
                    pts.push(b);
                }
            }
            Some(2) => {
                let mut pts = self.points().borrow_mut();
                if pts[0].position == pts[1].position || pts[1].position == pts[2].position {
                    self.imp().lut.borrow_mut().clear();
                    // Make quadratic
                    let a = pts[0].clone();
                    let b = pts[2].clone();
                    pts.clear();
                    pts.push(a);
                    pts.push(b);
                }
            }
            _ => {}
        }
    }

    pub fn emptiest_t(&self, starting_t: f64) -> (f64, Point) {
        use rand::distributions::{Distribution, Uniform};

        match self.imp().emptiest_t.get() {
            Some((t, _, true)) => {
                // Curve has been modified, look for new optimal and use it if it's significantly
                // far away from the previous optimal (otherwise the optimal point will jump around
                // in the UI erratically while the user is transforming the curve).
                self.imp().emptiest_t.set(None);
                let (new_t, new_p) = self.emptiest_t(starting_t);
                if (new_t - t).abs() < 0.05 {
                    // reject new optimal𝑡
                    let new_p = self.compute(t);
                    self.imp().emptiest_t.set(Some((t, new_p, false)));
                    return (t, new_p);
                } else {
                    return (new_t, new_p);
                }
            }
            Some((t, point, false)) => {
                return (t, point);
            }
            None => {}
        }

        // Evaluate candidate𝑡 by trying to minimise the difference of distances of the two closest
        // control points to the on-curve point of𝑡.
        fn eval(curv: &Bezier, p: Point) -> f64 {
            let pts = curv.points().borrow();
            let mut ds = pts
                .iter()
                .map(|cp| p.distance(cp.position))
                .collect::<Vec<f64>>();
            ds.sort_by(|a, b| a.partial_cmp(b).unwrap());
            (ds[0] - ds[1]).abs()
        }

        let mut curr_t = starting_t;
        let mut curr_p = self.compute(starting_t);
        let mut curr_eval = eval(self, curr_p);
        let mut best = curr_eval;

        let mut rng = rand::thread_rng();
        let die = Uniform::from(0.0..=1.0);
        let step_size = 0.1;

        let initial_temp = 10.0;

        for i in 1..1001 {
            let throw = die.sample(&mut rng);
            let candidate = (curr_t + throw * step_size).clamp(0.0, 1.0);
            let c_point = self.compute(candidate);
            let c_eval = eval(self, c_point);

            if c_eval < best {
                best = c_eval;
            }

            let diff = c_eval - curr_eval;
            let temperature = initial_temp / (i as f64);
            let metropolis = (-diff / temperature).exp();
            if diff < 0.0 || rand::random::<f64>() < metropolis {
                curr_t = candidate;
                curr_eval = c_eval;
                curr_p = c_point;
            }
        }

        self.imp().emptiest_t.set(Some((curr_t, curr_p, false)));
        (curr_t, curr_p)
    }

    pub fn set_modified(&self) {
        if let Some((distance, point, _)) = self.imp().emptiest_t.get() {
            self.imp().emptiest_t.set(Some((distance, point, true)));
        }
        self.imp().lut.borrow_mut().clear();
    }
}
