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

use glib::{clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::glyphs::{Contour, Glyph, GlyphDrawingOptions, Guideline};
use crate::project::Project;

mod bezier_pen;
mod viewhide;

const EM_SQUARE_PIXELS: f64 = 200.0;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum MotionMode {
    _Zoom = 0,
    Pan,
}

#[derive(Debug, Clone)]
enum ControlPointKind {
    Endpoint { handle: Option<usize> },
    Handle { end_points: Vec<usize> },
}

use ControlPointKind::*;

#[derive(Debug, Clone)]
struct ControlPoint {
    contour_index: usize,
    curve_index: usize,
    point_index: usize,
    position: (i64, i64),
    kind: ControlPointKind,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ControlPointMode {
    None,
    Drag,
    DragGuideline(usize),
}

impl Default for ControlPointMode {
    fn default() -> ControlPointMode {
        ControlPointMode::None
    }
}

#[derive(Debug, Clone)]
enum Tool {
    Manipulate { mode: ControlPointMode },
    BezierPen { state: bezier_pen::State },
}

impl Default for Tool {
    fn default() -> Tool {
        Tool::Manipulate {
            mode: ControlPointMode::default(),
        }
    }
}

impl Tool {
    fn is_manipulate(&self) -> bool {
        if let Tool::Manipulate { .. } = self {
            true
        } else {
            false
        }
    }

    fn is_bezier_pen(&self) -> bool {
        if let Tool::BezierPen { .. } = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct GlyphState {
    app: gtk::Application,
    glyph: Rc<RefCell<Glyph>>,
    reference: Rc<RefCell<Glyph>>,
    selection: Vec<usize>,
    tool: Tool,
    points: Rc<RefCell<Vec<ControlPoint>>>,
    points_map: Rc<RefCell<HashMap<(i64, i64), Vec<usize>>>>,
    kd_tree: Rc<RefCell<crate::utils::range_query::KdTree>>,
    drar: gtk::DrawingArea,
}

impl GlyphState {
    fn new(glyph: &Rc<RefCell<Glyph>>, app: gtk::Application, drar: gtk::DrawingArea) -> Self {
        let control_points = Rc::new(RefCell::new(vec![]));
        let points_map: Rc<RefCell<HashMap<(i64, i64), Vec<usize>>>> =
            Rc::new(RefCell::new(HashMap::default()));

        let mut ret = GlyphState {
            // FIXME: needs a deep clone (duplicate) here
            glyph: Rc::new(RefCell::new(glyph.borrow().clone())),
            app,
            reference: Rc::clone(glyph),
            points: control_points,
            points_map,
            tool: Tool::default(),
            selection: vec![],
            kd_tree: Rc::new(RefCell::new(crate::utils::range_query::KdTree::new(&[]))),
            drar,
        };

        for (contour_index, contour) in glyph.borrow().contours.iter().enumerate() {
            ret.add_contour(contour, contour_index);
        }
        ret
    }

    fn add_contour(&mut self, contour: &Contour, contour_index: usize) {
        let mut points = self.points.borrow_mut();
        let mut points_map = self.points_map.borrow_mut();
        let prev_len = points.len();
        for (curve_index, curve) in contour.curves().borrow().iter().enumerate() {
            match curve.points().borrow().len() {
                4 => {
                    for (endpoint, handle) in [(0, 1), (3, 2)] {
                        let mut point_index = points.len();
                        points.push(ControlPoint {
                            contour_index,
                            curve_index,
                            point_index: endpoint,
                            position: curve.points().borrow()[endpoint],
                            kind: Endpoint {
                                handle: Some(point_index + 1),
                            },
                        });
                        points_map
                            .entry(curve.points().borrow()[endpoint])
                            .or_default()
                            .push(point_index);
                        let endpoint_index = point_index;
                        point_index += 1;
                        points.push(ControlPoint {
                            contour_index,
                            curve_index,
                            point_index: handle,
                            position: curve.points().borrow()[handle],
                            kind: Handle {
                                end_points: vec![endpoint_index],
                            },
                        });
                        points_map
                            .entry(curve.points().borrow()[handle])
                            .or_default()
                            .push(point_index);
                    }
                }
                3 => {
                    let mut point_index = points.len();
                    points.push(ControlPoint {
                        contour_index,
                        curve_index,
                        point_index: 0,
                        position: curve.points().borrow()[0],
                        kind: Endpoint {
                            handle: Some(point_index + 1),
                        },
                    });
                    points_map
                        .entry(curve.points().borrow()[0])
                        .or_default()
                        .push(point_index);
                    point_index += 1;
                    points.push(ControlPoint {
                        contour_index,
                        curve_index,
                        point_index: 1,
                        position: curve.points().borrow()[1],
                        kind: Handle {
                            end_points: vec![point_index - 1, point_index + 1],
                        },
                    });
                    points_map
                        .entry(curve.points().borrow()[1])
                        .or_default()
                        .push(point_index);
                    point_index += 1;
                    points.push(ControlPoint {
                        contour_index,
                        curve_index,
                        point_index: 2,
                        position: curve.points().borrow()[2],
                        kind: Endpoint {
                            handle: Some(point_index - 1),
                        },
                    });
                    points_map
                        .entry(curve.points().borrow()[2])
                        .or_default()
                        .push(point_index);
                }
                2 => {
                    let mut point_index = points.len();
                    for endpoint in 0..=1 {
                        points.push(ControlPoint {
                            contour_index,
                            curve_index,
                            point_index: endpoint,
                            position: curve.points().borrow()[endpoint],
                            kind: Endpoint { handle: None },
                        });
                        points_map
                            .entry(curve.points().borrow()[endpoint])
                            .or_default()
                            .push(point_index);
                        point_index += 1;
                    }
                }
                1 => {}
                0 => {}
                _ => unreachable!(), //FIXME
            }
        }
        let mut kd_tree = self.kd_tree.borrow_mut();
        for i in prev_len..points.len() {
            let pos = points[i].position;
            kd_tree.add(pos, i);
        }
    }

    fn set_selection(&mut self, selection: &[(usize, (i64, i64))]) {
        self.selection.clear();
        let points_map = self.points_map.borrow();
        for (_, pt) in selection {
            if let Some(indices) = points_map.get(pt) {
                self.selection.extend(indices.iter().cloned());
            }
        }
    }

    fn update_positions(&mut self, new_pos: (i64, i64)) {
        let mut action = self.update_point(&self.selection, new_pos);
        (action.redo)();
        let app: &crate::Application =
            crate::Application::from_instance(&self.app.downcast_ref::<crate::GerbApp>().unwrap());
        let undo_db = app.undo_db.borrow_mut();
        undo_db.event(action);
    }

    fn new_guideline(&self, angle: f64, (x, y): (i64, i64)) -> crate::Action {
        let drar = self.drar.clone();
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: Box::new([]),
            },
            compress: false,
            redo: Box::new(clone!(@weak self.glyph as glyph, @weak drar => move || {
                glyph.borrow_mut().guidelines.push(Guideline::builder().angle(angle).x(x).y(y).build());
                drar.queue_draw();
            })),
            undo: Box::new(clone!(@weak self.glyph as glyph, @weak drar => move || {
                glyph.borrow_mut().guidelines.pop();
                drar.queue_draw();
            })),
        }
    }

    fn update_guideline(&self, idx: usize, position: (i64, i64)) -> crate::Action {
        let drar = self.drar.clone();
        let old_position: (i64, i64) = {
            let g = self.glyph.borrow();
            let x = g.guidelines[idx].property("x");
            let y = g.guidelines[idx].property("y");
            (x, y)
        };
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[idx]).into() },
            },
            compress: true,
            redo: Box::new(clone!(@weak self.glyph as glyph, @weak drar => move || {
                glyph.borrow().guidelines[idx].set_property("x", position.0);
                glyph.borrow().guidelines[idx].set_property("y", position.1);
                drar.queue_draw();
            })),
            undo: Box::new(clone!(@weak self.glyph as glyph, @weak drar => move || {
                glyph.borrow().guidelines[idx].set_property("x", old_position.0);
                glyph.borrow().guidelines[idx].set_property("y", old_position.1);
                drar.queue_draw();
            })),
        }
    }

    fn delete_guideline(&self, idx: usize) -> crate::Action {
        let drar = self.drar.clone();
        let json: serde_json::Value =
            { serde_json::to_value(&self.glyph.borrow().guidelines[idx].imp()).unwrap() };
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[idx]).into() },
            },
            compress: false,
            redo: Box::new(clone!(@weak self.glyph as glyph, @weak drar => move || {
                glyph.borrow_mut().guidelines.remove(idx);
                drar.queue_draw();
            })),
            undo: Box::new(clone!(@weak self.glyph as glyph, @weak drar => move || {
                glyph.borrow_mut().guidelines.push(Guideline::try_from(json.clone()).unwrap());
                drar.queue_draw();
            })),
        }
    }

    fn update_point(&self, idxs: &[usize], new_pos: (i64, i64)) -> crate::Action {
        let drar = self.drar.clone();
        let old_positions = {
            let mut v = Vec::with_capacity(idxs.len());
            for &idx in idxs {
                v.push(if let Some(p) = self.points.borrow().get(idx) {
                    p.position
                } else {
                    (0, 0)
                });
            }
            Rc::new(v)
        };
        let idxs = Rc::new(idxs.to_vec());
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "point",
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&idxs).into() },
            },
            compress: true,
            redo: Box::new(
                clone!(@strong old_positions, @strong idxs, @weak self.points as points, @weak self.points_map as points_map, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak drar => move || {
                    let mut points = points.borrow_mut();
                    let mut points_map = points_map.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for &idx in idxs.iter() {
                        if let Some(p) = points.get_mut(idx) {
                            /* update points_map */
                            points_map.entry(p.position).and_modify(|points_vec| {
                                points_vec.retain(|p| *p != idx);
                            });
                            points_map.entry(new_pos).or_default().push(idx);
                            /* update kd_tree */
                            assert!(kd_tree.remove(p.position, idx));
                            kd_tree.add(new_pos, idx);

                            /* finally update actual point */
                            p.position = new_pos;

                            let glyph = glyph.borrow();
                            let curves = glyph.contours[p.contour_index].curves().borrow_mut();
                            curves[p.curve_index]
                                .points()
                                .borrow_mut()[p.point_index] = new_pos;
                        }
                    }
                    drar.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@strong old_positions, @strong idxs, @weak self.points as points, @weak self.points_map as points_map, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak drar => move || {
                    let mut points = points.borrow_mut();
                    let mut points_map = points_map.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for (&idx, &old_position) in idxs.iter().zip(old_positions.iter()) {
                        if let Some(ref mut p) = points.get_mut(idx) {
                            /* update points_map */
                            points_map.entry(p.position).and_modify(|points_vec| {
                                points_vec.retain(|p| *p != idx);
                            });
                            points_map.entry(old_position).or_default().push(idx);
                            /* update kd_tree */
                            assert!(kd_tree.remove(p.position, idx));
                            kd_tree.add(old_position, idx);

                            /* finally update actual point */
                            p.position = old_position;
                            let glyph = glyph.borrow();
                            let curves = glyph.contours[p.contour_index].curves().borrow_mut();
                            curves[p.curve_index]
                                .points()
                                .borrow_mut()[p.point_index] = old_position;
                        }
                    }
                    drar.queue_draw();

                }),
            ),
        }
    }
}

