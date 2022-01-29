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
use std::cmp::Ordering;

use gtk::cairo::Context;
pub type Point = (i64, i64);

#[derive(Debug, Clone)]
pub struct Bezier {
    pub points: Vec<Point>,
}

impl Bezier {
    pub fn new(points: Vec<Point>) -> Self {
        Bezier { points }
    }

    pub fn get_point(&self, t: f64) -> Option<Point> {
        draw_curve_point(&self.points, t)
    }
}

fn draw_curve_point(points: &[Point], t: f64) -> Option<Point> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        //std::dbg!(points[0]);
        return Some(points[0]);
    }
    let mut new_points = Vec::with_capacity(points.len() - 1);
    for chunk in points.windows(2) {
        let p1 = chunk[0];
        let p2 = chunk[1];
        let x = (1. - t) * (p1.0 as f64) + t * (p2.0 as f64);
        let y = (1. - t) * (p1.1 as f64) + t * (p2.1 as f64);
        new_points.push((x as i64, y as i64));
    }
    assert_eq!(new_points.len(), points.len() - 1);
    draw_curve_point(&new_points, t)
}

#[derive(Debug, Clone)]
pub struct Contour {
    pub open: bool,
    pub curves: Vec<Bezier>,
}

#[derive(Debug, Clone)]
pub enum GlyphKind {
    Char(char),
    Component,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub name: Cow<'static, str>,
    pub kind: GlyphKind,
    pub width: Option<i64>,
    pub contours: Vec<Contour>,
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
        let g = glif::Glif::default();
        let g: Glyph = g.into();
        g
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

impl Glyph {
    pub fn from_ufo(path: &str) -> Vec<Self> {
        use std::path::Path;

        assert!(path.ends_with(".ufo"));
        let mut ret = vec![];
        let path = Path::new(path);
        let path = path.join("glyphs");

        for entry in path.read_dir().expect("read_dir call failed").flatten() {
            use std::fs::File;
            use std::io::prelude::*;
            let mut file = match File::open(&entry.path()) {
                Err(why) => panic!("couldn't open {}: {}", entry.path().display(), why),
                Ok(file) => file,
            };

            let mut s = String::new();
            if let Err(err) = file.read_to_string(&mut s) {
                panic!("couldn't read {}: {}", entry.path().display(), err);
            }
            let g: Result<glif::Glif, _> = glif::Glif::from_str(&s);
            match g {
                Err(err) => {
                    eprintln!("couldn't parse {}: {}", entry.path().display(), err);
                }
                Ok(g) => {
                    let g: Glyph = g.into();
                    ret.push(g);
                }
            }
        }
        ret
    }

    pub fn new(name: &'static str, char: char, curves: Vec<Bezier>) -> Self {
        Glyph {
            name: name.into(),
            kind: GlyphKind::Char(char),
            contours: vec![Contour {
                open: false,
                curves,
            }],
            width: None,
        }
    }

    pub fn new_empty(name: &'static str, char: char) -> Self {
        Glyph::new(name, char, vec![])
    }

    pub fn draw(
        &self,
        _drar: &gtk::DrawingArea,
        cr: &Context,
        (x, y): (f64, f64),
        (og_width, _og_height): (f64, f64),
    ) {
        if self.is_empty() {
            return;
        }
        let f = og_width
            / self
                .width
                .map(|w| if w == 0 { 1000 } else { w })
                .unwrap_or(1000) as f64;
        cr.save().expect("Invalid cairo surface state");
        cr.move_to(x, y);
        cr.set_source_rgba(0.2, 0.2, 0.2, 0.6);
        cr.set_line_width(2.0);
        cr.translate(0., -20.);
        for contour in self.contours.iter() {
            let mut strokes = vec![];
            let mut pen_position: Option<(f64, f64)> = None;
            if !contour.open {
                if let Some(point) = contour.curves.last().and_then(|b| b.points.last()) {
                    let point = (point.0 as f64, point.1 as f64);
                    pen_position = Some(point);
                }
            }
            for c in contour.curves.iter() {
                let prev_point = c.points[0];
                let mut prev_point = (prev_point.0 as f64, prev_point.1 as f64);
                let mut sample = 0;
                for t in (0..100).step_by(1) {
                    let t = (t as f64) / 100.;
                    if let Some(new_point) = c.get_point(t) {
                        let new_point = (new_point.0 as f64, new_point.1 as f64);
                        if sample == 0 {
                            if let Some(prev_position) = pen_position.take() {
                                strokes.push((prev_position, new_point));
                            }
                            //println!("{:?} {:?}", prev_point, new_point);
                            strokes
                                .push(((prev_point.0, prev_point.1), (new_point.0, new_point.1)));

                            sample = 5;
                            prev_point = new_point;
                        }
                        sample -= 1;
                    }
                }
                let new_point = *c.points.last().unwrap();
                let new_point = (new_point.0 as f64, new_point.1 as f64);
                strokes.push(((prev_point.0, prev_point.1), (new_point.0, new_point.1)));
                pen_position = Some(prev_point);
            }
            cr.new_path();
            let mut prev_point = (x, y);
            for ((ax, ay), (bx, by)) in &strokes {
                if ((prev_point.0 - *ax).powi(2) + (prev_point.1 - *ay).powi(2)).sqrt()
                    > f64::EPSILON
                {
                    cr.move_to(ax * f + x, ay * f + y);
                }
                prev_point = (*bx, *by);
                cr.line_to(bx * f + x, by * f + y);
            }
            cr.stroke().expect("Invalid cairo surface state");
        }
        cr.restore().expect("Invalid cairo surface state");
        /*
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(0.005);
        for &(x, y) in &c.points {
        cr.rectangle(x as f64 / width, y as f64 / height, 0.001, 0.001);
        cr.stroke_preserve().expect("Invalid cairo surface state");
        }
        */
    }

