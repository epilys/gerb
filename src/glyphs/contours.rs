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
    pub const OPEN: &'static str = "open";
    pub const BIGGEST_CURVE: &'static str = "biggest-curve";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret.imp().open.set(true);
        ret
    }

    pub fn new_with_curves(curves: Vec<Bezier>) -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret.imp().open.set(true);
        *ret.imp().curves.borrow_mut() = curves;
        ret.recalc_continuities();
        ret
    }

    pub fn curves(&self) -> crate::utils::FieldRef<'_, Vec<Bezier>> {
        self.imp().curves.borrow().into()
    }

    // [ref:needs_unit_test]
    pub fn recalc_continuities(&self) {
        let closed: bool = !self.imp().open.get();
        let curves = self.curves();
        if closed {
            let prev_iter = curves
                .iter()
                .enumerate()
                .cycle()
                .skip(curves.len().saturating_sub(1));
            let curr_iter = curves.iter().enumerate().cycle();
            let next_iter = curves.iter().enumerate().cycle().skip(1);
            for (((_prev_idx, prev), (_curr_idx, curr)), (_next_idx, next)) in
                prev_iter.zip(curr_iter).zip(next_iter).take(curves.len())
            {
                let before = {
                    let prevp = prev.points();
                    let currp = curr.points();
                    if curr.property::<bool>(Bezier::SMOOTH) {
                        Self::calc_smooth_continuity(
                            <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&prevp),
                            <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&currp),
                        )
                    } else {
                        Continuity::Positional
                    }
                };
                prev.set_property(Bezier::CONTINUITY_OUT, Some(before));
                curr.set_property(Bezier::CONTINUITY_IN, Some(before));
                let after = {
                    let currp = curr.points();
                    let nextp = next.points();
                    if next.property::<bool>(Bezier::SMOOTH) {
                        Self::calc_smooth_continuity(
                            <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&currp),
                            <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&nextp),
                        )
                    } else {
                        Continuity::Positional
                    }
                };
                curr.set_property(Bezier::CONTINUITY_OUT, Some(after));
                next.set_property(Bezier::CONTINUITY_IN, Some(after));
            }
        } else {
            let curr_iter = curves.iter().enumerate().cycle();
            let next_iter = curves.iter().enumerate().cycle().skip(1);
            for ((_curr_idx, curr), (_next_idx, next)) in
                curr_iter.zip(next_iter).take(curves.len() + 1)
            {
                let after = {
                    let currp = curr.points();
                    let nextp = next.points();
                    if next.property::<bool>(Bezier::SMOOTH) {
                        Self::calc_smooth_continuity(
                            <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&currp),
                            <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&nextp),
                        )
                    } else {
                        Continuity::Positional
                    }
                };
                curr.set_property(Bezier::CONTINUITY_OUT, Some(after));
                next.set_property(Bezier::CONTINUITY_IN, Some(after));
            }
        }
    }

    pub fn push_curve(&self, curve: Bezier) {
        let mut curves = self.imp().curves.borrow_mut();
        if curve.points().is_empty() {
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
        let prev = curves[curves.len() - 1].points();
        let curr = curve.points();
        let new = if curve.property::<bool>(Bezier::SMOOTH) {
            Self::calc_smooth_continuity(
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&prev),
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&curr),
            )
        } else {
            Continuity::Positional
        };
        drop(curr);
        drop(prev);
        curve.set_property(Bezier::CONTINUITY_IN, Some(new));
        if !curves.is_empty() {
            curves[curves.len() - 1].set_property(Bezier::CONTINUITY_OUT, Some(new));
        }
        curves.push(curve);
    }

    pub fn close(&self) {
        if !self.imp().open.get() {
            return;
        }
        self.imp().open.set(false);

        let curves = self.imp().curves.borrow();
        if curves.is_empty() {
            return;
        }
        let prev = curves[curves.len() - 1].points();
        let curr = curves[0].points();
        let new = if curves[0].property::<bool>(Bezier::SMOOTH) {
            Self::calc_smooth_continuity(
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&prev),
                <Vec<CurvePoint> as AsRef<[CurvePoint]>>::as_ref(&curr),
            )
        } else {
            Continuity::Positional
        };
        drop(curr);
        drop(prev);
        curves[0].set_property(Bezier::CONTINUITY_IN, Some(new));
        curves[curves.len() - 1].set_property(Bezier::CONTINUITY_OUT, Some(new));
    }

    // [ref:needs_unit_test]
    fn calc_smooth_continuity(prev: &[CurvePoint], curr: &[CurvePoint]) -> Continuity {
        match (prev, curr) {
            ([_, _, p2, p3_1], [p3_2, p4, _, _])
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
            ([_, _, p2, p3_1], [p3_2, p4, _, _])
            | ([_, p2, p3_1], [p3_2, p4, _, _])
            | ([_, p2, p3_1], [p3_2, p4, _])
            | ([_, _, p2, p3_1], [p3_2, p4, _])
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
        for c in curves.iter() {
            c.reverse();
        }
    }

    // [ref:needs_unit_test]
    pub fn transform_points(
        &self,
        contour_index: usize,
        idxs_slice: &[GlyphPointIndex],
        m: Matrix,
    ) -> Vec<(GlyphPointIndex, Point)> {
        self.is_contour_modified.set(true);

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
            ($b:expr, $result:expr) => {
                let (uuid, position) = $result.unwrap();
                updated_points.push((
                    GlyphPointIndex {
                        contour_index,
                        curve_index: $b,
                        uuid,
                    },
                    position,
                ));
                extra_uuids.insert(($b, uuid));
            };
        }
        let closed: bool = !self.imp().open.get();
        let curves = self.curves();
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
            let pts_len = curr.points().len();
            let pts_to_transform = curr
                .points()
                .iter()
                .enumerate()
                .filter(|(_, p)| uuids.contains(&(curr_idx, p.uuid)))
                .map(|(i, p)| (i, p.uuid))
                .collect::<Vec<(usize, Uuid)>>();
            for (i, _uuid) in pts_to_transform {
                updated!(
                    curr_idx,
                    curr.modify_point(i, |cp| {
                        cp.position *= m;
                    })
                );
                if i == 0 {
                    // Point is first oncurve point.
                    // also transform prev last oncurve point and its handle

                    /* Points handle if it's not quadratic */
                    {
                        if pts_len > 2 && !extra_uuids.contains(&(curr_idx, curr.points()[1].uuid))
                        {
                            updated!(curr_idx, curr.modify_point(1, |cp| { cp.position *= m }));
                        }
                    }
                    if closed || curr_idx + 1 != curves.len() {
                        let pts_len = prev.points().len();
                        assert!(pts_len != 0);
                        /* previous curve's last oncurve point */
                        if !extra_uuids.contains(&(prev_idx, prev.points()[pts_len - 1].uuid)) {
                            updated!(
                                prev_idx,
                                prev.modify_point(pts_len - 1, |cp| { cp.position *= m })
                            );
                        }
                        /* previous curve's last handle if it's not quadratic */
                        if pts_len > 2
                            && !extra_uuids.contains(&(prev_idx, prev.points()[pts_len - 2].uuid))
                        {
                            updated!(
                                prev_idx,
                                prev.modify_point(pts_len - 2, |cp| { cp.position *= m })
                            );
                        }
                    }
                } else if i + 1 == pts_len {
                    // Point is last oncurve point.
                    // also transform next first oncurve point and its handle

                    /* Points handle if it's not quadratic */
                    {
                        if pts_len > 2
                            && !extra_uuids.contains(&(curr_idx, curr.points()[i - 1].uuid))
                        {
                            updated!(
                                curr_idx,
                                curr.modify_point(i - 1, |cp| { cp.position *= m })
                            );
                        }
                    }
                    if closed || curr_idx + 1 != curves.len() {
                        let pts_len = next.points().len();
                        assert!(pts_len != 0);
                        /* next first oncurve point */
                        if !extra_uuids.contains(&(next_idx, next.points()[0].uuid)) {
                            updated!(next_idx, next.transform_point(0, m));
                        }
                        /* next curve's first handle if it's not quadratic */
                        if pts_len > 2 && !extra_uuids.contains(&(next_idx, next.points()[1].uuid))
                        {
                            updated!(next_idx, next.transform_point(1, m));
                        }
                    }
                } else if closed || (next_idx != 0 && curr_idx + 1 != curves.len()) {
                    // Point is handle.
                    // also transform neighbored handle if continuity constraints demand so
                    macro_rules! cont {
                        (between ($idx:expr) and next) => {{
                            // [ref:FIXME]
                            if $idx + 1 == curves.len() {
                                debug_assert!(closed);
                                curves[$idx]
                                    .property::<Option<Continuity>>(Bezier::CONTINUITY_OUT)
                                    .unwrap()
                            } else if $idx == 0 {
                                debug_assert!(closed);
                                curves[$idx]
                                    .property::<Option<Continuity>>(Bezier::CONTINUITY_OUT)
                                    .unwrap()
                            } else {
                                curves[$idx]
                                    .property::<Option<Continuity>>(Bezier::CONTINUITY_OUT)
                                    .unwrap()
                            }
                        }};
                    }
                    if i == 1 {
                        let pts_len = prev.points().len();
                        assert!(pts_len != 0);
                        if pts_len > 2
                            && !extra_uuids.contains(&(prev_idx, prev.points()[pts_len - 2].uuid))
                        {
                            match cont!(between (prev_idx) and next) {
                                Continuity::Positional => {}
                                Continuity::Velocity => {
                                    let new_val =
                                        curr.points()[1].position.mirror(curr.points()[0].position);
                                    updated!(
                                        prev_idx,
                                        prev.modify_point(pts_len - 2, |cp| {
                                            cp.position = new_val;
                                        })
                                    );
                                }
                                Continuity::Tangent { beta } => {
                                    let center = prev.points()[pts_len - 1].position;
                                    let b_ = curr.points()[1].position;
                                    assert_eq!(curr.points()[0].position, center);
                                    let m_ = b_ - center;
                                    let new_val = 2.0 * center - (m_ / beta + center);
                                    updated!(
                                        prev_idx,
                                        prev.modify_point(pts_len - 2, |cp| {
                                            cp.position = new_val;
                                        })
                                    );
                                }
                            }
                        }
                    } else {
                        let pts_len = next.points().len();
                        assert!(pts_len > 0);
                        if pts_len > 2 && !extra_uuids.contains(&(next_idx, next.points()[1].uuid))
                        {
                            match cont!(between (curr_idx) and next) {
                                Continuity::Positional => {}
                                Continuity::Velocity => {
                                    let new_val = curr.points()[i]
                                        .position
                                        .mirror(curr.points()[i + 1].position);
                                    updated!(
                                        next_idx,
                                        next.modify_point(1, |cp| {
                                            cp.position = new_val;
                                        })
                                    );
                                }
                                Continuity::Tangent { beta } => {
                                    let center = next.points()[0].position;
                                    let n_ =
                                        curr.points()[i].position - curr.points()[i + 1].position;
                                    let new_val = 2.0 * center - (beta * n_ + center);
                                    updated!(
                                        next_idx,
                                        next.modify_point(1, |cp| {
                                            cp.position = new_val;
                                        })
                                    );
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
                .get(curve_index)?
                .points()
                .iter()
                .find(|cp| cp.uuid == uuid)?
                .position,
        )
    }

    // [ref:needs_unit_test]
    // [ref:FIXME]: return None on line Beziers
    #[allow(clippy::type_complexity)]
    /// For an on-curve point, return neighboring control points.
    pub fn get_control_point(
        &self,
        GlyphPointIndex {
            contour_index,
            curve_index,
            uuid,
        }: GlyphPointIndex,
    ) -> (
        Option<(GlyphPointIndex, Point)>,
        Option<(GlyphPointIndex, Point)>,
    ) {
        let cl = || -> Option<(
            Option<(GlyphPointIndex, Point)>,
            Option<(GlyphPointIndex, Point)>,
        )> {
            let into_ret = |cp: &CurvePoint, curve_index: usize|-> (GlyphPointIndex, Point) {
                (cp.glyph_index(contour_index, curve_index), cp.position)
            };
            let curves = self.curves();
            let curve = curves.get(curve_index)?;
            let points = curve.points();
            let (index, continuity) = points
                .iter()
                .enumerate()
                .find(|(_, cp)| cp.uuid == uuid && cp.continuity.is_some())
                .and_then(|(i, cp)| Some((i, cp.continuity?)))?;
            Some(match index {
                _ if curve_index + 1 == curves.len() && index + 1 == points.len() && !self.open.get() => {
                    let prev_point = if index > 1 && !matches!(continuity, Continuity::Positional) {
                        points.get(index - 1).map(|cp| into_ret(cp, curve_index))
                    } else {
                        None
                    };
                    let next_point = curves[0].points().get(1).map(|cp| into_ret(cp, 0));
                    (prev_point, next_point)
                }
                _ if curve_index == curves.len() && index == points.len() && self.open.get() => {
                    (None, None)
                }
                _ if curve_index == 0 && index == 0 && !self.open.get() && curves.len() > 0 => {
                    let prev_point = if !matches!(continuity, Continuity::Positional) {
                        curves[curves.len()-1].points().iter().rev().nth(1).map(|cp| into_ret(cp, curves.len()-1))
                    } else {
                        None
                    };
                    let next_point = points.get(1).map(|cp| into_ret(cp, curve_index));
                    (prev_point, next_point)
                }
                _ if curve_index == 0 && index == 0 && self.open.get() => (None, None),
                _ if index + 1 == points.len() => {
                    let prev_point = if index > 1 && !matches!(continuity, Continuity::Positional) {
                        points.iter().rev().nth(1).map(|cp| into_ret(cp, curve_index))
                    } else {
                        None
                    };
                    let next_point = curves[curve_index+1].points().get(1).map(|cp| into_ret(cp, curve_index+1));
                    (prev_point, next_point)
                }
                _ if index == 0 => {
                    let prev_point =if !matches!(continuity, Continuity::Positional) {
                        curves[curve_index-1].points().iter().rev().nth(1).map(|cp| into_ret(cp, curve_index-1))
                    } else {
                        None
                    };
                    let next_point = points.get(1).map(|cp| into_ret(cp, curve_index));
                    (prev_point, next_point)
                }
                _ => {
                    let prev_point = if index > 1 && !matches!(continuity, Continuity::Positional) {
                        points.get(index - 1).map(|cp| into_ret(cp, curve_index))
                    } else {
                        None
                    };
                    let next_point = curves.get(curve_index+1).and_then(|c| c.points().get(1).map(|cp| into_ret(cp, curve_index+1)));
                    (prev_point, next_point)
                }
            })
        };
        cl().unwrap_or((None, None))
    }

    pub fn pop_curve(&self) -> Option<Bezier> {
        let mut curves = self.curves.borrow_mut();
        if curves.is_empty() {
            return None;
        }
        curves.pop()
    }

    // [ref:needs_unit_test]
    pub fn change_continuity(
        &self,
        GlyphPointIndex {
            contour_index: _,
            curve_index,
            uuid,
        }: GlyphPointIndex,
        continuity: Continuity,
    ) -> Option<Continuity> {
        let (index, degree) = self
            .curves()
            .get(curve_index)?
            .points()
            .iter()
            .enumerate()
            .find(|(_, cp)| cp.uuid == uuid)
            .map(|(i, cp)| (i, cp.degree))?;
        let prev_value = if degree == Some(index) {
            self.curves()[curve_index].property(Bezier::CONTINUITY_OUT)
        } else {
            self.curves()[curve_index].property(Bezier::CONTINUITY_IN)
        };
        match prev_value {
            None => todo!(),
            Some(c) if c != continuity => {
                let curves_no = self.curves().len();
                if degree == Some(index) {
                    self.curves
                        .borrow_mut()
                        .get(curve_index)?
                        .set_property(Bezier::CONTINUITY_OUT, Some(continuity));
                    self.curves
                        .borrow_mut()
                        .get(if curve_index == 0 {
                            curves_no - 1
                        } else {
                            curve_index - 1
                        })?
                        .set_property(Bezier::CONTINUITY_IN, Some(continuity));
                } else {
                    self.curves
                        .borrow_mut()
                        .get(curve_index)?
                        .set_property(Bezier::CONTINUITY_IN, Some(continuity));
                    self.curves
                        .borrow_mut()
                        .get(if curve_index == 0 {
                            curves_no - 1
                        } else {
                            curve_index - 1
                        })?
                        .set_property(Bezier::CONTINUITY_OUT, continuity);
                }

                Some(c)
            }
            Some(c) => Some(c),
        }
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
    curves: RefCell<Vec<Bezier>>,
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
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
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

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
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
}

/// Given two cubic Bézier curves with control points [P0, P1, P2, P3] and [P3, P4, P5, P6]
/// respectively, the constraints for ensuring continuity at P3 can be defined as follows:
#[derive(Clone, Debug, Default, PartialEq, Copy, glib::Boxed)]
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
    pub fn is_positional(self) -> bool {
        matches!(self, Self::Positional)
    }
}
