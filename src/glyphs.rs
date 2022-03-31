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

use gtk::subclass::prelude::*;

mod guidelines;
pub use guidelines::*;

mod glif;

mod contours;
pub use contours::*;

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
        let contour = Contour::new();
        *contour.imp().curves.borrow_mut() = curves;
        Glyph {
            name: name.into(),
            name2: char.char_name(),
            kind: GlyphKind::Char(char),
            contours: vec![contour],
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
            let curves = contour.imp().curves.borrow();
            if !*contour.imp().open.borrow() {
                if let Some(point) = curves
                    .last()
                    .and_then(|b| b.points().borrow().last().copied())
                {
                    let point = p_fn(point);
                    cr.move_to(point.0, point.1);
                    pen_position = Some(point);
                }
            } else if let Some(point) = curves
                .first()
                .and_then(|b| b.points().borrow().first().copied())
            {
                let point = p_fn(point);
                cr.move_to(point.0, point.1);
            }

            for (_jc, curv) in curves.iter().enumerate() {
                if !*curv.smooth().borrow() {
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
                    .map(|contour| contour.curves().clone().borrow()[curve_idx].clone())
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
            let point = p_fn(curv.points().borrow()[0]);
            cr.move_to(point.0, point.1);
            match degree {
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
            let mut curves = contour.imp().curves.borrow_mut();
            if !*contour.imp().open.borrow() {
                if let Some(point) = curves
                    .last()
                    .and_then(|b| b.points().borrow().last().copied())
                {
                    pen_position = Some(f_fn(point));
                }
            }

            for curv in curves.iter_mut() {
                if curv.points().borrow().len() == 3 {
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        f_fn(curv.points().borrow()[0])
                    };
                    let b = f_fn(curv.points().borrow()[1]);
                    let c = f_fn(curv.points().borrow()[2]);
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
                    let smooth = *curv.smooth().borrow();
                    *curv = Bezier::new(smooth, new_points);
                    pen_position = Some(c);
                } else {
                    if let Some(last_p) = curv.points().borrow().last() {
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
        (self.contours.is_empty()
            || self
                .contours
                .iter()
                .all(|c| c.imp().curves.borrow().is_empty()))
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
