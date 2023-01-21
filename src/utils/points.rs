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

mod points {
    use gtk::glib;
    use std::cmp::Ordering;
    use std::hash::{Hash, Hasher};
    use std::ops::{Add, Div, Mul, Sub};
    use uuid::Uuid;

    #[derive(Clone, Debug, Default, Copy, glib::Boxed)]
    #[boxed_type(name = "Point", nullable)]
    pub struct Point {
        pub uuid: Uuid,
        pub x: f64,
        pub y: f64,
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
    }

    impl PartialEq for Point {
        fn eq(&self, other: &Self) -> bool {
            self.uuid == other.uuid
        }
    }

    impl Eq for Point {}

    impl Hash for Point {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.uuid.hash(state);
        }
    }

    impl Into<(f64, f64)> for Point {
        fn into(self) -> (f64, f64) {
            (self.x, self.y)
        }
    }

    impl From<(f64, f64)> for Point {
        fn from((x, y): (f64, f64)) -> Point {
            Point {
                uuid: Uuid::from_u64_pair(x.to_bits(), y.to_bits()),
                x,
                y,
            }
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

    impl Mul<Point> for gtk::cairo::Matrix {
        type Output = Point;

        fn mul(self, point: Point) -> Self::Output {
            let (x, y) = self.transform_point(point.x, point.y);
            (x, y).into()
        }
    }

    #[derive(Clone, Hash, PartialEq, Debug, Default, Copy)]
    pub struct IPoint {
        pub x: i64,
        pub y: i64,
    }

    impl Into<IPoint> for Point {
        fn into(self) -> IPoint {
            IPoint {
                x: self.x as i64,
                y: self.y as i64,
            }
        }
    }

    impl Into<IPoint> for &Point {
        fn into(self) -> IPoint {
            IPoint {
                x: self.x as i64,
                y: self.y as i64,
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
}
pub use points::{IPoint, Point};
