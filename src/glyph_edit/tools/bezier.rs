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
use gtk::cairo::Matrix;
use gtk::Inhibit;

use crate::glyphs::{Contour, GlyphPointIndex};
use crate::prelude::*;
use crate::utils::{curves::Bezier, distance_between_two_points};

///```text
///   States                             Beginning                        Before transition
///==============================================================================================================
///   1) FirstHandle   { !handle! }      Contour { [p1,] }                Contour { [p1, handle == p2] }
///
///-> 2) OnCurve                         Contour {                        Contour {
///                                               [p1, p2, !p3, !p3]               [p1, p2, p3, p4 == p3]
///                                               }                                }
///
///-> 3) SecondHandle { !handle! }       Contour {                        Contour {
///                                               [p1, p2, !p3, p4]                [p1, p2, p3, p4],
///                                                                                [p5 == handle, ],
///                                               }                                }
///
///-> 4) FirstHandle { !handle! }        Contour {                        Contour {
///                                                [p1, p2, !p3, p4],               [p1, p2, p3, p4],
///                                                [p5,]                            [p5, p6 == handle]
///                                               }                                }
///
///-> 5) OnCurve                         Contour { [p1, p2, p3, p4],
///                                                [p5, p6, !p7 !p7]
///                                              }
///-> 6) ClosingHandle { !handle! }      Contour {                        Contour {
///                                              [p1, !p2, !p3, p4],                [p1, p2, p3, p4],
///                                              [p5, p6, !p7, p8 == p1],           [p5, p6, p7, p1],
///                                             ([p1, !p2,..])                      [p1, p2,..]
///                                              }                                }
///```
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum InnerState {
    #[default]
    Empty,
    FirstHandle {
        handle: Point,
        unlinked: bool,
        snap_to_angle: bool,
    },
    OnCurve,
    SecondHandle {
        handle: Point,
        unlinked: bool,
        snap_to_angle: bool,
    },
    ClosingHandle {
        handle: Point,
        unlinked: bool,
        snap_to_angle: bool,
    },
}

