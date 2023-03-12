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

use super::{new_contour_action, tool_impl::*};
use crate::glyphs::Contour;
use crate::utils::curves::Bezier;
use crate::views::canvas::{Layer, LayerBuilder};
use gtk::Inhibit;

use crate::prelude::*;

// [ref:needs_user_doc]
#[derive(Default)]
pub struct QuadrilateralToolInner {
    layer: OnceCell<Layer>,
    curves: Rc<RefCell<[Bezier; 4]>>,
    active: Cell<bool>,
    upper_left: Cell<Option<UnitPoint>>,
    cursor: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
}

#[glib::object_subclass]
impl ObjectSubclass for QuadrilateralToolInner {
    const NAME: &'static str = "QuadrilateralTool";
    type ParentType = ToolImpl;
    type Type = QuadrilateralTool;
}

impl ObjectImpl for QuadrilateralToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        for c in self.curves.borrow().iter() {
            c.set_property(Bezier::SMOOTH, true);
        }
        obj.set_property::<bool>(QuadrilateralTool::ACTIVE, false);
        obj.set_property::<String>(ToolImpl::NAME, "quadrilateral".to_string());
        obj.set_property::<String>(
            ToolImpl::DESCRIPTION,
            "Create quadrilateral path shape".to_string(),
        );
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::icons::RECTANGLE_ICON.to_image_widget(),
        );
        self.cursor
            .set(crate::resources::cursors::RECTANGLE_CURSOR.to_pixbuf())
            .unwrap();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    QuadrilateralTool::ACTIVE,
                    QuadrilateralTool::ACTIVE,
                    QuadrilateralTool::ACTIVE,
                    true,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            QuadrilateralTool::ACTIVE => self.active.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            QuadrilateralTool::ACTIVE => self.active.set(value.get().unwrap()),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

fn make_quadrilateral_bezier_curves(
    curves: &mut [Bezier; 4],
    (a, b, c, d): (Point, Point, Point, Point),
) {
    for c in curves.iter_mut() {
        c.clear_points();
    }
    {
        let c0 = &curves[0];
        c0.push_point(CurvePoint::new(a));
        c0.push_point(CurvePoint::new(b));
    }
    {
        let c1 = &curves[1];
        c1.push_point(CurvePoint::new(b));
        c1.push_point(CurvePoint::new(c));
    }
    {
        let c2 = &curves[2];
        c2.push_point(CurvePoint::new(c));
        c2.push_point(CurvePoint::new(d));
    }
    {
        let c3 = &curves[3];
        c3.push_point(CurvePoint::new(d));
        c3.push_point(CurvePoint::new(a));
    }
}

