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
use crate::glyphs;
use crate::utils::colors::Color;
use crate::utils::curves::Bezier;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use glib::subclass::types::ObjectSubclassIsExt;

fn color_serialize<S>(v: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *v {
        Some(Color((r, g, b, a))) => {
            let (r, g, b, a) = (
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0,
            );
            let cl = move |v: f64| {
                if v == 0.0 || v == 1.0 {
                    format!("{:.0}", v)
                } else {
                    format!("{:.2}", v)
                }
            };
            serializer.serialize_str(&format!("{},{},{},{}", cl(r), cl(g), cl(b), cl(a)))
        }
        None => serializer.serialize_str("0,0,0,0"),
    }
}

const fn f64_one_val() -> f64 {
    1.0
}

fn f64_is_zero(val: &f64) -> bool {
    *val == 0.0
}

fn f64_is_one(val: &f64) -> bool {
    *val == 1.0
}

fn pointkind_is_default(val: &PointKind) -> bool {
    *val == PointKind::Offcurve
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
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

fn smooth_deserialize<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|s| {
        if s == "yes" {
            Ok(Some(true))
        } else if s == "no" {
            Ok(Some(false))
        } else {
            Err(Error::custom(format!(
                "Invalid smooth value: expected either `yes` or `no`, got `{s}`"
            )))
        }
    })
}