    pub fn is_empty(&self) -> bool {
        self.contours.is_empty() || self.contours.iter().all(|c| c.curves.is_empty())
    }

    pub fn name_markup(&self) -> gtk::glib::GString {
        match self.kind {
            GlyphKind::Char(c) => {
                let mut b = [0; 4];

                gtk::glib::markup_escape_text(c.encode_utf8(&mut b))
            }
            GlyphKind::Component => gtk::glib::markup_escape_text(self.name.as_ref()),
        }
    }
}

mod glif {
    extern crate quick_xml;
    extern crate serde;

    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
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

    #[derive(Debug, Deserialize, PartialEq)]
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

    #[derive(Debug, Deserialize, PartialEq)]
    struct Contour {
        #[serde(rename = "$value", default)]
        point: Vec<Point>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Anchor {
        name: String,
        x: i64,
        y: i64,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Outline {
        #[serde(rename = "$value", default)]
        countours: Vec<Contour>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Unicode {
        hex: String,
    }

    #[derive(Debug, Deserialize, PartialEq, Default)]
    struct Advance {
        width: i64,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Glif {
        name: String,
        format: Option<String>,
        #[serde(default)]
        unicode: Option<Unicode>,
        #[serde(default)]
        advance: Option<Advance>,
        #[serde(default)]
        outline: Option<Outline>,
        #[serde(rename = "anchor", default)]
        anchors: Vec<Anchor>,
    }

    impl Into<super::Glyph> for Glif {
        fn into(self) -> super::Glyph {
            use super::{Bezier, Glyph};
            let Self {
                name,
                unicode,
                outline,
                advance,
                ..
            } = self;

            let kind = if let Some(val) = unicode
                .and_then(|unicode| u32::from_str_radix(unicode.hex.as_str(), 16).ok())
                .and_then(|n| n.try_into().ok())
            {
                super::GlyphKind::Char(val)
            } else {
                super::GlyphKind::Component
            };
            let mut ret = Glyph {
                name: name.into(),
                kind,
                width: advance.map(|a| a.width),
                contours: vec![],
            };

            if let Some(outline) = outline {
                for contour in outline.countours {
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
                    if points.front().unwrap().is_move() {
                        open = true;
                        // Open contour
                        let p = points.pop_front().unwrap();
                        prev_point = (p.x, 1000 - p.y);
                        c = vec![prev_point];
                    } else {
                        c = vec![];
                        // Closed contour
                        while points.front().unwrap().is_curve() {
                            points.rotate_left(1);
                        }
                        let last_point = points.back().unwrap();
                        prev_point = (last_point.x, 1000 - last_point.y);
                    }
                    if points.front().unwrap().is_line() {
                        let p = points.back().unwrap();
                        prev_point = (p.x, 1000 - p.y);
                    }
                    loop {
                        match points.pop_front() {
                            Some(Point {
                                type_: PointKind::Move,
                                ..
                            }) => {
                                panic!()
                            }
                            Some(Point {
                                type_: PointKind::Offcurve,
                                x,
                                y,
                                ..
                            }) => {
                                prev_point = (*x, 1000 - *y);
                                c.push(prev_point);
                            }
                            Some(Point {
                                type_: PointKind::Curve,
                                x,
                                y,
                                ..
                            }) => {
                                prev_point = (*x, 1000 - *y);
                                c.push(prev_point);
                                contour_acc.push(Bezier::new(c));
                                c = vec![];
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
                                c.push((*x, 1000 - *y));
                                contour_acc.push(Bezier::new(c));
                                c = vec![];
                                prev_point = (*x, 1000 - *y);
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
                                    contour_acc.push(Bezier::new(c));
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

            ret
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
