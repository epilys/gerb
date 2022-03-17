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

use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::unicode::names::CharName;
use crate::utils::curves::*;

use gtk::cairo::{Context, Matrix};

#[derive(Debug, Default, Clone)]
pub struct Guideline {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub color: Option<String>,
    pub angle: f64,
    pub x: i64,
    pub y: i64,
}

impl Guideline {
    pub fn draw(
        &self,
        cr: &Context,
        matrix: Matrix,
        (_width, height): (f64, f64),
        highlight: bool,
    ) {
        fn move_point(p: (f64, f64), d: f64, r: f64) -> (f64, f64) {
            let (x, y) = p;
            (x + (d * f64::cos(r)), y + (d * f64::sin(r)))
        }
        cr.save().unwrap();
        if highlight {
            cr.set_source_rgba(1., 0., 0., 0.8);
            cr.set_line_width(2.0);
        } else {
            cr.set_source_rgba(0., 0., 1., 0.8);
            cr.set_line_width(1.5);
        }
        let p = matrix.transform_point(self.x as f64, self.y as f64);
        let r = self.angle * 0.01745;
        let top = move_point(p, height * 10., r);
        cr.move_to(top.0, top.1);
        let bottom = move_point(p, -height * 10., r);
        cr.line_to(bottom.0, bottom.1);
        cr.stroke().unwrap();
        cr.restore().unwrap();
    }

    pub fn distance_from_point(&self, (xp, yp): Point) -> f64 {
        // Using an 𝐿 defined by a point 𝑃𝑙 and angle 𝜃
        //𝑑 = ∣cos(𝜃)(𝑃𝑙𝑦 − 𝑦𝑝) − sin(𝜃)(𝑃𝑙𝑥 − 𝑃𝑥)∣
        let r = -self.angle * 0.01745;
        let sin = f64::sin(r);
        let cos = f64::cos(r);
        (cos * (self.y - yp) as f64 - sin * (self.x - xp) as f64).abs()
    }

    pub fn on_line_query(&self, point: Point, error: Option<f64>) -> bool {
        let error = error.unwrap_or(5.0);
        self.distance_from_point(point) <= error
    }
}

#[derive(Debug, Clone)]
pub struct Contour {
    pub open: bool,
    pub curves: Vec<Bezier>,
}

