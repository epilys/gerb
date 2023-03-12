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

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::Path;
use std::rc::{Rc, Weak};

use crate::prelude::*;
use crate::ufo;
use crate::utils::{curves::*, *};

use gtk::cairo::Matrix;
use gtk::{glib::prelude::*, subclass::prelude::*};
use uuid::Uuid;

mod guidelines;
pub use guidelines::*;

pub use crate::ufo::glif::{self, Advance, Anchor, ImageRef, Unicode};

mod contours;
pub use contours::*;

pub mod obj;
pub use obj::GlyphMetadata;

#[derive(Debug, Clone)]
pub struct Component {
    pub base_name: String,
    pub base: Weak<RefCell<Glyph>>,
    pub x_offset: f64,
    pub y_offset: f64,
    pub x_scale: f64,
    pub xy_scale: f64,
    pub yx_scale: f64,
    pub y_scale: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlyphKind {
    Char(char),
    Component(String),
}

impl Default for GlyphKind {
    fn default() -> Self {
        Self::Char(' ')
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub contours: Vec<Contour>,
    pub components: Vec<Component>,
    guidelines: Vec<Guideline>,
    pub lib: IndexMap<String, plist::Value>,
    pub metadata: GlyphMetadata,
}

impl std::ops::Deref for Glyph {
    type Target = GlyphMetadata;

    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl Ord for Glyph {
    fn cmp(&self, other: &Self) -> Ordering {
        use GlyphKind::*;
        match (&self.kinds().0, &other.kinds().0) {
            (Char(s), Char(o)) => s.cmp(o),
            (Char(_), _) => Ordering::Less,
            (Component(ref name), Component(ref other_name)) => name.cmp(other_name),
            (Component(_), Char(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Glyph {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Glyph {
    fn eq(&self, other: &Self) -> bool {
        use GlyphKind::*;
        match (&self.kinds().0, &other.kinds().0) {
            (Char(s), Char(o)) => s == o,
            (Char(_), Component(_)) | (Component(_), Char(_)) => false,
            (Component(name), Component(other_name)) => name == other_name,
        }
    }
}

impl Eq for Glyph {}

impl Default for Glyph {
    fn default() -> Self {
        Self::new_empty("space", ' ')
    }
}

#[derive(Clone, Copy)]
pub struct GlyphDrawingOptions<'a> {
    pub outline: DrawOptions,
    pub inner_fill: Option<DrawOptions>,
    pub highlight: Option<(usize, usize)>,
    pub matrix: Matrix,
    pub units_per_em: f64,
    pub handle_connection: Option<DrawOptions>,
    pub handle: Option<DrawOptions>,
    pub corner: Option<DrawOptions>,
    pub smooth_corner: Option<DrawOptions>,
    pub direction_arrow: Option<DrawOptions>,
    pub selection: Option<&'a HashSet<Uuid>>,
}

impl Default for GlyphDrawingOptions<'_> {
    fn default() -> Self {
        Self {
            outline: DrawOptions {
                color: Color::BLACK,
                bg: None,
                size: 4.0,
                inherit_size: None,
            },
            inner_fill: None,
            highlight: None,
            matrix: Matrix::identity(),
            units_per_em: ufo::constants::UNITS_PER_EM,
            handle_connection: None,
            handle: None,
            corner: None,
            smooth_corner: None,
            direction_arrow: None,
            selection: None,
        }
    }
}

impl Glyph {
    #[allow(clippy::type_complexity)]
    pub fn from_ufo(
        root_path: PathBuf,
        contents: &ufo::Contents,
    ) -> Result<IndexMap<String, Rc<RefCell<Self>>>, Box<dyn std::error::Error>> {
        let mut ret: IndexMap<String, Rc<RefCell<Self>>> = IndexMap::default();
        let mut glyphs_with_refs: Vec<Rc<_>> = vec![];
        let mut path = root_path;

        for (name, filename) in contents.glyphs().iter() {
            path.push(filename);
            use std::fs::File;
            use std::io::prelude::*;
            let mut file = match File::open(&path) {
                Err(err) => return Err(format!("Couldn't open {}: {}", path.display(), err).into()),
                Ok(file) => file,
            };

            let mut s = String::new();
            if let Err(err) = file.read_to_string(&mut s) {
                return Err(format!("Couldn't read {}: {}", path.display(), err).into());
            }
            let g: Result<glif::Glif, _> = glif::Glif::from_str(&s);
            match g {
                Err(err) => {
                    eprintln!("couldn't parse {}: {}", path.display(), err);
                }
                Ok(g) => {
                    let glyph: Self = g.into();
                    *glyph.metadata.filename.borrow_mut() = filename.clone();
                    *glyph.metadata.glif_source.borrow_mut() = s;
                    let has_components = !glyph.components.is_empty();
                    let glyph = Rc::new(RefCell::new(glyph));
                    if has_components {
                        glyphs_with_refs.push(glyph.clone());
                    }
                    ret.insert(name.into(), glyph);
                }
            }
            path.pop();
        }

        for g in glyphs_with_refs {
            let mut deref = g.borrow_mut();
            for c in deref.components.iter_mut() {
                if let Some(o) = ret.get(&c.base_name) {
                    c.base = Rc::downgrade(o);
                }
            }
        }
        Ok(ret)
    }

    pub fn new(name: &'static str, char: char, curves: Vec<Bezier>) -> Self {
        let contours = if curves.is_empty() {
            vec![]
        } else {
            vec![Contour::new_with_curves(curves)]
        };
        let metadata = GlyphMetadata::new();
        *metadata.kinds.borrow_mut() = (GlyphKind::Char(char), vec![]);
        *metadata.name.borrow_mut() = name.into();
        Self {
            contours,
            components: vec![],
            guidelines: vec![],
            lib: IndexMap::default(),
            metadata,
        }
    }

    pub fn guidelines(&self) -> &[Guideline] {
        self.guidelines.as_slice()
    }

    pub fn add_guideline(&mut self, g: Guideline) {
        self.metadata.link(&g);
        self.guidelines.push(g);
    }

    pub fn remove_guideline(&mut self, g: Either<&Guideline, usize>) {
        let i = match g {
            Either::A(g) => {
                if let Some(i) = self.guidelines.iter().position(|el| el == g) {
                    i
                } else {
                    return;
                }
            }
            Either::B(i) => i,
        };
        self.guidelines.remove(i);
    }

    pub fn pop_guideline(&mut self) {
        self.guidelines.pop();
    }

    pub fn new_empty(name: &'static str, char: char) -> Self {
        Self::new(name, char, vec![])
    }

    pub fn draw(&self, mut cr: ContextRef, options: GlyphDrawingOptions<'_>) {
        if self.is_empty() {
            return;
        }
        let GlyphDrawingOptions {
            outline,
            inner_fill,
            highlight,
            matrix,
            units_per_em: _,
            handle_connection,
            handle,
            corner,
            smooth_corner: _,
            direction_arrow,
            selection,
        } = options;

        let mut cr1 = cr.push();
        cr1.transform(matrix);
        cr1.set_line_width(outline.size);
        cr1.set_source_color_alpha(outline.color);
        let mut pen_position: Option<Point> = None;
        for (_ic, contour) in self.contours.iter().enumerate() {
            let curves = contour.curves();
            if !contour.property::<bool>(Contour::OPEN) {
                if let Some(point) = curves.last().and_then(|b| b.points().last().cloned()) {
                    cr1.move_to(point.x, point.y);
                    pen_position = Some(point.position);
                }
            } else if let Some(point) = curves.first().and_then(|b| b.points().first().cloned()) {
                cr1.move_to(point.x, point.y);
            }

            for (_jc, curv) in curves.iter().enumerate() {
                let degree = curv.degree();
                let degree = if let Some(v) = degree {
                    v
                } else {
                    continue;
                };
                let curv_points = curv.points();
                match degree {
                    0 => { /* Single point */ }
                    1 => {
                        /* Line. */
                        let new_point = curv_points[1].position;
                        cr1.line_to(new_point.x, new_point.y);
                        pen_position = Some(new_point);
                    }
                    2 => {
                        /* Quadratic. */
                        let a = if let Some(v) = pen_position.take() {
                            v
                        } else {
                            curv_points[0].position
                        };
                        let b = curv_points[1].position;
                        let c = curv_points[2].position;
                        cr1.curve_to(
                            (2.0 / 3.0_f64).mul_add(b.x, 1.0 / 3.0 * a.x),
                            (2.0 / 3.0_f64).mul_add(b.y, 1.0 / 3.0 * a.y),
                            (2.0 / 3.0_f64).mul_add(b.x, 1.0 / 3.0 * c.x),
                            (2.0 / 3.0_f64).mul_add(b.y, 1.0 / 3.0 * c.y),
                            c.x,
                            c.y,
                        );
                        pen_position = Some(c);
                    }
                    3 => {
                        /* Cubic */
                        let _a = if let Some(v) = pen_position.take() {
                            v
                        } else {
                            curv_points[0].position
                        };
                        let b = curv_points[1].position;
                        let c = curv_points[2].position;
                        let d = curv_points[3].position;
                        cr1.curve_to(b.x, b.y, c.x, c.y, d.x, d.y);
                        pen_position = Some(d);
                    }
                    d => {
                        eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                        pen_position = Some(curv_points.last().unwrap().position);
                        continue;
                    }
                }
            }
        }

        if let Some(inner_fill) = inner_fill {
            let cr2 = cr1.push();
            cr2.close_path();
            cr2.set_source_color_alpha(inner_fill.color);
            cr2.fill_preserve().expect("Invalid cairo surface state");
        }

        cr1.stroke().expect("Invalid cairo surface state");

        if let Some((degree, curv)) = highlight.and_then(|(contour_idx, curve_idx)| {
            self.contours
                .get(contour_idx)
                .and_then(|contour| contour.curves().get(curve_idx).map(Clone::clone))
                .and_then(|curv| Some((curv.degree()?, curv)))
        }) {
            let curv_points = curv.points();
            cr1.set_source_color(Color::RED);
            let point = curv_points[0].position;
            cr1.move_to(point.x, point.y);
            match degree {
                0 => { /* Single point */ }
                1 => {
                    /* Line. */
                    let new_point = curv_points[1].position;
                    cr1.line_to(new_point.x, new_point.y);
                }
                2 => {
                    /* Quadratic. */
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        curv_points[0].position
                    };
                    let b = curv_points[1].position;
                    let c = curv_points[2].position;
                    cr1.curve_to(
                        (2.0 / 3.0_f64).mul_add(b.x, 1.0 / 3.0 * a.x),
                        (2.0 / 3.0_f64).mul_add(b.y, 1.0 / 3.0 * a.y),
                        (2.0 / 3.0_f64).mul_add(b.x, 1.0 / 3.0 * c.x),
                        (2.0 / 3.0_f64).mul_add(b.y, 1.0 / 3.0 * c.y),
                        c.x,
                        c.y,
                    );
                }
                3 => {
                    /* Cubic */
                    let _a = { curv_points[0].position };
                    cr1.move_to(_a.x, _a.y);
                    let b = curv_points[1].position;
                    let c = curv_points[2].position;
                    let d = curv_points[3].position;
                    cr1.curve_to(b.x, b.y, c.x, c.y, d.x, d.y);
                }
                d => {
                    eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                }
            }
            cr1.stroke().expect("Invalid cairo surface state");
        }
        if let Some(handle) = handle {
            let draw_oncurve = |cr: ContextRef, p: &CurvePoint, cont: Option<Continuity>| {
                if selection.map(|s| s.contains(&p.uuid)).unwrap_or(false) {
                    cr.set_draw_opts((Color::RED, outline.size).into());
                } else if let (Some(opts), true) =
                    (corner, cont.map(Continuity::is_positional).unwrap_or(true))
                {
                    cr.set_draw_opts((opts.color, outline.size).into());
                } else {
                    cr.set_draw_opts((handle.color, outline.size).into());
                }
                match cont.unwrap_or(Continuity::Positional) {
                    Continuity::Positional => {
                        cr.rectangle(
                            p.position.x - handle.size / 2.0,
                            p.position.y - handle.size / 2.0,
                            handle.size,
                            handle.size,
                        );
                    }
                    Continuity::Velocity => {
                        cr.arc(
                            p.position.x,
                            p.position.y,
                            handle.size / 2.0,
                            0.0,
                            2.0 * std::f64::consts::PI,
                        );
                    }
                    Continuity::Tangent { .. } => {
                        cr.move_to(
                            p.position.x - handle.size / 2.0,
                            p.position.y - handle.size / 2.0,
                        );
                        cr.rel_line_to(0.0, handle.size);
                        cr.move_to(p.position.x - handle.size / 2.0, p.position.y);
                        cr.rel_line_to(handle.size, 0.0);
                        cr.close_path();
                    }
                }
                cr.stroke().unwrap();
            };
            let draw_handle = |cr: ContextRef, p: &CurvePoint| {
                if selection.map(|s| s.contains(&p.uuid)).unwrap_or(false) {
                    cr.set_draw_opts((Color::RED, outline.size).into());
                } else {
                    cr.set_draw_opts((handle.color, outline.size).into());
                }
                cr.arc(
                    p.position.x,
                    p.position.y,
                    handle.size / 2.0,
                    0.0,
                    2.0 * std::f64::consts::PI,
                );
                cr.stroke_preserve().unwrap();
                if let Some(bg) = handle.bg {
                    cr.set_source_color_alpha(bg);
                    cr.fill().unwrap();
                }
            };
            let draw_handle_connection = |cr: ContextRef, h: Point, ep: Point| {
                if let Some(opts) = handle_connection {
                    cr.set_draw_opts(opts);
                    cr.move_to(h.x, h.y);
                    cr.line_to(ep.x, ep.y);
                    cr.stroke().unwrap();
                }
            };
            let draw_tangent = |cr: ContextRef, curv: &Bezier| {
                if let Some(opts) = direction_arrow {
                    let (t, p) = curv.emptiest_t(0.3);
                    let tangent = curv.tangent(t);
                    cr.set_source_color_alpha(opts.color);
                    cr.set_line_width(outline.size);
                    cr.translate(p.x, p.y);
                    cr.rotate(tangent.atan2());
                    let h: f64 = opts.size * 3.0;
                    let v: f64 = opts.size * 6.0;

                    cr.move_to(-h, v);

                    cr.line_to(-h, v / 2.0);
                    cr.line_to(-h * 3.0, v / 2.0);
                    cr.line_to(-h * 3.0, -v / 2.0);
                    cr.line_to(-h, -v / 2.0);

                    cr.line_to(-h, -v);
                    cr.line_to(h, 0.0);
                    cr.line_to(-h, v);
                    cr.close_path();
                    cr.stroke().unwrap();
                }
            };
            for contour in self.contours.iter() {
                let curves = contour.curves();
                let biggest = contour.property::<u64>(Contour::BIGGEST_CURVE) as usize;
                for (i, curv) in curves.iter().enumerate() {
                    let degree = curv.degree();
                    let degree = if let Some(v) = degree {
                        v
                    } else {
                        continue;
                    };
                    let curv_points = curv.points();
                    match degree {
                        0 => {
                            /* Single point */
                            draw_oncurve(
                                cr1.push(),
                                &curv_points[0],
                                curv.imp().continuity_in.get(),
                            );
                        }
                        1 => {
                            /* Line. */
                            if i == biggest {
                                draw_tangent(cr1.push(), curv);
                            }
                            draw_oncurve(
                                cr1.push(),
                                &curv_points[0],
                                curv.imp().continuity_in.get(),
                            );
                            draw_oncurve(
                                cr1.push(),
                                &curv_points[1],
                                curv.imp().continuity_out.get(),
                            );
                        }
                        2 => {
                            /* Quadratic. */
                            if i == biggest {
                                draw_tangent(cr1.push(), curv);
                            }
                            let handle = &curv_points[1];
                            let ep1 = &curv_points[0];
                            let ep2 = &curv_points[2];
                            draw_handle_connection(cr1.push(), handle.position, ep1.position);
                            draw_handle_connection(cr1.push(), handle.position, ep2.position);
                            draw_handle(cr1.push(), handle);
                            draw_oncurve(cr1.push(), ep1, curv.imp().continuity_in.get());
                            draw_oncurve(cr1.push(), ep2, curv.imp().continuity_out.get());
                        }
                        3 => {
                            /* Cubic */
                            if i == biggest {
                                draw_tangent(cr1.push(), curv);
                            }
                            let handle1 = &curv_points[1];
                            let handle2 = &curv_points[2];
                            let ep1 = &curv_points[0];
                            let ep2 = &curv_points[3];
                            draw_handle_connection(cr1.push(), handle1.position, ep1.position);
                            draw_handle_connection(cr1.push(), handle2.position, ep2.position);
                            draw_handle(cr1.push(), handle1);
                            draw_handle(cr1.push(), handle2);
                            draw_oncurve(cr1.push(), ep1, curv.imp().continuity_in.get());
                            draw_oncurve(cr1.push(), ep2, curv.imp().continuity_out.get());
                        }
                        d => {
                            eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                            continue;
                        }
                    }
                }
            }
        }
        drop(cr1);
        for component in self.components.iter() {
            if let Some(rc) = component.base.upgrade() {
                let glyph = rc.borrow();
                let crc = cr.push();
                crc.transform(matrix);
                let matrix = Matrix::new(
                    component.x_scale,
                    component.xy_scale,
                    component.yx_scale,
                    component.y_scale,
                    component.x_offset,
                    component.y_offset,
                );
                glyph.draw(
                    crc,
                    GlyphDrawingOptions {
                        matrix,
                        handle: None,
                        corner: None,
                        smooth_corner: None,
                        direction_arrow: None,
                        selection: None,
                        ..options
                    },
                );
            }
        }
    }

    /*
    pub fn into_cubic(&mut self) {
        if self.is_empty() {
            return;
        }
        for contour in self.contours.iter_mut() {
            let mut pen_position: Option<Point> = None;
            let mut curves = contour.imp().curves.borrow_mut();
            if !contour.property::<bool>(Contour::OPEN) {
                if let Some(point) = curves.last().and_then(|b| b.points().last().cloned()) {
                    pen_position = Some(point.position);
                }
            }

            for curv in curves.iter_mut() {
                let curv_points = curv.points();
                if curv_points.len() == 3 {
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        curv_points[0].position
                    };
                    let b = curv_points[1].position;
                    let c = curv_points[2].position;
                    let new_points = vec![
                        a,
                        (
                            2.0 / 3.0 * b.x + 1.0 / 3.0 * a.x,
                            2.0 / 3.0 * b.y + 1.0 / 3.0 * a.y,
                        )
                            .into(),
                        (
                            2.0 / 3.0 * b.x + 1.0 / 3.0 * c.x,
                            2.0 / 3.0 * b.y + 1.0 / 3.0 * c.y,
                        )
                            .into(),
                        c,
                    ];
                    drop(curv_points);
                    *curv = Bezier::new(new_points);
                    pen_position = Some(c);
                } else if let Some(last_p) = curv_points.last() {
                    pen_position = Some(last_p.position);
                }
            }
        }
    }
    */

    pub fn save_to_svg<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let surface = gtk::cairo::SvgSurface::new(
            self.width().unwrap_or(ufo::constants::UNITS_PER_EM),
            ufo::constants::UNITS_PER_EM,
            Some(path),
        )?;
        let ctx = gtk::cairo::Context::new(&surface)?;

        let options = GlyphDrawingOptions {
            matrix: Matrix::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0),
            ..Default::default()
        };
        self.draw((&ctx).push(), options);
        surface.flush();
        surface.finish();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        (self.contours.is_empty() || self.contours.iter().all(|c| c.curves().is_empty()))
            && self.components.is_empty()
    }

    pub fn name_markup(&self) -> gtk::glib::GString {
        match self.kinds().0 {
            GlyphKind::Char(c) => {
                let mut b = [0; 4];
                gtk::glib::markup_escape_text(c.encode_utf8(&mut b).replace('\0', "").trim())
            }
            GlyphKind::Component(ref name) => {
                gtk::glib::markup_escape_text(name.as_str().replace('\0', "").trim())
            }
        }
    }

    pub fn on_curve_query(
        &self,
        position: Point,
        pts: &[GlyphPointIndex],
    ) -> Option<((usize, usize), Bezier)> {
        for (ic, contour) in self.contours.iter().enumerate() {
            for (jc, curve) in contour.curves().iter().enumerate() {
                if curve.on_curve_query(position, None) {
                    return Some(((ic, jc), curve.clone()));
                }
                for GlyphPointIndex {
                    contour_index,
                    curve_index,
                    uuid,
                } in pts
                {
                    if (*contour_index, *curve_index) != (ic, jc) {
                        continue;
                    }
                    if curve.points().iter().any(|cp| cp.uuid == *uuid) {
                        return Some(((ic, jc), curve.clone()));
                    }
                }
            }
        }
        None
    }

    pub fn save(&self, prefix: &Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let glif: glif::Glif = self.into();
        let path = prefix.join(&*self.filename());
        let mut file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(glif.to_xml().as_bytes())?;
        for g in self.guidelines.iter().filter(|obj| obj.modified()) {
            g.set_property(Guideline::MODIFIED, false);
        }
        Ok(())
    }
}

impl From<glif::Glif> for Glyph {
    fn from(val: glif::Glif) -> Self {
        use glif::{Component, OutlineEntry, Point, PointKind};
        let glif::Glif {
            name,
            outline,
            advance,
            image,
            anchors,
            guidelines,
            unicode,
            format: _,
            lib,
        } = val;

        let kinds = if unicode.is_empty() {
            (GlyphKind::Component(name.clone()), vec![])
        } else {
            let mut iter = unicode
                .iter()
                .filter_map(|unicode| u32::from_str_radix(unicode.hex(), 16).ok())
                .filter_map(|n| n.try_into().ok())
                .map(GlyphKind::Char);
            let first = iter.next().unwrap();
            (first, iter.collect::<Vec<_>>())
        };
        let mut ret = Self {
            guidelines: guidelines
                .into_iter()
                .map(|g| {
                    Guideline::builder()
                        .name(g.name)
                        .identifier(g.identifier)
                        .color(g.color)
                        .angle(g.angle)
                        .x(g.x)
                        .y(g.y)
                        .build()
                })
                .collect::<Vec<_>>(),
            lib,
            ..Self::default()
        };
        *ret.metadata.name.borrow_mut() = name;
        *ret.metadata.kinds.borrow_mut() = kinds;
        *ret.metadata.unicode.borrow_mut() = unicode;
        *ret.metadata.anchors.borrow_mut() = anchors;
        *ret.metadata.image.borrow_mut() = image;
        ret.metadata.advance.set(advance);
        ret.metadata.width.set(advance.map(|a| a.width));

        if let Some(outline) = outline {
            for contour in outline.contours {
                let contour = match contour {
                    OutlineEntry::Contour(c) => c,
                    OutlineEntry::Component(Component {
                        base,
                        x_offset,
                        y_offset,
                        x_scale,
                        xy_scale,
                        yx_scale,
                        y_scale,
                    }) => {
                        ret.components.push(crate::glyphs::Component {
                            base_name: base,
                            base: std::rc::Weak::new(),
                            x_offset,
                            y_offset,
                            x_scale,
                            xy_scale,
                            yx_scale,
                            y_scale,
                        });
                        continue;
                    }
                };

                let mut open = false;
                let mut points = contour
                    .point
                    .iter()
                    .collect::<std::collections::VecDeque<&_>>();
                if points.is_empty() {
                    continue;
                }
                let mut c;
                let mut prev_point;
                let mut last_oncurve;
                if points.front().unwrap().is_move() {
                    open = true;
                    // Open contour
                    let p = points.pop_front().unwrap();
                    prev_point = (p.x, p.y);
                    last_oncurve = prev_point;
                    c = vec![prev_point];
                } else {
                    c = vec![];
                    let first_point = points.front().unwrap();
                    last_oncurve = (first_point.x, first_point.y);
                    // Closed contour
                    while points.front().unwrap().is_curve() {
                        points.rotate_left(1);
                    }
                    let last_point = points.back().unwrap();
                    prev_point = (last_point.x, last_point.y);
                }
                if points.front().unwrap().is_line() {
                    let p = points.back().unwrap();
                    prev_point = (p.x, p.y);
                }
                let super_ = crate::glyphs::Contour::new();
                loop {
                    match points.pop_front() {
                        Some(Point {
                            type_: PointKind::Move,
                            ..
                        }) => {
                            panic!() // [ref:FIXME] return Err
                        }
                        Some(Point {
                            type_: PointKind::Offcurve,
                            x,
                            y,
                            ..
                        }) => {
                            prev_point = (*x, *y);
                            c.push(prev_point);
                        }
                        Some(Point {
                            type_: PointKind::Curve,
                            x,
                            y,
                            smooth,
                            ..
                        }) => {
                            prev_point = (*x, *y);
                            c.push(prev_point);
                            c.insert(0, last_oncurve);
                            let curv = Bezier::new(c.into_iter().map(Into::into).collect());
                            if *smooth == Some(true) {
                                curv.set_property(Bezier::SMOOTH, true);
                            }
                            super_.push_curve(curv);
                            c = vec![];
                            last_oncurve = prev_point;
                        }
                        Some(Point {
                            type_: PointKind::Line,
                            x,
                            y,
                            ..
                        }) => {
                            assert!(c.is_empty() || c.len() == 1);
                            if c.is_empty() {
                                c.push(prev_point);
                            }
                            c.push((*x, *y));
                            super_.push_curve(Bezier::new(c.into_iter().map(Into::into).collect()));
                            c = vec![];
                            prev_point = (*x, *y);
                            last_oncurve = prev_point;
                        }
                        Some(Point {
                            type_: PointKind::Qcurve,
                            ..
                        }) => {
                            todo!()
                        }
                        None => {
                            if !c.is_empty() {
                                if !c.contains(&prev_point) {
                                    c.push(prev_point);
                                }
                                super_.push_curve(Bezier::new(
                                    c.into_iter().map(Into::into).collect(),
                                ));
                            }
                            break;
                        }
                    }
                }
                if !open {
                    super_.close();
                }
                super_.is_contour_modified.set(true);
                ret.contours.push(super_);
            }
        }

        ret
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Default, Copy)]
#[repr(C)]
pub struct GlyphPointIndex {
    pub contour_index: usize,
    pub curve_index: usize,
    pub uuid: Uuid,
}

impl GlyphPointIndex {
    const U: usize = std::mem::size_of::<Uuid>();
    const USZ: usize = std::mem::size_of::<usize>();
    const N: usize = Self::USZ * 2 + Self::U;

    pub fn as_bytes(&self) -> [u8; Self::N] {
        let mut ret: [u8; Self::N] = [0; Self::N];
        ret[..Self::USZ].copy_from_slice(&self.contour_index.to_le_bytes());
        ret[Self::USZ..(Self::USZ * 2)].copy_from_slice(&self.curve_index.to_le_bytes());
        ret[(Self::USZ * 2)..].copy_from_slice(self.uuid.as_bytes().as_slice());
        ret
    }
}
