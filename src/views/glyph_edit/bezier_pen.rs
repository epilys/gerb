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

use crate::glyphs::{Bezier, Contour, Point};

#[derive(Debug, Clone, Copy, PartialEq)]
enum InnerState {
    AddControlPoint,
    AddControlHandle,
}

#[derive(Debug, Clone)]
pub struct State {
    inner: InnerState,
    current_curve: Bezier,
    curves: Vec<Bezier>,
}

impl Default for State {
    fn default() -> Self {
        State {
            inner: InnerState::AddControlPoint,
            current_curve: Bezier::new(true, vec![]),
            curves: vec![],
        }
    }
}

impl State {
    pub fn insert_point(&mut self, point: (i64, i64)) {
        match self.inner {
            InnerState::AddControlPoint => {
                self.inner = InnerState::AddControlHandle;
                self.current_curve.points.push(point);
                if self.current_curve.points.len() == 4 {
                    /* current_curve is cubic, so split it. */
                    let curv =
                        std::mem::replace(&mut self.current_curve, Bezier::new(true, vec![]));
                    self.curves.push(curv);
                }
            }
            InnerState::AddControlHandle => {
                self.inner = InnerState::AddControlPoint;
                self.current_curve.points.push(point);
            }
        }
    }

    pub fn close(self) -> Contour {
        let State {
            inner: _,
            current_curve,
            mut curves,
        } = self;
        if current_curve.degree().is_some() {
            curves.push(current_curve);
        }

        Contour { open: true, curves }
    }
}