#[derive(Debug, Clone)]
pub struct Component {
    base_name: String,
    base: Weak<RefCell<Glyph>>,
    x_offset: f64,
    y_offset: f64,
    x_scale: f64,
    xy_scale: f64,
    yx_scale: f64,
    y_scale: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlyphKind {
    Char(char),
    Component,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub name: Cow<'static, str>,
    pub name2: Option<crate::unicode::names::Name>,
    pub kind: GlyphKind,
    pub width: Option<f64>,
    pub contours: Vec<Contour>,
    pub components: Vec<Component>,
    pub guidelines: Vec<Guideline>,
    pub glif_source: String,
}

impl Ord for Glyph {
    fn cmp(&self, other: &Self) -> Ordering {
        use GlyphKind::*;
        match (&self.kind, &other.kind) {
            (Char(s), Char(o)) => s.cmp(o),
            (Char(_), _) => Ordering::Less,
            (Component, Component) => self.name.cmp(&other.name),
            (Component, Char(_)) => Ordering::Greater,
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
        match (&self.kind, &other.kind) {
            (Char(s), Char(o)) => s == o,
            (Char(_), Component) | (Component, Char(_)) => false,
            (Component, Component) => self.name == other.name,
        }
    }
}

impl Eq for Glyph {}

impl Default for Glyph {
    fn default() -> Self {
        Glyph::new_empty("space", ' ')
        /*
        let curves = vec![
            Bezier::new(vec![(54, 72), (55, 298)]),
            Bezier::new(vec![(27, 328), (61, 333), (55, 299)]),
            Bezier::new(vec![(26, 328), (27, 338)]),
            Bezier::new(vec![(27, 339), (124, 339)]),
            Bezier::new(vec![(98, 306), (97, 209)]),
            Bezier::new(vec![(97, 301), (98, 334), (123, 330)]),
            Bezier::new(vec![(123, 330), (124, 337)]),
            Bezier::new(vec![(12, 53), (54, 55), (53, 72)]),
            Bezier::new(vec![(11, 52), (174, 53)]),
            Bezier::new(vec![(174, 55), (251, 63), (266, 124)]),
            Bezier::new(vec![(183, 192), (265, 182), (266, 127)]),
            Bezier::new(vec![(100, 180), (101, 78)]),
            Bezier::new(vec![(100, 79), (125, 78)]),
            Bezier::new(vec![(126, 79), (209, 67), (216, 120)]),
            Bezier::new(vec![(136, 177), (217, 178), (218, 122)]),
            Bezier::new(vec![(105, 176), (135, 176)]),
            Bezier::new(vec![(96, 209), (138, 209)]),
            Bezier::new(vec![(140, 210), (183, 201), (203, 243)]),
            Bezier::new(vec![(205, 245), (215, 296), (241, 327)]),
            Bezier::new(vec![(187, 192), (244, 197), (252, 237)]),
            Bezier::new(vec![(253, 241), (263, 304), (290, 317)]),
            Bezier::new(vec![(241, 327), (287, 359), (339, 301)]),
            Bezier::new(vec![(292, 317), (316, 318), (332, 294)]),
            Bezier::new(vec![(335, 295), (339, 303)]),
        ];
        Glyph {
            name: "R".into(),
            char: 'R',
            curves,
        }
        */
        /*
        let outline = vec![
            vec![
                ContourPoint::Curve(Bezier::new(vec![(201, 11), (252, -12), (297, -12)]));
        ContourPointBezier::new(vec![(408, -12), (507, 85), (507, 251)])),
            <point x="507" y="401"/>
            <point x="440" y="498"/>
            <point x="314" y="498" type="curve" smooth="yes"/>
            <point x="260" y="498"/>
            <point x="206" y="469"/>
            <point x="162" y="430" type="curve"/>
            <point x="164" y="518" type="line"/>
            <point x="164" y="712" type="line"/>
            <point x="82" y="712" type="line"/>
            <point x="82" y="0" type="line"/>
            <point x="148" y="0" type="line"/>
            <point x="155" y="50" type="line"/>
            <point x="158" y="50" type="line"/>
            ],
            vec![
            <point x="283" y="57" type="curve" smooth="yes"/>
            <point x="251" y="57"/>
            <point x="207" y="71"/>
            <point x="164" y="108" type="curve"/>
            <point x="164" y="363" type="line"/>
            <point x="210" y="406"/>
            <point x="253" y="429"/>
            <point x="294" y="429" type="curve" smooth="yes"/>
            <point x="386" y="429"/>
            <point x="422" y="357"/>
            <point x="422" y="250" type="curve" smooth="yes"/>
            <point x="422" y="130"/>
            <point x="363" y="57"/>
            ],
        ];

        Glyph {
            name: "b",
            char: 'b',
            outline,
        }
        */
    }
}

#[derive(Clone, Copy)]
pub struct GlyphDrawingOptions {
    pub outline: (f64, f64, f64, f64),
    pub inner_fill: Option<(f64, f64, f64, f64)>,
    pub highlight: Option<(usize, usize)>,
    pub matrix: Matrix,
    pub units_per_em: f64,
}

impl Default for GlyphDrawingOptions {
    fn default() -> Self {
        Self {
            outline: (1., 1., 1., 1.),
            inner_fill: None,
            highlight: None,
            matrix: Matrix::identity(),
            units_per_em: 1000.,
        }
    }
}

impl Glyph {
    pub fn from_ufo(
        path: &str,
    ) -> Result<HashMap<String, Rc<RefCell<Glyph>>>, Box<dyn std::error::Error>> {
        use std::path::Path;

        //assert!(path.ends_with(".ufo"));
        let mut ret: HashMap<String, Rc<RefCell<Glyph>>> = HashMap::default();

        let mut glyphs_with_refs: Vec<Rc<_>> = vec![];
        let path = Path::new(path);
        let path = path.join("glyphs");

        for entry in path
            .read_dir()
            .map_err(|err| format!("Reading directory {} failed: {}", path.display(), err))?
            .flatten()
        {
            use std::fs::File;
            use std::io::prelude::*;
            let mut file = match File::open(&entry.path()) {
                Err(err) => {
                    return Err(format!("Couldn't open {}: {}", entry.path().display(), err).into())
                }
                Ok(file) => file,
            };

            let mut s = String::new();
            if let Err(err) = file.read_to_string(&mut s) {
                return Err(format!("Couldn't read {}: {}", entry.path().display(), err).into());
            }
            let g: Result<glif::Glif, _> = glif::Glif::from_str(&s);
            match g {
                Err(err) => {
                    eprintln!("couldn't parse {}: {}", entry.path().display(), err);
                }
                Ok(g) => {
                    for mut g in g.into_iter() {
                        g.glif_source = s.clone();
                        let has_components = !g.components.is_empty();
                        let name = g.name.clone();
                        let g = Rc::new(RefCell::new(g));
                        if has_components {
                            glyphs_with_refs.push(g.clone());
                        }
                        ret.insert(name.into(), g);
                    }
                }
            }
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
        Glyph {
            name: name.into(),
            name2: char.char_name(),
            kind: GlyphKind::Char(char),
            contours: vec![Contour {
                open: false,
                curves,
            }],
            components: vec![],
            guidelines: vec![],
            width: None,
            glif_source: String::new(),
        }
    }

    pub fn new_empty(name: &'static str, char: char) -> Self {
        Glyph::new(name, char, vec![])
    }

    pub fn draw(&self, cr: &Context, options: GlyphDrawingOptions) {
        if self.is_empty() {
            return;
        }
        let GlyphDrawingOptions {
            outline,
            inner_fill,
            highlight,
            matrix,
            units_per_em,
        } = options;

        cr.save().expect("Invalid cairo surface state");
        cr.set_line_width(4.0);
        cr.transform(matrix);
        cr.transform(Matrix::new(1.0, 0., 0., -1.0, 0., units_per_em.abs()));
        cr.set_source_rgba(outline.0, outline.1, outline.2, outline.3);
        let p_fn = |p: (i64, i64)| -> (f64, f64) { (p.0 as f64, p.1 as f64) };
        let mut pen_position: Option<(f64, f64)> = None;
        for (_ic, contour) in self.contours.iter().enumerate() {
            if !contour.open {
                if let Some(point) = contour.curves.last().and_then(|b| b.points.last()) {
                    let point = p_fn(*point);
                    cr.move_to(point.0, point.1);
                    pen_position = Some(point);
                }
            } else if let Some(point) = contour.curves.first().and_then(|b| b.points.first()) {
                let point = p_fn(*point);
                cr.move_to(point.0, point.1);
            }

            for (_jc, curv) in contour.curves.iter().enumerate() {
                if !curv.smooth {
                    //cr.stroke().expect("Invalid cairo surface state");
                }
                let degree = curv.degree();
                let degree = if let Some(v) = degree {
                    v
                } else {
                    continue;
                };
                match degree {
                    1 => {
                        /* Line. */
                        let new_point = p_fn(curv.points[1]);
                        cr.line_to(new_point.0, new_point.1);
                        pen_position = Some(new_point);
                    }
                    2 => {
                        /* Quadratic. */
                        let a = if let Some(v) = pen_position.take() {
                            v
                        } else {
                            p_fn(curv.points[0])
                        };
                        let b = p_fn(curv.points[1]);
                        let c = p_fn(curv.points[2]);
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
                            p_fn(curv.points[0])
                        };
                        let b = p_fn(curv.points[1]);
                        let c = p_fn(curv.points[2]);
                        let d = p_fn(curv.points[3]);
                        cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                        pen_position = Some(d);
                    }
                    d => {
                        eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                        pen_position = Some(p_fn(*curv.points.last().unwrap()));
                        continue;
                    }
                }
            }
        }

        if let Some(inner_fill) = inner_fill {
            cr.save().unwrap();
            cr.close_path();
            cr.set_source_rgba(inner_fill.0, inner_fill.1, inner_fill.2, inner_fill.3);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.restore().expect("Invalid cairo surface state");
        }

        cr.stroke().expect("Invalid cairo surface state");

        for curv in highlight
            .and_then(|(contour_idx, curve_idx)| {
                self.contours
                    .get(contour_idx)
                    .map(|contour| &contour.curves[curve_idx])
            })
            .into_iter()
        {
            cr.set_source_rgba(1.0, 0., 0., 1.0);
            let degree = curv.degree();
            let degree = if let Some(v) = degree {
                v
            } else {
                continue;
            };
            let point = p_fn(curv.points[0]);
            cr.move_to(point.0, point.1);
            match degree {
                1 => {
                    /* Line. */
                    let new_point = p_fn(curv.points[1]);
                    cr.line_to(new_point.0, new_point.1);
                    pen_position = Some(new_point);
                }
                2 => {
                    /* Quadratic. */
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        p_fn(curv.points[0])
                    };
                    let b = p_fn(curv.points[1]);
                    let c = p_fn(curv.points[2]);
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
                        p_fn(curv.points[0])
                    };
                    let b = p_fn(curv.points[1]);
                    let c = p_fn(curv.points[2]);
                    let d = p_fn(curv.points[3]);
                    cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                    pen_position = Some(d);
                }
                d => {
                    eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                    pen_position = Some(p_fn(*curv.points.last().unwrap()));
                    continue;
                }
            }
            cr.stroke().expect("Invalid cairo surface state");
        }
        cr.restore().expect("Invalid cairo surface state");
        for component in self.components.iter() {
            if let Some(rc) = component.base.upgrade() {
                let glyph = rc.borrow();
                cr.save().unwrap();
                cr.transform(matrix);
                let matrix = Matrix::new(
                    component.x_scale,
                    component.xy_scale,
                    component.yx_scale,
                    component.y_scale,
                    component.x_offset,
                    -component.y_offset,
                );
                glyph.draw(cr, GlyphDrawingOptions { matrix, ..options });
                cr.restore().expect("Invalid cairo surface state");
            }
        }
    }