impl ToolImplImpl for QuadrilateralToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        if !self.active.get() {
            return Inhibit(false);
        }
        match self.upper_left.get() {
            Some(UnitPoint(upper_left)) => match event.button() {
                gtk::gdk::BUTTON_PRIMARY => {
                    self.upper_left.set(None);
                    let event_position = event.position();
                    let UnitPoint(bottom_right) =
                        viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    let mut curves = self.curves.borrow_mut();
                    let Point { x: width, y: _, .. }: Point = bottom_right - upper_left;
                    let a: Point = upper_left;
                    let b: Point = upper_left + (width, 0.0).into();
                    let c: Point = bottom_right;
                    let d: Point = bottom_right - (width, 0.0).into();
                    make_quadrilateral_bezier_curves(&mut curves, (a, b, c, d));
                    let contour = Contour::new();
                    let curves = std::mem::take(&mut *curves);
                    for c in curves {
                        contour.push_curve(c);
                    }
                    contour.close();
                    let mut state = view.state().borrow_mut();
                    let contour_index = state.glyph.borrow().contours.len();
                    let subaction = state.add_contour(&contour, contour_index);
                    let mut action = new_contour_action(state.glyph.clone(), contour, subaction);
                    (action.redo)();
                    state.add_undo_action(action);
                    self.upper_left.set(None);
                    self.instance()
                        .set_property::<bool>(QuadrilateralTool::ACTIVE, false);
                    state.active_tool = glib::types::Type::INVALID;
                    viewport.set_cursor("default");
                }
                gtk::gdk::BUTTON_SECONDARY => {
                    self.upper_left.set(None);
                    self.instance()
                        .set_property::<bool>(QuadrilateralTool::ACTIVE, false);
                    let mut state = view.state().borrow_mut();
                    state.active_tool = glib::types::Type::INVALID;
                    viewport.set_cursor("default");
                }
                _ => return Inhibit(false),
            },
            None => match event.button() {
                gtk::gdk::BUTTON_PRIMARY => {
                    let event_position = event.position();
                    let upper_left = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    self.upper_left.set(Some(upper_left));
                    let mut curves = self.curves.borrow_mut();
                    let a: Point = upper_left.0;
                    let b: Point = upper_left.0;
                    let c: Point = upper_left.0;
                    let d: Point = upper_left.0;
                    make_quadrilateral_bezier_curves(&mut curves, (a, b, c, d));
                }
                _ => return Inhibit(false),
            },
        }
        Inhibit(true)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        if !self.active.get() {
            return Inhibit(false);
        }
        match self.upper_left.get() {
            Some(UnitPoint(upper_left)) => {
                let event_position = event.position();
                let UnitPoint(bottom_right) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                let mut curves = self.curves.borrow_mut();
                let Point { x: width, y: _, .. }: Point = bottom_right - upper_left;
                let a: Point = upper_left;
                let b: Point = upper_left + (width, 0.0).into();
                let c: Point = bottom_right;
                let d: Point = bottom_right - (width, 0.0).into();
                make_quadrilateral_bezier_curves(&mut curves, (a, b, c, d));
                viewport.queue_draw();
            }
            None => return Inhibit(false),
        }
        Inhibit(true)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &Editor) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("quadrilateral"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: ContextRef| {
                    QuadrilateralTool::draw_layer(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(QuadrilateralTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.viewport.add_post_layer(layer);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &Editor) {
        self.instance()
            .set_property::<bool>(QuadrilateralTool::ACTIVE, true);
        if let Some(pixbuf) = self.cursor.get().unwrap().clone() {
            view.viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.viewport.set_cursor("grab");
        }
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &Editor) {
        self.upper_left.set(None);
        self.instance()
            .set_property::<bool>(QuadrilateralTool::ACTIVE, false);
        view.viewport.set_cursor("default");
        self.parent_on_deactivate(obj, view)
    }
}

impl QuadrilateralToolInner {}

glib::wrapper! {
    pub struct QuadrilateralTool(ObjectSubclass<QuadrilateralToolInner>)
        @extends ToolImpl;
}

impl Default for QuadrilateralTool {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadrilateralTool {
    pub const ACTIVE: &str = "active";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn draw_layer(viewport: &Canvas, cr: ContextRef, obj: Editor) -> Inhibit {
        let state = obj.state().borrow();
        if Self::static_type() != state.active_tool {
            return Inhibit(false);
        }
        let t = state.tools[&state.active_tool]
            .clone()
            .downcast::<Self>()
            .unwrap();
        if !t.imp().active.get() {
            return Inhibit(false);
        }
        if t.imp().upper_left.get().is_none() {
            return Inhibit(false);
        }
        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let curves = t.imp().curves.borrow();
        let line_width = obj
            .settings
            .get()
            .unwrap()
            .property::<f64>(Settings::LINE_WIDTH)
            / (scale * ppu);
        cr.transform(viewport.transformation.matrix());
        cr.set_line_width(line_width);
        let outline = (0.2, 0.2, 0.2, 0.6);
        cr.set_source_rgba(outline.0, outline.1, outline.2, outline.3);

        for curv in curves.iter() {
            let points = curv.points();
            let (a, b) = (points[0].position, points[1].position);
            cr.move_to(a.x, a.y);
            cr.line_to(b.x, b.y);
        }
        cr.stroke().expect("Invalid cairo surface state");

        Inhibit(true)
    }
}

#[derive(Default)]
pub struct EllipseToolInner {
    layer: OnceCell<Layer>,
    curves: Rc<RefCell<[Bezier; 4]>>,
    active: Cell<bool>,
    center: Cell<Option<UnitPoint>>,
    cursor: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
}

#[glib::object_subclass]
impl ObjectSubclass for EllipseToolInner {
    const NAME: &'static str = "EllipseTool";
    type ParentType = ToolImpl;
    type Type = EllipseTool;
}

impl ObjectImpl for EllipseToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        for c in self.curves.borrow().iter() {
            c.set_property(Bezier::SMOOTH, true);
        }
        obj.set_property::<bool>(EllipseTool::ACTIVE, false);
        obj.set_property::<String>(ToolImpl::NAME, "ellipse".to_string());
        obj.set_property::<String>(
            ToolImpl::DESCRIPTION,
            "Create ellipses path shapes".to_string(),
        );
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::icons::ELLIPSE_ICON.to_image_widget(),
        );
        self.cursor
            .set(crate::resources::cursors::CIRCLE_CURSOR.to_pixbuf())
            .unwrap();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    EllipseTool::ACTIVE,
                    EllipseTool::ACTIVE,
                    EllipseTool::ACTIVE,
                    true,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            EllipseTool::ACTIVE => self.active.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            EllipseTool::ACTIVE => self.active.set(value.get().unwrap()),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

fn make_circle_bezier_curves(curves: &mut [Bezier; 4], (center, radius): (Point, f64)) {
    /*
     * Source: https://spencermortensen.com/articles/bezier-circle/
     */
    const A: f64 = 1.00005519;
    const B: f64 = 0.55342686;
    const C: f64 = 0.99873585;
    for (i, c) in curves.iter_mut().enumerate() {
        c.clear_points();
        let mut matrix = gtk::cairo::Matrix::identity();
        matrix.translate(center.x, center.y);
        matrix.rotate(i as f64 * std::f64::consts::FRAC_PI_2);
        c.push_point(CurvePoint::new(
            matrix * <_ as Into<Point>>::into((A * radius, 0.0)),
        ));
        c.push_point(CurvePoint::new(
            matrix * <_ as Into<Point>>::into((C * radius, B * radius)),
        ));
        c.push_point(CurvePoint::new(
            matrix * <_ as Into<Point>>::into((B * radius, C * radius)),
        ));
        c.push_point(CurvePoint::new(
            matrix * <_ as Into<Point>>::into((0.0, A * radius)),
        ));
    }
    /* ensure continuities after rotation */
    let mut last_point = curves[3].points()[3].position;
    curves[0].modify_point(0, |cp| {
        cp.position = last_point;
    });
    last_point = curves[0].points()[3].position;

    curves[1].modify_point(0, |cp| {
        cp.position = last_point;
    });
    last_point = curves[1].points()[3].position;

    curves[2].modify_point(0, |cp| {
        cp.position = last_point;
    });
    last_point = curves[3].points()[3].position;

    curves[3].modify_point(0, |cp| {
        cp.position = last_point;
    });
}

impl ToolImplImpl for EllipseToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        if !self.active.get() {
            return Inhibit(false);
        }
        match self.center.get() {
            Some(UnitPoint(center)) => match event.button() {
                gtk::gdk::BUTTON_PRIMARY => {
                    self.center.set(None);
                    let event_position = event.position();
                    let UnitPoint(r) =
                        viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    let mut curves = self.curves.borrow_mut();
                    let radius: f64 = crate::utils::distance_between_two_points(center, r);
                    make_circle_bezier_curves(&mut curves, (center, radius));
                    let contour = Contour::new();
                    let curves = std::mem::take(&mut *curves);
                    for c in curves {
                        contour.push_curve(c);
                    }
                    contour.close();
                    let mut state = view.state().borrow_mut();
                    let contour_index = state.glyph.borrow().contours.len();
                    let subaction = state.add_contour(&contour, contour_index);
                    let mut action = new_contour_action(state.glyph.clone(), contour, subaction);
                    (action.redo)();
                    state.add_undo_action(action);
                    viewport.queue_draw();
                    self.instance()
                        .set_property::<bool>(EllipseTool::ACTIVE, false);
                    state.active_tool = glib::types::Type::INVALID;
                    viewport.set_cursor("default");
                }
                gtk::gdk::BUTTON_SECONDARY => {
                    self.center.set(None);
                    self.instance()
                        .set_property::<bool>(EllipseTool::ACTIVE, false);
                    let mut state = view.state().borrow_mut();
                    state.active_tool = glib::types::Type::INVALID;
                    viewport.set_cursor("default");
                }
                _ => return Inhibit(false),
            },
            None => match event.button() {
                gtk::gdk::BUTTON_PRIMARY => {
                    let event_position = event.position();
                    let center = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    self.center.set(Some(center));
                    let mut curves = self.curves.borrow_mut();
                    make_circle_bezier_curves(&mut curves, (center.0, 0.0));
                }
                _ => return Inhibit(false),
            },
        }
        Inhibit(true)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        if !self.active.get() {
            return Inhibit(false);
        }
        match self.center.get() {
            Some(UnitPoint(center)) => {
                let event_position = event.position();
                let UnitPoint(r) = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                let mut curve = self.curves.borrow_mut();
                let radius: f64 = crate::utils::distance_between_two_points(center, r);
                make_circle_bezier_curves(&mut curve, (center, radius));
                viewport.queue_draw();
            }
            None => return Inhibit(false),
        }
        Inhibit(true)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &Editor) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("ellipse"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: ContextRef| {
                    EllipseTool::draw_layer(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(EllipseTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.viewport.add_post_layer(layer);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &Editor) {
        self.instance()
            .set_property::<bool>(EllipseTool::ACTIVE, true);
        if let Some(pixbuf) = self.cursor.get().unwrap().clone() {
            view.viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.viewport.set_cursor("grab");
        }
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &Editor) {
        self.center.set(None);
        self.instance()
            .set_property::<bool>(EllipseTool::ACTIVE, false);
        view.viewport.set_cursor("default");
        self.parent_on_deactivate(obj, view)
    }
}

impl EllipseToolInner {}

glib::wrapper! {
    pub struct EllipseTool(ObjectSubclass<EllipseToolInner>)
        @extends ToolImpl;
}

impl Default for EllipseTool {
    fn default() -> Self {
        Self::new()
    }
}

impl EllipseTool {
    pub const ACTIVE: &str = "active";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn draw_layer(viewport: &Canvas, cr: ContextRef, obj: Editor) -> Inhibit {
        let state = obj.state().borrow();
        if Self::static_type() != state.active_tool {
            return Inhibit(false);
        }
        let t = state.tools[&state.active_tool]
            .clone()
            .downcast::<Self>()
            .unwrap();
        if !t.imp().active.get() {
            return Inhibit(false);
        }
        if t.imp().center.get().is_none() {
            return Inhibit(false);
        }
        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let curves = t.imp().curves.borrow();
        let line_width = obj
            .settings
            .get()
            .unwrap()
            .property::<f64>(Settings::LINE_WIDTH)
            / (scale * ppu);
        cr.transform(viewport.transformation.matrix());
        cr.set_line_width(line_width);
        let outline = (0.2, 0.2, 0.2, 0.6);
        cr.set_source_rgba(outline.0, outline.1, outline.2, outline.3);

        for curv in curves.iter() {
            let points = curv.points();
            if points.len() != 4 {
                continue;
            }
            let (a, b, c, d) = (
                points[0].position,
                points[1].position,
                points[2].position,
                points[3].position,
            );
            cr.move_to(a.x, a.y);
            cr.curve_to(b.x, b.y, c.x, c.y, d.x, d.y);
        }
        cr.stroke().expect("Invalid cairo surface state");

        Inhibit(true)
    }
}
