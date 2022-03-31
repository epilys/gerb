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

use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::Ref;
use std::cell::RefCell;

pub type Point = (i64, i64);

glib::wrapper! {
    pub struct Bezier(ObjectSubclass<imp::Bezier>);
}

impl Bezier {
    pub fn smooth(&self) -> &RefCell<bool> {
        &self.imp().smooth
    }

    pub fn points(&self) -> &RefCell<Vec<Point>> {
        &self.imp().points
    }
}

mod imp {
    use super::*;
    #[derive(Debug, Default)]
    pub struct Bezier {
        pub smooth: RefCell<bool>,
        pub points: RefCell<Vec<Point>>,
        pub lut: RefCell<Vec<Point>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Bezier {
        const NAME: &'static str = "Bezier";
        type Type = super::Bezier;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    impl ObjectImpl for Bezier {}
}

impl Bezier {
    pub fn new(smooth: bool, points: Vec<Point>) -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        *ret.imp().smooth.borrow_mut() = smooth;
        *ret.imp().points.borrow_mut() = points;
        ret
    }

    pub fn get_point(&self, t: f64) -> Option<Point> {
        let points = self.points().borrow();
        draw_curve_point(&points, t)
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
            let ret = (
                (mt * p[0].0 as f64 + t * p[1].0 as f64) as i64,
                (mt * p[0].1 as f64 + t * p[1].1 as f64) as i64,
            );
            return ret;
        }

        // quadratic/cubic curve?
        if order < 4 {
            let p2 = &[p[0], p[1], p[2], (0, 0)];
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
                (a * p[0].0 as f64 + b * p[1].0 as f64 + c * p[2].0 as f64 + d * p[3].0 as f64)
                    as i64,
                (a * p[0].1 as f64 + b * p[1].1 as f64 + c * p[2].1 as f64 + d * p[3].1 as f64)
                    as i64,
            );
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
}

fn draw_curve_point(points: &[Point], t: f64) -> Option<Point> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        //std::dbg!(points[0]);
        return Some(points[0]);
    }
    let mut new_points = Vec::with_capacity(points.len() - 1);
    for chunk in points.windows(2) {
        let p1 = chunk[0];
        let p2 = chunk[1];
        let x = (1. - t) * (p1.0 as f64) + t * (p2.0 as f64);
        let y = (1. - t) * (p1.1 as f64) + t * (p2.1 as f64);
        new_points.push((x as i64, y as i64));
    }
    assert_eq!(new_points.len(), points.len() - 1);
    draw_curve_point(&new_points, t)
}