fn smooth_serialize<S>(s: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match s.as_ref() {
        Some(true) => serializer.serialize_str("yes"),
        None | Some(false) => serializer.serialize_str("no"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename = "point")]
struct Point {
    #[serde(rename = "@x")]
    x: f64,
    #[serde(rename = "@y")]
    y: f64,
    #[serde(default)]
    #[serde(rename = "@name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default)]
    #[serde(rename = "@identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    identifier: Option<String>,
    #[serde(rename = "@type", default)]
    #[serde(skip_serializing_if = "pointkind_is_default")]
    type_: PointKind,
    #[serde(rename = "@smooth")]
    #[serde(default)]
    #[serde(serialize_with = "smooth_serialize")]
    #[serde(deserialize_with = "smooth_deserialize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    smooth: Option<bool>,
}

impl Point {
    #[inline(always)]
    fn is_curve(&self) -> bool {
        matches!(self.type_, PointKind::Curve)
    }

    #[inline(always)]
    fn is_move(&self) -> bool {
        matches!(self.type_, PointKind::Move)
    }

    #[inline(always)]
    fn is_line(&self) -> bool {
        matches!(self.type_, PointKind::Line)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename = "contour")]
struct Contour {
    #[serde(default)]
    #[serde(rename = "@identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    identifier: Option<String>,
    #[serde(default)]
    point: Vec<Point>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Component {
    #[serde(rename = "@base")]
    base: String,
    #[serde(rename = "@xOffset")]
    #[serde(default)]
    x_offset: f64,
    #[serde(default)]
    #[serde(rename = "@yOffset")]
    y_offset: f64,
    #[serde(default = "f64_one_val")]
    #[serde(rename = "@xScale")]
    x_scale: f64,
    #[serde(default)]
    #[serde(rename = "@xyScale")]
    xy_scale: f64,
    #[serde(default)]
    #[serde(rename = "@yxScale")]
    yx_scale: f64,
    #[serde(default = "f64_one_val")]
    #[serde(rename = "@yScale")]
    y_scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum OutlineEntry {
    Contour(Contour),
    Component(Component),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anchor {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@x")]
    x: f64,
    #[serde(rename = "@y")]
    y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Guideline {
    #[serde(default)]
    #[serde(rename = "@name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default)]
    #[serde(rename = "@identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    identifier: Option<String>,
    #[serde(default)]
    #[serde(rename = "@color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "color_serialize")]
    color: Option<Color>,
    #[serde(default)]
    #[serde(rename = "@angle")]
    angle: f64,
    #[serde(default)]
    #[serde(rename = "@x")]
    x: f64,
    #[serde(default)]
    #[serde(rename = "@y")]
    y: f64,
}

impl From<&glyphs::Guideline> for Guideline {
    fn from(g: &glyphs::Guideline) -> Guideline {
        Guideline {
            name: g.imp().name.borrow().clone(),
            identifier: g.imp().identifier.borrow().clone(),
            color: g.imp().color.get(),
            angle: g.imp().angle.get(),
            x: g.imp().x.get(),
            y: g.imp().y.get(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Outline {
    #[serde(default)]
    #[serde(rename = "$value")]
    contours: Vec<OutlineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", rename = "unicode")]
pub struct Unicode {
    #[serde(rename = "@hex")]
    hex: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Advance {
    #[serde(rename = "@width")]
    #[serde(default)]
    #[serde(skip_serializing_if = "f64_is_zero")]
    width: f64,
    #[serde(rename = "@height")]
    #[serde(default)]
    #[serde(skip_serializing_if = "f64_is_zero")]
    height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase", rename = "image")]
pub struct ImageRef {
    #[serde(default)]
    #[serde(rename = "@fileName")]
    pub file_name: Option<String>,
    #[serde(default = "f64_one_val")]
    #[serde(rename = "@xScale")]
    #[serde(skip_serializing_if = "f64_is_one")]
    pub x_scale: f64,
    #[serde(default)]
    #[serde(rename = "@xyScale")]
    #[serde(skip_serializing_if = "f64_is_zero")]
    pub xy_scale: f64,
    #[serde(default)]
    #[serde(rename = "@yxScale")]
    #[serde(skip_serializing_if = "f64_is_zero")]
    pub yx_scale: f64,
    #[serde(default = "f64_one_val")]
    #[serde(rename = "@yScale")]
    #[serde(skip_serializing_if = "f64_is_one")]
    pub y_scale: f64,
    #[serde(default)]
    #[serde(rename = "@xScale")]
    #[serde(skip_serializing_if = "f64_is_zero")]
    pub x_offset: f64,
    #[serde(default)]
    #[serde(rename = "@yScale")]
    #[serde(skip_serializing_if = "f64_is_zero")]
    pub y_offset: f64,
    #[serde(default)]
    #[serde(rename = "@color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "color_serialize")]
    pub color: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename = "glyph")]
pub struct Glif {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@format")]
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    unicode: Vec<Unicode>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<ImageRef>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    advance: Option<Advance>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    outline: Option<Outline>,
    #[serde(rename = "anchor", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    anchors: Vec<Anchor>,
    #[serde(rename = "guideline", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    guidelines: Vec<Guideline>,
    //#[serde(
    //    rename = "lib",
    //    default,
    //    serialize_with = "plist_serialize",
    //    deserialize_with = "plist_deserialize",
    //    skip_serializing_if = "Option::is_none"
    //)]
    //lib: Option<plist::Dictionary>,
}

//fn plist_deserialize<'de, D>(deserializer: D) -> Result<Option<plist::Dictionary>, D::Error>
//where
//    D: serde::Deserializer<'de>,
//{
//    let dict = plist::Dictionary::deserialize(deserializer)?;
//    if dict.is_empty() {
//        Ok(None)
//    } else {
//        Ok(Some(dict))
//    }
//}
//
//fn plist_serialize<S>(s: &Option<plist::Dictionary>, serializer: S) -> Result<S::Ok, S::Error>
//where
//    S: Serializer,
//{
//    match s.as_ref() {
//        Some(dict) => dict.serialize(serializer),
//        None => serializer.serialize_str(""),
//    }
//}

impl Glif {
    #[allow(dead_code)]
    pub fn to_xml(&self) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}\n",
            quick_xml::se::to_string(&self).unwrap(),
        )
    }
}

impl From<Glif> for glyphs::Glyph {
    fn from(val: Glif) -> glyphs::Glyph {
        use glyphs::Glyph;
        let Glif {
            name,
            outline,
            advance,
            image,
            anchors,
            guidelines,
            unicode,
            format: _,
            //lib,
        } = val;

        let kinds = if unicode.is_empty() {
            (glyphs::GlyphKind::Component(name.clone()), vec![])
        } else {
            let mut iter = unicode
                .iter()
                .filter_map(|unicode| u32::from_str_radix(unicode.hex.as_str(), 16).ok())
                .filter_map(|n| n.try_into().ok())
                .map(glyphs::GlyphKind::Char);
            let first = iter.next().unwrap();
            (first, iter.collect::<Vec<_>>())
        };
        let mut ret = Glyph {
            guidelines: guidelines
                .into_iter()
                .map(|g| {
                    glyphs::Guideline::builder()
                        .name(g.name)
                        .identifier(g.identifier)
                        .color(g.color)
                        .angle(g.angle)
                        .x(g.x)
                        .y(g.y)
                        .build()
                })
                .collect::<Vec<_>>(),
            ..Glyph::default()
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
                        ret.components.push(glyphs::Component {
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
                let super_ = glyphs::Contour::new();
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

impl From<&glyphs::Glyph> for Glif {
    fn from(glyph: &glyphs::Glyph) -> Glif {
        let mut outline: Vec<OutlineEntry> =
            Vec::with_capacity(glyph.components.len() + glyph.contours.len());
        outline.extend(glyph.components.iter().map(|c| {
            OutlineEntry::Component(Component {
                base: c.base_name.clone(),
                x_offset: c.x_offset,
                y_offset: c.y_offset,
                x_scale: c.x_scale,
                xy_scale: c.xy_scale,
                yx_scale: c.yx_scale,
                y_scale: c.y_scale,
            })
        }));
        outline.extend(glyph.contours.iter().map(|c| {
            let mut point = vec![];
            if c.imp().open.get() {
                let mut first = true;
                for curv in c.curves().iter() {
                    let degree = curv.degree();
                    point.extend(curv.points().iter().enumerate().map(|(i, cp)| Point {
                        x: cp.position.x,
                        y: cp.position.y,
                        name: None,
                        identifier: None,
                        type_: if first {
                            first = false;
                            PointKind::Move
                        } else {
                            match (i, degree) {
                                (0 | 3, Some(3)) => PointKind::Curve,
                                (1 | 2, Some(3)) => PointKind::Offcurve,
                                (0 | 2, Some(2)) => PointKind::Curve,
                                (1, Some(2)) => PointKind::Qcurve,
                                (0 | 1, Some(1)) => PointKind::Line,
                                _ => PointKind::Move,
                            }
                        },
                        smooth: None,
                    }));
                }
            } else {
                let mut last = false;
                for curv in c.curves().iter() {
                    let degree = curv.degree();
                    point.extend(curv.points().iter().enumerate().filter_map(|(i, cp)| {
                        if last {
                            last = false;
                            None
                        } else {
                            if Some(i + 1) == degree {
                                last = true;
                            }

                            Some(Point {
                                x: cp.position.x,
                                y: cp.position.y,
                                name: None,
                                identifier: None,
                                type_: match (i, degree) {
                                    (0 | 3, Some(3)) => PointKind::Curve,
                                    (1 | 2, Some(3)) => PointKind::Offcurve,
                                    (0 | 2, Some(2)) => PointKind::Curve,
                                    (1, Some(2)) => PointKind::Qcurve,
                                    (0 | 1, Some(1)) => PointKind::Line,
                                    _ => PointKind::Move,
                                },
                                smooth: None,
                            })
                        }
                    }));
                }
            }
            OutlineEntry::Contour(Contour {
                identifier: None,
                point,
            })
        }));

        Glif {
            name: glyph.name().to_string(),
            format: Some("2".to_string()),
            unicode: glyph.metadata.unicode.borrow().clone(),
            image: glyph.metadata.image.borrow().clone(),
            advance: glyph.metadata.advance.get(),
            outline: Some(Outline { contours: outline }),
            anchors: glyph.metadata.anchors.borrow().clone(),
            guidelines: glyph.guidelines.iter().map(Into::into).collect(),
            //lib: glyph.lib.clone(),
        }
    }
}

impl std::str::FromStr for Glif {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
    let _: glyphs::Glyph = g.into();
    let glif: Glif = quick_xml::de::from_str(EXCLAM_GLYPH).unwrap();
    let glyph: glyphs::Glyph = glif.clone().into();
    let _glif2: Glif = Glif::from(&glyph);
    //print!("{}\n\n{}", glif.to_xml(), glif2.to_xml());
    //assert_eq!(glif.to_xml(), glif2.to_xml());
}

#[test]
fn test_glif_write() {
    let g: Glif = quick_xml::de::from_str(_UPPERCASE_A_GLIF).unwrap();
    let _: glyphs::Glyph = g.into();
    let g: Glif = quick_xml::de::from_str(_UPPERCASE_A_GLIF).unwrap();
    let g2: Glif = quick_xml::de::from_str(&g.to_xml()).unwrap();
    assert_eq!(g.to_xml(), g2.to_xml());
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

#[cfg(test)]
const _UPPERCASE_A_GLIF: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="A" format="2">
	<unicode hex="0041"/>
  <image fileName="Sketch 1.png" xOffset="100" yOffset="200"
    xScale=".75" yScale=".75" color="1,0,0,.5" />
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

#[cfg(test)]
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

//@ SportingNormal.ufo/glyphs/exclam.glif
#[cfg(test)]
const EXCLAM_GLYPH: &str = r##"<?xml version='1.0' encoding='UTF-8'?>
<glyph name="exclam" format="2">
  <advance width="290"/>
  <unicode hex="0021"/>
  <outline>
    <contour>
      <point x="80" y="777" type="line"/>
      <point x="90" y="240" type="line"/>
      <point x="200" y="240" type="line"/>
      <point x="210" y="777" type="line"/>
    </contour>
    <contour>
      <point x="80" y="0" type="line"/>
      <point x="210" y="0" type="line"/>
      <point x="210" y="145" type="line"/>
      <point x="80" y="145" type="line"/>
    </contour>
  </outline>
  <lib>
    <dict>
      <key>com.schriftgestaltung.Glyphs.lastChange</key>
      <string>2018-04-05 15:21:53 +0000</string>
      <key>com.typemytype.robofont.mark</key>
      <array>
        <real>0.6</real>
        <real>0.609</real>
        <integer>1</integer>
        <integer>1</integer>
      </array>
    </dict>
  </lib>
</glyph>
"##;
