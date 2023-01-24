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

use glib::{
    clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecString, Value,
};
use gtk::cairo::Matrix;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use indexmap::IndexMap;
use once_cell::unsync::OnceCell;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::glyphs::{Contour, Glyph, GlyphDrawingOptions, GlyphPointIndex, Guideline};
use crate::project::Project;
use crate::utils::Point;
use crate::views::{
    canvas::{Layer, LayerBuilder},
    overlay::Child,
};
use crate::Settings;

//mod bezier_pen;
mod layers;
mod tools;
mod visibility_toggles;

use super::{Canvas, Transformation, UnitPoint, ViewPoint};
use tools::{PanningTool, Tool, ToolImpl};

#[derive(Debug, Clone)]
pub enum ControlPointKind {
    Endpoint { handle: Option<GlyphPointIndex> },
    Handle { end_points: Vec<GlyphPointIndex> },
}

use ControlPointKind::*;

#[derive(Debug, Clone)]
pub struct ControlPoint {
    contour_index: usize,
    curve_index: usize,
    point_index: usize,
    position: Point,
    kind: ControlPointKind,
}

#[derive(Debug, Clone)]
pub struct GlyphState {
    pub app: gtk::Application,
    pub glyph: Rc<RefCell<Glyph>>,
    pub reference: Rc<RefCell<Glyph>>,
    pub viewport: Canvas,
    pub tools: IndexMap<glib::types::Type, ToolImpl>,
    pub active_tool: glib::types::Type,
    pub panning_tool: glib::types::Type,
    selection: Vec<GlyphPointIndex>,
    pub points: Rc<RefCell<HashMap<GlyphPointIndex, ControlPoint>>>,
    pub kd_tree: Rc<RefCell<crate::utils::range_query::KdTree>>,
}

impl GlyphState {
    fn new(glyph: &Rc<RefCell<Glyph>>, app: gtk::Application, viewport: Canvas) -> Self {
        let mut ret = Self {
            app,
            glyph: Rc::new(RefCell::new(glyph.borrow().clone())),
            reference: Rc::clone(glyph),
            viewport,
            tools: IndexMap::default(),
            active_tool: glib::types::Type::INVALID,
            panning_tool: PanningTool::static_type(),
            selection: vec![],
            points: Rc::new(RefCell::new(HashMap::default())),
            kd_tree: Rc::new(RefCell::new(crate::utils::range_query::KdTree::new(&[]))),
        };

        for (contour_index, contour) in glyph.borrow().contours.iter().enumerate() {
            (ret.add_contour(contour, contour_index).redo)();
        }
        ret
    }

