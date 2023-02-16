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
use glib::{ParamFlags, ParamSpec, ParamSpecBoxed, ParamSpecDouble, ParamSpecString, Value};

use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuidelineInner {
    pub name: RefCell<Option<String>>,
    pub identifier: RefCell<Option<String>>,
    pub angle: Cell<f64>,
    pub x: Cell<f64>,
    pub y: Cell<f64>,
    pub color: Cell<Option<Color>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GuidelineInner {
    const NAME: &'static str = "Guideline";
    type Type = Guideline;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for GuidelineInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        Guideline::NAME,
                        Guideline::NAME,
                        Guideline::NAME,
                        None,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecString::new(
                        Guideline::IDENTIFIER,
                        Guideline::IDENTIFIER,
                        Guideline::IDENTIFIER,
                        None,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Guideline::ANGLE,
                        Guideline::ANGLE,
                        Guideline::ANGLE,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Guideline::X,
                        Guideline::X,
                        Guideline::X,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Guideline::Y,
                        Guideline::Y,
                        Guideline::Y,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Guideline::COLOR,
                        Guideline::COLOR,
                        Guideline::COLOR,
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Guideline::NAME => self.name.borrow().to_value(),
            Guideline::IDENTIFIER => self.identifier.borrow().to_value(),
            Guideline::ANGLE => self.angle.get().to_value(),
            Guideline::X => self.x.get().to_value(),
            Guideline::Y => self.y.get().to_value(),
            Guideline::COLOR => self.color.get().unwrap_or(Self::COLOR).to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Guideline::NAME => {
                *self.name.borrow_mut() = value.get().unwrap();
            }
            Guideline::IDENTIFIER => {
                *self.identifier.borrow_mut() = value.get().unwrap();
            }
            Guideline::ANGLE => {
                let mut val: f64 = value.get().unwrap();
                if val.is_finite() {
                    while val < 0.0 {
                        val += 360.0;
                    }
                    self.angle.set(val % 180.0);
                }
            }
            Guideline::X => {
                self.x.set(value.get().unwrap());
            }
            Guideline::Y => {
                self.y.set(value.get().unwrap());
            }
            Guideline::COLOR => {
                self.color.set(Some(value.get().unwrap()));
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl GuidelineInner {
    const COLOR: Color = Color::from_hex("#0000ff").with_alpha((0.5 * 255.0) as u8);
    const HIGHLIGHT_COLOR: Color = Color::from_hex("#ff0000").with_alpha((0.5 * 255.0) as u8);

    pub fn draw(
        &self,
        cr: ContextRef,
        (_width, height): (f64, f64),
        highlight: bool,
        show_origin: bool,
    ) {
        fn move_point(p: (f64, f64), d: f64, r: f64) -> (f64, f64) {
            let (x, y) = p;
            (x + (d * f64::cos(r)), y + (d * f64::sin(r)))
        }
        if highlight {
            cr.set_source_color_alpha(Self::HIGHLIGHT_COLOR);
            let curr_width = cr.line_width();
            cr.set_line_width(curr_width + 1.0);
        } else {
            cr.set_source_color_alpha(self.color.get().unwrap_or(Self::COLOR));
        }
        let p = (self.x.get(), self.y.get());
        if show_origin {
            cr.arc(p.0, p.1, 8.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap();
        }
        let r = self.angle.get() * 0.01745;
        let top = move_point(p, height * 10.0, r);
        cr.move_to(top.0, top.1);
        let bottom = move_point(p, -height * 10.0, r);
        cr.line_to(bottom.0, bottom.1);
        cr.stroke().unwrap();
    }

    pub fn distance_from_point(&self, p: Point) -> f64 {
        let (xp, yp) = (p.x, p.y);
        // Using an 𝐿 defined by a point 𝑃𝑙 and angle 𝜃
        //𝑑 = ∣cos(𝜃)(𝑃𝑙𝑦 − 𝑦𝑝) − sin(𝜃)(𝑃𝑙𝑥 − 𝑃𝑥)∣
        // d = |cos(θˆ)⋅(Pₗᵧ - yₚ) - sin(θˆ)⋅(Pₗₓ - Pₓ)|
        let r = self.angle.get() * 0.01745;
        let (sin, cos) = r.sin_cos();
        (cos * (self.y.get() - yp) - sin * (self.x.get() - xp)).abs()
    }

    pub fn on_line_query(&self, point: Point, error: Option<f64>) -> bool {
        let error = error.unwrap_or(5.0);
        self.distance_from_point(point) <= error
    }

    pub fn project_point(&self, p: Point) -> Point {
        let r = self.angle.get() * 0.01745;
        let (sin, cos) = r.sin_cos();
        let p1 = Point::from((self.x.get(), self.y.get()));
        let p2 = Point::from((p1.x + 10.0 * cos, p1.y + 10.0 * sin));
        let alpha = p - p1;
        let beta = p2 - p1;
        let bunit = beta / beta.norm();
        let scalar = alpha.dot(beta) / beta.norm();
        scalar * bunit + p1
    }
}

glib::wrapper! {
    pub struct Guideline(ObjectSubclass<GuidelineInner>);
}

impl std::ops::Deref for Guideline {
    type Target = GuidelineInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Default for Guideline {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<serde_json::Value> for Guideline {
    type Error = serde_json::Error;
    fn try_from(v: serde_json::Value) -> Result<Guideline, Self::Error> {
        let inner: GuidelineInner = serde_json::from_value(v)?;
        let ret = Self::new();
        ret.name.swap(&inner.name);
        ret.identifier.swap(&inner.identifier);
        ret.color.swap(&inner.color);
        ret.angle.swap(&inner.angle);
        ret.x.swap(&inner.x);
        ret.y.swap(&inner.y);
        Ok(ret)
    }
}

impl TryFrom<ufo::GuidelineInfo> for Guideline {
    type Error = String;

    fn try_from(v: ufo::GuidelineInfo) -> Result<Guideline, Self::Error> {
        let ret = Self::new();
        // FIXME: check for invalid optional set value combinations
        let ufo::GuidelineInfo {
            x,
            y,
            angle,
            name,
            color,
            identifier,
        } = v;

        if let Some(x) = x {
            ret.x.set(x);
        }
        if let Some(y) = y {
            ret.y.set(y);
        }
        if let Some(name) = name {
            *ret.name.borrow_mut() = Some(name);
        }
        if let Some(identifier) = identifier {
            *ret.identifier.borrow_mut() = Some(identifier);
        }
        if let Some(color) = color {
            ret.color.set(Some(color));
        }
        if let Some(angle) = angle {
            ret.angle.set(angle);
        }
        Ok(ret)
    }
}

impl Guideline {
    pub const NAME: &str = "name";
    pub const COLOR: &str = "color";
    pub const IDENTIFIER: &str = "identifier";
    pub const ANGLE: &str = "angle";
    pub const X: &str = "x";
    pub const Y: &str = "y";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn name(&self) -> Option<String> {
        self.property(Guideline::NAME)
    }

    pub fn identifier(&self) -> Option<String> {
        self.property(Guideline::IDENTIFIER)
    }

    pub fn color(&self) -> Option<Color> {
        self.property(Guideline::COLOR)
    }

    pub fn angle(&self) -> f64 {
        self.property(Guideline::ANGLE)
    }

    pub fn x(&self) -> f64 {
        self.property(Guideline::X)
    }

    pub fn y(&self) -> f64 {
        self.property(Guideline::Y)
    }

    pub fn builder() -> GuidelineBuilder {
        GuidelineBuilder::new()
    }
}

pub struct GuidelineBuilder(Guideline);

impl Default for GuidelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GuidelineBuilder {
    pub fn new() -> Self {
        GuidelineBuilder(Guideline::new())
    }

    pub fn name(self, name: Option<String>) -> Self {
        *self.0.name.borrow_mut() = name;
        self
    }

    pub fn identifier(self, identifier: Option<String>) -> Self {
        *self.0.identifier.borrow_mut() = identifier;
        self
    }

    pub fn with_random_identifier(self) -> Self {
        *self.0.identifier.borrow_mut() = Some(ufo::make_random_identifier());
        self
    }

    pub fn color(self, color: Option<Color>) -> Self {
        self.0.color.set(color);
        self
    }

    pub fn angle(self, angle: f64) -> Self {
        self.0.angle.set(angle);
        self
    }

    pub fn x(self, x: f64) -> Self {
        self.0.x.set(x);
        self
    }

    pub fn y(self, y: f64) -> Self {
        self.0.y.set(y);
        self
    }

    pub fn build(self) -> Guideline {
        self.0
    }
}