    pub fn into_cubic(&mut self) {
        if self.is_empty() {
            return;
        }
        let f_fn = |(x, y): (i64, i64)| -> (f64, f64) { (x as f64, y as f64) };
        let i_fn = |(x, y): (f64, f64)| -> (i64, i64) { (x as i64, y as i64) };
        for contour in self.contours.iter_mut() {
            let mut pen_position: Option<(f64, f64)> = None;
            if !contour.open {
                if let Some(point) = contour.curves.last().and_then(|b| b.points.last()) {
                    pen_position = Some(f_fn(*point));
                }
            }

            for curv in contour.curves.iter_mut() {
                if curv.points.len() == 3 {
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        f_fn(curv.points[0])
                    };
                    let b = f_fn(curv.points[1]);
                    let c = f_fn(curv.points[2]);
                    let new_points = vec![
                        i_fn(a),
                        i_fn((
                            2.0 / 3.0 * b.0 + 1.0 / 3.0 * a.0,
                            2.0 / 3.0 * b.1 + 1.0 / 3.0 * a.1,
                        )),
                        i_fn((
                            2.0 / 3.0 * b.0 + 1.0 / 3.0 * c.0,
                            2.0 / 3.0 * b.1 + 1.0 / 3.0 * c.1,
                        )),
                        i_fn((c.0, c.1)),
                    ];
                    *curv = Bezier::new(curv.smooth, new_points);
                    pen_position = Some(c);
                } else {
                    if let Some(last_p) = curv.points.last() {
                        pen_position = Some(f_fn(*last_p));
                    }
                }
            }
        }
    }

    #[cfg(feature = "svg")]
    pub fn save_to_svg<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let surface = gtk::cairo::SvgSurface::new(self.width.unwrap_or(500.0), 1000., Some(path))?;
        let ctx = gtk::cairo::Context::new(&surface)?;

        let options = GlyphDrawingOptions {
            outline: (0., 0., 0., 1.),
            inner_fill: None,
            highlight: None,
            ..Default::default()
        };
        self.draw(&ctx, options);
        surface.flush();
        surface.finish();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        (self.contours.is_empty() || self.contours.iter().all(|c| c.curves.is_empty()))
            && self.components.is_empty()
    }

    pub fn name_markup(&self) -> gtk::glib::GString {
        match self.kind {
            GlyphKind::Char(c) => {
                let mut b = [0; 4];
                gtk::glib::markup_escape_text(c.encode_utf8(&mut b).replace('\0', "").trim())
            }
            GlyphKind::Component => {
                gtk::glib::markup_escape_text(self.name.as_ref().replace('\0', "").trim())
            }
        }
    }

    /*
    pub fn points(&self) -> Vec<Point> {
        self.contours
            .clone()
            .into_iter()
            .map(|v| v.curves.into_iter().map(|b| b.points.into_iter()).flatten())
            .flatten()
            .collect::<Vec<Point>>()
    }
    */
}