    fn add_contour(&mut self, contour: &Contour, contour_index: usize) -> crate::Action {
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "contour",
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[contour_index]).into() },
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.points as points, @weak self.kd_tree as kd_tree, @weak contour as contour  => move || {
                    let mut points = points.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for (curve_index, curve) in contour.curves().borrow().iter().enumerate() {
                        match curve.points().borrow().len() {
                            4 => {
                                for (endpoint, handle) in [(0, 1), (3, 2)] {
                                    let end_p = &curve.points().borrow()[endpoint];
                                    let handle_p = &curve.points().borrow()[handle];
                                    points.insert(
                                        end_p.glyph_index(contour_index, curve_index),
                                        ControlPoint {
                                            contour_index,
                                            curve_index,
                                            point_index: endpoint,
                                            position: *end_p,
                                            kind: Endpoint {
                                                handle: Some(handle_p.glyph_index(contour_index, curve_index)),
                                            },
                                        },
                                    );
                                    kd_tree.add(end_p.glyph_index(contour_index, curve_index), *end_p);
                                    points.insert(
                                        handle_p.glyph_index(contour_index, curve_index),
                                        ControlPoint {
                                            contour_index,
                                            curve_index,
                                            point_index: handle,
                                            position: *handle_p,
                                            kind: Handle {
                                                end_points: vec![end_p.glyph_index(contour_index, curve_index)],
                                            },
                                        },
                                    );
                                    kd_tree.add(handle_p.glyph_index(contour_index, curve_index), *handle_p);
                                }
                            }
                            3 => {
                                let p0 = &curve.points().borrow()[0];
                                let p1 = &curve.points().borrow()[1];
                                let p2 = &curve.points().borrow()[2];
                                points.insert(
                                    p0.glyph_index(contour_index, curve_index),
                                    ControlPoint {
                                        contour_index,
                                        curve_index,
                                        point_index: 0,
                                        position: *p0,
                                        kind: Endpoint {
                                            handle: Some(p1.glyph_index(contour_index, curve_index)),
                                        },
                                    },
                                );
                                points.insert(
                                    p1.glyph_index(contour_index, curve_index),
                                    ControlPoint {
                                        contour_index,
                                        curve_index,
                                        point_index: 1,
                                        position: *p1,
                                        kind: Handle {
                                            end_points: vec![
                                                p0.glyph_index(contour_index, curve_index),
                                                p2.glyph_index(contour_index, curve_index),
                                            ],
                                        },
                                    },
                                );
                                points.insert(
                                    p2.glyph_index(contour_index, curve_index),
                                    ControlPoint {
                                        contour_index,
                                        curve_index,
                                        point_index: 2,
                                        position: *p2,
                                        kind: Endpoint {
                                            handle: Some(p1.glyph_index(contour_index, curve_index)),
                                        },
                                    },
                                );
                                for p in [p0, p1, p2] {
                                    kd_tree.add(p.glyph_index(contour_index, curve_index), *p);
                                }
                            }
                            2 => {
                                for endpoint in 0..=1 {
                                    let p = &curve.points().borrow()[endpoint];
                                    points.insert(
                                        p.glyph_index(contour_index, curve_index),
                                        ControlPoint {
                                            contour_index,
                                            curve_index,
                                            point_index: endpoint,
                                            position: *p,
                                            kind: Endpoint { handle: None },
                                        },
                                    );
                                    kd_tree.add(p.glyph_index(contour_index, curve_index), *p);
                                }
                            }
                            1 => {}
                            0 => {}
                            _ => unreachable!(), //FIXME
                        }
                    }
                }),
            ),
            undo: Box::new(
                clone!(@weak self.points as points, @weak self.kd_tree as kd_tree, @weak contour as contour => move || {
                    let mut points = points.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for (curve_index, curve) in contour.curves().borrow().iter().enumerate() {
                        match curve.points().borrow().len() {
                            4 => {
                                for (endpoint, handle) in [(0, 1), (3, 2)] {
                                    let end_p = &curve.points().borrow()[endpoint];
                                    let handle_p = &curve.points().borrow()[handle];
                                    points.remove(&end_p.glyph_index(contour_index, curve_index));
                                    kd_tree.remove(end_p.glyph_index(contour_index, curve_index), *end_p);
                                    points.remove(&handle_p.glyph_index(contour_index, curve_index));
                                    kd_tree.remove(handle_p.glyph_index(contour_index, curve_index), *handle_p);
                                }
                            }
                            3 => {
                                let p0 = &curve.points().borrow()[0];
                                let p1 = &curve.points().borrow()[1];
                                let p2 = &curve.points().borrow()[2];
                                points.remove(
                                    &p0.glyph_index(contour_index, curve_index),
                                );
                                points.remove(
                                    &p1.glyph_index(contour_index, curve_index),
                                );
                                points.remove(
                                    &p2.glyph_index(contour_index, curve_index),
                                );
                                for p in [p0, p1, p2] {
                                    kd_tree.remove(p.glyph_index(contour_index, curve_index), *p);
                                }
                            }
                            2 => {
                                for endpoint in 0..=1 {
                                    let p = &curve.points().borrow()[endpoint];
                                    points.remove(
                                        &p.glyph_index(contour_index, curve_index),
                                    );
                                    kd_tree.remove(p.glyph_index(contour_index, curve_index), *p);
                                }
                            }
                            1 => {}
                            0 => {}
                            _ => unreachable!(), //FIXME
                        }
                    }
                }),
            ),
        }
    }

    fn new_guideline(&self, angle: f64, p: Point) -> crate::Action {
        let (x, y) = (p.x, p.y);
        let viewport = self.viewport.clone();
        let guideline = Guideline::builder()
            .angle(angle)
            .x(x)
            .y(y)
            .with_random_identifier()
            .build();
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: Box::new([]),
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport, @strong guideline => move || {
                    glyph.borrow_mut().guidelines.push(guideline.clone());
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow_mut().guidelines.pop();
                    viewport.queue_draw();
                }),
            ),
        }
    }

    fn update_guideline(&self, idx: usize, position: Point) -> crate::Action {
        let viewport = self.viewport.clone();
        let old_position: Point = {
            let g = self.glyph.borrow();
            let x = g.guidelines[idx].property("x");
            let y = g.guidelines[idx].property("y");
            (x, y).into()
        };
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[idx]).into() },
            },
            compress: true,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow().guidelines[idx].set_property("x", position.x);
                    glyph.borrow().guidelines[idx].set_property("y", position.y);
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow().guidelines[idx].set_property("x", old_position.x);
                    glyph.borrow().guidelines[idx].set_property("y", old_position.y);
                    viewport.queue_draw();
                }),
            ),
        }
    }

    #[allow(dead_code)]
    fn delete_guideline(&self, idx: usize) -> crate::Action {
        let viewport = self.viewport.clone();
        let json: serde_json::Value =
            { serde_json::to_value(self.glyph.borrow().guidelines[idx].imp()).unwrap() };
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[idx]).into() },
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow_mut().guidelines.remove(idx);
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow_mut().guidelines.push(Guideline::try_from(json.clone()).unwrap());
                    viewport.queue_draw();
                }),
            ),
        }
    }

    fn add_undo_action(&self, action: crate::Action) {
        let app: &crate::Application =
            crate::Application::from_instance(self.app.downcast_ref::<crate::GerbApp>().unwrap());
        app.undo_db.borrow_mut().event(action);
    }

    fn transform_selection(&self, m: Matrix) {
        let mut action = self.transform_points(&self.selection, m);
        (action.redo)();
        self.add_undo_action(action);
    }

    fn transform_points(&self, idxs: &[GlyphPointIndex], m: Matrix) -> crate::Action {
        let viewport = self.viewport.clone();
        let idxs = Rc::new(idxs.to_vec());
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "point",
                id: unsafe { std::mem::transmute::<&[GlyphPointIndex], &[u8]>(&idxs).into() },
            },
            compress: true,
            redo: Box::new(
                clone!(@strong idxs, @weak self.points as points, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak viewport => move || {
                    let mut points = points.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for idx in idxs.iter() {
                        if let Some(p) = points.get_mut(idx) {
                            let old_pos = p.position;
                            let new_pos = m * old_pos;
                            /* update kd_tree */
                            assert!(kd_tree.remove(*idx, p.position));
                            kd_tree.add(*idx, new_pos);

                            /* finally update actual point */
                            p.position = new_pos;

                            let glyph = glyph.borrow();
                            let curves = glyph.contours[p.contour_index].curves().borrow_mut();
                            curves[p.curve_index]
                                .points()
                                .borrow_mut()[p.point_index] = new_pos;
                        }
                    }
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@strong idxs, @weak self.points as points, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak viewport => move || {
                    let m = if let Ok(m) = m.try_invert() {m} else {return;};

                    let mut points = points.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for idx in idxs.iter() {
                        if let Some(ref mut p) = points.get_mut(idx) {
                            let old_pos = p.position;
                            let new_pos = m * old_pos;
                            /* update kd_tree */
                            assert!(kd_tree.remove(*idx, old_pos));
                            kd_tree.add(*idx, new_pos);

                            /* finally update actual point */
                            p.position = new_pos;
                            let glyph = glyph.borrow();
                            let curves = glyph.contours[p.contour_index].curves().borrow_mut();
                            curves[p.curve_index]
                                .points()
                                .borrow_mut()[p.point_index] = new_pos;
                        }
                    }
                    viewport.queue_draw();
                }),
            ),
        }
    }

    fn set_selection(&mut self, selection: &[(GlyphPointIndex, crate::utils::IPoint)]) {
        self.selection.clear();
        self.selection.extend(selection.iter().map(|(u, _)| u));
    }

    fn get_selection(&self) -> &[GlyphPointIndex] {
        &self.selection
    }
}

