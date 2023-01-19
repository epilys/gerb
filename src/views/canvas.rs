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

mod layers;
mod transformation;
use crate::utils::Point;
pub use layers::*;
pub use transformation::*;

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecObject, Value};

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const RULER_BREADTH: f64 = 13.0;
#[derive(Debug, Copy, Clone, Default)]
#[repr(transparent)]
pub struct UnitPoint(pub Point);

#[derive(Debug, Copy, Clone, Default)]
#[repr(transparent)]
pub struct ViewPoint(pub Point);

#[derive(Debug, Default)]
pub struct CanvasInner {
    pub show_grid: Cell<bool>,
    pub show_guidelines: Cell<bool>,
    pub show_handles: Cell<bool>,
    pub inner_fill: Cell<bool>,
    pub transformation: Transformation,
    pub show_total_area: Cell<bool>,
    pub show_rulers: Cell<bool>,
    pub warp_cursor: Cell<bool>,
    view_height: Cell<f64>,
    view_width: Cell<f64>,
    mouse: Cell<ViewPoint>,
    pub pre_layers: Rc<RefCell<Vec<Layer>>>,
    pub layers: Rc<RefCell<Vec<Layer>>>,
    pub post_layers: Rc<RefCell<Vec<Layer>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for CanvasInner {
    const NAME: &'static str = "Canvas";
    type Type = Canvas;
    type ParentType = gtk::DrawingArea;
}

impl ObjectImpl for CanvasInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.show_grid.set(true);
        self.show_guidelines.set(true);
        self.show_handles.set(true);
        self.inner_fill.set(false);
        self.show_total_area.set(true);
        self.show_rulers.set(true);
        self.warp_cursor.set(true);
        self.pre_layers.borrow_mut().push(
            LayerBuilder::new()
                .set_name(Some("grid"))
                .set_active(true)
                .set_hidden(true)
                .set_callback(Some(Box::new(Canvas::draw_grid)))
                .build(),
        );
        obj.set_tooltip_text(None);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK
                | gtk::gdk::EventMask::BUTTON_RELEASE_MASK
                | gtk::gdk::EventMask::BUTTON_MOTION_MASK
                | gtk::gdk::EventMask::SCROLL_MASK
                | gtk::gdk::EventMask::SMOOTH_SCROLL_MASK
                | gtk::gdk::EventMask::POINTER_MOTION_MASK,
        );
        obj.connect_size_allocate(|self_, _rect| {
            self_.set_property::<f64>(Canvas::VIEW_HEIGHT, self_.allocated_height() as f64);
            self_.set_property::<f64>(Canvas::VIEW_WIDTH, self_.allocated_width() as f64);
        });
        obj.connect_draw(
            move |viewport: &Canvas, cr: &gtk::cairo::Context| -> Inhibit {
                let mut retval = Inhibit(false);
                for layer in viewport
                    .imp()
                    .pre_layers
                    .borrow()
                    .iter()
                    .chain(viewport.imp().layers.borrow().iter())
                    .chain(viewport.imp().post_layers.borrow().iter())
                    .filter(Layer::is_active)
                {
                    retval = (layer.get_callback())(viewport, cr);
                }
                retval
            },
        );
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        Canvas::SHOW_GRID,
                        Canvas::SHOW_GRID,
                        Canvas::SHOW_GRID,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_GUIDELINES,
                        Canvas::SHOW_GUIDELINES,
                        Canvas::SHOW_GUIDELINES,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_HANDLES,
                        Canvas::SHOW_HANDLES,
                        Canvas::SHOW_HANDLES,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::INNER_FILL,
                        Canvas::INNER_FILL,
                        Canvas::INNER_FILL,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecObject::new(
                        Canvas::TRANSFORMATION,
                        Canvas::TRANSFORMATION,
                        Canvas::TRANSFORMATION,
                        Transformation::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_TOTAL_AREA,
                        Canvas::SHOW_TOTAL_AREA,
                        Canvas::SHOW_TOTAL_AREA,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_RULERS,
                        Canvas::SHOW_RULERS,
                        Canvas::SHOW_RULERS,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::WARP_CURSOR,
                        Canvas::WARP_CURSOR,
                        Canvas::WARP_CURSOR,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::VIEW_HEIGHT,
                        Canvas::VIEW_HEIGHT,
                        Canvas::VIEW_HEIGHT,
                        std::f64::MIN,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::VIEW_WIDTH,
                        Canvas::VIEW_WIDTH,
                        Canvas::VIEW_WIDTH,
                        std::f64::MIN,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::RULER_BREADTH_PIXELS,
                        Canvas::RULER_BREADTH_PIXELS,
                        Canvas::RULER_BREADTH_PIXELS,
                        0.0,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Canvas::SHOW_GRID => self.show_grid.get().to_value(),
            Canvas::SHOW_GUIDELINES => self.show_guidelines.get().to_value(),
            Canvas::SHOW_HANDLES => self.show_handles.get().to_value(),
            Canvas::INNER_FILL => self.inner_fill.get().to_value(),
            Canvas::TRANSFORMATION => self.transformation.to_value(),
            Canvas::SHOW_TOTAL_AREA => self.show_total_area.get().to_value(),
            Canvas::SHOW_RULERS => self.show_rulers.get().to_value(),
            Canvas::WARP_CURSOR => self.warp_cursor.get().to_value(),
            Canvas::VIEW_HEIGHT => (self.instance().allocated_height() as f64).to_value(),
            Canvas::VIEW_WIDTH => (self.instance().allocated_width() as f64).to_value(),
            Canvas::RULER_BREADTH_PIXELS => RULER_BREADTH.to_value(),
            /*Canvas::RULER_BREADTH_UNITS => {
                let ppu = self
                    .transformation
                    .property::<f64>(Transformation::PIXELS_PER_UNIT);
                let scale: f64 = self.transformation.property::<f64>(Transformation::SCALE);
                (RULER_BREADTH / (scale * ppu)).to_value()
            }*/
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Canvas::SHOW_GRID => {
                self.show_grid.set(value.get().unwrap());
            }
            Canvas::SHOW_GUIDELINES => {
                self.show_guidelines.set(value.get().unwrap());
            }
            Canvas::SHOW_HANDLES => {
                self.show_handles.set(value.get().unwrap());
            }
            Canvas::INNER_FILL => {
                self.inner_fill.set(value.get().unwrap());
            }
            Canvas::TRANSFORMATION => {}
            Canvas::SHOW_TOTAL_AREA => {
                self.show_total_area.set(value.get().unwrap());
            }
            Canvas::SHOW_RULERS => {
                self.show_rulers.set(value.get().unwrap());
            }
            Canvas::WARP_CURSOR => {
                self.warp_cursor.set(value.get().unwrap());
            }
            Canvas::VIEW_WIDTH => {
                self.view_width.set(value.get().unwrap());
            }
            Canvas::VIEW_HEIGHT => {
                self.view_height.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl CanvasInner {}

impl DrawingAreaImpl for CanvasInner {}
impl WidgetImpl for CanvasInner {}

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<CanvasInner>)
        @extends gtk::DrawingArea, gtk::Widget;
}