struct ContourState {
    first_point: Point,
    contour: Contour,
    first_curve: Bezier,
    current_curve: Bezier,
    last_point: CurvePoint,
    curve_index: usize,
    contour_index: usize,
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
            .field("contour_index", &self.contour_index)
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
            crate::resources::icons::BEZIER_ICON.to_image_widget(),
        );
        self.cursor
            .set(crate::resources::cursors::PEN_CURSOR.to_pixbuf())
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
            let UnitPoint(point) = viewport.view_to_unit_point(ViewPoint(event.position().into()));
            if c.is_none() {
                let mut glyph_state = view.glyph_state.get().unwrap().borrow_mut();
                let current_curve = Bezier::new(vec![point]);
                current_curve.set_property(Bezier::SMOOTH, true);
                let contour_index = glyph_state.glyph.borrow().contours.len();
                let new_state = ContourState {
                    last_point: {
                        let p = current_curve.points().borrow()[0].clone();
                        p
                    },
                    first_point: point,
                    contour: {
                        let contour = Contour::new();
                        contour.push_curve(current_curve.clone());
                        contour
                    },
                    first_curve: current_curve.clone(),
                    current_curve,
                    curve_index: 0,
                    contour_index,
                };
                let subaction = glyph_state.add_contour(&new_state.contour, contour_index);
                let mut action = new_contour_action(
                    glyph_state.glyph.clone(),
                    new_state.contour.clone(),
                    subaction,
                );
                (action.redo)();
                glyph_state.add_undo_action(action);
                *c = Some(new_state);
                self.inner.set(InnerState::FirstHandle {
                    handle: point,
                    unlinked: false,
                    snap_to_angle: false,
                });
                return Inhibit(true);
            }

            if matches!(
                self.inner.get(),
                InnerState::OnCurve | InnerState::SecondHandle { .. }
            ) {
                self.insert_point(obj, &view, c, point);
                return Inhibit(true);
            }
        } else if event.button() == gtk::gdk::BUTTON_SECONDARY {
            let c = self.contour.borrow_mut();
            if c.is_none() {
                return Inhibit(false);
            }
            if matches!(self.inner.get(), InnerState::ClosingHandle { .. }) {
                let UnitPoint(point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                self.insert_point(obj, &view, c, point);
            } else {
                self.close(obj, &view, c);
            }

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
            (gtk::gdk::BUTTON_PRIMARY, InnerState::Empty) => Inhibit(true),
            (gtk::gdk::BUTTON_PRIMARY, _) => {
                let c = self.contour.borrow_mut();
                if c.is_none() {
                    return Inhibit(false);
                }
                let UnitPoint(point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                self.insert_point(obj, &view, c, point);
                viewport.queue_draw();
                Inhibit(true)
            }
            _ => Inhibit(false),
        }
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
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

        /* first update state if any modifiers are active (Shift, Ctrl, etc) */
        match inner {
            InnerState::FirstHandle {
                handle,
                unlinked,
                snap_to_angle,
            } if event_state.intersects(gtk::gdk::ModifierType::SHIFT_MASK) => {
                /* toggle snap */
                self.inner.set(InnerState::FirstHandle {
                    handle,
                    unlinked,
                    snap_to_angle: !snap_to_angle,
                });
            }
            InnerState::SecondHandle {
                handle,
                unlinked,
                snap_to_angle,
            } if event_state.intersects(gtk::gdk::ModifierType::SHIFT_MASK) => {
                /* toggle snap */
                self.inner.set(InnerState::SecondHandle {
                    handle,
                    unlinked,
                    snap_to_angle: !snap_to_angle,
                });
            }
            InnerState::ClosingHandle {
                handle,
                unlinked,
                snap_to_angle,
            } if event_state.intersects(gtk::gdk::ModifierType::SHIFT_MASK) => {
                /* toggle snap */
                self.inner.set(InnerState::ClosingHandle {
                    handle,
                    unlinked,
                    snap_to_angle: !snap_to_angle,
                });
            }
            InnerState::FirstHandle {
                handle,
                unlinked,
                snap_to_angle,
            } if event_state.intersects(
                gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK,
            ) =>
            {
                /* toggle linked */
                self.inner.set(InnerState::FirstHandle {
                    handle,
                    unlinked: !unlinked,
                    snap_to_angle,
                });
                let state = c.as_mut().unwrap();
                state.current_curve.set_property(Bezier::SMOOTH, unlinked);
            }
            InnerState::SecondHandle {
                handle,
                unlinked,
                snap_to_angle,
            } if event_state.intersects(
                gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK,
            ) =>
            {
                /* toggle linked */
                self.inner.set(InnerState::SecondHandle {
                    handle,
                    unlinked: !unlinked,
                    snap_to_angle,
                });
                let state = c.as_mut().unwrap();
                state.current_curve.set_property(Bezier::SMOOTH, unlinked);
            }
            InnerState::ClosingHandle {
                handle,
                unlinked,
                snap_to_angle,
            } if event_state.intersects(
                gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK,
            ) =>
            {
                /* toggle linked */
                self.inner.set(InnerState::ClosingHandle {
                    handle,
                    unlinked: !unlinked,
                    snap_to_angle,
                });
                let state = c.as_mut().unwrap();
                state.current_curve.set_property(Bezier::SMOOTH, unlinked);
            }
            _ => {}
        }

        let inner = self.inner.get();

        match inner {
            InnerState::Empty => {}
            InnerState::FirstHandle {
                handle: _,
                unlinked,
                snap_to_angle,
            } => {
                let state = c.as_mut().unwrap();
                let UnitPoint(point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                let handle = point;
                if !unlinked && state.curve_index != 0 {
                    let linked_curve_point = state.contour.curves().borrow()[state.curve_index - 1]
                        .points()
                        .borrow()
                        .iter()
                        .rev()
                        .nth(1)
                        .unwrap()
                        .clone();
                    let new_mirrored_position = handle.mirror(state.last_point.position);
                    let diff_vector = new_mirrored_position - linked_curve_point.position;
                    let mut m = Matrix::identity();
                    m.translate(diff_vector.x, diff_vector.y);
                    self.transform_point(
                        m,
                        &view,
                        state,
                        state.curve_index - 1,
                        linked_curve_point,
                    );
                }
                self.inner.set(InnerState::FirstHandle {
                    handle,
                    unlinked,
                    snap_to_angle,
                });
                return Inhibit(true);
            }
            InnerState::SecondHandle {
                handle: _,
                unlinked,
                snap_to_angle,
            } => {
                let state = c.as_mut().unwrap();
                let UnitPoint(point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                let handle = point;
                if !unlinked {
                    let linked_curve_point = state.contour.curves().borrow()[state.curve_index]
                        .points()
                        .borrow()
                        .iter()
                        .rev()
                        .nth(1)
                        .unwrap()
                        .clone();
                    let new_mirrored_position = handle.mirror(state.last_point.position);
                    let diff_vector = new_mirrored_position - linked_curve_point.position;
                    let mut m = Matrix::identity();
                    m.translate(diff_vector.x, diff_vector.y);
                    self.transform_point(m, &view, state, state.curve_index, linked_curve_point);
                }
                self.inner.set(InnerState::SecondHandle {
                    handle,
                    unlinked,
                    snap_to_angle,
                });
                return Inhibit(true);
            }
            InnerState::ClosingHandle {
                handle: _,
                unlinked,
                snap_to_angle,
            } => {
                let state = c.as_mut().unwrap();
                let UnitPoint(point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                let handle = point;
                {
                    let handle_point = state
                        .current_curve
                        .points()
                        .borrow()
                        .iter()
                        .rev()
                        .nth(1)
                        .unwrap()
                        .clone();
                    let diff_vector = handle - handle_point.position;
                    let mut m = Matrix::identity();
                    m.translate(diff_vector.x, diff_vector.y);
                    self.transform_point(m, &view, state, state.curve_index, handle_point);
                }
                if !unlinked {
                    let linked_curve_point = state.contour.curves().borrow()[0]
                        .points()
                        .borrow()
                        .iter()
                        .nth(1)
                        .unwrap()
                        .clone();
                    let new_mirrored_position = handle.mirror(state.first_point);
                    let diff_vector = new_mirrored_position - linked_curve_point.position;
                    let mut m = Matrix::identity();
                    m.translate(diff_vector.x, diff_vector.y);
                    self.transform_point(m, &view, state, 0, linked_curve_point);
                }
                self.inner.set(InnerState::ClosingHandle {
                    handle,
                    unlinked,
                    snap_to_angle,
                });
                return Inhibit(true);
            }
            InnerState::OnCurve => {
                let state = c.as_mut().unwrap();
                let UnitPoint(point) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                assert_eq!(
                    state.contour.curves().borrow()[state.curve_index]
                        .points()
                        .borrow()
                        .len(),
                    4
                );
                let curve_point1 = state.contour.curves().borrow()[state.curve_index]
                    .points()
                    .borrow()
                    .last()
                    .unwrap()
                    .clone();
                let curve_point2 = state.contour.curves().borrow()[state.curve_index]
                    .points()
                    .borrow()
                    .iter()
                    .rev()
                    .nth(1)
                    .unwrap()
                    .clone();
                assert_eq!(curve_point1.position, curve_point2.position);
                let diff_vector = point - curve_point1.position;
                if diff_vector.norm() >= 0.1 {
                    let mut m = Matrix::identity();
                    m.translate(diff_vector.x, diff_vector.y);
                    self.transform_point(m, &view, state, state.curve_index, curve_point1);
                }
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("bezier"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: ContextRef| {
                    BezierTool::draw_layer(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(BezierTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.viewport.add_post_layer(layer);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(BezierTool::ACTIVE, true);
        if let Some(pixbuf) = self.cursor.get().unwrap().clone() {
            view.viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.viewport.set_cursor("grab");
        }
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(BezierTool::ACTIVE, false);
        view.viewport.set_cursor("default");
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
            let add_to_kdtree =
                |state: &mut ContourState, curve_index: usize, curve_point: CurvePoint| {
                    let glyph_state = view.glyph_state.get().unwrap().borrow();
                    let contour_index = state.contour_index;
                    let uuid = curve_point.uuid;
                    let idx = GlyphPointIndex {
                        contour_index,
                        curve_index,
                        uuid,
                    };
                    let mut kd_tree = glyph_state.kd_tree.borrow_mut();
                    /* update kd_tree */
                    kd_tree.add(idx, curve_point.position);
                    glyph_state.viewport.queue_draw();
                };
            match self.inner.get() {
                InnerState::Empty => {
                    unreachable!()
                }
                InnerState::FirstHandle {
                    handle,
                    unlinked: _,
                    snap_to_angle: _,
                } => {
                    {
                        let curve_point = CurvePoint::new(handle);
                        state.last_point = curve_point.clone();
                        state.current_curve.points().borrow_mut().push(curve_point);
                        add_to_kdtree(state, state.curve_index, state.last_point.clone());
                    }

                    // p3 handle
                    let curve_point = CurvePoint::new(point);
                    state.last_point = curve_point.clone();
                    state.current_curve.points().borrow_mut().push(curve_point);
                    add_to_kdtree(state, state.curve_index, state.last_point.clone());

                    // p3 oncurve
                    let curve_point = CurvePoint::new(point);
                    state.last_point = curve_point.clone();
                    state.current_curve.points().borrow_mut().push(curve_point);
                    add_to_kdtree(state, state.curve_index, state.last_point.clone());

                    self.inner.set(InnerState::OnCurve);
                }
                InnerState::OnCurve
                    if distance_between_two_points(point, state.first_point) < 20.0 =>
                {
                    state.contour.close();
                    self.inner.set(InnerState::ClosingHandle {
                        handle: state.first_point,
                        unlinked: false,
                        snap_to_angle: false,
                    });
                }
                InnerState::ClosingHandle {
                    handle,
                    unlinked: _,
                    snap_to_angle: _,
                } => {
                    let previous_handle = state
                        .current_curve
                        .points()
                        .borrow()
                        .iter()
                        .rev()
                        .nth(1)
                        .unwrap()
                        .clone();
                    let diff_vector = handle - previous_handle.position;
                    let mut m = Matrix::identity();
                    m.translate(diff_vector.x, diff_vector.y);
                    self.transform_point(m, view, state, state.curve_index, previous_handle);
                    let curv = state.contour.curves().borrow_mut().pop();
                    if let Some(curv) = curv {
                        state.contour.continuities().borrow_mut().pop();
                        state.contour.push_curve(curv);
                        state.contour.close();
                    }
                    self.close(obj, view, state_opt);
                }
                InnerState::OnCurve => {
                    self.inner.set(InnerState::SecondHandle {
                        handle: point,
                        unlinked: false,
                        snap_to_angle: false,
                    });
                }
                InnerState::SecondHandle {
                    handle,
                    unlinked: _,
                    snap_to_angle: _,
                } => {
                    let new_bezier = Bezier::new(vec![]);
                    new_bezier.set_property(Bezier::SMOOTH, true);
                    let h = CurvePoint::new(
                        state
                            .current_curve
                            .points()
                            .borrow()
                            .last()
                            .unwrap()
                            .position,
                    );
                    state.last_point = h;
                    new_bezier
                        .points()
                        .borrow_mut()
                        .push(state.last_point.clone());
                    add_to_kdtree(state, state.curve_index + 1, state.last_point.clone());

                    let curve = std::mem::replace(&mut state.current_curve, new_bezier);

                    state.contour.curves().borrow_mut().pop();
                    state.contour.continuities().borrow_mut().pop();
                    state.contour.push_curve(curve);
                    state.contour.push_curve(state.current_curve.clone());
                    state.curve_index += 1;
                    self.inner.set(InnerState::FirstHandle {
                        handle,
                        unlinked: false,
                        snap_to_angle: false,
                    });
                    self.insert_point(obj, view, state_opt, handle);
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
            view.glyph_state.get().unwrap().borrow_mut().active_tool = glib::types::Type::INVALID;
            self.on_deactivate(obj, view);
            self.contour.borrow_mut().take();
        }
    }

    fn transform_point(
        &self,
        m: Matrix,
        view: &GlyphEditView,
        state: &mut ContourState,
        curve_index: usize,
        curve_point_to_move: CurvePoint,
    ) {
        let glyph_state = view.glyph_state.get().unwrap().borrow();
        let contour_index = state.contour_index;
        let uuid = curve_point_to_move.uuid;
        let idxs = [GlyphPointIndex {
            contour_index,
            curve_index,
            uuid,
        }];
        let mut kd_tree = glyph_state.kd_tree.borrow_mut();
        for (idx, new_pos) in state.contour.transform_points(contour_index, &idxs, m) {
            if idx.uuid == uuid && curve_point_to_move.uuid == state.last_point.uuid {
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

    pub fn draw_layer(viewport: &Canvas, cr: ContextRef, obj: GlyphEditView) -> Inhibit {
        let glyph_state = obj.glyph_state.get().unwrap().borrow();
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
        let line_width = obj
            .settings
            .get()
            .unwrap()
            .property::<f64>(Settings::LINE_WIDTH);
        let outline =
            Color::from_hex("#3333FF").with_alpha(if inner_fill { 0 } else { (0.6 * 255.0) as u8 });
        let matrix = viewport.transformation.matrix();
        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let handle_size: f64 = if viewport.property::<bool>(Canvas::SHOW_HANDLES) {
            obj.settings
                .get()
                .unwrap()
                .property::<f64>(Settings::HANDLE_SIZE)
                / (scale * ppu)
        } else {
            0.0
        };

        /*{
            let cr1 = cr.push();
            let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
            let _height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
            cr1.set_font_size(9.5);
            let line_height = cr1.text_extents("BezierTool").unwrap().height * 1.5;
            cr1.show_text("BezierTool").unwrap();
            for (i, line) in Some(format!("state: {:?}", t.imp().inner))
                .into_iter()
                //.chain(Some(format!("snap_to_angle: {:?}", t.imp().snap_to_angle)).into_iter())
                .chain(Some(format!("last_point: {:?}", &state.last_point)).into_iter())
                .chain(Some(format!("curve_index: {:?}", &state.curve_index)).into_iter())
                .chain(Some(format!("contour_index: {:?}", &state.contour_index)).into_iter())
                .chain(
                    format!("current_curve: {:#?}", state.current_curve.imp())
                        .lines()
                        .map(str::to_string),
                )
                .chain(
                    Some(format!(
                        "contour open: {:?}",
                        &state.contour.imp().open.get()
                    ))
                    .into_iter(),
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
                cr1.move_to(width / 2.0, 95.0 + (i + 1) as f64 * line_height);
                cr1.show_text(&line).unwrap();
            }
        }
        */
        cr.transform(matrix);
        cr.set_line_width(line_width);
        cr.set_source_color_alpha(outline);

        let draw_handle = |p: Point| {
            if inner_fill {
                cr.set_source_rgba(0.9, 0.9, 0.9, 1.0);
            } else {
                cr.set_source_rgba(0.0, 0.0, 1.0, 0.5);
            }
            cr.arc(p.x, p.y, handle_size / 2.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap();
            cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
            cr.arc(
                p.x,
                p.y,
                handle_size / 2.0 + 1.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cr.stroke().unwrap();
        };

        let draw_handle_connection = |h: Point, ep: Point| {
            cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
            cr.move_to(h.x - 2.5, h.y - 2.5);
            cr.line_to(ep.x, ep.y);
            cr.stroke().unwrap();
        };

        match t.imp().inner.get() {
            InnerState::Empty | InnerState::OnCurve => {}
            InnerState::FirstHandle { handle, .. } => {
                // draw handle
                draw_handle_connection(handle, state.current_curve.points().borrow()[0].position);
                draw_handle(handle);
            }
            InnerState::SecondHandle { handle, .. } => {
                // draw handle
                draw_handle_connection(
                    handle,
                    state
                        .current_curve
                        .points()
                        .borrow()
                        .last()
                        .unwrap()
                        .position,
                );
                draw_handle(handle);
            }
            InnerState::ClosingHandle { handle, .. } => {
                // draw handle
                draw_handle_connection(handle, state.first_point);
                draw_handle(handle);
            }
        }

        Inhibit(true)
    }
}
