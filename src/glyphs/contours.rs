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
use std::collections::BTreeSet;

glib::wrapper! {
    pub struct Contour(ObjectSubclass<ContourInner>);
}

impl Default for Contour {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for Contour {
    type Target = ContourInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Contour {
    pub const OPEN: &str = "open";
    pub const CONTINUITIES: &str = "continuities";
    pub const CONTINUITY: &str = "continuity";
    pub const BIGGEST_CURVE: &str = "biggest-curve";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret.imp().open.set(true);
        ret
    }

    pub fn curves(&self) -> &RefCell<Vec<Bezier>> {
        &self.imp().curves
    }

    pub fn continuities(&self) -> &RefCell<Vec<Continuity>> {
        &self.imp().continuities
    }

    pub fn push_curve(&self, curve: Bezier) {
        let mut curves = self.imp().curves.borrow_mut();
        let mut continuities = self.imp().continuities.borrow_mut();
        if curve.points().borrow().is_empty() {
            return;
        }
        let new_len = curve.approx_length();
        if let Some(b) = self
            .imp()
            .biggest_curve
            .get()
            .filter(
                |&BiggestCurve {
                     approx_length: len, ..
                 }| new_len > len && (new_len - len).abs() < 0.05,
            )
            .or(Some(BiggestCurve {
                index: curves.len(),
                approx_length: new_len,
            }))
        {
            self.imp().is_contour_modified.set(false);
            self.imp().biggest_curve.set(Some(b));
        }

        if curves.is_empty() {
            curves.push(curve);
            return;
        }
        let prev = curves[curves.len() - 1].points().borrow();
        let curr = curve.points().borrow();
        if curve.property::<bool>(Bezier::SMOOTH) {
            continuities.push(Self::calc_smooth_continuity(
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&prev),
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&curr),
            ));
        } else {
            continuities.push(Continuity::Positional);
        }
        drop(curr);
        drop(prev);
        curves.push(curve);
    }

    pub fn close(&self) {
        if !self.imp().open.get() {
            return;
        }
        self.imp().open.set(false);

        let curves = self.imp().curves.borrow();
        let mut continuities = self.imp().continuities.borrow_mut();
        if curves.is_empty() {
            return;
        }
        let prev = curves[curves.len() - 1].points().borrow();
        let curr = curves[0].points().borrow();
        if curves[0].property::<bool>(Bezier::SMOOTH) {
            continuities.push(Self::calc_smooth_continuity(
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&prev),
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&curr),
            ));
        } else {
            continuities.push(Continuity::Positional);
        }
        assert_eq!(continuities.len(), curves.len());
    }

    fn calc_smooth_continuity(prev: &[CurvePoint], curr: &[CurvePoint]) -> Continuity {
        match (prev, curr) {
            (&[_, _, ref p2, ref p3_1], &[ref p3_2, ref p4, _, _])
                if p3_1.position == p3_2.position
                    && p2.position.collinear(&p3_1.position, &p4.position) =>
            {
                let beta =
                    (p4.position - p3_1.position).norm() / (p3_1.position - p2.position).norm();
                if beta.is_nan() {
                    Continuity::Positional
                } else if beta == 1.0
                    || (beta.round() == 1.0 && (beta.fract() - 1.0).abs() < (1e-2 / 2.0))
                {
                    Continuity::Velocity
                } else {
                    Continuity::Tangent { beta }
                }
            }
            (&[_, _, ref p2, ref p3_1], &[ref p3_2, ref p4, _, _])
            | (&[_, ref p2, ref p3_1], &[ref p3_2, ref p4, _, _])
            | (&[_, ref p2, ref p3_1], &[ref p3_2, ref p4, _])
            | (&[_, _, ref p2, ref p3_1], &[ref p3_2, ref p4, _])
                if p3_1.position == p3_2.position
                    && p4.position == 2.0 * p3_1.position - p2.position =>
            {
                Continuity::Velocity
            }
            _ => Continuity::Positional,
        }
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

    pub fn transform_points(
        &self,
        contour_index: usize,
        idxs_slice: &[GlyphPointIndex],
        m: Matrix,
    ) -> Vec<(GlyphPointIndex, Point)> {
        if !self.is_contour_modified.get() {
            self.is_contour_modified.set(true);
        }

        let uuids = idxs_slice
            .iter()
            .filter(|i| i.contour_index == contour_index)
            .map(|i| (i.curve_index, i.uuid))
            .collect::<BTreeSet<_>>();
        let mut extra_uuids = uuids.clone();
        let curves_idxs = idxs_slice
            .iter()
            .filter(|i| i.contour_index == contour_index)
            .map(|i| i.curve_index)
            .collect::<BTreeSet<_>>();
        let mut updated_points = vec![];
        macro_rules! updated {
            ($b:expr, $point:expr) => {
                updated_points.push((
                    GlyphPointIndex {
                        contour_index,
                        curve_index: $b,
                        uuid: $point.uuid,
                    },
                    $point.position,
                ));
                extra_uuids.insert(($b, $point.uuid));
            };
        }
        let closed: bool = !self.imp().open.get();
        let continuities = self.imp().continuities.borrow();
        let curves = self.imp().curves.borrow();
        let prev_iter = curves
            .iter()
            .enumerate()
            .cycle()
            .skip(curves.len().saturating_sub(1));
        let curr_iter = curves.iter().enumerate().cycle();
        let next_iter = curves.iter().enumerate().cycle().skip(1);
        for (((prev_idx, prev), (curr_idx, curr)), (next_idx, next)) in prev_iter
            .zip(curr_iter)
            .zip(next_iter)
            .take(curves.len())
            .filter(|((_, (curr_idx, _)), _)| curves_idxs.contains(curr_idx))
        {
            let mut pts = curr.points().borrow_mut();
            let pts_len = pts.len();
            let pts_to_transform = pts
                .iter()
                .enumerate()
                .filter(|(_, p)| uuids.contains(&(curr_idx, p.uuid)))
                .map(|(i, p)| (i, p.uuid))
                .collect::<Vec<(usize, Uuid)>>();
            for (i, _uuid) in pts_to_transform {
                macro_rules! points_mut {
                    (prev) => {{
                        Some(prev.points().borrow_mut())
                        //if prev_idx == curr_idx {
                        //    None
                        //} else {
                        //    Some(prev.points().borrow_mut())
                        //}
                    }};
                    (next) => {{
                        Some(next.points().borrow_mut())
                        //if next_idx == curr_idx {
                        //    None
                        //} else {
                        //    Some(next.points().borrow_mut())
                        //}
                    }};
                }
                pts[i].position *= m;
                updated!(curr_idx, pts[i]);
                curr.set_modified();
                if i == 0 {
                    // Point is first oncurve point.
                    // also transform prev last oncurve point and its handle

                    /* Points handle if it's not quadratic */
                    {
                        if pts_len > 2 && !extra_uuids.contains(&(curr_idx, pts[1].uuid)) {
                            pts[1].position *= m;
                            updated!(curr_idx, pts[1]);
                        }
                    }
                    if closed || curr_idx + 1 != curves.len() {
                        if let Some(mut prev_points) = points_mut!(prev) {
                            let pts_len = prev_points.len();
                            assert!(!prev_points.is_empty());
                            /* previous curve's last oncurve point */
                            if !extra_uuids.contains(&(prev_idx, prev_points[pts_len - 1].uuid)) {
                                prev_points[pts_len - 1].position *= m;
                                updated!(prev_idx, prev_points[pts_len - 1]);
                                prev.set_modified();
                            }
                            /* previous curve's last handle if it's not quadratic */
                            if pts_len > 2
                                && !extra_uuids.contains(&(prev_idx, prev_points[pts_len - 2].uuid))
                            {
                                prev_points[pts_len - 2].position *= m;
                                updated!(prev_idx, prev_points[pts_len - 2]);
                                prev.set_modified();
                            }
                        }
                    }
                } else if i + 1 == pts_len {
                    // Point is last oncurve point.
                    // also transform next first oncurve point and its handle

                    /* Points handle if it's not quadratic */
                    {
                        if pts_len > 2 && !extra_uuids.contains(&(curr_idx, pts[i - 1].uuid)) {
                            pts[i - 1].position *= m;
                            updated!(curr_idx, pts[i - 1]);
                        }
                    }
                    if closed || curr_idx + 1 != curves.len() {
                        if let Some(mut next_points) = points_mut!(next) {
                            let pts_len = next_points.len();
                            assert!(!next_points.is_empty());
                            /* next first oncurve point */
                            if !extra_uuids.contains(&(next_idx, next_points[0].uuid)) {
                                next_points[0].position *= m;
                                updated!(next_idx, next_points[0]);
                                next.set_modified();
                            }
                            /* next curve's first handle if it's not quadratic */
                            if pts_len > 2
                                && !extra_uuids.contains(&(next_idx, next_points[1].uuid))
                            {
                                next_points[1].position *= m;
                                updated!(next_idx, next_points[1]);
                                next.set_modified();
                            }
                        }
                    }
                } else if closed || (next_idx != 0 && curr_idx + 1 != curves.len()) {
                    // Point is handle.
                    // also transform neighbored handle if continuity constraints demand so
                    macro_rules! cont {
                        (between ($idx:expr) and next) => {{
                            if $idx + 1 == continuities.len() {
                                debug_assert!(closed);
                                debug_assert_eq!(continuities.len(), curves.len());
                                continuities[$idx - 1]
                            } else if $idx == 0 {
                                debug_assert!(closed);
                                continuities[curves.len() - 1]
                            } else {
                                continuities[$idx - 1]
                            }
                        }};
                    }
                    if i == 1 {
                        if let Some(mut prev_points) = points_mut!(prev) {
                            let pts_len = prev_points.len();
                            assert!(!prev_points.is_empty());
                            if pts_len > 2
                                && !extra_uuids.contains(&(prev_idx, prev_points[pts_len - 2].uuid))
                            {
                                prev.set_modified();
                                match cont!(between (curr_idx) and next) {
                                    Continuity::Positional => {}
                                    Continuity::Velocity => {
                                        prev_points[pts_len - 2].position =
                                            pts[1].position.mirror(pts[0].position);
                                        updated!(prev_idx, prev_points[pts_len - 2]);
                                    }
                                    Continuity::Tangent { beta } => {
                                        let center = prev_points[pts_len - 1].position;
                                        let b_ = pts[1].position;
                                        assert_eq!(pts[0].position, center);
                                        let m_ = b_ - center;
                                        let n_ = 2.0 * center - (m_ / beta + center);
                                        prev_points[pts_len - 2].position = n_;
                                        updated!(prev_idx, prev_points[pts_len - 2]);
                                    }
                                }
                            }
                        }
                    } else if let Some(mut next_points) = points_mut!(next) {
                        let pts_len = next_points.len();
                        assert!(!next_points.is_empty());
                        if pts_len > 2 && !extra_uuids.contains(&(next_idx, next_points[1].uuid)) {
                            next.set_modified();
                            match cont!(between (next_idx) and next) {
                                Continuity::Positional => {}
                                Continuity::Velocity => {
                                    next_points[1].position =
                                        pts[i].position.mirror(pts[i + 1].position);
                                    updated!(next_idx, next_points[1]);
                                }
                                Continuity::Tangent { beta } => {
                                    let center = next_points[0].position;
                                    let n_ = pts[i].position - pts[i + 1].position;
                                    let m_ = 2.0 * center - (beta * n_ + center);
                                    next_points[1].position = m_;
                                    updated!(next_idx, next_points[1]);
                                }
                            }
                        }
                    }
                } else {
                    // Point is handle.
                    // Transform nothing else
                }
            }
        }
        updated_points
    }

    pub fn get_point(
        &self,
        GlyphPointIndex {
            contour_index: _,
            curve_index,
            uuid,
        }: GlyphPointIndex,
    ) -> Option<Point> {
        Some(
            self.curves()
                .borrow()
                .get(curve_index)?
                .points()
                .borrow()
                .iter()
                .find(|cp| cp.uuid == uuid)?
                .position,
        )
    }
}