impl Canvas {
    pub const INNER_FILL: &str = "inner-fill";
    pub const VIEW_HEIGHT: &str = "view-height";
    pub const VIEW_WIDTH: &str = "view-width";
    pub const SHOW_GRID: &str = "show-grid";
    pub const SHOW_GUIDELINES: &str = "show-guidelines";
    pub const SHOW_HANDLES: &str = "show-handles";
    pub const SHOW_TOTAL_AREA: &str = "show-total-area";
    pub const SHOW_RULERS: &str = "show-rules";
    pub const TRANSFORMATION: &str = "transformation";
    pub const WARP_CURSOR: &str = "warp-cursor";
    pub const MOUSE: &str = "mouse";
    pub const RULER_BREADTH_PIXELS: &str = "ruler-breadth-pixels";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Canvas");
        for prop in [Self::VIEW_WIDTH, Self::VIEW_HEIGHT] {
            ret.bind_property(prop, &ret.imp().transformation, prop)
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
                .build();
        }
        ret
    }

    pub fn add_pre_layer(&self, new_layer: Layer) {
        self.imp().pre_layers.borrow_mut().push(new_layer);
    }

    pub fn add_layer(&self, new_layer: Layer) {
        self.imp().layers.borrow_mut().push(new_layer);
    }

    pub fn add_post_layer(&self, new_layer: Layer) {
        self.imp().post_layers.borrow_mut().push(new_layer);
    }

    pub fn draw_grid(&self, cr: &gtk::cairo::Context) -> Inhibit {
        if !self.property::<bool>(Canvas::SHOW_GRID) {
            return Inhibit(false);
        }

        let scale: f64 = self
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu = self
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let width: f64 = self.property::<f64>(Canvas::VIEW_WIDTH);
        let height: f64 = self.property::<f64>(Canvas::VIEW_HEIGHT);
        let ViewPoint(camera) = self.imp().transformation.camera();

        cr.save().unwrap();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();
        cr.set_line_width(1.5);

        for &(color, step) in &[(0.9, 50.0 * scale * ppu), (0.8, 200.0 * scale * ppu)] {
            cr.set_source_rgb(color, color, color);
            let mut y = camera.y.rem_euclid(step).floor() + 0.5;
            while y < height {
                cr.move_to(0.0, y);
                cr.line_to(width, y);
                y += step;
            }
            cr.stroke().unwrap();
            let mut x = camera.x.rem_euclid(step).floor() + 0.5;
            while x < width {
                cr.move_to(x, 0.0);
                cr.line_to(x, height);
                x += step;
            }
            cr.stroke().unwrap();
        }
        cr.restore().unwrap();
        Inhibit(false)
    }

    pub fn draw_rulers(&self, cr: &gtk::cairo::Context) -> Inhibit {
        if !self.imp().show_rulers.get() {
            return Inhibit(false);
        }
        let width: f64 = self.property::<f64>(Canvas::VIEW_WIDTH);
        let height: f64 = self.property::<f64>(Canvas::VIEW_HEIGHT);
        let ruler_breadth: f64 = self.property::<f64>(Canvas::RULER_BREADTH_PIXELS);
        let scale = self
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu = self
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let ViewPoint(camera) = self.imp().transformation.camera();

        cr.save().unwrap();
        cr.set_line_width(1.0);

        let font_size = 6.0;
        let v @ ViewPoint(view_mouse) = self.get_mouse();
        let UnitPoint(mouse) = self.view_to_unit_point(v);

        /* Draw rulers */

        cr.save().unwrap();
        cr.rectangle(0.0, ruler_breadth, ruler_breadth, height);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.fill_preserve().expect("Invalid cairo surface state");
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke_preserve().unwrap();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(0.0, view_mouse.y);
        cr.line_to(ruler_breadth, view_mouse.y);
        cr.stroke().unwrap();
        cr.move_to(2.0 * ruler_breadth / 3.0, view_mouse.y - 1.0);
        cr.set_font_size(font_size);
        cr.rotate(-std::f64::consts::FRAC_PI_2);
        cr.show_text(&format!("{:.0}", mouse.y)).unwrap();
        cr.restore().expect("Invalid cairo surface state");

        cr.save().unwrap();
        cr.rectangle(0.0, 0.0, width, ruler_breadth);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.fill_preserve().expect("Invalid cairo surface state");
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke_preserve().unwrap();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke().unwrap();
        cr.arc(
            camera.x,
            camera.y,
            5.0 + 1.0,
            0.,
            2.0 * std::f64::consts::PI,
        );
        cr.stroke().unwrap();
        let step: f64 = 200.0 * (scale * ppu);
        let mut x = camera.x.rem_euclid(step).floor() + 0.5;
        while x < width {
            cr.move_to(x, 0.0);
            cr.line_to(x, ruler_breadth);
            cr.stroke().unwrap();
            for i in 1..10 {
                cr.move_to(x + i as f64 * step / 10.0, ruler_breadth / 2.0);
                cr.line_to(x + i as f64 * step / 10.0, ruler_breadth);
                cr.stroke().unwrap();
            }
            x += step;
        }
        cr.move_to(view_mouse.x, 0.0);
        cr.line_to(view_mouse.x, ruler_breadth);
        cr.stroke().unwrap();
        cr.move_to(view_mouse.x + 1.0, 2.0 * ruler_breadth / 3.0);
        cr.set_font_size(font_size);
        cr.show_text(&format!("{:.0}", mouse.x)).unwrap();
        cr.restore().expect("Invalid cairo surface state");

        cr.restore().unwrap();

        Inhibit(false)
    }

    pub fn view_to_unit_point(&self, ViewPoint(viewpoint): ViewPoint) -> UnitPoint {
        let mut matrix = self.imp().transformation.matrix();
        matrix.invert();
        UnitPoint(matrix.transform_point(viewpoint.x, viewpoint.y).into())
    }

    pub fn unit_to_view_point(&self, UnitPoint(unitpoint): UnitPoint) -> ViewPoint {
        ViewPoint(
            self.imp()
                .transformation
                .matrix()
                .transform_point(unitpoint.x, unitpoint.y)
                .into(),
        )
    }

    pub fn set_mouse(&self, new_value: ViewPoint) {
        self.imp().mouse.set(new_value);
    }

    pub fn get_mouse(&self) -> ViewPoint {
        self.imp().mouse.get()
    }

    pub fn set_cursor(&self, name: &str) {
        if let Some(screen) = self.window() {
            let display = screen.display();
            screen.set_cursor(Some(&gtk::gdk::Cursor::from_name(&display, name).unwrap()));
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}
