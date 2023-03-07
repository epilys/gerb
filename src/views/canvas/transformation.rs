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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, Value};
use gtk::cairo::Matrix;

use crate::prelude::*;

#[derive(Debug, Default)]
pub struct TransformationInner {
    scale: Cell<f64>,
    previous_scale: Cell<Option<f64>>,
    /// In units (not scaled).
    camera_x: Cell<f64>,
    /// In units (not scaled).
    camera_y: Cell<f64>,
    pixels_per_unit: Cell<f64>,
    units_per_em: Cell<f64>,
    view_height: Cell<f64>,
    view_width: Cell<f64>,
    centered: Cell<bool>,
    fit_view: Cell<bool>,
    content_width: Cell<f64>,
}

impl TransformationInner {
    const INIT_SCALE_VAL: f64 = 1.0;
    const INIT_CAMERA_X_VAL: f64 = 0.0;
    const INIT_CAMERA_Y_VAL: f64 = 0.0;
    const INIT_UNITS_PER_EM_VAL: f64 = ufo::constants::UNITS_PER_EM;
    const INIT_PIXELS_PER_UNIT_VAL: f64 = 200.0 / Self::INIT_UNITS_PER_EM_VAL;
    const INIT_VIEW_WIDTH_VAL: f64 = 800.0;
    const INIT_VIEW_HEIGHT_VAL: f64 = 600.0;
    const EM_SQUARE_PIXELS: f64 = 200.0;
}