#[derive(Copy, Clone, Debug)]
struct BiggestCurve {
    index: usize,
    approx_length: f64,
}

#[derive(Default)]
pub struct ContourInner {
    pub open: Cell<bool>,
    pub curves: RefCell<Vec<Bezier>>,
    pub continuities: RefCell<Vec<Continuity>>,
    biggest_curve: Cell<Option<BiggestCurve>>,
    pub is_contour_modified: Cell<bool>,
}

impl std::fmt::Debug for ContourInner {
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
            .field("biggest_curve", &self.biggest_curve.get())
            .finish()
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ContourInner {
    const NAME: &'static str = "Contour";
    type Type = Contour;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for ContourInner {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecValueArray::new(
                        Contour::CONTINUITIES,
                        Contour::CONTINUITIES,
                        Contour::CONTINUITIES,
                        &glib::ParamSpecBoxed::new(
                            Contour::CONTINUITY,
                            Contour::CONTINUITY,
                            Contour::CONTINUITY,
                            Continuity::static_type(),
                            glib::ParamFlags::READWRITE,
                        ),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Contour::OPEN,
                        Contour::OPEN,
                        Contour::OPEN,
                        true,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecUInt64::new(
                        Contour::BIGGEST_CURVE,
                        Contour::BIGGEST_CURVE,
                        Contour::BIGGEST_CURVE,
                        0,
                        u64::MAX,
                        0,
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Contour::CONTINUITIES => {
                let continuities = self.continuities.borrow();
                let mut ret = glib::ValueArray::new(continuities.len() as u32);
                for c in continuities.iter() {
                    ret.append(&c.to_value());
                }
                ret.to_value()
            }
            Contour::OPEN => self.open.get().to_value(),
            Contour::BIGGEST_CURVE => {
                let prev = match (self.is_contour_modified.get(), self.biggest_curve.get()) {
                    (false, Some(BiggestCurve { index: ret, .. })) => {
                        return (ret as u64).to_value()
                    }
                    (
                        true,
                        Some(BiggestCurve {
                            index: ret,
                            approx_length: d,
                            ..
                        }),
                    ) => Some((ret, d)),
                    (_, None) => None,
                };
                let curves = self.curves.borrow();
                let ret = curves
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.approx_length()))
                    .fold((0, f64::NEG_INFINITY), |(prev, acc), (i, len)| {
                        if len > acc {
                            (i, len)
                        } else {
                            (prev, acc)
                        }
                    });

                if let Some((ret, d)) = prev.filter(|&(_, prev)| (prev - ret.1).abs() < 20.00) {
                    self.biggest_curve.set(Some(BiggestCurve {
                        index: ret,
                        approx_length: d,
                    }));
                    self.is_contour_modified.set(false);
                    return (ret as u64).to_value();
                }
                self.biggest_curve.set(Some(BiggestCurve {
                    index: ret.0,
                    approx_length: ret.1,
                }));
                self.is_contour_modified.set(false);
                (ret.0 as u64).to_value()
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Contour::CONTINUITIES => {
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

impl Continuity {
    pub fn is_positional(&self) -> bool {
        matches!(self, Self::Positional)
    }
}
