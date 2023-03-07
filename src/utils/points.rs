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

use crate::prelude::*;
use gtk::cairo::Matrix;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Sub};

/// This type does not implement copy and is not meant to be modified by anyone except for its
/// container struct (usually a [`Bezier`]).
#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "CurvePoint", nullable)]
pub struct CurvePoint {
    pub uuid: Uuid,
    pub position: Point,
    pub degree: Option<usize>,
    pub continuity: Option<Continuity>,
}

impl Default for CurvePoint {
    fn default() -> CurvePoint {
        CurvePoint {
            uuid: Uuid::new_v4(),
            position: Point::default(),
            degree: None,
            continuity: None,
        }
    }
}

impl CurvePoint {
    pub fn new(position: Point) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }

    pub fn glyph_index(&self, contour_index: usize, curve_index: usize) -> GlyphPointIndex {
        GlyphPointIndex {
            contour_index,
            curve_index,
            uuid: self.uuid,
        }
    }
}

impl PartialEq for CurvePoint {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for CurvePoint {}

impl Hash for CurvePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl From<CurvePoint> for (f64, f64) {
    fn from(p: CurvePoint) -> (f64, f64) {
        (p.position.x, p.position.y)
    }
}

impl std::ops::Deref for CurvePoint {
    type Target = Point;

    fn deref(&self) -> &Self::Target {
        &self.position
    }
}

#[derive(Clone, Debug, Default, Copy, glib::Boxed)]
#[boxed_type(name = "Point", nullable)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.x.to_bits(), self.y.to_bits()) == (other.x.to_bits(), other.y.to_bits())
    }
}

impl Eq for Point {}

impl From<CurvePoint> for Point {
    fn from(cp: CurvePoint) -> Point {
        cp.position
    }
}

impl From<&CurvePoint> for Point {
    fn from(cp: &CurvePoint) -> Point {
        cp.position
    }
}

impl Point {
    pub fn collinear(&self, other_a: &Self, other_b: &Self) -> bool {
        //Putting all this together, the points (a,b), (m,n) and (x,y) are collinear if and only if
        //    (n−b)(x−m)=(y−n)(m−a)
        let (a, b) = (self.x, self.y);
        let (m, n) = (other_a.x, other_a.y);
        let (x, y) = (other_b.x, other_b.y);
        (n - b) * (x - m) == (y - n) * (m - a)
    }

    pub fn transform(&mut self, m: Matrix) -> Self {
        let old_val = *self;
        let (x, y) = m.transform_point(self.x, self.y);
        self.x = x;
        self.y = y;
        old_val
    }

    pub fn mirror(&self, c: Self) -> Self {
        let line = Line::from_two_points(*self, c);
        let perp = line.perpendicular(c);

        let (x, y) = (self.x, self.y);
        let Line { a, b, c } = perp;
        let b2a = (b * b) / a;
        let mx = (b2a * x - c - b * y) / (a + b2a);
        let my = (-a * mx - c) / b;
        (2.0 * mx - x, 2.0 * my - y).into()
    }

    pub fn norm(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn angle(&self, rhs: Self) -> f64 {
        let a = rhs.x * self.y - self.x * rhs.y;
        let b = self.x * self.y + rhs.x * rhs.y;
        a.atan2(b)
    }

    pub fn atan2(&self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn distance(&self, rhs: Self) -> f64 {
        let xlk = rhs.x - self.x;
        let ylk = rhs.y - self.y;
        (xlk * xlk + ylk * ylk).sqrt()
    }

    pub fn dot(&self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn unit(&self) -> Self {
        let norm = self.norm();
        if !norm.is_normal() || norm <= f64::MIN_POSITIVE {
            return (0.0, 0.0).into();
        }
        *self / norm
    }
}

impl From<Point> for (f64, f64) {
    fn from(p: Point) -> (f64, f64) {
        (p.x, p.y)
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Point {
        Point { x, y }
    }
}

impl Add<Self> for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.x + rhs.x, self.y + rhs.y).into()
    }
}

impl Sub<Self> for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

impl Div<Self> for Point {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        (self.x / rhs.x, self.y / rhs.y).into()
    }
}

impl Mul<Point> for f64 {
    type Output = Point;

    fn mul(self, p: Point) -> Self::Output {
        (p.x * self, p.y * self).into()
    }
}

impl Mul<f64> for Point {
    type Output = Self;

    fn mul(self, f: f64) -> Self::Output {
        (self.x / f, self.y / f).into()
    }
}

impl Div<Point> for f64 {
    type Output = Point;

    fn div(self, p: Point) -> Self::Output {
        (self / p.x, self / p.y).into()
    }
}

impl Div<f64> for Point {
    type Output = Self;

    fn div(self, f: f64) -> Self::Output {
        (self.x / f, self.y / f).into()
    }
}

impl std::ops::DivAssign<f64> for Point {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl std::ops::MulAssign<f64> for Point {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, point: Point) -> Self::Output {
        let (x, y) = self.transform_point(point.x, point.y);
        (x, y).into()
    }
}

impl std::ops::MulAssign<Matrix> for Point {
    fn mul_assign(&mut self, m: Matrix) {
        let (x, y) = m.transform_point(self.x, self.y);
        self.x = x;
        self.y = y;
    }
}

#[derive(Clone, Hash, PartialEq, Debug, Default, Copy)]
pub struct IPoint {
    pub x: i64,
    pub y: i64,
}

impl From<Point> for IPoint {
    fn from(p: Point) -> IPoint {
        IPoint {
            x: p.x as i64,
            y: p.y as i64,
        }
    }
}

impl From<&Point> for IPoint {
    fn from(p: &Point) -> IPoint {
        IPoint {
            x: p.x as i64,
            y: p.y as i64,
        }
    }
}

impl From<(i64, i64)> for IPoint {
    fn from((x, y): (i64, i64)) -> IPoint {
        IPoint { x, y }
    }
}

impl Ord for IPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.x, self.y).cmp(&(other.x, other.y))
    }
}

impl Eq for IPoint {}

impl PartialOrd for IPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Default, Copy)]
pub struct Line {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Line {
    pub fn from_two_points(point_a: Point, point_b: Point) -> Self {
        let (xa, ya) = (point_a.x, point_a.y);
        let (xb, yb) = (point_b.x, point_b.y);
        let a = yb - ya;
        let b = xa - xb;
        let c = xb * ya - xa * yb;
        let mut ret = [a, b, c];
        while ret.iter().any(|i| *i == 0.0) {
            ret[0] += 1.0;
            ret[1] += 1.0;
            ret[2] += 1.0;
        }
        let [a, b, c] = ret;
        Self { a, b, c }
    }

    pub fn perpendicular(self: Line, p: Point) -> Self {
        let Self { a, b, c: _ } = self;
        Self {
            a: b,
            b: -1.0 * a,
            c: a * p.y - b * p.x,
        }
    }
}
