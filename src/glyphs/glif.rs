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

extern crate quick_xml;
extern crate serde;

use crate::glib::ObjectExt;
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
    x: f64,
    y: f64,
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
                .map(|g| {
                    super::Guideline::builder()
                        .name(g.name)
                        .identifier(g.identifier)
                        .color(g.color)
                        .angle(g.angle)
                        .x(g.x)
                        .y(g.y)
                        .build()
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
                        //let Point { x, y, .. } = points.front().unwrap();
                        //c.push((*x, *y));
                        points.rotate_left(1);
                    }
                    let last_point = points.back().unwrap();
                    prev_point = (last_point.x, last_point.y);
                }
                if points.front().unwrap().is_line() {
                    let p = points.back().unwrap();
                    prev_point = (p.x, p.y);
                }
                let super_ = super::Contour::new();
                super_.set_property::<bool>(super::Contour::OPEN, open);
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
                            super_.push_curve(Bezier::new(
                                smooth,
                                c.into_iter().map(Into::into).collect(),
                            ));
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
                            super_.push_curve(Bezier::new(
                                false,
                                c.into_iter().map(Into::into).collect(),
                            ));
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
                                    false,
                                    c.into_iter().map(Into::into).collect(),
                                ));
                            }
                            break;
                        }
                    }
                }
                ret.contours.push(super_);
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
    let g: super::Glyph = g.into_iter().next().unwrap();
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

const _AE_GLIF: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="ae" format="2">
	<unicode hex="00E6"/>
	<advance width="778"/>
	<outline>
		<contour>
			<point x="194" y="-12" type="curve" smooth="yes"/>
			<point x="257" y="-12"/>
			<point x="327" y="19"/>
			<point x="392" y="79" type="curve"/>
			<point x="430" y="30"/>
			<point x="482" y="-12"/>
			<point x="564" y="-12" type="curve" smooth="yes"/>
			<point x="630" y="-12"/>
			<point x="680" y="8"/>
			<point x="722" y="38" type="curve"/>
			<point x="693" y="92" type="line"/>
			<point x="658" y="68"/>
			<point x="620" y="55"/>
			<point x="574" y="55" type="curve" smooth="yes"/>
			<point x="490" y="55"/>
			<point x="427" y="121"/>
			<point x="423" y="221" type="curve"/>
			<point x="737" y="221" type="line"/>
			<point x="739" y="234"/>
			<point x="741" y="251"/>
			<point x="741" y="269" type="curve" smooth="yes"/>
			<point x="741" y="408"/>
			<point x="677" y="498"/>
			<point x="555" y="498" type="curve" smooth="yes"/>
			<point x="488" y="498"/>
			<point x="432" y="458"/>
			<point x="396" y="395" type="curve"/>
			<point x="377" y="458"/>
			<point x="328" y="498"/>
			<point x="256" y="498" type="curve" smooth="yes"/>
			<point x="185" y="498"/>
			<point x="118" y="465"/>
			<point x="73" y="435" type="curve"/>
			<point x="105" y="379" type="line"/>
			<point x="143" y="403"/>
			<point x="194" y="429"/>
			<point x="246" y="429" type="curve" smooth="yes"/>
			<point x="326" y="429"/>
			<point x="345" y="371"/>
			<point x="346" y="309" type="curve"/>
			<point x="145" y="286"/>
			<point x="51" y="235"/>
			<point x="51" y="126" type="curve" smooth="yes"/>
			<point x="51" y="38"/>
			<point x="113" y="-12"/>
		</contour>
		<contour>
			<point x="217" y="56" type="curve" smooth="yes"/>
			<point x="170" y="56"/>
			<point x="134" y="77"/>
			<point x="134" y="131" type="curve" smooth="yes"/>
			<point x="134" y="195"/>
			<point x="191" y="231"/>
			<point x="345" y="250" type="curve"/>
			<point x="346" y="228" type="line" smooth="yes"/>
			<point x="348" y="192"/>
			<point x="353" y="154"/>
			<point x="364" y="128" type="curve"/>
			<point x="320" y="81"/>
			<point x="263" y="56"/>
		</contour>
		<contour>
			<point x="424" y="284" type="line"/>
			<point x="434" y="373"/>
			<point x="487" y="431"/>
			<point x="553" y="431" type="curve" smooth="yes"/>
			<point x="623" y="431"/>
			<point x="666" y="381"/>
			<point x="666" y="284" type="curve"/>
		</contour>
	</outline>
	<anchor name="aboveLC" x="406" y="509"/>
	<anchor name="belowLC" x="413" y="-22"/>
</glyph>"##;
