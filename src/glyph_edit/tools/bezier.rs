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
use crate::glyphs::{Contour, GlyphPointIndex};
use crate::utils::{curves::Bezier, distance_between_two_points, CurvePoint, Point};
use crate::views::{
    canvas::{Layer, LayerBuilder, UnitPoint, ViewPoint},
    Canvas,
};
use crate::GlyphEditView;
use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::cairo::{Context, Matrix};
use gtk::Inhibit;
use gtk::{glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;
use std::cell::{Cell, RefCell, RefMut};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum InnerState {
    #[default]
    OnCurvePoint,
    Handle,
    HandleUnlinked,
}

struct ContourState {
    first_point: Point,
    contour: Contour,
    first_curve: Bezier,
    current_curve: Bezier,
    last_point: CurvePoint,
    curve_index: usize,
}

impl std::fmt::Debug for ContourState {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ContourState")
            .field("first_point", &self.first_point)
            .field("contour", &self.contour.imp())
            .field("first_curve", &self.first_curve.imp())
            .field("current_curve", &self.current_curve.imp())
            .field("last_point", &self.last_point)
            .field("curve_index", &self.curve_index)
            .finish()
    }
}

#[derive(Default)]
pub struct BezierToolInner {
    layer: OnceCell<Layer>,
    active: Cell<bool>,
    cursor: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
    inner: Cell<InnerState>,
    contour: RefCell<Option<ContourState>>,
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
            crate::resources::BEZIER_ICON.to_image_widget(),
        );
        self.cursor
            .set(crate::resources::PEN_CURSOR.to_pixbuf())
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
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        if event.button() == gtk::gdk::BUTTON_PRIMARY {
            let mut c = self.contour.borrow_mut();
            if c.is_none() {
                let UnitPoint(first_point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                let current_curve = Bezier::new(vec![first_point]);
                let new_state = ContourState {
                    last_point: {
                        let p = current_curve.points().borrow()[0].clone();
                        p
                    },
                    first_point,
                    contour: {
                        let contour = Contour::new();
                        contour.push_curve(current_curve.clone());
                        contour
                    },
                    first_curve: current_curve.clone(),
                    current_curve,
                    curve_index: 0,
                };
                let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
                let contour_index = glyph_state.glyph.borrow().contours.len();
                let subaction = glyph_state.add_contour(&new_state.contour, contour_index);
                let mut action = new_contour_action(
                    glyph_state.glyph.clone(),
                    new_state.contour.clone(),
                    subaction,
                );
                (action.redo)();
                glyph_state.add_undo_action(action);
                *c = Some(new_state);
                return Inhibit(true);
            }

            if self.inner.get() == InnerState::OnCurvePoint {
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                self.insert_point(obj, &view, c, position);
                // b.start

                return Inhibit(true);
            }
        } else if event.button() == gtk::gdk::BUTTON_SECONDARY {
            let c = self.contour.borrow_mut();
            if c.is_none() {
                return Inhibit(false);
            }
            self.close(obj, &view, c);

            return Inhibit(true);
        }
        Inhibit(false)
    }

    fn on_button_release_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        match (event.button(), self.inner.get()) {
            (gtk::gdk::BUTTON_PRIMARY, InnerState::Handle | InnerState::HandleUnlinked) => {
                let c = self.contour.borrow_mut();
                if c.is_none() {
                    return Inhibit(false);
                }
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                self.inner.set(InnerState::OnCurvePoint);
                self.insert_point(obj, &view, c, position);
                viewport.queue_draw();
                Inhibit(true)
            }
            _ => Inhibit(false),
        }
    }

    fn on_motion_notify_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let mut c = self.contour.borrow_mut();
        if c.is_none() {
            return Inhibit(false);
        }

        let event_state = event.state();
        let inner = self.inner.get();

        if event_state.intersects(gtk::gdk::ModifierType::BUTTON1_MASK)
            && inner == InnerState::OnCurvePoint
        {
            let UnitPoint(position) =
                viewport.view_to_unit_point(ViewPoint(event.position().into()));
            self.insert_point(obj, &view, c, position);
            self.inner.set(InnerState::Handle);
            return Inhibit(true);
        } else if event_state.intersects(gtk::gdk::ModifierType::SHIFT_MASK) {
            /* snap */
            return Inhibit(true);
        } else if event_state.intersects(gtk::gdk::ModifierType::META_MASK)
            && matches!(inner, InnerState::Handle | InnerState::HandleUnlinked)
        {
            self.inner.set(if inner == InnerState::Handle {
                InnerState::HandleUnlinked
            } else {
                InnerState::Handle
            });

            return Inhibit(true);
        } else if inner == InnerState::Handle {
            let state = c.as_mut().unwrap();
            let UnitPoint(point) = viewport.view_to_unit_point(ViewPoint(event.position().into()));
            let diff_vector = point - state.last_point.position;
            if diff_vector.norm() >= 0.1 {
                let mut m = Matrix::identity();
                m.translate(diff_vector.x, diff_vector.y);
                self.transform_point(m, &view, state);
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
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: &Context| {
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

impl BezierToolInner {
    fn insert_point(
        &self,
        obj: &ToolImpl,
        view: &GlyphEditView,
        /* to ensure we don't reborrow ContourState */
        mut state_opt: RefMut<'_, Option<ContourState>>,
        point: Point,
    ) {
        if let Some(mut state) = state_opt.as_mut() {
            let add_to_kdtree = |state: &mut ContourState| {
                let glyph_state = view.imp().glyph_state.get().unwrap().borrow();
                let contour_index = glyph_state.glyph.borrow().contours.len();
                let curve_index = state.curve_index;
                let uuid = state.last_point.uuid;
                let idx = GlyphPointIndex {
                    contour_index,
                    curve_index,
                    uuid,
                };
                let mut kd_tree = glyph_state.kd_tree.borrow_mut();
                /* update kd_tree */
                kd_tree.add(idx, state.last_point.position);
                glyph_state.viewport.queue_draw();
            };
            let curve_point = CurvePoint::new(point);
            if distance_between_two_points(point, state.first_point) < 20.0
                && self.inner.get() == InnerState::Handle
            {
                state.first_curve.set_property(Bezier::SMOOTH, true);
                state.last_point = curve_point.clone();
                state.current_curve.points().borrow_mut().push(curve_point);
                add_to_kdtree(state);
                self.close(obj, view, state_opt);
            } else {
                state.last_point = curve_point.clone();
                state.current_curve.points().borrow_mut().push(curve_point);
                add_to_kdtree(state);
                if state.current_curve.degree() == Some(3) {
                    /* current_curve is cubic, so split it. */
                    let curv =
                        std::mem::replace(&mut state.current_curve, Bezier::new(vec![point]));
                    curv.clean_up();
                    state.last_point = state.current_curve.points().borrow()[0].clone();
                    state.contour.push_curve(state.current_curve.clone());
                    state.curve_index += 1;
                    add_to_kdtree(state);
                }
            }
        }
    }

    fn close(
        &self,
        obj: &ToolImpl,
        view: &GlyphEditView,
        /* to ensure we don't reborrow ContourState */
        state_opt: RefMut<'_, Option<ContourState>>,
    ) {
        if state_opt.as_ref().is_some() {
            drop(state_opt);
            view.imp()
                .glyph_state
                .get()
                .unwrap()
                .borrow_mut()
                .active_tool = glib::types::Type::INVALID;
            self.on_deactivate(obj, view);
            self.contour.borrow_mut().take();
        }
    }

    fn transform_point(&self, m: Matrix, view: &GlyphEditView, state: &mut ContourState) {
        let glyph_state = view.imp().glyph_state.get().unwrap().borrow();
        let contour_index = glyph_state.glyph.borrow().contours.len();
        let curve_index = state.curve_index;
        let uuid = state.last_point.uuid;
        let idxs = [GlyphPointIndex {
            contour_index,
            curve_index,
            uuid,
        }];
        let mut kd_tree = glyph_state.kd_tree.borrow_mut();
        for (idx, new_pos) in state.contour.transform_points(contour_index, &idxs, m) {
            if idx.uuid == uuid {
                state.last_point.position = new_pos;
            }
            /* update kd_tree */
            kd_tree.add(idx, new_pos);
            glyph_state.viewport.queue_draw();
        }
    }
}

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

    pub fn draw_layer(_viewport: &Canvas, _cr: &Context, _obj: GlyphEditView) -> Inhibit {
        Inhibit(false)
        /*
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
        let c = t.imp().contour.borrow();
        if c.is_none() {
            return Inhibit(false);
        }
        let state = c.as_ref().unwrap();

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
        //t.imp().state.borrow().draw(
        //    viewport,
        //    cr,
        //    options,
        //    viewport.view_to_unit_point(viewport.get_mouse()).0,
        //);

        let cursor: Point = viewport.view_to_unit_point(viewport.get_mouse()).0;
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
            for (i, line) in Some(format!("state: {:?}", t.imp().inner))
                .into_iter()
                .chain(Some(format!("snap_to_angle: {:?}", t.imp().snap_to_angle)).into_iter())
                .chain(Some(format!("first_point: {:?}", &state.first_point)).into_iter())
                .chain(
                    format!("current_curve: {:#?}", state.current_curve.imp())
                        .lines()
                        .map(str::to_string),
                )
                .chain(
                    format!(
                        "curves: {:#?}",
                        state
                            .contour
                            .imp()
                            .curves
                            .borrow()
                            .iter()
                            .map(Bezier::imp)
                            .collect::<Vec<_>>()
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
        let draw_endpoint = |p: Point| {
            cr.rectangle(p.x - 2.5, p.y - 2.5, 5.0, 5.0);
            cr.stroke().expect("Invalid cairo surface state");
        };
        let draw_handle = |p: Point, ep: Point| {
            cr.arc(p.x - 2.5, p.y - 2.5, 2.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap();
            cr.move_to(p.x - 2.5, p.y - 2.5);
            cr.line_to(ep.x, ep.y);
            cr.stroke().unwrap();
        };
        let fp = state.first_point;
        draw_endpoint(fp);
        let mut pen_position: Option<Point> = Some(fp);
        for curv in state.contour.imp().curves.borrow().iter() {
            let degree = curv.degree();
            let degree = if let Some(v) = degree {
                v
            } else {
                continue;
            };
            if let Some(p) = pen_position.take() {
                cr.move_to(p.x, p.y);
            }
            match degree {
                0 => { /* ignore */ }
                1 => {
                    /* Line. */
                    let new_point = curv.points().borrow()[1].position;
                    cr.line_to(new_point.x, new_point.y);
                    pen_position = Some(new_point);
                }
                2 => {
                    /* Quadratic. */
                    let a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        curv.points().borrow()[0].position
                    };
                    let b = curv.points().borrow()[1].position;
                    let c = curv.points().borrow()[2].position;
                    cr.curve_to(
                        2.0 / 3.0 * b.x + 1.0 / 3.0 * a.x,
                        2.0 / 3.0 * b.y + 1.0 / 3.0 * a.y,
                        2.0 / 3.0 * b.x + 1.0 / 3.0 * c.x,
                        2.0 / 3.0 * b.y + 1.0 / 3.0 * c.y,
                        c.x,
                        c.y,
                    );
                    pen_position = Some(c);
                }
                3 => {
                    /* Cubic */
                    let _a = if let Some(v) = pen_position.take() {
                        v
                    } else {
                        curv.points().borrow()[0].position
                    };
                    let b = curv.points().borrow()[1].position;
                    let c = curv.points().borrow()[2].position;
                    let d = curv.points().borrow()[3].position;
                    cr.curve_to(b.x, b.y, c.x, c.y, d.x, d.y);
                    pen_position = Some(d);
                }
                d => {
                    eprintln!("Something's wrong. Bezier of degree {}: {:?}", d, curv);
                    pen_position = Some(curv.points().borrow().last().unwrap().position);
                    continue;
                }
            }
        }
        cr.stroke().expect("Invalid cairo surface state");
        if let Some(pos) = pen_position {
            cr.move_to(pos.x, pos.y);
            draw_handle(pos, state.last_point.position);
            cr.stroke().expect("Invalid cairo surface state");
        }
        cr.restore().expect("Invalid cairo surface state");

        Inhibit(true)
        */
    }
}