#[derive(Debug, Default)]
pub struct GlyphEditArea {
    app: OnceCell<gtk::Application>,
    glyph: OnceCell<Rc<RefCell<Glyph>>>,
    glyph_state: OnceCell<RefCell<GlyphState>>,
    drawing_area: OnceCell<gtk::DrawingArea>,
    hovering: Cell<Option<(usize, usize)>>,
    statusbar_context_id: Cell<Option<u32>>,
    overlay: OnceCell<gtk::Overlay>,
    pub toolbar_box: OnceCell<gtk::Box>,
    pub viewhidebox: OnceCell<viewhide::ViewHideBox>,
    zoom_percent_label: OnceCell<gtk::Label>,
    resized: Cell<bool>,
    camera: Cell<(f64, f64)>,
    mouse: Cell<(f64, f64)>,
    transformed_mouse: Cell<(i64, i64)>,
    zoom: Cell<f64>,
    button: Cell<Option<MotionMode>>,
    project: OnceCell<Project>,
}

const RULER_BREADTH: f64 = 13.;

#[glib::object_subclass]
impl ObjectSubclass for GlyphEditArea {
    const NAME: &'static str = "GlyphEditArea";
    type Type = GlyphEditView;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GlyphEditArea {
    // Here we are overriding the glib::Object::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.hovering.set(None);
        self.resized.set(true);
        self.statusbar_context_id.set(None);
        self.camera.set((0., 0.));
        self.mouse.set((0., 0.));
        self.transformed_mouse.set((0, 0));
        self.zoom.set(1.);

        let drawing_area = gtk::DrawingArea::builder()
            .expand(true)
            .visible(true)
            .build();
        drawing_area.set_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK
                | gtk::gdk::EventMask::BUTTON_RELEASE_MASK
                | gtk::gdk::EventMask::BUTTON_MOTION_MASK
                | gtk::gdk::EventMask::SCROLL_MASK
                | gtk::gdk::EventMask::SMOOTH_SCROLL_MASK
                | gtk::gdk::EventMask::POINTER_MOTION_MASK,
        );
        drawing_area.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                obj.imp().mouse.set(event.position());
                let zoom_factor = obj.imp().zoom.get();
                let camera = obj.imp().camera.get();
                let event_position = event.position();
                let units_per_em = *obj.imp().project.get().unwrap().imp().units_per_em.borrow();
                let f = units_per_em / EM_SQUARE_PIXELS;
                let position = (((event_position.0 * f - camera.0 * f * zoom_factor) / zoom_factor) as i64, (units_per_em - ((event_position.1 * f - camera.1 * f * zoom_factor) / zoom_factor)) as i64);
                obj.imp().transformed_mouse.set(position);
                match event.button() {
                    gtk::gdk::BUTTON_PRIMARY => {
                        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                        if event_position.0 < RULER_BREADTH || event_position.1 < RULER_BREADTH {
                            let angle = if event_position.0 < RULER_BREADTH && event_position.1 < RULER_BREADTH {
                                -45.
                            } else if event_position.0 < RULER_BREADTH {
                                90.
                            } else {
                                0.
                            };
                            let mut action = glyph_state.new_guideline(angle, position);
                            (action.redo)();
                            let app: &crate::Application =
                                crate::Application::from_instance(&obj.imp().app.get().unwrap().downcast_ref::<crate::GerbApp>().unwrap());
                            let undo_db = app.undo_db.borrow_mut();
                            undo_db.event(action);
                        }

                        if glyph_state.tool.is_manipulate() {
                            let mut is_guideline: bool = false;
                            let GlyphState {
                                ref mut tool,
                                ref glyph,
                                ..
                            } = *glyph_state;
                            for (i, g) in glyph.borrow().guidelines.iter().enumerate() {
                                if g.imp().on_line_query(position, None) {
                                    obj.imp().select_object(Some(g.clone().upcast::<gtk::glib::Object>()));
                                    *tool = Tool::Manipulate { mode: ControlPointMode::DragGuideline(i) };
                                    is_guideline = true;
                                    break;
                                }
                            }
                            if !is_guideline {
                                let pts = glyph_state.kd_tree.borrow().query(position, 10);
                                glyph_state.tool = Tool::Manipulate { mode: ControlPointMode::Drag };
                                glyph_state.set_selection(&pts);
                            }
                        } else if let Tool::BezierPen { ref mut state } = glyph_state.tool {
                            if !state.insert_point(position) {
                                let state = std::mem::replace(state, Default::default());
                                glyph_state.tool = Tool::Manipulate { mode: Default::default() };
                                let new_contour = state.close(false);
                                let contour_index = glyph_state.glyph.borrow().contours.len();
                                glyph_state.add_contour(&new_contour, contour_index);
                                glyph_state.glyph.borrow_mut().contours.push(new_contour);
                            }
                        }
                    },
                    gtk::gdk::BUTTON_MIDDLE => {
                        obj.imp().button.set(Some(MotionMode::Pan));
                    },
                    gtk::gdk::BUTTON_SECONDARY => {
                        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
                        if glyph_state.tool.is_manipulate() {
                            let glyph = glyph_state.glyph.borrow_mut();
                            for (i, g) in glyph.guidelines.iter().enumerate() {
                                if g.imp().on_line_query(position, None) {
                                    let menu = gtk::Menu::builder().attach_widget(_self).take_focus(true).visible(true).build();
                                    let name = gtk::MenuItem::builder().label(&format!("{} - {}", g.name().as_ref().map(String::as_str).unwrap_or("Anonymous guideline"), g.identifier().as_ref().map(String::as_str).unwrap_or("No identifier"))).sensitive(false).visible(true).build();
                                    menu.append(&name);
                                    menu.append(&gtk::SeparatorMenuItem::builder().visible(true).build());
                                    let delete = gtk::MenuItem::builder().label("Delete").sensitive(true).visible(true).build();
                                    drop(glyph);
                                    drop(glyph_state);
                                    delete.connect_activate(clone!(@weak obj, @weak _self as drar => move |_del_self| {
                                        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                                        if glyph_state.glyph.borrow().guidelines.get(i).is_some() { // Prevent panic if `i` out of bounds
                                            let mut action = glyph_state.delete_guideline(i);
                                            (action.redo)();
                                            let app: &crate::Application =
                                                crate::Application::from_instance(&obj.imp().app.get().unwrap().downcast_ref::<crate::GerbApp>().unwrap());
                                            let undo_db = app.undo_db.borrow_mut();
                                            undo_db.event(action);
                                            drar.queue_draw();
                                        }
                                    }));
                                    menu.append(&delete);
                                    menu.show_all();
                                    menu.popup_easy(event.button(), event.time());
                                    break;
                                }
                            }
                        }
                        return Inhibit(true);
                    }
                    _ => {},
                }
                Inhibit(false)
            }),
        );
        drawing_area.connect_button_release_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                //obj.imp().mouse.set((0., 0.));
                obj.imp().button.set(None);
                let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                if let Tool::Manipulate { ref mut mode } = glyph_state.tool {
                    *mode = ControlPointMode::None;
                } else if let Tool::BezierPen { ref mut state } = glyph_state.tool {
                    if event.button() == gtk::gdk::BUTTON_PRIMARY {
                        let zoom_factor = obj.imp().zoom.get();
                        let camera = obj.imp().camera.get();
                        let position = event.position();
                        let units_per_em = *obj.imp().project.get().unwrap().imp().units_per_em.borrow();
                        let f = units_per_em / EM_SQUARE_PIXELS;
                        let position = (((position.0*f - camera.0*f * zoom_factor)/zoom_factor) as i64, (units_per_em - ((position.1*f-camera.1*f * zoom_factor)/zoom_factor)) as i64);
                        obj.imp().transformed_mouse.set(position);
                        if !state.insert_point(position) {
                            let state = std::mem::replace(state, Default::default());
                            glyph_state.tool = Tool::Manipulate { mode: Default::default() };
                            let new_contour = state.close(true);
                            let contour_index = glyph_state.glyph.borrow().contours.len();
                            glyph_state.add_contour(&new_contour, contour_index);
                            glyph_state.glyph.borrow_mut().contours.push(new_contour);
                        }
                    } else if event.button() == gtk::gdk::BUTTON_SECONDARY {
                        let state = std::mem::replace(state, Default::default());
                        glyph_state.tool = Tool::Manipulate { mode: Default::default() };
                        let new_contour = state.close(true);
                        let contour_index = glyph_state.glyph.borrow().contours.len();
                        glyph_state.add_contour(&new_contour, contour_index);
                        glyph_state.glyph.borrow_mut().contours.push(new_contour);
                    }
                }
                if let Some(screen) = _self.window() {
                    let display = screen.display();
                    screen.set_cursor(Some(
                            &gtk::gdk::Cursor::from_name(&display, "default").unwrap(),
                    ));
                }

                Inhibit(false)
            }),
        );
        drawing_area.connect_motion_notify_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                if let Some(MotionMode::Pan) = obj.imp().button.get(){
                    let mut camera = obj.imp().camera.get();
                    let mouse = obj.imp().mouse.get();
                    camera.0 += event.position().0 - mouse.0;
                    camera.1 += event.position().1 - mouse.1;
                    obj.imp().camera.set(camera);
                    if let Some(screen) = _self.window() {
                        let display = screen.display();
                        screen.set_cursor(Some(
                                &gtk::gdk::Cursor::from_name(&display, "grab").unwrap(),
                        ));
                    }
                } else {
                    let zoom_factor = obj.imp().zoom.get();
                    let camera = obj.imp().camera.get();
                    let event_position = event.position();
                    let units_per_em = *obj.imp().project.get().unwrap().imp().units_per_em.borrow();
                    let f = units_per_em / EM_SQUARE_PIXELS;
                    let position = (((event_position.0 * f - camera.0 * f * zoom_factor) / zoom_factor) as i64, (units_per_em - ((event_position.1 * f - camera.1 * f * zoom_factor) / zoom_factor)) as i64);
                    obj.imp().transformed_mouse.set(position);
                    let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                    if let Tool::Manipulate { mode: ControlPointMode::Drag } = glyph_state.tool {
                        glyph_state.update_positions(position);
                    } else if let Tool::Manipulate { mode: ControlPointMode::DragGuideline(idx) } = glyph_state.tool {
                        let mut action = glyph_state.update_guideline(idx, position);
                        (action.redo)();
                        let app: &crate::Application =
                            crate::Application::from_instance(&obj.imp().app.get().unwrap().downcast_ref::<crate::GerbApp>().unwrap());
                        let undo_db = app.undo_db.borrow_mut();
                        undo_db.event(action);
                    }

                    let pts = glyph_state.kd_tree.borrow().query(position, 10);
                    if pts.is_empty() {
                        obj.imp().hovering.set(None);
                        if let Some(screen) = _self.window() {
                            let display = screen.display();
                            screen.set_cursor(Some(
                                    &if glyph_state.tool.is_manipulate() {
                                        gtk::gdk::Cursor::from_name(&display, "default").unwrap()
                                    } else if glyph_state.tool.is_bezier_pen() {
                                        gtk::gdk::Cursor::from_name(&display, "crosshair").unwrap()
                                    } else {
                                        gtk::gdk::Cursor::from_name(&display, "default").unwrap()
                                    }
                            ));
                        }
                    } else if let Some(screen) = _self.window() {
                        let display = screen.display();
                        screen.set_cursor(Some(
                                &gtk::gdk::Cursor::from_name(&display, "grab").unwrap(),
                        ));
                    }

                    let glyph = glyph_state.glyph.borrow();
                    'hover: for (ic, contour) in glyph.contours.iter().enumerate() {
                        for (jc, curve) in contour.curves().borrow().iter().enumerate() {
                            if curve.on_curve_query(position, None) {
                                obj.imp().new_statusbar_message(&format!("{:?}", curve));
                                obj.imp().hovering.set(Some((ic, jc)));
                                break 'hover;
                            }
                            for p in &pts {
                                if curve.points().borrow().contains(&p.1) {
                                    obj.imp().new_statusbar_message(&format!("{:?}", curve));
                                    obj.imp().hovering.set(Some((ic, jc)));
                                    break 'hover;
                                }
                            }
                        }
                    }
                }
                obj.imp().mouse.set(event.position());
                _self.queue_draw();

                Inhibit(false)
            }),
        );

        drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |drar: &gtk::DrawingArea, cr: &gtk::cairo::Context| {
            let (show_grid, show_guidelines, show_handles, inner_fill) = {
                let viewhide = obj.imp().viewhidebox.get().unwrap();
                let show_grid = viewhide.property::<bool>("show-grid");
                let show_guidelines = viewhide.property::<bool>("show-guidelines");
                let show_handles = viewhide.property::<bool>("show-handles");
                let inner_fill = viewhide.property::<bool>("inner-fill");
                (show_grid, show_guidelines, show_handles, inner_fill)
            };
            let app: &crate::GerbApp =
                obj.imp().app.get().unwrap().downcast_ref::<crate::GerbApp>().unwrap();
            let settings = app.imp().settings.clone();
            let width = drar.allocated_width() as f64;
            let height = drar.allocated_height() as f64;
            let project = obj.imp().project.get().unwrap().imp();
            let units_per_em = *project.units_per_em.borrow();
            let x_height = *project.x_height.borrow();
            let cap_height = *project.cap_height.borrow();
            let _ascender = *project.ascender.borrow();
            let _descender = *project.descender.borrow();
            let f = EM_SQUARE_PIXELS / units_per_em;
            let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
            let glyph_width = f * glyph_state.glyph.borrow().width.unwrap_or(units_per_em);

            if obj.imp().resized.get() {
                obj.imp().resized.set(false);
                /* resize and center glyph to view */
                let target_width = width / 3. * 0.8;
                let target_zoom_factor = target_width / glyph_width;
                obj.imp().set_zoom(target_zoom_factor);
                obj.imp().camera.set((EM_SQUARE_PIXELS / 2.0, 0.0));
            }
            let zoom_factor = obj.imp().zoom.get();
            cr.save().unwrap();
            cr.scale(zoom_factor, zoom_factor);
            cr.set_source_rgb(1., 1., 1.);
            cr.paint().expect("Invalid cairo surface state");

            cr.set_line_width(0.5);

            let glyph_line_width = settings.borrow().property("line-width");

            let camera = obj.imp().camera.get();
            let mouse = obj.imp().mouse.get();

            if show_grid {
                for &(color, step) in &[(0.9, 5.0), (0.8, 100.0)] {
                    cr.set_source_rgb(color, color, color);
                    let mut y = (camera.1 % step).floor();
                    while y < (height/zoom_factor) {
                        cr.move_to(0., y);
                        cr.line_to(width/zoom_factor, y);
                        y += step;
                    }
                    cr.stroke().unwrap();
                    let mut x = (camera.0 % step).floor();
                    while x < (width/zoom_factor) {
                        cr.move_to(x, 0.);
                        cr.line_to(x, height/zoom_factor);
                        x += step;
                    }
                    cr.stroke().unwrap();
                }
            }
            /* Draw em square of units_per_em units: */

            cr.save().unwrap();
            cr.translate(camera.0, camera.1);

            cr.set_source_rgba(210./255., 227./255., 252./255., 0.6);
            cr.rectangle(0., 0., glyph_width, EM_SQUARE_PIXELS);
            cr.fill().unwrap();

            if show_guidelines {
                /* Draw x-height */
                cr.set_source_rgba(0., 0., 1., 0.6);
                cr.set_line_width(2.0);
                cr.move_to(0., (units_per_em - x_height) * f);
                cr.line_to(glyph_width * 1.2, (units_per_em - x_height) * f);
                cr.stroke().unwrap();
                cr.move_to(glyph_width * 1.2, (units_per_em - x_height) * f);
                cr.show_text("x-height").unwrap();

                /* Draw baseline */
                cr.move_to(0., units_per_em * f);
                cr.line_to(glyph_width * 1.2, units_per_em * f);
                cr.stroke().unwrap();
                cr.move_to(glyph_width * 1.2, units_per_em * f);
                cr.show_text("baseline").unwrap();

                /* Draw cap height */
                cr.move_to(0., EM_SQUARE_PIXELS - cap_height * f);
                cr.line_to(glyph_width * 1.2, EM_SQUARE_PIXELS - cap_height * f);
                cr.stroke().unwrap();
                cr.move_to(glyph_width * 1.2, EM_SQUARE_PIXELS - cap_height * f);
                cr.show_text("cap height").unwrap();
            }

            /* Draw the glyph */

            let mut matrix = gtk::cairo::Matrix::identity();
            matrix.scale(EM_SQUARE_PIXELS / units_per_em, EM_SQUARE_PIXELS / units_per_em);
            let options = GlyphDrawingOptions {
                outline: (0.2, 0.2, 0.2, if inner_fill { 0. } else { 0.6 }),
                inner_fill: if inner_fill {
                    Some((0., 0., 0., 1.))
                } else {
                    None
                },
                highlight: obj.imp().hovering.get(),
                matrix,
                units_per_em,
                line_width: glyph_line_width,
            };
            glyph_state.glyph.borrow().draw(cr, options);

            if let Tool::BezierPen { ref state } = glyph_state.tool {
                let position = (((mouse.0 - camera.0 * zoom_factor) / (f * zoom_factor)) as i64, (units_per_em - ((mouse.1 - camera.1 * zoom_factor) / (f * zoom_factor))) as i64);
                state.draw(cr, options, position);
            }
            cr.save().unwrap();
            cr.set_source_rgba(0.0, 0.0, 1.0, 0.5);

            cr.set_line_width(1.0 / (2.0 * f));
            if show_handles {
                let handle_size: f64 = settings.borrow().property("handle-size");
                cr.transform(matrix);
                cr.transform(gtk::cairo::Matrix::new(1.0, 0., 0., -1.0, 0., units_per_em.abs()));
                for cp in glyph_state.points.borrow().iter() {
                    let p = cp.position;
                    match &cp.kind {
                        Endpoint { .. } => {
                            cr.rectangle(p.0 as f64 - handle_size / (2.0 * f), p.1 as f64 - handle_size / (2.0 * f), handle_size / (2.0 * f), handle_size / (2.0 * f));
                            cr.stroke().unwrap();
                        }
                        Handle { ref end_points } => {
                            cr.arc(p.0 as f64, p.1 as f64, handle_size / (2.0 * f), 0., 2.0 * std::f64::consts::PI);
                            cr.stroke().unwrap();
                            for ep in end_points {
                                let ep = glyph_state.points.borrow()[*ep].position;
                                cr.move_to(p.0 as f64, p.1 as f64);
                                cr.line_to(ep.0 as f64, ep.1 as f64);
                                cr.stroke().unwrap();
                            }
                        }
                    }
                }
            }

            cr.restore().unwrap();
            cr.restore().unwrap();
            cr.restore().unwrap();

            if show_guidelines {
                let mut matrix = gtk::cairo::Matrix::identity();
                matrix.scale(zoom_factor, zoom_factor);
                matrix.translate(camera.0, camera.1);
                matrix.scale(EM_SQUARE_PIXELS / units_per_em, EM_SQUARE_PIXELS / units_per_em);
                matrix.translate(0., units_per_em.abs());
                matrix.scale(1.0, -1.0);
                for g in glyph_state.glyph.borrow().guidelines.iter() {
                    let highlight = g.imp().on_line_query(obj.imp().transformed_mouse.get(), None);
                    g.imp().draw(cr, matrix, (width, height), highlight);
                    if highlight {
                        cr.move_to(mouse.0, mouse.1);
                        let line_height = cr.text_extents("Guideline").unwrap().height * 1.5;
                        cr.show_text("Guideline").unwrap();
                        for (i, line) in [
                            format!(
                                "Name: {}",
                                g.name().as_ref().map(String::as_str).unwrap_or("-")
                            ),
                            format!(
                                "Identifier: {}",
                                g.identifier().as_ref().map(String::as_str).unwrap_or("-")
                            ),
                            format!("Point: ({}, {})", g.x(), g.y()),
                            format!("Angle: {:02}deg", g.angle()),
                        ]
                            .into_iter()
                            .enumerate()
                            {
                                cr.move_to(
                                    mouse.0,
                                    mouse.1 + (i + 1) as f64 * line_height,
                                );
                                cr.show_text(&line).unwrap();
                            }
                    }
                }
            }

            /* Draw rulers */
            cr.rectangle(0., 0., width, RULER_BREADTH);
            cr.set_source_rgb(1., 1., 1.);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0., 0., 0.);
            cr.stroke_preserve().unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.move_to(mouse.0, 0.);
            cr.line_to(mouse.0, RULER_BREADTH);
            cr.stroke().unwrap();
            cr.move_to(mouse.0+1., 2.*RULER_BREADTH/3.);
            cr.set_font_size(6.);
            cr.show_text(&format!("{:.0}", (mouse.0 - camera.0 * zoom_factor) / (f * zoom_factor))).unwrap();


            cr.rectangle(0., RULER_BREADTH, RULER_BREADTH, height);
            cr.set_source_rgb(1., 1., 1.);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0., 0., 0.);
            cr.stroke_preserve().unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.move_to(0., mouse.1);
            cr.line_to(RULER_BREADTH, mouse.1);
            cr.stroke().unwrap();
            cr.move_to(2.*RULER_BREADTH/3., mouse.1-1.);
            cr.set_font_size(6.);
            cr.save().expect("Invalid cairo surface state");
            cr.rotate(-std::f64::consts::FRAC_PI_2);
            cr.show_text(&format!("{:.0}", (mouse.1 - camera.1 * zoom_factor) / (f * zoom_factor))).unwrap();
            cr.restore().expect("Invalid cairo surface state");


           Inhibit(false)
        }));
        let toolbar_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .spacing(5)
            .visible(true)
            .can_focus(true)
            .build();
        let toolbar = gtk::Toolbar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            //.toolbar_style(gtk::ToolbarStyle::Both)
            .visible(true)
            .can_focus(true)
            .build();

        let manipulate_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::GRAB_ICON_SVG,
            )),
            Some("Manipulate"),
        );
        manipulate_button.set_visible(true);
        // FIXME: doesn't seem to work?
        manipulate_button.set_tooltip_text(Some("Pan"));
        manipulate_button.connect_clicked(clone!(@weak obj => move |_self| {
            let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
            glyph_state.tool = Tool::Manipulate { mode: Default::default() };
            if let Some(screen) = _self.window() {
                let display = screen.display();
                screen.set_cursor(Some(
                        &gtk::gdk::Cursor::from_name(&display, "default").unwrap(),
                ));
            }
        }));

        let bezier_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::BEZIER_ICON_SVG,
            )),
            Some("Create Bézier curve"),
        );
        bezier_button.set_visible(true);
        // FIXME: doesn't seem to work?
        bezier_button.set_tooltip_text(Some("Create Bézier curve"));
        bezier_button.connect_clicked(clone!(@weak obj => move |_self| {
            let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
            glyph_state.tool = Tool::BezierPen { state: Default::default() };
            if let Some(screen) = _self.window() {
                let display = screen.display();
                screen.set_cursor(Some(
                        &gtk::gdk::Cursor::from_name(&display, "crosshair").unwrap(),
                ));
            }
        }));

        let bspline_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::BSPLINE_ICON_SVG,
            )),
            Some("Create b-spline curve"),
        );
        bspline_button.set_visible(true);
        // FIXME: doesn't seem to work?
        bspline_button.set_tooltip_text(Some("Create b-spline curve"));

        /*let pen_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::PEN_ICON_SVG,
            )),
            Some("Pen"),
        );
        pen_button.set_visible(true);
        pen_button.set_tooltip_text(Some("Pen"));*/

        let zoom_in_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::ZOOM_IN_ICON_SVG,
            )),
            Some("Zoom in"),
        );
        zoom_in_button.set_visible(true);
        zoom_in_button.set_tooltip_text(Some("Zoom in"));
        zoom_in_button.connect_clicked(clone!(@weak obj => move |_| {
            let imp = obj.imp();
            let zoom_factor = imp.zoom.get() + 0.25;
            imp.set_zoom(zoom_factor);
        }));
        let zoom_out_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::ZOOM_OUT_ICON_SVG,
            )),
            Some("Zoom out"),
        );
        zoom_out_button.set_visible(true);
        zoom_out_button.set_tooltip_text(Some("Zoom out"));
        zoom_out_button.connect_clicked(clone!(@weak obj => move |_| {
            let imp = obj.imp();
            let zoom_factor = imp.zoom.get() - 0.25;
            imp.set_zoom(zoom_factor);
        }));

        drawing_area.connect_scroll_event(clone!(@weak obj => @default-return Inhibit(false), move |_drar, event| {
            if event.state().contains(gtk::gdk::ModifierType::SHIFT_MASK) {
                obj.imp().mouse.set(event.position());
                let (mut dx, mut dy) = event.delta();
                if event.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    if dy.abs() > dx.abs() {
                        dx = dy;
                    }
                    dy = 0.0;
                }

                let dx = dx * EM_SQUARE_PIXELS/2. * obj.imp().zoom.get();
                let dy = dy * EM_SQUARE_PIXELS/2. * obj.imp().zoom.get();
                let mut camera = obj.imp().camera.get();
                let mouse = obj.imp().mouse.get();
                camera.0 += event.position().0 - mouse.0 - dx;
                camera.1 += event.position().1 - mouse.1 - dy;
                obj.imp().camera.set(camera);
                _drar.queue_draw();
                return Inhibit(false);
            }
            match event.direction() {
                gtk::gdk::ScrollDirection::Up | gtk::gdk::ScrollDirection::Down | gtk::gdk::ScrollDirection::Smooth => {
                    /* zoom */
                    let (_, dy) = event.delta();
                    let imp = obj.imp();
                    let mut camera =imp.camera.get();
                    let mouse = imp.mouse.get();
                    camera.0 += event.position().0 - mouse.0;
                    camera.1 += event.position().1 - mouse.1;
                    imp.mouse.set(event.position());
                    imp.camera.set(camera);
                    let zoom_factor = imp.zoom.get() - 0.25* dy;
                    imp.set_zoom(zoom_factor);
                    _drar.queue_draw();
                },
                _ => {
                    /* ignore */
                }
            }
            Inhibit(false)
        }));

        let zoom_percent_label = gtk::Label::new(Some("100%"));
        zoom_percent_label.set_visible(true);
        zoom_percent_label.set_selectable(true); // So that the widget can receive the button-press event
        zoom_percent_label.set_width_chars(5); // So that if 2 digit zoom (<100%) has the same length as a widget with a three digit zoom value. For example 75% and 125% should result in the same width
        zoom_percent_label.set_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK);
        zoom_percent_label.set_tooltip_text(Some("Interface zoom percentage"));

        zoom_percent_label.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                if event.button() == gtk::gdk::BUTTON_PRIMARY &&
                 event.event_type() == gtk::gdk::EventType::DoubleButtonPress {
                     let imp = obj.imp();
                     let zoom_factor = imp.zoom.get();
                     if (zoom_factor - 1.0).abs() > f64::EPSILON {
                         imp.set_zoom(1.0);
                     }
                }
                Inhibit(false)
            }),
        );
        let debug_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Debug info"));
        debug_button.set_visible(true);
        debug_button.set_tooltip_text(Some("Debug info"));
        debug_button.connect_clicked(clone!(@weak obj => move |_| {
            let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
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
            let glyph_info = gtk::Label::new(Some(&format!("{:#?}", glyph.contours)));
            glyph_info.set_halign(gtk::Align::Start);
            let scrolled_window = gtk::ScrolledWindow::builder()
                .expand(true)
                .visible(true)
                .can_focus(true)
                .margin_start(5)
                .build();
            scrolled_window.set_child(Some(&glyph_info));
            hbox.pack_start(&scrolled_window, true, true, 0);
            hbox.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, true, 0);
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
            window.add(&hbox);
            window.show_all();
        }));
        toolbar.add(&manipulate_button);
        toolbar.set_item_homogeneous(&manipulate_button, false);
        toolbar.add(&bezier_button);
        toolbar.set_item_homogeneous(&bezier_button, false);
        toolbar.add(&bspline_button);
        toolbar.set_item_homogeneous(&bspline_button, false);
        toolbar.add(&zoom_in_button);
        toolbar.set_item_homogeneous(&zoom_in_button, false);
        toolbar.add(&zoom_out_button);
        toolbar.set_item_homogeneous(&zoom_out_button, false);
        toolbar_box.pack_start(&toolbar, false, false, 0);
        toolbar_box.pack_start(&zoom_percent_label, false, false, 0);
        toolbar_box.pack_start(&debug_button, false, false, 0);
        toolbar_box.style_context().add_class("glyph-edit-toolbox");
        let viewhidebox = viewhide::ViewHideBox::new();
        viewhidebox.connect_notify_local(
            Some("show-grid"),
            clone!(@weak drawing_area => move |_self, _| {
                drawing_area.queue_draw();
            }),
        );
        let overlay = gtk::Overlay::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        overlay.set_child(Some(&drawing_area));
        overlay.add_overlay(
            &gtk::Expander::builder()
                .child(&viewhidebox)
                .expanded(true)
                .visible(true)
                .can_focus(true)
                .tooltip_text("Toggle overlay visibilities")
                .halign(gtk::Align::End)
                .valign(gtk::Align::End)
                .build(),
        );
        overlay.add_overlay(&toolbar_box);
        obj.add(&overlay);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);

        self.zoom_percent_label
            .set(zoom_percent_label)
            .expect("Failed to initialize window state");
        self.overlay
            .set(overlay)
            .expect("Failed to initialize window state");
        self.toolbar_box
            .set(toolbar_box)
            .expect("Failed to initialize window state");
        self.viewhidebox
            .set(viewhidebox)
            .expect("Failed to initialize window state");
        self.drawing_area
            .set(drawing_area)
            .expect("Failed to initialize window state");
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        // Name
                        "tab-title",
                        // Nickname
                        "tab-title",
                        // Short description
                        "tab-title",
                        // Default value
                        Some("edit glyph"),
                        // The property can be read and written to
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "tab-can-close",
                        "tab-can-close",
                        "tab-can-close",
                        true,
                        ParamFlags::READABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "tab-title" => {
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
            "tab-can-close" => true.to_value(),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for GlyphEditArea {}
impl ContainerImpl for GlyphEditArea {}
impl BinImpl for GlyphEditArea {}

impl GlyphEditArea {
    fn set_zoom(&self, new_val: f64) {
        if new_val > 0.09 && new_val < 15.26 {
            self.zoom.set(new_val);
            self.zoom_percent_label
                .get()
                .unwrap()
                .set_text(&format!("{:.0}%", new_val * 100.));
            self.overlay.get().unwrap().queue_draw();
        }
    }

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
                        .context_id(&format!("GlyphEditArea-{:?}", &self.glyph.get().unwrap())),
                ));
            }
            if let Some(cid) = self.statusbar_context_id.get().as_ref() {
                statusbar.push(*cid, msg);
            }
        }
    }

    fn select_object(&self, new_obj: Option<glib::Object>) {
        if let Some(app) = self
            .app
            .get()
            .and_then(|app| app.downcast_ref::<crate::GerbApp>())
        {
            let tabinfo = app.tabinfo();
            tabinfo.set_object(new_obj);
        }
    }
}

glib::wrapper! {
    pub struct GlyphEditView(ObjectSubclass<GlyphEditArea>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl GlyphEditView {
    pub fn new(app: gtk::Application, project: Project, glyph: Rc<RefCell<Glyph>>) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp()
            .glyph_state
            .set(RefCell::new(GlyphState::new(
                &glyph,
                app.clone(),
                ret.imp().drawing_area.get().unwrap().clone(),
            )))
            .expect("Failed to create glyph state");
        ret.imp().glyph.set(glyph).unwrap();
        ret.imp().app.set(app).unwrap();
        ret.imp().project.set(project).unwrap();
        ret
    }
}