#[derive(Debug, Default)]
pub struct GlyphEditViewInner {
    app: OnceCell<gtk::Application>,
    project: OnceCell<Project>,
    glyph: OnceCell<Rc<RefCell<Glyph>>>,
    glyph_state: OnceCell<Rc<RefCell<GlyphState>>>,
    viewport: Canvas,
    statusbar_context_id: Cell<Option<u32>>,
    overlay: super::Overlay,
    hovering: Cell<Option<(usize, usize)>>,
    pub toolbar_box: gtk::Box,
    pub viewhidebox: OnceCell<visibility_toggles::ViewHideBox>,
    units_per_em: Cell<f64>,
    descender: Cell<f64>,
    x_height: Cell<f64>,
    cap_height: Cell<f64>,
    ascender: Cell<f64>,
    lock_guidelines: Cell<bool>,
    settings: OnceCell<Settings>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphEditViewInner {
    const NAME: &'static str = "GlyphEditView";
    type Type = GlyphEditView;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GlyphEditViewInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.lock_guidelines.set(true);
        self.statusbar_context_id.set(None);
        self.viewport.set_mouse(ViewPoint((0.0, 0.0).into()));

        self.viewport.connect_scroll_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                if event.state().contains(gtk::gdk::ModifierType::SHIFT_MASK) {
                    let (mut dx, mut dy) = event.delta();
                    if event.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                        if dy.abs() > dx.abs() {
                            dx = dy;
                        }
                        dy = 0.0;
                    }
                    viewport.imp().transformation.move_camera_by_delta(ViewPoint(<_ as Into<Point>>::into((5.0 * dx, 5.0 * dy))));
                    viewport.queue_draw();
                    return Inhibit(false);
                }
                Inhibit(false)
            }),
        );

        self.viewport.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let retval = Tool::on_button_press_event(obj, viewport, event);
                if retval == Inhibit(true) {
                    viewport.queue_draw();
                }
                viewport.set_mouse(ViewPoint(event.position().into()));
                retval
            }),
        );

        self.viewport.connect_button_release_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let retval = Tool::on_button_release_event(obj, viewport, event);
                if retval == Inhibit(true) {
                    viewport.queue_draw();
                }
                viewport.set_mouse(ViewPoint(event.position().into()));
                retval
            }),
        );

        self.viewport.connect_motion_notify_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let retval = Tool::on_motion_notify_event(obj, viewport, event);
                viewport.set_mouse(ViewPoint(event.position().into()));
                if let Inhibit(true) = retval {
                    viewport.queue_draw();
                }
                retval
            }),
        );

        self.viewport.add_layer(
            LayerBuilder::new()
                .set_name(Some("glyph"))
                .set_active(true)
                .set_hidden(false)
                .set_callback(Some(Box::new(clone!(@weak obj => @default-return Inhibit(false), move |viewport: &Canvas, cr: &gtk::cairo::Context| {
                    layers::draw_glyph_layer(viewport, cr, obj)
                }))))
                .build(),
        );
        self.viewport.add_pre_layer(
            LayerBuilder::new()
                .set_name(Some("guidelines"))
                .set_active(true)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak obj => @default-return Inhibit(false), move |viewport: &Canvas, cr: &gtk::cairo::Context| {
                    layers::draw_guidelines(viewport, cr, obj)
                }))))
                .build(),
        );
        self.viewport.add_post_layer(
            LayerBuilder::new()
                .set_name(Some("rules"))
                .set_active(true)
                .set_hidden(true)
                .set_callback(Some(Box::new(Canvas::draw_rulers)))
                .build(),
        );
        let viewhidebox = visibility_toggles::ViewHideBox::new(&self.viewport);
        self.overlay.set_child(&self.viewport);
        self.overlay.add_overlay(Child::new(
            gtk::Expander::builder()
                .child(&viewhidebox)
                .expanded(false)
                .visible(true)
                .can_focus(true)
                .tooltip_text("Toggle overlay visibilities")
                .halign(gtk::Align::End)
                .valign(gtk::Align::End)
                .build(),
            true,
        ));
        self.overlay
            .add_overlay(Child::new(self.toolbar_box.clone(), true));
        let listbox = gtk::ListBox::builder()
            .name("layers")
            .expand(false)
            .visible(true)
            .can_focus(true)
            .tooltip_text("layers")
            .halign(gtk::Align::Start)
            .valign(gtk::Align::End)
            .build();
        let label = gtk::Label::new(Some("layers"));
        label.set_visible(true);
        listbox.add(&label);
        for layer in self
            .viewport
            .imp()
            .pre_layers
            .borrow()
            .iter()
            .chain(self.viewport.imp().layers.borrow().iter())
            .chain(self.viewport.imp().post_layers.borrow().iter())
        {
            let label = gtk::Label::new(Some(&layer.property::<String>(Layer::NAME)));
            label.set_visible(true);
            let button = gtk::ToggleButton::builder()
                .child(&label)
                .visible(true)
                .active(true)
                .build();
            layer
                .bind_property(Layer::ACTIVE, &button, "active")
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();
            button.connect_toggled(clone!(@strong self.viewport as viewport => move |button| {
                if button.is_active() {
                    button.style_context().add_class("active");
                } else {
                    button.style_context().remove_class("active");
                }
                viewport.queue_draw();
            }));
            button.toggled();
            listbox.add(&button);
        }
        self.overlay.add_overlay(Child::new(listbox, true));
        obj.add(&self.overlay);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);

        self.viewhidebox
            .set(viewhidebox)
            .expect("Failed to initialize window state");
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        GlyphEditView::TITLE,
                        GlyphEditView::TITLE,
                        GlyphEditView::TITLE,
                        Some("edit glyph"),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        GlyphEditView::CLOSEABLE,
                        GlyphEditView::CLOSEABLE,
                        GlyphEditView::CLOSEABLE,
                        true,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::UNITS_PER_EM,
                        GlyphEditView::UNITS_PER_EM,
                        GlyphEditView::UNITS_PER_EM,
                        1.0,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::X_HEIGHT,
                        GlyphEditView::X_HEIGHT,
                        GlyphEditView::X_HEIGHT,
                        1.0,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::ASCENDER,
                        GlyphEditView::ASCENDER,
                        GlyphEditView::ASCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        700.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::DESCENDER,
                        GlyphEditView::DESCENDER,
                        GlyphEditView::DESCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        -200.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::CAP_HEIGHT,
                        GlyphEditView::CAP_HEIGHT,
                        GlyphEditView::CAP_HEIGHT,
                        std::f64::MIN,
                        std::f64::MAX,
                        650.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        GlyphEditView::LOCK_GUIDELINES,
                        GlyphEditView::LOCK_GUIDELINES,
                        GlyphEditView::LOCK_GUIDELINES,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        GlyphEditView::ACTIVE_TOOL,
                        GlyphEditView::ACTIVE_TOOL,
                        GlyphEditView::ACTIVE_TOOL,
                        ToolImpl::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        GlyphEditView::PANNING_TOOL,
                        GlyphEditView::PANNING_TOOL,
                        GlyphEditView::PANNING_TOOL,
                        ToolImpl::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            GlyphEditView::TITLE => {
                if let Some(name) = obj
                    .imp()
                    .glyph_state
                    .get()
                    .map(|s| s.borrow().glyph.borrow().name_markup())
                {
                    format!("edit <i>{}</i>", name).to_value()
                } else {
                    "edit glyph".to_value()
                }
            }
            GlyphEditView::CLOSEABLE => true.to_value(),
            GlyphEditView::UNITS_PER_EM => self.units_per_em.get().to_value(),
            GlyphEditView::X_HEIGHT => self.x_height.get().to_value(),
            GlyphEditView::ASCENDER => self.ascender.get().to_value(),
            GlyphEditView::DESCENDER => self.descender.get().to_value(),
            GlyphEditView::CAP_HEIGHT => self.cap_height.get().to_value(),
            GlyphEditView::LOCK_GUIDELINES => self.lock_guidelines.get().to_value(),
            GlyphEditView::ACTIVE_TOOL => {
                let state = self.glyph_state.get().unwrap().borrow();
                let active_tool = state.active_tool;
                state.tools.get(&active_tool).map(Clone::clone).to_value()
            }
            GlyphEditView::PANNING_TOOL => {
                let state = self.glyph_state.get().unwrap().borrow();
                let panning_tool = state.panning_tool;
                state.tools.get(&panning_tool).map(Clone::clone).to_value()
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            GlyphEditView::UNITS_PER_EM => {
                self.units_per_em.set(value.get().unwrap());
            }
            GlyphEditView::X_HEIGHT => {
                self.x_height.set(value.get().unwrap());
            }
            GlyphEditView::ASCENDER => {
                self.ascender.set(value.get().unwrap());
            }
            GlyphEditView::DESCENDER => {
                self.descender.set(value.get().unwrap());
            }
            GlyphEditView::CAP_HEIGHT => {
                self.cap_height.set(value.get().unwrap());
            }
            GlyphEditView::LOCK_GUIDELINES => {
                self.lock_guidelines.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl WidgetImpl for GlyphEditViewInner {}
impl ContainerImpl for GlyphEditViewInner {}
impl BinImpl for GlyphEditViewInner {}

impl GlyphEditViewInner {
    fn new_statusbar_message(&self, msg: &str) {
        if let Some(app) = self
            .app
            .get()
            .and_then(|app| app.downcast_ref::<crate::GerbApp>())
        {
            let statusbar = app.statusbar();
            if self.statusbar_context_id.get().is_none() {
                self.statusbar_context_id.set(Some(
                    statusbar
                        .context_id(&format!("GlyphEditView-{:?}", &self.glyph.get().unwrap())),
                ));
            }
            if let Some(cid) = self.statusbar_context_id.get().as_ref() {
                statusbar.push(*cid, msg);
            }
        }
    }

    fn select_object(&self, _new_obj: Option<glib::Object>) {
        if let Some(_app) = self
            .app
            .get()
            .and_then(|app| app.downcast_ref::<crate::GerbApp>())
        {
            //let tabinfo = app.tabinfo();
            //tabinfo.set_object(new_obj);
        }
    }
}

glib::wrapper! {
    pub struct GlyphEditView(ObjectSubclass<GlyphEditViewInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl GlyphEditView {
    pub const ASCENDER: &str = Project::ASCENDER;
    pub const CAP_HEIGHT: &str = Project::CAP_HEIGHT;
    pub const CLOSEABLE: &str = "closeable";
    pub const DESCENDER: &str = Project::DESCENDER;
    pub const TITLE: &str = "title";
    pub const UNITS_PER_EM: &str = Project::UNITS_PER_EM;
    pub const X_HEIGHT: &str = Project::X_HEIGHT;
    pub const LOCK_GUIDELINES: &str = "lock-guidelines";
    pub const ACTIVE_TOOL: &str = "active-tool";
    pub const PANNING_TOOL: &str = "panning-tool";

    pub fn new(app: gtk::Application, project: Project, glyph: Rc<RefCell<Glyph>>) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp().glyph.set(glyph.clone()).unwrap();
        ret.imp().app.set(app.clone()).unwrap();
        {
            let property = GlyphEditView::UNITS_PER_EM;
            ret.bind_property(property, &ret.imp().viewport.imp().transformation, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        for property in [
            GlyphEditView::ASCENDER,
            GlyphEditView::CAP_HEIGHT,
            GlyphEditView::DESCENDER,
            GlyphEditView::UNITS_PER_EM,
            GlyphEditView::X_HEIGHT,
        ] {
            project
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        let settings = app
            .downcast_ref::<crate::GerbApp>()
            .unwrap()
            .imp()
            .settings
            .borrow()
            .clone();
        settings
            .bind_property(
                Canvas::WARP_CURSOR,
                &ret.imp().viewport,
                Canvas::WARP_CURSOR,
            )
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        for prop in [Settings::HANDLE_SIZE, Settings::LINE_WIDTH] {
            settings.connect_notify_local(
                Some(prop),
                clone!(@strong ret => move |_self, _| {
                    ret.imp().viewport.queue_draw();
                }),
            );
        }
        ret.imp().settings.set(settings).unwrap();
        let action_map = gtk::gio::SimpleActionGroup::new();
        for prop in [
            Canvas::SHOW_GRID,
            Canvas::SHOW_GUIDELINES,
            Canvas::SHOW_HANDLES,
            Canvas::INNER_FILL,
            Canvas::SHOW_TOTAL_AREA,
        ] {
            let prop_action = gtk::gio::PropertyAction::new(prop, &ret.imp().viewport, prop);
            action_map.add_action(&prop_action);
        }
        ret.insert_action_group("edit", Some(&action_map));
        ret.imp()
            .glyph_state
            .set(Rc::new(RefCell::new(GlyphState::new(
                &glyph,
                app,
                ret.imp().viewport.clone(),
            ))))
            .expect("Failed to create glyph state");
        Tool::setup_toolbox(&ret);
        ret.imp().project.set(project).unwrap();
        ret
    }

    pub fn make_debug_window(&self) {
        let glyph_state = self.imp().glyph_state.get().unwrap().borrow();
        let glyph = glyph_state.glyph.borrow();
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_default_size(640, 480);
        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::Fill)
            .expand(false)
            .spacing(5)
            .visible(true)
            .can_focus(true)
            .build();
        let glyph_info = gtk::Label::new(Some(&format!(
            "{:#?}",
            glyph.contours.iter().map(Contour::imp).collect::<Vec<_>>()
        )));
        glyph_info.set_halign(gtk::Align::Start);
        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .margin_start(5)
            .build();
        scrolled_window.set_child(Some(&glyph_info));
        hbox.pack_start(&scrolled_window, true, true, 0);
        hbox.pack_start(
            &gtk::Separator::new(gtk::Orientation::Horizontal),
            false,
            true,
            0,
        );
        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .margin_start(5)
            .build();
        let glif_info = gtk::Label::new(Some(&glyph.glif_source));
        glif_info.set_halign(gtk::Align::Start);
        scrolled_window.set_child(Some(&glif_info));
        hbox.pack_start(&scrolled_window, true, true, 0);
        #[cfg(feature = "svg")]
        {
            let save_to_svg = gtk::Button::builder()
                .label("Save to SVG")
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .visible(true)
                .build();
            save_to_svg.connect_clicked(
                clone!(@strong glyph_state.glyph as glyph, @strong window => move |_| {
                    let dialog = gtk::FileChooserDialog::builder()
                        .create_folders(true)
                        .do_overwrite_confirmation(true)
                        .action(gtk::FileChooserAction::Save)
                        .visible(true)
                        .build();
                    dialog.add_button("Save", gtk::ResponseType::Ok);
                    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                    dialog.set_current_name(&format!("{}.svg", glyph.borrow().name.as_ref()));
                    if dialog.run() == gtk::ResponseType::Ok {
                        if let Some(f) = dialog.filename() {
                            if let Err(err) = glyph.borrow().save_to_svg(f) {
                                let dialog = gtk::MessageDialog::new(
                                    Some(&window),
                                    gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Close,
                                    &err.to_string(),
                                );
                                dialog.set_title("Error: Could not svg file");
                                dialog.set_use_markup(true);
                                dialog.run();
                                dialog.hide();
                            }
                        }
                    }
                    dialog.hide();
                }),
            );
            hbox.pack_start(&save_to_svg, false, true, 5);
        }

        window.add(&hbox);
        window.show_all();
    }
}
