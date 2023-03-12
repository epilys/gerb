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

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuidelineInner {
    name: RefCell<Option<String>>,
    identifier: RefCell<Option<String>>,
    angle: Cell<Option<f64>>,
    x: Cell<Option<f64>>,
    y: Cell<Option<f64>>,
    color: Cell<Option<Color>>,
    pub modified: Cell<bool>,
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
                    glib::ParamSpecBoolean::new(
                        Guideline::MODIFIED,
                        Guideline::MODIFIED,
                        Guideline::MODIFIED,
                        false,
                        ParamFlags::READWRITE,
                    ),
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
            Guideline::ANGLE => self.angle().to_value(),
            Guideline::X => self.x().to_value(),
            Guideline::Y => self.y().to_value(),
            Guideline::COLOR => self.color().to_value(),
            Guideline::MODIFIED => self.modified.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        macro_rules! set_cell {
            ($field:tt) => {{
                let val = value.get().unwrap();
                set_cell!(val, $field);
            }};
            ($val:ident, $field:tt) => {{
                if Some($val) != self.$field.get() {
                    self.instance().set_property(Guideline::MODIFIED, true);
                    self.$field.set(Some($val));
                }
            }};
        }
        macro_rules! set_refcell {
            ($field:ident) => {{
                let val: String = value.get().unwrap();
                if val != self.$field.borrow() {
                    self.instance().set_property(Guideline::MODIFIED, true);
                    *self.$field.borrow_mut() = val;
                }
            }};
            (opt $field:tt) => {{
                let val: String = value.get().unwrap();
                if Some(&val) != self.$field.borrow().as_ref() {
                    self.instance().set_property(Guideline::MODIFIED, true);
                    *self.$field.borrow_mut() = Some(val);
                }
            }};
        }
        match pspec.name() {
            Guideline::NAME => {
                set_refcell!(opt name);
            }
            Guideline::IDENTIFIER => {
                set_refcell!(opt identifier);
            }
            Guideline::ANGLE => {
                let mut val: f64 = value.get().unwrap();
                if val.is_finite() {
                    while val < 0.0 {
                        val += 360.0;
                    }
                    val %= 180.0;
                    set_cell!(val, angle);
                }
            }
            Guideline::X => set_cell!(x),
            Guideline::Y => set_cell!(y),
            Guideline::COLOR => set_cell!(color),
            Guideline::MODIFIED => {
                self.modified.set(value.get().unwrap());
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
            (d.mul_add(f64::cos(r), x), d.mul_add(f64::sin(r), y))
        }
        if highlight {
            cr.set_source_color_alpha(Self::HIGHLIGHT_COLOR);
            let curr_width = cr.line_width();
            cr.set_line_width(curr_width + 1.0);
        } else {
            cr.set_source_color_alpha(self.color.get().unwrap_or(Self::COLOR));
        }
        let p = (self.x(), self.y());
        if show_origin {
            cr.arc(p.0, p.1, 8.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap();
        }
        let r = self.angle() * 0.01745;
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
        let r = self.angle() * 0.01745;
        let (sin, cos) = r.sin_cos();
        (cos * (self.y() - yp) - sin * (self.x() - xp)).abs()
    }

    pub fn on_line_query(&self, point: Point, error: Option<f64>) -> bool {
        let error = error.unwrap_or(5.0);
        self.distance_from_point(point) <= error
    }

    pub fn project_point(&self, p: Point) -> Point {
        let r = self.angle() * 0.01745;
        let (sin, cos) = r.sin_cos();
        let p1 = Point::from((self.x(), self.y()));
        let p2 = Point::from((10.0f64.mul_add(cos, p1.x), 10.0f64.mul_add(sin, p1.y)));
        let alpha = p - p1;
        let beta = p2 - p1;
        let bunit = beta / beta.norm();
        let scalar = alpha.dot(beta) / beta.norm();
        scalar * bunit + p1
    }

    #[inline(always)]
    pub fn name(&self) -> Option<String> {
        self.name.borrow().clone()
    }

    #[inline(always)]
    pub fn identifier(&self) -> Option<String> {
        self.identifier.borrow().clone()
    }

    #[inline(always)]
    pub fn color(&self) -> Color {
        self.color.get().unwrap_or(Self::COLOR)
    }

    #[inline(always)]
    pub fn color_inner(&self) -> Option<Color> {
        self.color.get()
    }

    #[inline(always)]
    pub fn angle(&self) -> f64 {
        self.angle.get().unwrap_or_default()
    }

    #[inline(always)]
    pub fn angle_inner(&self) -> Option<f64> {
        self.angle.get()
    }

    #[inline(always)]
    pub fn x(&self) -> f64 {
        self.x.get().unwrap_or_default()
    }

    #[inline(always)]
    pub fn x_inner(&self) -> Option<f64> {
        self.x.get()
    }

    #[inline(always)]
    pub fn y(&self) -> f64 {
        self.y.get().unwrap_or_default()
    }

    #[inline(always)]
    pub fn y_inner(&self) -> Option<f64> {
        self.y.get()
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
    fn try_from(v: serde_json::Value) -> Result<Self, Self::Error> {
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

impl From<ufo::GuidelineInfo> for Guideline {
    fn from(v: ufo::GuidelineInfo) -> Self {
        let ret = Self::new();
        let ufo::GuidelineInfo {
            x,
            y,
            angle,
            name,
            color,
            identifier,
        } = v;

        ret.x.set(x);
        ret.y.set(y);
        *ret.name.borrow_mut() = name;
        *ret.identifier.borrow_mut() = identifier;
        ret.color.set(color);
        ret.angle.set(angle);
        ret
    }
}

impl From<&Guideline> for ufo::GuidelineInfo {
    fn from(v: &Guideline) -> Self {
        Self {
            x: v.x_inner(),
            y: v.y_inner(),
            angle: v.angle_inner(),
            name: v.name(),
            identifier: v.identifier(),
            color: v.color_inner(),
        }
    }
}

impl Guideline {
    pub const NAME: &str = "name";
    pub const COLOR: &str = "color";
    pub const IDENTIFIER: &str = "identifier";
    pub const ANGLE: &str = "angle";
    pub const X: &str = "x";
    pub const Y: &str = "y";
    pub const MODIFIED: &str = "modified";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    #[inline(always)]
    pub fn modified(&self) -> bool {
        self.imp().modified.get()
    }

    pub fn builder() -> GuidelineBuilder {
        GuidelineBuilder::new()
    }
}

impl_modified!(Guideline);
impl_property_window!(Guideline);

pub struct GuidelineBuilder(Guideline);

impl Default for GuidelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GuidelineBuilder {
    pub fn new() -> Self {
        Self(Guideline::new())
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

    pub fn angle(self, angle: Option<f64>) -> Self {
        self.0.angle.set(angle);
        self
    }

    pub fn x(self, x: Option<f64>) -> Self {
        self.0.x.set(x);
        self
    }

    pub fn y(self, y: Option<f64>) -> Self {
        self.0.y.set(y);
        self
    }

    pub fn build(self) -> Guideline {
        self.0.imp().modified.set(true);
        self.0
    }
}
