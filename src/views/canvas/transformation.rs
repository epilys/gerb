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

use crate::views::{UnitPoint, ViewPoint};
use glib::{
    clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecString, Value,
};
use gtk::cairo::Matrix;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct TransformationInner {
    scale: Cell<f64>,
    previous_scale: Cell<Option<f64>>,
    /// In units (not scaled).
    camera_x: Cell<f64>,
    /// In units (not scaled).
    camera_y: Cell<f64>,
    pixels_per_unit: Cell<f64>,
    view_height: Cell<f64>,
    view_width: Cell<f64>,
}

impl TransformationInner {
    const INIT_SCALE_VAL: f64 = 1.0;
    const INIT_CAMERA_X_VAL: f64 = 0.0;
    const INIT_CAMERA_Y_VAL: f64 = 0.0;
}

#[glib::object_subclass]
impl ObjectSubclass for TransformationInner {
    const NAME: &'static str = "TransformationInner";
    type Type = Transformation;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for TransformationInner {
    fn constructed(&self, obj: &Self::Type) {
        self.scale.set(TransformationInner::INIT_SCALE_VAL);
        self.camera_x.set(TransformationInner::INIT_CAMERA_X_VAL);
        self.camera_y.set(TransformationInner::INIT_CAMERA_Y_VAL);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecDouble::new(
                        Transformation::SCALE,
                        Transformation::SCALE,
                        Transformation::SCALE,
                        0.001,
                        10.0,
                        TransformationInner::INIT_SCALE_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::CAMERA_X,
                        Transformation::CAMERA_X,
                        Transformation::CAMERA_X,
                        std::f64::MIN,
                        std::f64::MAX,
                        TransformationInner::INIT_CAMERA_X_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::CAMERA_Y,
                        Transformation::CAMERA_Y,
                        Transformation::CAMERA_Y,
                        std::f64::MIN,
                        std::f64::MAX,
                        TransformationInner::INIT_CAMERA_Y_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::PIXELS_PER_UNIT,
                        Transformation::PIXELS_PER_UNIT,
                        Transformation::PIXELS_PER_UNIT,
                        std::f64::MIN,
                        std::f64::MAX,
                        200.0 / 1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::VIEW_HEIGHT,
                        Transformation::VIEW_HEIGHT,
                        Transformation::VIEW_HEIGHT,
                        std::f64::MIN,
                        std::f64::MAX,
                        600.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::VIEW_WIDTH,
                        Transformation::VIEW_WIDTH,
                        Transformation::VIEW_WIDTH,
                        std::f64::MIN,
                        std::f64::MAX,
                        800.0,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            Transformation::SCALE => self.scale.get().to_value(),
            Transformation::CAMERA_X => self.camera_x.get().to_value(),
            Transformation::CAMERA_Y => self.camera_y.get().to_value(),
            Transformation::PIXELS_PER_UNIT => self.pixels_per_unit.get().to_value(),
            Transformation::VIEW_HEIGHT => self.view_height.get().to_value(),
            Transformation::VIEW_WIDTH => self.view_width.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Transformation::SCALE => {
                self.scale.set(value.get().unwrap());
                self.previous_scale.set(None);
            }
            Transformation::CAMERA_X => {
                self.camera_x.set(value.get().unwrap());
            }
            Transformation::CAMERA_Y => {
                self.camera_y.set(value.get().unwrap());
            }
            Transformation::PIXELS_PER_UNIT => self.pixels_per_unit.set(value.get().unwrap()),
            Transformation::VIEW_HEIGHT => {
                self.view_height.set(value.get().unwrap());
            }
            Transformation::VIEW_WIDTH => {
                self.view_width.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct Transformation(ObjectSubclass<TransformationInner>);
}

impl Transformation {
    pub const VIEW_HEIGHT: &str = super::Canvas::VIEW_HEIGHT;
    pub const VIEW_WIDTH: &str = super::Canvas::VIEW_WIDTH;
    pub const SCALE: &str = "scale";
    pub const CAMERA_X: &str = "camera-x";
    pub const CAMERA_Y: &str = "camera-y";
    pub const PIXELS_PER_UNIT: &str = "pixels-per-unit";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Transformation");
        ret
    }

    pub fn matrix(&self) -> Matrix {
        let mut retval = Matrix::identity();
        let scale = self.imp().scale.get();
        let ppu = self.imp().pixels_per_unit.get();
        let ch = self.property::<f64>(Self::VIEW_HEIGHT);
        let cw = self.property::<f64>(Self::VIEW_WIDTH);
        retval.translate(cw / 2.0, ch / 2.0);
        retval.scale(ppu * scale, -ppu * scale);
        retval.translate(
            self.property::<f64>(Self::CAMERA_X),
            self.property::<f64>(Self::CAMERA_Y),
        );
        retval
    }

    pub fn camera(&self) -> UnitPoint {
        UnitPoint(
            (
                self.property::<f64>(Self::CAMERA_X),
                self.property::<f64>(Self::CAMERA_Y),
            )
                .into(),
        )
    }

    pub fn set_camera(&self, UnitPoint(new_value): UnitPoint) -> UnitPoint {
        let oldval = self.camera();
        self.set_property::<f64>(Self::CAMERA_X, new_value.x);
        self.set_property::<f64>(Self::CAMERA_Y, new_value.y);
        oldval
    }

    pub fn move_camera_by_delta(&self, mut delta: ViewPoint) -> UnitPoint {
        let mut camera = self.camera();
        let scale = self.imp().scale.get();
        let ppu = self.imp().pixels_per_unit.get();
        //delta.0 = delta.0 / (scale * ppu);
        let ch = self.property::<f64>(Self::VIEW_HEIGHT);
        let cw = self.property::<f64>(Self::VIEW_WIDTH);
        delta.0 /= (scale * ppu);
        delta.0.y *= -1.0;

        camera.0 = camera.0 + delta.0;
        self.set_camera(camera)
    }

    pub fn set_zoom(&self, new_value: f64) -> bool {
        if new_value < 0.2 || new_value > 5.0 {
            return false;
        }
        self.set_property::<f64>(Self::SCALE, new_value);

        true
    }

    pub fn zoom_in(&self) -> bool {
        self.set_zoom(self.property::<f64>(Transformation::SCALE) + 0.05)
    }

    pub fn zoom_out(&self) -> bool {
        self.set_zoom(self.property::<f64>(Transformation::SCALE) - 0.05)
    }

    pub fn reset_zoom(&self) {
        let previous_value = self.imp().previous_scale.get();
        let current_value = self.property::<f64>(Transformation::SCALE);
        if let (Some(v), 1.0) = (previous_value, current_value) {
            self.set_zoom(v);
        } else {
            self.set_zoom(1.0);
            self.imp().previous_scale.set(Some(current_value));
        }
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new()
    }
}
