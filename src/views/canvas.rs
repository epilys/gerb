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
mod settings;
mod transformation;
use crate::prelude::*;
pub use layers::*;
pub use settings::*;
pub use transformation::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct UnitPoint(pub Point);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ViewPoint(pub Point);

#[derive(Debug, Default)]
pub struct CanvasInner {
    pub handle_size: Cell<f64>,
    pub line_width: Cell<f64>,
    pub show_grid: Cell<bool>,
    pub show_guidelines: Cell<bool>,
    pub show_handles: Cell<bool>,
    pub show_direction: Cell<bool>,
    pub show_outline: Cell<bool>,
    pub inner_fill: Cell<bool>,
    pub transformation: Transformation,
    pub show_total_area: Cell<bool>,
    pub show_rulers: Cell<bool>,
    pub warp_cursor: Cell<bool>,
    view_height: Cell<f64>,
    view_width: Cell<f64>,
    content_width: Cell<f64>,
    mouse: Cell<ViewPoint>,
    pub pre_layers: Rc<RefCell<Vec<Layer>>>,
    pub layers: Rc<RefCell<Vec<Layer>>>,
    pub post_layers: Rc<RefCell<Vec<Layer>>>,
    pub bg_color: Cell<Color>,
    pub glyph_bbox_bg_color: Cell<Color>,
    pub glyph_inner_fill_color: Cell<Color>,
    pub ruler_fg_color: Cell<Color>,
    pub ruler_indicator_color: Cell<Color>,
    pub ruler_bg_color: Cell<Color>,
    pub outline_options: Cell<DrawOptions>,
    pub handle_options: Cell<DrawOptions>,
    pub smooth_corner_options: Cell<DrawOptions>,
    pub corner_options: Cell<DrawOptions>,
    pub direction_options: Cell<DrawOptions>,
    pub handle_connection_options: Cell<DrawOptions>,
}