#[glib::object_subclass]
impl ObjectSubclass for TransformationInner {
    const NAME: &'static str = "Transformation";
    type Type = Transformation;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for TransformationInner {
    fn constructed(&self, _obj: &Self::Type) {
        self.scale.set(Self::INIT_SCALE_VAL);
        self.camera_x.set(Self::INIT_CAMERA_X_VAL);
        self.camera_y.set(Self::INIT_CAMERA_Y_VAL);
        self.centered.set(true);
        self.fit_view.set(true);
        self.units_per_em.set(Self::INIT_UNITS_PER_EM_VAL);
        self.pixels_per_unit.set(Self::INIT_PIXELS_PER_UNIT_VAL);
        self.view_width.set(Self::INIT_VIEW_WIDTH_VAL);
        self.view_height.set(Self::INIT_VIEW_HEIGHT_VAL);
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
                        20.0,
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
                        TransformationInner::INIT_PIXELS_PER_UNIT_VAL,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::UNITS_PER_EM,
                        Transformation::UNITS_PER_EM,
                        Transformation::UNITS_PER_EM,
                        std::f64::MIN,
                        std::f64::MAX,
                        TransformationInner::INIT_UNITS_PER_EM_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::VIEW_HEIGHT,
                        Transformation::VIEW_HEIGHT,
                        Transformation::VIEW_HEIGHT,
                        std::f64::MIN,
                        std::f64::MAX,
                        TransformationInner::INIT_VIEW_HEIGHT_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::VIEW_WIDTH,
                        Transformation::VIEW_WIDTH,
                        Transformation::VIEW_WIDTH,
                        std::f64::MIN,
                        std::f64::MAX,
                        TransformationInner::INIT_VIEW_WIDTH_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Transformation::CENTERED,
                        Transformation::CENTERED,
                        Transformation::CENTERED,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Transformation::FIT_VIEW,
                        Transformation::FIT_VIEW,
                        Transformation::FIT_VIEW,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Transformation::CONTENT_WIDTH,
                        Transformation::CONTENT_WIDTH,
                        Transformation::CONTENT_WIDTH,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            Transformation::SCALE => self.scale.get().to_value(),
            Transformation::CAMERA_X => self.camera_x.get().to_value(),
            Transformation::CAMERA_Y => self.camera_y.get().to_value(),
            Transformation::PIXELS_PER_UNIT => self.pixels_per_unit.get().to_value(),
            Transformation::VIEW_HEIGHT => self.view_height.get().to_value(),
            Transformation::VIEW_WIDTH => self.view_width.get().to_value(),
            Transformation::UNITS_PER_EM => self.units_per_em.get().to_value(),
            Transformation::CENTERED => self.centered.get().to_value(),
            Transformation::FIT_VIEW => self.fit_view.get().to_value(),
            Transformation::CONTENT_WIDTH => self.content_width.get().to_value(),
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
            Transformation::UNITS_PER_EM => {
                let units_per_em: f64 = value.get().unwrap();
                self.pixels_per_unit
                    .set(Self::EM_SQUARE_PIXELS / units_per_em);
                self.units_per_em.set(units_per_em);
                if self.centered.get() {
                    self.instance().center_camera();
                }
                if self.fit_view.get() {
                    self.instance().fit_view();
                }
            }
            Transformation::VIEW_HEIGHT => {
                self.view_height.set(value.get().unwrap());
                if self.centered.get() {
                    self.instance().center_camera();
                }
                if self.fit_view.get() {
                    self.instance().fit_view();
                }
            }
            Transformation::VIEW_WIDTH => {
                self.view_width.set(value.get().unwrap());
                if self.centered.get() {
                    self.instance().center_camera();
                }
                if self.fit_view.get() {
                    self.instance().fit_view();
                }
            }
            Transformation::CONTENT_WIDTH => {
                self.content_width.set(value.get().unwrap());
                if self.centered.get() {
                    self.instance().center_camera();
                }
            }
            Transformation::CENTERED => {
                let val = value.get().unwrap();
                if val {
                    self.instance().center_camera();
                    if self.fit_view.get() {
                        self.instance().fit_view();
                    }
                }
                self.centered.set(val);
            }
            Transformation::FIT_VIEW => {
                let val = value.get().unwrap();
                if val {
                    self.instance().fit_view();
                    if self.centered.get() {
                        self.instance().center_camera();
                    }
                }
                self.fit_view.set(val);
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct Transformation(ObjectSubclass<TransformationInner>);
}

impl std::ops::Deref for Transformation {
    type Target = TransformationInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Transformation {
    pub const CENTERED: &str = "centered";
    pub const FIT_VIEW: &str = "fit-view";
    pub const VIEW_HEIGHT: &str = super::Canvas::VIEW_HEIGHT;
    pub const VIEW_WIDTH: &str = super::Canvas::VIEW_WIDTH;
    pub const CONTENT_WIDTH: &str = super::Canvas::CONTENT_WIDTH;
    pub const SCALE: &str = "scale";
    pub const CAMERA_X: &str = "camera-x";
    pub const CAMERA_Y: &str = "camera-y";
    pub const PIXELS_PER_UNIT: &str = "pixels-per-unit";
    pub const UNITS_PER_EM: &str = Project::UNITS_PER_EM;

    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Transformation");
        ret
    }

    fn center_camera(&self) {
        let width: f64 = self.property::<f64>(Self::VIEW_WIDTH);
        let height: f64 = self.property::<f64>(Self::VIEW_HEIGHT);
        let units_per_em = self.property::<f64>(Self::UNITS_PER_EM);
        let content_width = self.property::<f64>(Self::CONTENT_WIDTH);
        let ppu = self.property::<f64>(Self::PIXELS_PER_UNIT);
        let half_unit = (ppu * content_width, ppu * units_per_em / 4.0);
        let (x, y) = (width / 2.0 - half_unit.0, height / 2.0 + half_unit.1 * 2.0);
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        self.set_property::<f64>(Self::CAMERA_X, x);
        self.set_property::<f64>(Self::CAMERA_Y, y);
    }

    fn fit_view(&self) {
        let height: f64 = self.property::<f64>(Self::VIEW_HEIGHT);
        let units_per_em = self.property::<f64>(Self::UNITS_PER_EM);
        let ppu = self.property::<f64>(Self::PIXELS_PER_UNIT);
        _ = self.try_set_property::<f64>(Self::SCALE, 0.8 * height / (units_per_em * ppu));
    }

    pub fn matrix(&self) -> Matrix {
        let mut retval = Matrix::identity();
        let factor = self.pixels_per_unit.get() * self.scale.get();
        retval.translate(
            self.property::<f64>(Self::CAMERA_X),
            self.property::<f64>(Self::CAMERA_Y),
        );
        if factor.is_finite() {
            retval.scale(factor, factor);
        }
        retval.scale(1.0, -1.0);
        retval
    }

    pub fn camera(&self) -> ViewPoint {
        ViewPoint(
            (
                self.property::<f64>(Self::CAMERA_X),
                self.property::<f64>(Self::CAMERA_Y),
            )
                .into(),
        )
    }

    pub fn set_camera(&self, ViewPoint(new_value): ViewPoint) -> ViewPoint {
        self.set_property::<bool>(Self::CENTERED, false);
        self.set_property::<bool>(Self::FIT_VIEW, false);
        let oldval = self.camera();
        self.set_property::<f64>(Self::CAMERA_X, new_value.x);
        self.set_property::<f64>(Self::CAMERA_Y, new_value.y);
        oldval
    }

    pub fn move_camera_by_delta(&self, delta: ViewPoint) -> ViewPoint {
        self.set_property::<bool>(Self::CENTERED, false);
        self.set_property::<bool>(Self::FIT_VIEW, false);
        let mut camera = self.camera();
        camera.0 = camera.0 + delta.0;
        self.set_camera(camera)
    }

    pub fn set_zoom(&self, new_value: f64) -> bool {
        if !(0.2..=20.0).contains(&new_value) {
            return false;
        }
        self.set_property::<f64>(Self::SCALE, new_value);

        true
    }

    pub fn zoom_in(&self) -> bool {
        self.set_property::<bool>(Self::FIT_VIEW, false);
        self.set_zoom(self.property::<f64>(Transformation::SCALE) + 0.1)
    }

    pub fn zoom_out(&self) -> bool {
        self.set_property::<bool>(Self::FIT_VIEW, false);
        self.set_zoom(self.property::<f64>(Transformation::SCALE) - 0.1)
    }

    pub fn reset_zoom(&self) {
        let previous_value = self.previous_scale.get();
        let current_value = self.property::<f64>(Transformation::SCALE);
        match previous_value {
            None if current_value == 1.0 => {}
            Some(v) if current_value == 1.0 => {
                self.set_zoom(v);
            }
            _ => {
                self.set_zoom(1.0);
                self.previous_scale.set(Some(current_value));
            }
        }
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new()
    }
}