mod glif {
    extern crate quick_xml;
    extern crate serde;

    use crate::unicode::names::CharName;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum PointKind {
        /// A point of this type MUST be the first in a contour. The reverse is not true: a contour does not necessarily start with a move point. When a contour does start with a move point, it signifies the beginning of an open contour. A closed contour does not start with a move and is defined as a cyclic list of points, with no predominant start point. There is always a next point and a previous point. For this purpose the list of points can be seen as endless in both directions. The actual list of points can be rotated arbitrarily (by removing the first N points and appending them at the end) while still describing the same outline.
        Move,
        /// Draw a straight line from the previous point to this point. The previous point may be a move, a line, a curve or a qcurve, but not an offcurve.
        Line,
        /// This point is part of a curve segment, that goes up to the next point that is either a curve or a qcurve.
        Offcurve,
        /// Draw a cubic bezier curve from the last non-offcurve point to this point. If the number of offcurve points is zero, a straight line is drawn. If it is one, a quadratic curve is drawn. If it is two, a regular cubic bezier is drawn. If it is larger than 2, a series of cubic bezier segments are drawn, as defined by the Super Bezier algorithm.
        Curve,
        /// Similar to curve, but uses quadratic curves, using the TrueType “implied on-curve points” principle.
        Qcurve,
    }