impl CanvasInner {
    fn get_opts(&self, retval: DrawOptions) -> DrawOptions {
        if let Some((inherit, true)) = retval.inherit_size {
            DrawOptions {
                size: self.instance().property(inherit),
                ..retval
            }
        } else {
            retval
        }
    }
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
            self_.set_property::<f64>(Canvas::VIEW_HEIGHT, f64::from(self_.allocated_height()));
            self_.set_property::<f64>(Canvas::VIEW_WIDTH, f64::from(self_.allocated_width()));
        });
        obj.connect_draw(
            move |viewport: &Canvas, mut cr: &gtk::cairo::Context| -> Inhibit {
                let mut retval = Inhibit(false);
                for layer in viewport
                    .pre_layers
                    .borrow()
                    .iter()
                    .chain(viewport.layers.borrow().iter())
                    .chain(viewport.post_layers.borrow().iter())
                    .filter(Layer::is_active)
                {
                    retval = (layer.get_callback())(viewport, cr.push());
                }
                retval
            },
        );
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> = once_cell::sync::Lazy::new(
            || {
                vec![
                    ParamSpecDouble::new(
                        Canvas::HANDLE_SIZE,
                        Canvas::HANDLE_SIZE,
                        "Diameter of round control point handle.",
                        0.0001,
                        10.0,
                        CanvasSettingsInner::HANDLE_SIZE_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::LINE_WIDTH,
                        Canvas::LINE_WIDTH,
                        "Width of lines in pixels.",
                        0.0001,
                        10.0,
                        CanvasSettingsInner::LINE_WIDTH_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_GRID,
                        Canvas::SHOW_GRID,
                        "Show/hide grid.",
                        CanvasSettingsInner::SHOW_GRID_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_GUIDELINES,
                        Canvas::SHOW_GUIDELINES,
                        "Show/hide all guidelines.",
                        CanvasSettingsInner::SHOW_GUIDELINES_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_HANDLES,
                        Canvas::SHOW_HANDLES,
                        "Show/hide handles.",
                        CanvasSettingsInner::SHOW_HANDLES_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::INNER_FILL,
                        Canvas::INNER_FILL,
                        "Show/hide inner glyph fill.",
                        CanvasSettingsInner::INNER_FILL_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
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
                        "Show/hide total glyph area.",
                        CanvasSettingsInner::SHOW_TOTAL_AREA_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_RULERS,
                        Canvas::SHOW_RULERS,
                        "Show/hide canvas rulers.",
                        CanvasSettingsInner::SHOW_RULERS_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::SHOW_DIRECTION,
                        Canvas::SHOW_DIRECTION,
                        "Show/hide contour direction arrows.",
                        CanvasSettingsInner::SHOW_DIRECTION_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Canvas::WARP_CURSOR,
                        Canvas::WARP_CURSOR,
                        Canvas::WARP_CURSOR,
                        CanvasSettingsInner::WARP_CURSOR_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::VIEW_HEIGHT,
                        Canvas::VIEW_HEIGHT,
                        Canvas::VIEW_HEIGHT,
                        f64::MIN,
                        f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::VIEW_WIDTH,
                        Canvas::VIEW_WIDTH,
                        Canvas::VIEW_WIDTH,
                        f64::MIN,
                        f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::CONTENT_WIDTH,
                        Canvas::CONTENT_WIDTH,
                        Canvas::CONTENT_WIDTH,
                        f64::MIN,
                        f64::MAX,
                        0.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        Canvas::RULER_BREADTH_PIXELS,
                        Canvas::RULER_BREADTH_PIXELS,
                        Canvas::RULER_BREADTH_PIXELS,
                        0.0,
                        f64::MAX,
                        0.0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::BG_COLOR,
                        Canvas::BG_COLOR,
                        "Background color of canvas.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::GLYPH_INNER_FILL_COLOR,
                        Canvas::GLYPH_INNER_FILL_COLOR,
                        "Color of glyph's inner fill.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::GLYPH_BBOX_BG_COLOR,
                        Canvas::GLYPH_BBOX_BG_COLOR,
                        "Background color of glyph's bounding box (total area).",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::RULER_FG_COLOR,
                        Canvas::RULER_FG_COLOR,
                        "Foreground color of canvas rulers.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::RULER_BG_COLOR,
                        Canvas::RULER_BG_COLOR,
                        "Background color of canvas rulers.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::RULER_INDICATOR_COLOR,
                        Canvas::RULER_INDICATOR_COLOR,
                        "Color of mouse pointer in ruler.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::DIRECTION_OPTIONS,
                        Canvas::DIRECTION_OPTIONS,
                        "Theming options of contour direction arrow.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::HANDLE_CONNECTION_OPTIONS,
                        Canvas::HANDLE_CONNECTION_OPTIONS,
                        "Theming options of handle connections (lines between handle and on-curve points).",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::HANDLE_OPTIONS,
                        Canvas::HANDLE_OPTIONS,
                        "Theming options of handles.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::SMOOTH_CORNER_OPTIONS,
                        Canvas::SMOOTH_CORNER_OPTIONS,
                        "Theming options of smooth (non-positional continuity) on-curve points.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::CORNER_OPTIONS,
                        Canvas::CORNER_OPTIONS,
                        "Theming options of positional continuity on-curve points.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        Canvas::OUTLINE_OPTIONS,
                        Canvas::OUTLINE_OPTIONS,
                        "Theming options of glyph outline.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            },
        );
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Canvas::HANDLE_SIZE => self.handle_size.get().to_value(),
            Canvas::LINE_WIDTH => self.line_width.get().to_value(),
            Canvas::SHOW_GRID => self.show_grid.get().to_value(),
            Canvas::SHOW_GUIDELINES => self.show_guidelines.get().to_value(),
            Canvas::SHOW_HANDLES => self.show_handles.get().to_value(),
            Canvas::INNER_FILL => self.inner_fill.get().to_value(),
            Canvas::TRANSFORMATION => self.transformation.to_value(),
            Canvas::SHOW_TOTAL_AREA => self.show_total_area.get().to_value(),
            Canvas::SHOW_RULERS => self.show_rulers.get().to_value(),
            Canvas::WARP_CURSOR => self.warp_cursor.get().to_value(),
            Canvas::VIEW_HEIGHT => (f64::from(self.instance().allocated_height())).to_value(),
            Canvas::VIEW_WIDTH => (f64::from(self.instance().allocated_width())).to_value(),
            Canvas::CONTENT_WIDTH => self.content_width.get().to_value(),
            Canvas::BG_COLOR => self.bg_color.get().to_value(),
            Canvas::GLYPH_INNER_FILL_COLOR => self.glyph_inner_fill_color.get().to_value(),
            Canvas::GLYPH_BBOX_BG_COLOR => self.glyph_bbox_bg_color.get().to_value(),
            Canvas::RULER_BREADTH_PIXELS => CanvasSettingsInner::RULER_BREADTH.to_value(),
            Canvas::RULER_FG_COLOR => self.ruler_fg_color.get().to_value(),
            Canvas::RULER_BG_COLOR => self.ruler_bg_color.get().to_value(),
            Canvas::RULER_INDICATOR_COLOR => self.ruler_indicator_color.get().to_value(),
            Canvas::SHOW_DIRECTION => self.show_direction.get().to_value(),
            Canvas::HANDLE_OPTIONS => { self.get_opts(self.handle_options.get()) }.to_value(),
            Canvas::SMOOTH_CORNER_OPTIONS => {
                { self.get_opts(self.smooth_corner_options.get()) }.to_value()
            }
            Canvas::CORNER_OPTIONS => { self.get_opts(self.corner_options.get()) }.to_value(),
            Canvas::DIRECTION_OPTIONS => { self.get_opts(self.direction_options.get()) }.to_value(),
            Canvas::HANDLE_CONNECTION_OPTIONS => {
                { self.get_opts(self.handle_connection_options.get()) }.to_value()
            }
            Canvas::OUTLINE_OPTIONS => { self.get_opts(self.outline_options.get()) }.to_value(),
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

    fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        obj.queue_draw();
        match pspec.name() {
            Canvas::HANDLE_SIZE => {
                self.handle_size.set(value.get().unwrap());
            }
            Canvas::LINE_WIDTH => {
                self.line_width.set(value.get().unwrap());
            }
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
            Canvas::CONTENT_WIDTH => {
                self.content_width.set(value.get().unwrap());
            }
            Canvas::BG_COLOR => {
                self.bg_color.set(value.get().unwrap());
            }
            Canvas::GLYPH_INNER_FILL_COLOR => {
                self.glyph_inner_fill_color.set(value.get().unwrap());
            }
            Canvas::GLYPH_BBOX_BG_COLOR => {
                self.glyph_bbox_bg_color.set(value.get().unwrap());
            }
            Canvas::RULER_FG_COLOR => {
                self.ruler_fg_color.set(value.get().unwrap());
            }
            Canvas::RULER_BG_COLOR => {
                self.ruler_bg_color.set(value.get().unwrap());
            }
            Canvas::RULER_INDICATOR_COLOR => {
                self.ruler_indicator_color.set(value.get().unwrap());
            }
            Canvas::SHOW_DIRECTION => {
                self.show_direction.set(value.get().unwrap());
            }
            Canvas::HANDLE_OPTIONS => {
                self.handle_options.set(value.get().unwrap());
            }
            Canvas::SMOOTH_CORNER_OPTIONS => {
                self.smooth_corner_options.set(value.get().unwrap());
            }
            Canvas::CORNER_OPTIONS => {
                self.corner_options.set(value.get().unwrap());
            }
            Canvas::DIRECTION_OPTIONS => {
                self.direction_options.set(value.get().unwrap());
            }
            Canvas::HANDLE_CONNECTION_OPTIONS => {
                self.handle_connection_options.set(value.get().unwrap());
            }
            Canvas::OUTLINE_OPTIONS => {
                self.outline_options.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl DrawingAreaImpl for CanvasInner {}
impl WidgetImpl for CanvasInner {}

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<CanvasInner>)
        @extends gtk::DrawingArea, gtk::Widget;
}

impl std::ops::Deref for Canvas {
    type Target = CanvasInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Canvas {
    inherit_property!(
        CanvasSettings,
        HANDLE_SIZE,
        LINE_WIDTH,
        INNER_FILL,
        VIEW_HEIGHT,
        VIEW_WIDTH,
        SHOW_GRID,
        SHOW_GUIDELINES,
        SHOW_HANDLES,
        SHOW_DIRECTION,
        HANDLE_OPTIONS,
        SMOOTH_CORNER_OPTIONS,
        CORNER_OPTIONS,
        DIRECTION_OPTIONS,
        HANDLE_CONNECTION_OPTIONS,
        OUTLINE_OPTIONS,
        SHOW_TOTAL_AREA,
        SHOW_RULERS,
        TRANSFORMATION,
        WARP_CURSOR,
        MOUSE,
        BG_COLOR,
        GLYPH_INNER_FILL_COLOR,
        GLYPH_BBOX_BG_COLOR,
        RULER_BREADTH_PIXELS,
        RULER_FG_COLOR,
        RULER_BG_COLOR,
        RULER_INDICATOR_COLOR,
        CONTENT_WIDTH,
    );

    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Canvas");
        for prop in [Self::VIEW_WIDTH, Self::VIEW_HEIGHT] {
            ret.bind_property(prop, &ret.transformation, prop)
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
                .build();
        }
        ret.transformation
            .bind_property(Self::CONTENT_WIDTH, &ret, Self::CONTENT_WIDTH)
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
            .build();
        ret
    }

    pub fn add_pre_layer(&self, new_layer: Layer) {
        self.pre_layers.borrow_mut().push(new_layer);
    }

    pub fn add_layer(&self, new_layer: Layer) {
        self.layers.borrow_mut().push(new_layer);
    }

    pub fn add_post_layer(&self, new_layer: Layer) {
        self.post_layers.borrow_mut().push(new_layer);
    }

    pub fn draw_grid(&self, cr: ContextRef<'_, '_>) -> Inhibit {
        cr.set_source_color(self.property::<Color>(Self::BG_COLOR));
        cr.paint().unwrap();

        if self.property::<bool>(Self::SHOW_GRID) {
            let scale: f64 = self.transformation.property::<f64>(Transformation::SCALE);
            let ppu = self
                .transformation
                .property::<f64>(Transformation::PIXELS_PER_UNIT);
            let width: f64 = self.property::<f64>(Self::VIEW_WIDTH);
            let height: f64 = self.property::<f64>(Self::VIEW_HEIGHT);
            let ViewPoint(camera) = self.transformation.camera();

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
        }
        Inhibit(false)
    }

    pub fn draw_rulers(&self, mut cr: ContextRef<'_, '_>) -> Inhibit {
        if !self.show_rulers.get() {
            return Inhibit(false);
        }
        let width: f64 = self.property::<f64>(Self::VIEW_WIDTH);
        let height: f64 = self.property::<f64>(Self::VIEW_HEIGHT);
        let ruler_breadth: f64 = self.property::<f64>(Self::RULER_BREADTH_PIXELS);
        let scale = self.transformation.property::<f64>(Transformation::SCALE);
        let ppu = self
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let (ruler_fg, ruler_bg, ruler_indicator_color) = (
            self.property::<Color>(Self::RULER_FG_COLOR),
            self.property::<Color>(Self::RULER_BG_COLOR),
            self.property::<Color>(Self::RULER_INDICATOR_COLOR),
        );
        let ViewPoint(camera) = self.transformation.camera();

        cr.set_line_width(1.0);

        let font_size = 6.0;
        let v @ ViewPoint(view_mouse) = self.get_mouse();
        let UnitPoint(mouse) = self.view_to_unit_point(v);

        /* Draw rulers */

        let mut cr1 = cr.push();

        cr1.rectangle(0.0, ruler_breadth, ruler_breadth, height);
        cr1.set_source_color_alpha(ruler_bg);
        cr1.fill_preserve().expect("Invalid cairo surface state");
        cr1.set_source_color_alpha(ruler_fg);
        cr1.stroke().unwrap();

        let step: f64 = 200.0 * (scale * ppu);
        let mut y = camera.y.rem_euclid(step).floor() + 0.5;
        while y < height {
            cr1.move_to(0.0, y);
            cr1.line_to(ruler_breadth, y);
            for i in 1..10 {
                cr1.move_to(ruler_breadth / 2.0, y + f64::from(i) * step / 10.0);
                cr1.line_to(ruler_breadth, y + f64::from(i) * step / 10.0);
            }
            y += step;
        }
        cr1.stroke().unwrap();

        {
            let cr2 = cr1.push();
            cr2.move_to(2.0 * ruler_breadth / 3.0, view_mouse.y - 1.0);
            cr2.set_font_size(font_size);
            cr2.rotate(-std::f64::consts::FRAC_PI_2);
            cr2.set_source_color_alpha(ruler_fg);
            cr2.show_text_with_bg(&format!("{:.0}", mouse.y), 0.5, ruler_fg, ruler_bg);
        }

        {
            let cr3 = cr1.push();
            cr3.set_source_color_alpha(ruler_indicator_color);
            cr3.move_to(0.0, view_mouse.y);
            cr3.line_to(ruler_breadth, view_mouse.y);
            cr3.stroke().unwrap();
        }

        drop(cr1);

        let mut cr4 = cr.push();

        cr4.rectangle(0.0, 0.0, width, ruler_breadth);
        cr4.set_source_color_alpha(ruler_bg);
        cr4.fill_preserve().expect("Invalid cairo surface state");
        cr4.set_source_color_alpha(ruler_fg);
        cr4.stroke().unwrap();

        let step: f64 = 200.0 * (scale * ppu);
        let mut x = camera.x.rem_euclid(step).floor() + 0.5;
        while x < width {
            cr4.move_to(x, 0.0);
            cr4.line_to(x, ruler_breadth);
            for i in 1..10 {
                cr4.move_to(x + f64::from(i) * step / 10.0, ruler_breadth / 2.0);
                cr4.line_to(x + f64::from(i) * step / 10.0, ruler_breadth);
            }
            x += step;
        }
        cr4.stroke().unwrap();

        {
            let cr5 = cr4.push();
            cr5.move_to(view_mouse.x + 1.0, 2.0 * ruler_breadth / 3.0);
            cr5.set_font_size(font_size);
            cr5.show_text_with_bg(&format!("{:.0}", mouse.x), 0.5, ruler_fg, ruler_bg);
        }

        {
            let cr6 = cr4.push();
            cr6.set_source_color_alpha(ruler_indicator_color);
            cr6.move_to(view_mouse.x, 0.0);
            cr6.line_to(view_mouse.x, ruler_breadth);
            cr6.stroke().unwrap();
        }

        Inhibit(false)
    }

    pub fn view_to_unit_point(&self, ViewPoint(viewpoint): ViewPoint) -> UnitPoint {
        let mut matrix = self.transformation.matrix();
        matrix.invert();
        UnitPoint(matrix.transform_point(viewpoint.x, viewpoint.y).into())
    }

    pub fn unit_to_view_point(&self, UnitPoint(unitpoint): UnitPoint) -> ViewPoint {
        ViewPoint(
            self.transformation
                .matrix()
                .transform_point(unitpoint.x, unitpoint.y)
                .into(),
        )
    }

    pub fn set_mouse(&self, new_value: ViewPoint) {
        self.mouse.set(new_value);
    }

    pub fn get_mouse(&self) -> ViewPoint {
        self.mouse.get()
    }

    pub fn set_cursor(&self, name: &str) {
        if let Some(window) = self.window() {
            let display = window.display();
            window.set_cursor(Some(&gtk::gdk::Cursor::from_name(&display, name).unwrap()));
        }
    }

    pub fn set_cursor_from_pixbuf(&self, mut pixbuf: gtk::gdk_pixbuf::Pixbuf) {
        if let Some(window) = self.window() {
            let display = window.display();
            let scale_factor = window.scale_factor();
            if scale_factor == 1 {
                pixbuf = pixbuf
                    .scale_simple(16, 16, gtk::gdk_pixbuf::InterpType::Bilinear)
                    .unwrap();
            }
            if let Some(surf) = pixbuf.create_surface(scale_factor, Some(&window)) {
                window.set_cursor(Some(&gtk::gdk::Cursor::from_surface(
                    &display, &surf, 0.0, 0.0,
                )));
            } else {
                window.set_cursor(Some(&gtk::gdk::Cursor::from_pixbuf(
                    &display, &pixbuf, 0, 0,
                )));
            }
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}
