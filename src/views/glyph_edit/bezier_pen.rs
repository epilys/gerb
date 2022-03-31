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

use crate::glyphs::{Contour, GlyphDrawingOptions};
use crate::utils::{
    curves::{Bezier, Point},
    distance_between_two_points,
};
use gtk::cairo::{Context, Matrix};

#[derive(Debug, Clone, Copy, PartialEq)]
enum InnerState {
    AddControlPoint,
    AddControlHandle,
}

#[derive(Debug, Clone)]
pub struct State {
    inner: InnerState,
    first_point: Option<Point>,
    current_curve: Bezier,
    curves: Vec<Bezier>,
}

impl Default for State {
    fn default() -> Self {
        State {
            inner: InnerState::AddControlPoint,
            current_curve: Bezier::new(true, vec![]),
            curves: vec![],
            first_point: None,
        }
    }
}

impl State {
    pub fn insert_point(&mut self, point: (i64, i64)) -> bool {
        match self.inner {
            InnerState::AddControlPoint => {
                self.inner = InnerState::AddControlHandle;
                match self.first_point.as_ref() {
                    None => {
                        self.first_point = Some(point);
                    }
                    Some(fp) if distance_between_two_points(point, *fp) < 10.0 => {
                        return false;
                    }
                    _ => {}
                }
                self.current_curve.points().borrow_mut().push(point);

                true
            }
            InnerState::AddControlHandle => {
                self.inner = InnerState::AddControlPoint;
                self.current_curve.points().borrow_mut().push(point);
                if self.current_curve.points().borrow().len() == 4 {
                    /* current_curve is cubic, so split it. */
                    let curv =
                        std::mem::replace(&mut self.current_curve, Bezier::new(true, vec![]));
                    self.curves.push(curv);
                }
                true
            }
        }
    }

    pub fn close(self, open: bool) -> Contour {
        let State {
            inner: _,
            first_point: _,
            current_curve,
            mut curves,
        } = self;
        if current_curve.degree().is_some() {
            curves.push(current_curve);
        }

        let ret = Contour::new();
        *ret.open().borrow_mut() = open;
        *ret.curves().borrow_mut() = curves;
        ret
    }

