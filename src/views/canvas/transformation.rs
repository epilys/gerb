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

use gtk::cairo::Matrix;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct TransformationInner {
    pub matrix: Cell<Matrix>,
}

#[glib::object_subclass]
impl ObjectSubclass for TransformationInner {
    const NAME: &'static str = "TransformationInner";
    type Type = Transformation;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for TransformationInner {}

glib::wrapper! {
    pub struct Transformation(ObjectSubclass<TransformationInner>);
}

impl Transformation {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Transformation");
        ret
    }

    pub fn set_scale(&self, factor: f64) {
        self.apply_scale(factor);
    }

    pub fn apply_scale(&self, factor: f64) {
        let mut m = self.imp().matrix.get();
        m.xx = factor;
        m.yy = factor;
        self.imp().matrix.set(m);
    }

    pub fn scale(&self) -> f64 {
        let m = self.imp().matrix.get();
        m.xx
    }

    pub fn scale_towards_point(&self, factor: f64, (x, y): (f64, f64)) {
        let mut m = self.imp().matrix.get();
        let m_ = m;
        let scale = m.xx;
        eprintln!("scale_towards_point {factor:?} ({x:?}, {y:?})");
        let mut to_pt_mx = Matrix::identity();
        to_pt_mx.translate(-x, -y);
        let mut scale_mx = Matrix::identity();
        scale_mx.scale(factor / scale, factor / scale);
        let mut from_pt_mx = Matrix::identity();
        from_pt_mx.translate(x, y);
        m = Matrix::multiply(&m, &to_pt_mx);
        m = Matrix::multiply(&m, &scale_mx);
        m = Matrix::multiply(&m, &from_pt_mx);
        //m.translate(-x, -y);
        //m.scale(factor, factor);
        //m.translate(x, y);
        std::dbg!(m_, m);
        //  let mut ident = Matrix::identity();
        //  let scale = m.xx;
        //  let m_ = m;
        //  m.translate(-x, -y);
        //  m.scale(factor/scale, factor/scale);
        //  m.translate(x, y);
        //  std::dbg!(m_,m);
        //m.xx = factor;
        //m.yy = factor;
        //let (lx, ly) = m.transform_point(x, y);
        //std::dbg!((lx,ly));
        ////m.translate(-lx, -ly);
        ////m.translate(lx, ly);
        //m.x0 = lx;
        //m.y0 = ly;
        self.imp().matrix.set(m); //Matrix::multiply(&ident, &m));
    }

    pub fn pan(&self, (dx, dy): (f64, f64)) {
        let mut m = self.imp().matrix.get();
        m.translate(dx, dy);
        self.imp().matrix.set(m);
    }

    pub fn set_pan(&self, (x, y): (f64, f64)) {
        let mut m = self.imp().matrix.get();
        m.x0 += x;
        m.y0 += y;
        self.imp().matrix.set(m);
    }

    pub fn reset(&self) {
        self.imp().matrix.set(Matrix::identity());
    }

    pub fn matrix(&self) -> Matrix {
        self.imp().matrix.get()
    }

    pub fn camera(&self) -> (f64, f64) {
        let m = self.imp().matrix.get();
        (m.x0, m.y0)
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new()
    }
}