    impl Default for PointKind {
        fn default() -> Self {
            PointKind::Offcurve
        }
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Point {
        x: i64,
        y: i64,
        #[serde(rename = "type", default)]
        type_: PointKind,
        smooth: Option<String>,
    }

    impl Point {
        #[inline(always)]
        fn is_curve(&self) -> bool {
            matches!(self.type_, PointKind::Curve)
        }

        /*
        #[inline(always)]
        fn is_offcurve(&self) -> bool {
            if let PointKind::Offcurve = self.type_ {
                true
            } else {
                false
            }
        }
        */

        #[inline(always)]
        fn is_move(&self) -> bool {
            matches!(self.type_, PointKind::Move)
        }

        #[inline(always)]
        fn is_line(&self) -> bool {
            matches!(self.type_, PointKind::Line)
        }
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Contour {
        #[serde(rename = "$value", default)]
        point: Vec<Point>,
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct Component {
        base: String,
        #[serde(default)]
        x_offset: f64,
        #[serde(default)]
        y_offset: f64,
        #[serde(default = "one_fn")]
        x_scale: f64,
        #[serde(default)]
        xy_scale: f64,
        #[serde(default)]
        yx_scale: f64,
        #[serde(default = "one_fn")]
        y_scale: f64,
    }

    const fn one_fn() -> f64 {
        1.0
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum OutlineEntry {
        Contour(Contour),
        Component(Component),
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Anchor {
        name: String,
        x: f64,
        y: f64,
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Guideline {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        identifier: Option<String>,
        #[serde(default)]
        color: Option<String>,
        #[serde(default)]
        angle: f64,
        #[serde(default)]
        x: f64,
        #[serde(default)]
        y: f64,
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Outline {
        #[serde(rename = "$value", default)]
        countours: Vec<OutlineEntry>,
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Unicode {
        hex: String,
    }

    #[derive(Debug, Clone, Deserialize, PartialEq, Default)]
    struct Advance {
        width: f64,
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    pub struct Glif {
        name: String,
        format: Option<String>,
        #[serde(default)]
        unicode: Vec<Unicode>,
        #[serde(default)]
        advance: Option<Advance>,
        #[serde(default)]
        outline: Option<Outline>,
        #[serde(rename = "anchor", default)]
        anchors: Vec<Anchor>,
        #[serde(rename = "guideline", default)]
        guidelines: Vec<Guideline>,
    }

    pub struct GlifIterator {
        glif: Glif,
        kinds: Vec<(super::GlyphKind, Option<crate::unicode::names::Name>)>,
    }

    impl IntoIterator for Glif {
        type Item = super::Glyph;
        type IntoIter = GlifIterator;

        fn into_iter(mut self) -> Self::IntoIter {
            let unicodes = std::mem::take(&mut self.unicode);

            let kinds = if unicodes.is_empty() {
                vec![(super::GlyphKind::Component, None)]
            } else {
                unicodes
                    .into_iter()
                    .filter_map(|unicode| u32::from_str_radix(unicode.hex.as_str(), 16).ok())
                    .filter_map(|n| n.try_into().ok())
                    .map(|val| (super::GlyphKind::Char(val), val.char_name()))
                    .collect::<Vec<_>>()
            };
            GlifIterator { glif: self, kinds }
        }
    }

    impl Iterator for GlifIterator {
        type Item = super::Glyph;

        fn next(&mut self) -> Option<Self::Item> {
            use super::{Bezier, Glyph};
            let (kind, name2) = self.kinds.pop()?;
            let Glif {
                name,
                outline,
                advance,
                anchors: _,
                guidelines,
                ..
            } = self.glif.clone();
            let mut ret = Glyph {
                name: name.into(),
                name2,
                kind,
                width: advance.map(|a| a.width),
                contours: vec![],
                components: vec![],
                guidelines: guidelines
                    .into_iter()
                    .map(|g| super::Guideline {
                        name: g.name,
                        identifier: g.identifier,
                        color: g.color,
                        angle: g.angle,
                        x: g.x as i64,
                        y: g.y as i64,
                    })
                    .collect::<Vec<_>>(),
                glif_source: String::new(),
            };

            if let Some(outline) = outline {
                for contour in outline.countours {
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
                            ret.components.push(super::Component {
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

                    let mut contour_acc = vec![];
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
                        // Closed contour
                        while points.front().unwrap().is_curve() {
                            points.rotate_left(1);
                        }
                        let last_point = points.back().unwrap();
                        prev_point = (last_point.x, last_point.y);
                        let first_point = points.front().unwrap();
                        last_oncurve = (first_point.x, first_point.y);
                    }
                    if points.front().unwrap().is_line() {
                        let p = points.back().unwrap();
                        prev_point = (p.x, p.y);
                    }
                    loop {
                        match points.pop_front() {
                            Some(Point {
                                type_: PointKind::Move,
                                ..
                            }) => {
                                panic!() // FIXME return Err
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
                                let smooth = smooth.as_ref().map(|s| s == "yes").unwrap_or(false);
                                contour_acc.push(Bezier::new(smooth, c));
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
                                contour_acc.push(Bezier::new(false, c));
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
                                    contour_acc.push(Bezier::new(false, c));
                                }
                                break;
                            }
                        }
                    }
                    ret.contours.push(super::Contour {
                        open,
                        curves: contour_acc,
                    });
                }
            }

            Some(ret)
        }
    }

    impl Glif {
        pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let g: Glif = quick_xml::de::from_str(s)?;
            Ok(g)
        }
    }

    impl Default for Glif {
        fn default() -> Self {
            let g: Glif = quick_xml::de::from_str(_LOWERCASE_B_GLIF).unwrap();
            g
        }
    }

    #[test]
    fn test_glif_parse() {
        let g: Glif = quick_xml::de::from_str(_UPPERCASE_A_GLIF).unwrap();
        println!("{:#?}", g);
        let g: super::Glyph = g.into();
        println!("\n\n{:#?}", g);
    }

    const _LOWERCASE_B_GLIF: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="b" format="2">
	<unicode hex="0062"/>
	<advance width="553"/>
	<outline>
		<contour>
			<point x="297" y="-12" type="curve" smooth="yes"/>
			<point x="408" y="-12"/>
			<point x="507" y="85"/>
			<point x="507" y="251" type="curve" smooth="yes"/>
			<point x="507" y="401"/>
			<point x="440" y="498"/>
			<point x="314" y="498" type="curve" smooth="yes"/>
			<point x="260" y="498"/>
			<point x="206" y="469"/>
			<point x="162" y="430" type="curve"/>
			<point x="164" y="518" type="line"/>
			<point x="164" y="712" type="line"/>
			<point x="82" y="712" type="line"/>
			<point x="82" y="0" type="line"/>
			<point x="148" y="0" type="line"/>
			<point x="155" y="50" type="line"/>
			<point x="158" y="50" type="line"/>
			<point x="201" y="11"/>
			<point x="252" y="-12"/>
		</contour>
		<contour>
			<point x="283" y="57" type="curve" smooth="yes"/>
			<point x="251" y="57"/>
			<point x="207" y="71"/>
			<point x="164" y="108" type="curve"/>
			<point x="164" y="363" type="line"/>
			<point x="210" y="406"/>
			<point x="253" y="429"/>
			<point x="294" y="429" type="curve" smooth="yes"/>
			<point x="386" y="429"/>
			<point x="422" y="357"/>
			<point x="422" y="250" type="curve" smooth="yes"/>
			<point x="422" y="130"/>
			<point x="363" y="57"/>
		</contour>
	</outline>
	<anchor name="aboveUC" x="295" y="728"/>
	<anchor name="belowLC" x="296" y="-22"/>
	<anchor name="center" x="125" y="593"/>
</glyph>"##;

    const _UPPERCASE_A_GLIF: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="A" format="2">
	<unicode hex="0041"/>
	<advance width="544"/>
	<outline>
		<contour>
			<point x="3" y="0" type="line"/>
			<point x="88" y="0" type="line"/>
			<point x="203" y="367" type="line" smooth="yes"/>
			<point x="227" y="440"/>
			<point x="248" y="512"/>
			<point x="268" y="588" type="curve"/>
			<point x="272" y="588" type="line"/>
			<point x="293" y="512"/>
			<point x="314" y="440"/>
			<point x="338" y="367" type="curve" smooth="yes"/>
			<point x="452" y="0" type="line"/>
			<point x="541" y="0" type="line"/>
			<point x="319" y="656" type="line"/>
			<point x="225" y="656" type="line"/>
		</contour>
		<contour>
			<point x="119" y="200" type="line"/>
			<point x="422" y="200" type="line"/>
			<point x="422" y="267" type="line"/>
			<point x="119" y="267" type="line"/>
		</contour>
	</outline>
	<anchor name="aboveUC" x="271" y="678"/>
	<anchor name="belowLC" x="271" y="-22"/>
	<anchor name="ogonekUC" x="483" y="0"/>
</glyph>"##;
}