    pub fn draw(&self, cr: &Context, options: GlyphDrawingOptions, cursor_position: (i64, i64)) {
        let first_point = match self.first_point {
            Some(v) => v,
            None => return,
        };
        let GlyphDrawingOptions {
            outline,
            inner_fill: _,
            highlight: _,
            matrix,
            units_per_em,
        } = options;

        cr.save().expect("Invalid cairo surface state");
        cr.set_line_width(4.0);
        cr.transform(matrix);
        cr.transform(Matrix::new(1.0, 0., 0., -1.0, 0., units_per_em.abs()));
        cr.set_source_rgba(outline.0, outline.1, outline.2, outline.3);
        let draw_endpoint = |p: (f64, f64)| {
            cr.rectangle(p.0 - 2.5, p.1 - 2.5, 5., 5.);
            cr.stroke().expect("Invalid cairo surface state");
        };
        let draw_handle = |p: (f64, f64), ep: (f64, f64)| {
            cr.arc(p.0 - 2.5, p.1 - 2.5, 2.0, 0., 2. * std::f64::consts::PI);
            cr.fill().unwrap();
            cr.move_to(p.0 - 2.5, p.1 - 2.5);
            cr.line_to(ep.0, ep.1);
            cr.stroke().unwrap();
        };
        let p_fn = |p: (i64, i64)| -> (f64, f64) { (p.0 as f64, p.1 as f64) };
        let fp = p_fn(first_point);
        draw_endpoint(fp);
        let mut pen_position: Option<(f64, f64)> = Some(fp);
        for curv in self.curves.iter() {
            if !*curv.smooth().borrow() {
                //cr.stroke().expect("Invalid cairo surface state");
            }
            let degree = curv.degree();
            let degree = if let Some(v) = degree {
                v
            } else {
                continue;
            };
            if let Some(p) = pen_position.take() {
                cr.move_to(p.0, p.1);
            }
            match degree {
                0 => { /* ignore */ }
                1 => {
                    /* Line. */
                    let new_point = p_fn(curv.points().borrow()[1]);
                    cr.line_to(new_point.0, new_point.1);
                    pen_position = Some(new_point);
                }
                2 => {
                    /* Quadratic. */
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        p_fn(curv.points().borrow()[0])
                    };
                    let b = p_fn(curv.points().borrow()[1]);
                    let c = p_fn(curv.points().borrow()[2]);
                    cr.curve_to(
                        2.0 / 3.0 * b.0 + 1.0 / 3.0 * a.0,
                        2.0 / 3.0 * b.1 + 1.0 / 3.0 * a.1,
                        2.0 / 3.0 * b.0 + 1.0 / 3.0 * c.0,
                        2.0 / 3.0 * b.1 + 1.0 / 3.0 * c.1,
                        c.0,
                        c.1,
                    );
                    pen_position = Some(c);
                }
                3 => {
                    /* Cubic */
                    let _a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        p_fn(curv.points().borrow()[0])
                    };
                    let b = p_fn(curv.points().borrow()[1]);
                    let c = p_fn(curv.points().borrow()[2]);
                    let d = p_fn(curv.points().borrow()[3]);
                    cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                    pen_position = Some(d);
                }
                d => {
                    eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                    pen_position = Some(p_fn(*curv.points().borrow().last().unwrap()));
                    continue;
                }
            }
        }
        cr.stroke().expect("Invalid cairo surface state");
        if let Some(pos) = pen_position {
            cr.move_to(pos.0, pos.1);
        }
        let (pos_x, pos_y) = p_fn(cursor_position);
        cr.set_dash(&[3., 2., 1.], 1.);
        cr.set_line_width(2.5);
        cr.set_source_rgba(outline.0, outline.1, outline.2, 0.5 * outline.3);
        match self.current_curve.degree() {
            None => {
                cr.line_to(pos_x, pos_y);
                cr.stroke().expect("Invalid cairo surface state");
            }
            Some(0) => {
                let new_point = p_fn(self.current_curve.points().borrow()[0]);
                cr.line_to(new_point.0, new_point.1);
                cr.line_to(pos_x, pos_y);
                cr.stroke().expect("Invalid cairo surface state");
                cr.set_dash(&[], 0.);
                draw_endpoint(new_point);
            }
            Some(1) => {
                let a = p_fn(self.current_curve.points().borrow()[0]);
                cr.line_to(a.0, a.1);
                let b = p_fn(self.current_curve.points().borrow()[1]);
                let c = (pos_x, pos_y);
                let d = (pos_x, pos_y);
                cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                cr.stroke().expect("Invalid cairo surface state");
                cr.set_dash(&[], 0.);
                draw_endpoint(a);
                draw_endpoint(d);
                draw_handle(b, a);
            }
            Some(2) => {
                let a = p_fn(self.current_curve.points().borrow()[0]);
                cr.line_to(a.0, a.1);
                let b = p_fn(self.current_curve.points().borrow()[1]);
                let c = p_fn(self.current_curve.points().borrow()[2]);
                let d = (pos_x, pos_y);
                cr.set_line_width(2.5);
                cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                cr.stroke().expect("Invalid cairo surface state");
                cr.set_dash(&[], 0.);
                draw_endpoint(a);
                draw_endpoint(d);
                draw_handle(b, a);
                draw_handle(c, d);
            }
            Some(d) => {
                eprintln!(
                    "Something's wrong in current curve. Bezier of degree {}: {:?}",
                    d, self.current_curve
                );
            }
        }

        /*
        let (pos_x, pos_y) = p_fn(cursor_position);
        match self.inner {
            InnerState::AddControlPoint => {
            }
            InnerState::AddControlHandle => {
                cr.set_line_width(0.8);
            }
        }
        cr.line_to(pos_x, pos_y);
        */

        cr.restore().expect("Invalid cairo surface state");
    }
}
