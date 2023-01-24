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
use crate::ufo;
use crate::utils::colors::*;
use glib::{ParamFlags, ParamSpec, ParamSpecBoxed, ParamSpecDouble, ParamSpecString, Value};
use gtk::glib;

use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuidelineInner {
    pub name: RefCell<Option<String>>,
    pub identifier: RefCell<Option<String>>,
    pub angle: Cell<f64>,
    pub x: Cell<f64>,
    pub y: Cell<f64>,
    pub color: Cell<Color>,
    pub highlight_color: Cell<Color>,
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
        self.color.set(Color::new_alpha(0.0, 0.0, 1.0, 0.8));
        self.highlight_color
            .set(Color::new_alpha(1.0, 0.0, 0.0, 0.8));
        *self.identifier.borrow_mut() = Some(crate::ufo::make_random_identifier());
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
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        Guideline::IDENTIFIER,
                        Guideline::IDENTIFIER,
                        Guideline::IDENTIFIER,
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Guideline::ANGLE,
                        Guideline::ANGLE,
                        Guideline::ANGLE,
                        -360.0,
                        360.0,
                        0.,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Guideline::X,
                        Guideline::X,
                        Guideline::X,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Guideline::Y,
                        Guideline::Y,
                        Guideline::Y,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoxed::new(
                        Guideline::COLOR,
                        Guideline::COLOR,
                        Guideline::COLOR,
                        Color::static_type(),
                        ParamFlags::READWRITE,
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
            Guideline::COLOR => self.color.get().to_value(),
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
                self.angle.set(value.get().unwrap());
            }
            Guideline::X => {
                self.x.set(value.get().unwrap());
            }
            Guideline::Y => {
                self.y.set(value.get().unwrap());
            }
            Guideline::COLOR => {
                self.color.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl GuidelineInner {
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
            cr.set_source_color_alpha(self.highlight_color.get());
            cr.set_line_width(2.0);
        } else {
            cr.set_source_color_alpha(self.color.get());
            cr.set_line_width(1.5);
        }
        let p = matrix.transform_point(self.x.get(), self.y.get());
        let r = self.angle.get() * 0.01745;
        if let Some(name) = self.name.borrow().as_ref() {
            cr.save().unwrap();
            cr.move_to(p.0, p.1);
            cr.rotate(r);
            cr.show_text(name).unwrap();
            cr.restore().unwrap();
        }
        let top = move_point(p, height * 10., r);
        cr.move_to(top.0, top.1);
        let bottom = move_point(p, -height * 10., r);
        cr.line_to(bottom.0, bottom.1);
        cr.stroke().unwrap();
        cr.restore().unwrap();
    }

    pub fn distance_from_point(&self, p: Point) -> f64 {
        let (xp, yp) = (p.x, p.y);
        // Using an 𝐿 defined by a point 𝑃𝑙 and angle 𝜃
        //𝑑 = ∣cos(𝜃)(𝑃𝑙𝑦 − 𝑦𝑝) − sin(𝜃)(𝑃𝑙𝑥 − 𝑃𝑥)∣
        let r = -self.angle.get() * 0.01745;
        let sin = f64::sin(r);
        let cos = f64::cos(r);
        (cos * (self.y.get() - yp) - sin * (self.x.get() - xp)).abs()
    }

    pub fn on_line_query(&self, point: Point, error: Option<f64>) -> bool {
        let error = error.unwrap_or(5.0);
        self.distance_from_point(point) <= error
    }
}

glib::wrapper! {
    pub struct Guideline(ObjectSubclass<GuidelineInner>);
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
        ret.imp().name.swap(&inner.name);
        ret.imp().identifier.swap(&inner.identifier);
        ret.imp().color.swap(&inner.color);
        ret.imp().angle.swap(&inner.angle);
        ret.imp().x.swap(&inner.x);
        ret.imp().y.swap(&inner.y);
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
            ret.imp().x.set(x);
        }
        if let Some(y) = y {
            ret.imp().y.set(y);
        }
        if let Some(name) = name {
            *ret.imp().name.borrow_mut() = Some(name);
        }
        if let Some(identifier) = identifier {
            *ret.imp().identifier.borrow_mut() = Some(identifier);
        }
        if let Some(_color) = color {
            //ret.imp().color.swap(&inner.color);
        }
        if let Some(angle) = angle {
            ret.imp().angle.set(angle);
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
        *self.0.imp().name.borrow_mut() = name;
        self
    }

    pub fn identifier(self, identifier: Option<String>) -> Self {
        *self.0.imp().identifier.borrow_mut() = identifier;
        self
    }

    pub fn color(self, color: Option<String>) -> Self {
        if let Some(color) = color.as_deref().and_then(Color::try_parse) {
            self.0.imp().color.set(color);
        }
        self
    }

    pub fn angle(self, angle: f64) -> Self {
        self.0.imp().angle.set(angle);
        self
    }

    pub fn x(self, x: f64) -> Self {
        self.0.imp().x.set(x);
        self
    }

    pub fn y(self, y: f64) -> Self {
        self.0.imp().y.set(y);
        self
    }

    pub fn build(self) -> Guideline {
        self.0
    }
}
