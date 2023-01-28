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
use crate::glyphs::{Contour, GlyphDrawingOptions};
use crate::utils::colors::*;
use crate::utils::{curves::Bezier, distance_between_two_points, Point};
use crate::views::{
    canvas::{Layer, LayerBuilder, UnitPoint, ViewPoint},
    Canvas, Transformation,
};
use crate::GlyphEditView;
use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::cairo::Context;
use gtk::Inhibit;
use gtk::{glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Default)]
pub struct BezierToolInner {
    layer: OnceCell<Layer>,
    state: Rc<RefCell<State>>,
    active: Cell<bool>,
    cursor: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
}

#[glib::object_subclass]
impl ObjectSubclass for BezierToolInner {
    const NAME: &'static str = "BezierTool";
    type ParentType = ToolImpl;
    type Type = BezierTool;
}

impl ObjectImpl for BezierToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_property::<bool>(BezierTool::ACTIVE, false);
        obj.set_property::<String>(ToolImpl::NAME, "Create Bézier curve".to_string());
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::svg_to_image_widget(crate::resources::BEZIER_ICON_SVG),
        );
        self.cursor
            .set(crate::resources::svg_to_pixbuf(
                crate::resources::PEN_CURSOR_SVG,
            ))
            .unwrap();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    BezierTool::ACTIVE,
                    BezierTool::ACTIVE,
                    BezierTool::ACTIVE,
                    true,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            BezierTool::ACTIVE => self.active.get().to_value(),
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
            BezierTool::ACTIVE => self.active.set(value.get().unwrap()),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl ToolImplImpl for BezierToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        if event.button() == gtk::gdk::BUTTON_PRIMARY {
            let mut state = self.state.borrow_mut();
            if state.inner == InnerState::OnCurvePoint {
                let event_position = event.position();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                if !state.insert_point(position) {
                    let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
                    if let Some(contour) = state.close() {
                        let contour_index = glyph_state.glyph.borrow().contours.len();
                        let subaction = glyph_state.add_contour(&contour, contour_index);
                        let mut action =
                            new_contour_action(glyph_state.glyph.clone(), contour, subaction);
                        (action.redo)();
                        glyph_state.add_undo_action(action);
                        glyph_state.active_tool = glib::types::Type::INVALID;
                        self.on_deactivate(_obj, &view);
                        return Inhibit(true);
                    }
                }
                // b.start
                assert!(state.inner != InnerState::OnCurvePoint);

                return Inhibit(true);
            }
        } else if event.button() == gtk::gdk::BUTTON_SECONDARY {
            let mut state = self.state.borrow_mut();
            let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
            if let Some(contour) = state.close() {
                let contour_index = glyph_state.glyph.borrow().contours.len();
                let subaction = glyph_state.add_contour(&contour, contour_index);
                let mut action = new_contour_action(glyph_state.glyph.clone(), contour, subaction);
                (action.redo)();
                glyph_state.add_undo_action(action);
                glyph_state.active_tool = glib::types::Type::INVALID;
                self.on_deactivate(_obj, &view);
            }
            viewport.queue_draw();

            return Inhibit(true);
        }
        Inhibit(false)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mut state = self.state.borrow_mut();
        //let is_smooth: bool = state.current_curve.property(Bezier::SMOOTH);
        match (event.button(), state.inner) {
            /*(gtk::gdk::BUTTON_PRIMARY, InnerState::OnCurvePoint, false) => {
                let mut state = self.state.borrow_mut();
                state.add_corner();
                viewport.queue_draw();
                Inhibit(true)
            }*/
            (
                gtk::gdk::BUTTON_PRIMARY,
                prev @ InnerState::Handle | prev @ InnerState::HandleUnlinked,
            ) => {
                let event_position = event.position();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                state.current_curve.set_property(Bezier::SMOOTH, true);
                if !state.insert_point(position) {
                    let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
                    if let Some(contour) = state.close() {
                        let contour_index = glyph_state.glyph.borrow().contours.len();
                        let subaction = glyph_state.add_contour(&contour, contour_index);
                        let mut action =
                            new_contour_action(glyph_state.glyph.clone(), contour, subaction);
                        (action.redo)();
                        glyph_state.add_undo_action(action);
                        glyph_state.active_tool = glib::types::Type::INVALID;
                        self.on_deactivate(_obj, &view);
                    }
                    viewport.queue_draw();
                }
                assert!(state.inner != prev);
                Inhibit(true)
            }
            _ => Inhibit(false),
        }
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let event_state = event.state();
        let mut state = self.state.borrow_mut();

        if event_state.intersects(gtk::gdk::ModifierType::BUTTON1_MASK)
            && state.inner == InnerState::OnCurvePoint
        {
            //state.current_curve.set_property(Bezier::SMOOTH, true);
            state.inner = InnerState::Handle;
            return Inhibit(true);
        } else if event_state.intersects(gtk::gdk::ModifierType::SHIFT_MASK) {
            /* snap */
            return Inhibit(true);
        } else if event_state.intersects(gtk::gdk::ModifierType::META_MASK)
            && matches!(state.inner, InnerState::Handle | InnerState::HandleUnlinked)
        {
            state.inner = if state.inner == InnerState::Handle {
                InnerState::HandleUnlinked
            } else {
                InnerState::Handle
            };
            return Inhibit(true);
        } else if state.inner == InnerState::Handle && !state.curves.is_empty() {
            let event_position = event.position();
            let UnitPoint(point) = viewport.view_to_unit_point(ViewPoint(event_position.into()));

            let mut current_points = state.current_curve.points().borrow_mut();
            let p_len = current_points.len();
            let end_point = current_points[p_len - 1];
            let dist = distance_between_two_points(point, end_point);
            if dist >= 0.1 {
                if p_len == 1 {
                    let mut prev_points = state.curves.last().unwrap().points().borrow_mut();
                    let prev_points_len = prev_points.len();
                    prev_points[prev_points_len - 2] = point.mirror(end_point);
                } else {
                    current_points[p_len - 2] = point.mirror(end_point);
                }
            }
            return Inhibit(true);
        }
        Inhibit(false)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("bezier"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: &gtk::cairo::Context| {
                    BezierTool::draw_layer(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(BezierTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.imp().viewport.add_post_layer(layer);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(BezierTool::ACTIVE, true);
        if let Some(pixbuf) = self.cursor.get().unwrap().as_ref() {
            view.imp().viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.imp().viewport.set_cursor("grab");
        }
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(BezierTool::ACTIVE, false);
        view.imp().viewport.set_cursor("default");
        self.parent_on_deactivate(obj, view)
    }
}

impl BezierToolInner {}

glib::wrapper! {
    pub struct BezierTool(ObjectSubclass<BezierToolInner>)
        @extends ToolImpl;
}

impl Default for BezierTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BezierTool {
    pub const ACTIVE: &str = "active";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn draw_layer(viewport: &Canvas, cr: &gtk::cairo::Context, obj: GlyphEditView) -> Inhibit {
        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
        if BezierTool::static_type() != glyph_state.active_tool {
            return Inhibit(false);
        }
        let t = glyph_state.tools[&glyph_state.active_tool]
            .clone()
            .downcast::<BezierTool>()
            .unwrap();
        if !t.imp().active.get() {
            return Inhibit(false);
        }
        let inner_fill = viewport.property::<bool>(Canvas::INNER_FILL);
        let units_per_em = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::UNITS_PER_EM);
        let options = GlyphDrawingOptions {
            outline: Color::new_alpha(0.2, 0.2, 0.2, if inner_fill { 0. } else { 0.6 }),
            inner_fill: if inner_fill {
                Some(Color::BLACK)
            } else {
                Some(viewport.property::<Color>(Canvas::GLYPH_INNER_FILL_COLOR))
            },
            highlight: obj.imp().hovering.get(),
            matrix: viewport.imp().transformation.matrix(),
            units_per_em,
            line_width: obj
                .imp()
                .settings
                .get()
                .unwrap()
                .property::<f64>(crate::Settings::LINE_WIDTH),
        };
        t.imp().state.borrow().draw(
            viewport,
            cr,
            options,
            viewport.view_to_unit_point(viewport.get_mouse()).0,
        );

        Inhibit(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InnerState {
    OnCurvePoint,
    Handle,
    HandleUnlinked,
}

#[derive(Debug, Clone)]
pub struct State {
    inner: InnerState,
    snap_to_angle: bool,
    first_point: Option<Point>,
    current_curve: Bezier,
    curves: Vec<Bezier>,
}

impl Default for State {
    fn default() -> Self {
        State {
            inner: InnerState::OnCurvePoint,
            snap_to_angle: false,
            current_curve: Bezier::new(true, vec![]),
            curves: vec![],
            first_point: None,
        }
    }
}

impl State {
    pub fn insert_point(&mut self, point: Point) -> bool {
        match self.inner {
            InnerState::OnCurvePoint => {
                // (a).end
                match self.first_point.as_ref() {
                    None => {
                        self.first_point = Some(point);
                        self.current_curve
                            .set_property::<bool>(Bezier::SMOOTH, false);
                    }
                    Some(fp) if distance_between_two_points(point, *fp) < 20.0 => {
                        let mut current_points = self.current_curve.points().borrow_mut();
                        current_points.push(*fp);
                        return false;
                    }
                    _ => {}
                }
                self.inner = InnerState::Handle;
            }
            InnerState::Handle => {
                self.inner = InnerState::OnCurvePoint;
                // (b).end
                let current_points = self.current_curve.points().borrow();
                let p_len = current_points.len();
                let end_point = current_points[p_len - 1];
                drop(current_points);
                if distance_between_two_points(point, end_point) < 20.0
                    && self.current_curve.degree() == Some(2)
                {
                    /* current_curve should be quadratic, so split it. */
                    let curv = std::mem::replace(
                        &mut self.current_curve,
                        Bezier::new(true, vec![point, point]),
                    );
                    curv.clean_up();
                    self.curves.push(curv);
                    return true;
                }

                let mut current_points = self.current_curve.points().borrow_mut();
                if !self.curves.is_empty() {
                    let dist = distance_between_two_points(point, end_point);
                    if dist >= 0.1 {
                        current_points[p_len - 2] = point.mirror(end_point);
                    }
                }
            }
            InnerState::HandleUnlinked => {
                self.inner = InnerState::OnCurvePoint;
                // (c).end
                let current_points = self.current_curve.points().borrow();
                let p_len = current_points.len();
                let end_point = current_points[p_len - 1];
                drop(current_points);
                if distance_between_two_points(point, end_point) < 20.0 {
                    if self.current_curve.degree() == Some(2) {
                        /* current_curve should be quadratic, so split it. */
                        let curv = std::mem::replace(
                            &mut self.current_curve,
                            Bezier::new(true, vec![point, point]),
                        );
                        curv.clean_up();
                        self.curves.push(curv);
                    }
                    return true;
                }
            }
        }
        /*if !self.current_curve.property::<bool>(Bezier::SMOOTH) {
            match self.first_point.as_ref() {
                None => {
                    self.first_point = Some(point);
                }
                Some(fp) if distance_between_two_points(point, *fp) < 20.0 => {
                    self.current_curve.points().borrow_mut().push(*fp);
                    return false;
                }
                _ => {}
            }
            self.current_curve.points().borrow_mut().push(point);
            let curv = std::mem::replace(&mut self.current_curve, Bezier::new(false, vec![]));
            self.current_curve.points().borrow_mut().push(point);
            self.curves.push(curv);
            return true;
        }
        match self.inner {
            InnerState::OnCurvePoint | InnerState::HandleUnlinked => {
                match self.first_point.as_ref() {
                    None => {
                        self.first_point = Some(point);
                        self.current_curve
                            .set_property::<bool>(Bezier::SMOOTH, false);
                    }
                    Some(fp) if distance_between_two_points(point, *fp) < 20.0 => {
                        self.current_curve.points().borrow_mut().push(*fp);
                        return false;
                    }
                    _ => {}
                }
                self.current_curve.points().borrow_mut().push(point);
                //self.inner = InnerState::Handle;
            }
            InnerState::Handle => {
                self.inner = InnerState::OnCurvePoint;
                self.current_curve.points().borrow_mut().push(point);
            }
        }
        */
        self.current_curve.points().borrow_mut().push(point);
        if self.current_curve.degree() == Some(3) {
            /* current_curve is cubic, so split it. */
            let curv = std::mem::replace(
                &mut self.current_curve,
                Bezier::new(
                    true,
                    if self.inner == InnerState::OnCurvePoint {
                        vec![point, point]
                    } else {
                        vec![point]
                    },
                ),
            );
            curv.clean_up();
            self.curves.push(curv);
        }
        true
    }

    pub fn close(&mut self) -> Option<Contour> {
        let State {
            inner: _,
            first_point: _,
            current_curve,
            snap_to_angle: _,
            mut curves,
        } = std::mem::take(self);
        if current_curve.degree().is_some() {
            current_curve.clean_up();
            curves.push(current_curve);
        }
        curves.retain(|c| !matches!(c.degree(), Some(0) | None));
        if curves.is_empty() {
            return None;
        }

        let ret = Contour::new();
        ret.set_property::<bool>(
            Contour::OPEN,
            curves.first().unwrap().points().borrow()[0]
                != *curves.last().unwrap().points().borrow().last().unwrap(),
        );
        if !ret.property::<bool>(Contour::OPEN) && curves.len() > 2 {
            let l = curves.len();
            let (first, prev, last) = (
                curves.first().unwrap().points().borrow(),
                curves[l - 2].points().borrow(),
                &curves[l - 1],
            );
            if last.degree() == Some(1) {
                let mut pts = last.points().borrow_mut();
                let p_l = prev.len();
                pts.pop();
                pts.push(prev[p_l - 2].mirror(prev[p_l - 1]));
                pts.push(first[1].mirror(first[0]));
                pts.push(first[0]);
            }
        }

        for c in curves {
            if matches!(c.degree(), Some(0) | None) {
                continue;
            }
            ret.push_curve(c);
        }
        if ret.curves().borrow().is_empty() {
            None
        } else {
            Some(ret)
        }
    }

    pub fn draw(
        &self,
        viewport: &Canvas,
        cr: &Context,
        options: GlyphDrawingOptions,
        cursor_position: Point,
    ) {
        let first_point = match self.first_point {
            Some(v) => v,
            None => return,
        };
        let GlyphDrawingOptions {
            outline,
            inner_fill: _,
            highlight: _,
            matrix,
            units_per_em: _,
            line_width,
        } = options;

        cr.save().expect("Invalid cairo surface state");
        {
            let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
            let _height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
            cr.set_font_size(11.0);
            let line_height = cr.text_extents("BezierTool").unwrap().height * 1.5;
            cr.show_text("BezierTool").unwrap();
            for (i, line) in Some(format!("state: {:?}", self.inner))
                .into_iter()
                .chain(Some(format!("snap_to_angle: {:?}", self.snap_to_angle)).into_iter())
                .chain(Some(format!("first_point: {:?}", self.first_point.as_ref())).into_iter())
                .chain(
                    format!("current_curve: {:#?}", self.current_curve.imp())
                        .lines()
                        .map(str::to_string),
                )
                .chain(
                    format!(
                        "curves: {:#?}",
                        self.curves.iter().map(Bezier::imp).collect::<Vec<_>>()
                    )
                    .lines()
                    .map(str::to_string),
                )
                .enumerate()
            {
                cr.move_to(width / 2.0, 95.0 + (i + 1) as f64 * line_height);
                cr.show_text(&line).unwrap();
            }
        }
        cr.transform(matrix);
        cr.set_line_width(line_width);
        cr.set_source_color_alpha(outline);
        let draw_endpoint = |p: (f64, f64)| {
            cr.rectangle(p.0 - 2.5, p.1 - 2.5, 5., 5.);
            cr.stroke().expect("Invalid cairo surface state");
        };
        let draw_handle = |p: (f64, f64), ep: (f64, f64)| {
            cr.arc(p.0 - 2.5, p.1 - 2.5, 2.0, 0., 2. * std::f64::consts::PI);
            cr.fill().unwrap();
            cr.move_to(p.0 - 2.5, p.1 - 2.5);
            cr.line_to(ep.0, ep.1);
            cr.stroke().unwrap();
        };
        let p_fn = |p: Point| -> (f64, f64) { (p.x as f64, p.y as f64) };
        let fp = p_fn(first_point);
        draw_endpoint(fp);
        let mut pen_position: Option<(f64, f64)> = Some(fp);
        for curv in self.curves.iter() {
            if !curv.property::<bool>(Bezier::SMOOTH) {
                //cr.stroke().expect("Invalid cairo surface state");
            }
            let degree = curv.degree();
            let degree = if let Some(v) = degree {
                v
            } else {
                continue;
            };
            if let Some(p) = pen_position.take() {
                cr.move_to(p.0, p.1);
            }
            match degree {
                0 => { /* ignore */ }
                1 => {
                    /* Line. */
                    let new_point = p_fn(curv.points().borrow()[1]);
                    cr.line_to(new_point.0, new_point.1);
                    pen_position = Some(new_point);
                }
                2 => {
                    /* Quadratic. */
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        p_fn(curv.points().borrow()[0])
                    };
                    let b = p_fn(curv.points().borrow()[1]);
                    let c = p_fn(curv.points().borrow()[2]);
                    cr.curve_to(
                        2.0 / 3.0 * b.0 + 1.0 / 3.0 * a.0,
                        2.0 / 3.0 * b.1 + 1.0 / 3.0 * a.1,
                        2.0 / 3.0 * b.0 + 1.0 / 3.0 * c.0,
                        2.0 / 3.0 * b.1 + 1.0 / 3.0 * c.1,
                        c.0,
                        c.1,
                    );
                    pen_position = Some(c);
                }
                3 => {
                    /* Cubic */
                    let _a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        p_fn(curv.points().borrow()[0])
                    };
                    let b = p_fn(curv.points().borrow()[1]);
                    let c = p_fn(curv.points().borrow()[2]);
                    let d = p_fn(curv.points().borrow()[3]);
                    cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                    pen_position = Some(d);
                }
                d => {
                    eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                    pen_position = Some(p_fn(*curv.points().borrow().last().unwrap()));
                    continue;
                }
            }
        }
        cr.stroke().expect("Invalid cairo surface state");
        if let Some(pos) = pen_position {
            cr.move_to(pos.0, pos.1);
        }
        let (pos_x, pos_y) = p_fn(cursor_position);
        //cr.set_source_color_alpha(outline.0, outline.1, outline.2, 0.5 * outline.3);
        match self.current_curve.degree() {
            None => {
                cr.line_to(pos_x, pos_y);
                cr.stroke().expect("Invalid cairo surface state");
            }
            Some(0) => {
                let new_point = p_fn(self.current_curve.points().borrow()[0]);
                cr.line_to(new_point.0, new_point.1);
                cr.line_to(pos_x, pos_y);
                cr.stroke().expect("Invalid cairo surface state");
                draw_endpoint(new_point);
            }
            Some(1) => {
                let a = p_fn(self.current_curve.points().borrow()[0]);
                cr.line_to(a.0, a.1);
                let b = p_fn(self.current_curve.points().borrow()[1]);
                let c = (pos_x, pos_y);
                let d = (pos_x, pos_y);
                cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                cr.stroke().expect("Invalid cairo surface state");
                draw_endpoint(a);
                draw_endpoint(d);
                draw_handle(b, a);
            }
            Some(2) => {
                let a = p_fn(self.current_curve.points().borrow()[0]);
                cr.line_to(a.0, a.1);
                let b = p_fn(self.current_curve.points().borrow()[1]);
                let c = p_fn(self.current_curve.points().borrow()[2]);
                let d = (pos_x, pos_y);
                cr.curve_to(b.0, b.1, c.0, c.1, d.0, d.1);
                cr.stroke().expect("Invalid cairo surface state");
                draw_endpoint(a);
                draw_endpoint(d);
                draw_handle(b, a);
                draw_handle(c, d);
            }
            Some(d) => {
                eprintln!(
                    "Something's wrong in current curve. Bezier of degree {}: {:?}",
                    d, self.current_curve
                );
            }
        }

        cr.restore().expect("Invalid cairo surface state");
    }
}
