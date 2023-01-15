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
use once_cell::unsync::OnceCell;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::glyphs::{Contour, Glyph, GlyphDrawingOptions, Guideline};
use crate::project::Project;
use crate::utils::Point;
use crate::views::{canvas::LayerBuilder, overlay::Child};
use crate::Settings;

//mod bezier_pen;
mod layers;
mod tools;
mod visibility_toggles;

use super::{Canvas, Transformation, UnitPoint, ViewPoint};
use tools::Tool;

const EM_SQUARE_PIXELS: f64 = 200.0;

#[derive(Debug, Clone)]
pub enum ControlPointKind {
    Endpoint {
        handle: Option<((usize, usize), Uuid)>,
    },
    Handle {
        end_points: Vec<((usize, usize), Uuid)>,
    },
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
    pub tool: Tool,
    selection: Vec<((usize, usize), Uuid)>,
    pub points: Rc<RefCell<HashMap<((usize, usize), Uuid), ControlPoint>>>,
    pub kd_tree: Rc<RefCell<crate::utils::range_query::KdTree>>,
}

impl GlyphState {
    fn new(glyph: &Rc<RefCell<Glyph>>, app: gtk::Application, viewport: Canvas) -> Self {
        let mut ret = Self {
            app,
            glyph: Rc::new(RefCell::new(glyph.borrow().clone())),
            reference: Rc::clone(glyph),
            viewport,
            tool: Tool::default(),
            selection: vec![],
            points: Rc::new(RefCell::new(HashMap::default())),
            kd_tree: Rc::new(RefCell::new(crate::utils::range_query::KdTree::new(&[]))),
        };

        for (contour_index, contour) in glyph.borrow().contours.iter().enumerate() {
            ret.add_contour(contour, contour_index);
        }
        ret
    }

    fn add_contour(&mut self, contour: &Contour, contour_index: usize) {
        let mut points = self.points.borrow_mut();
        let mut kd_tree = self.kd_tree.borrow_mut();
        for (curve_index, curve) in contour.curves().borrow().iter().enumerate() {
            match curve.points().borrow().len() {
                4 => {
                    for (endpoint, handle) in [(0, 1), (3, 2)] {
                        let end_p = &curve.points().borrow()[endpoint];
                        let handle_p = &curve.points().borrow()[handle];
                        points.insert(
                            ((contour_index, curve_index), end_p.uuid),
                            ControlPoint {
                                contour_index,
                                curve_index,
                                point_index: endpoint,
                                position: *end_p,
                                kind: Endpoint {
                                    handle: Some(((contour_index, curve_index), handle_p.uuid)),
                                },
                            },
                        );
                        kd_tree.add(((contour_index, curve_index), end_p.uuid), *end_p);
                        points.insert(
                            ((contour_index, curve_index), handle_p.uuid),
                            ControlPoint {
                                contour_index,
                                curve_index,
                                point_index: handle,
                                position: *handle_p,
                                kind: Handle {
                                    end_points: vec![((contour_index, curve_index), end_p.uuid)],
                                },
                            },
                        );
                        kd_tree.add(((contour_index, curve_index), handle_p.uuid), *handle_p);
                    }
                }
                3 => {
                    let p0 = &curve.points().borrow()[0];
                    let p1 = &curve.points().borrow()[1];
                    let p2 = &curve.points().borrow()[2];
                    points.insert(
                        ((contour_index, curve_index), p0.uuid),
                        ControlPoint {
                            contour_index,
                            curve_index,
                            point_index: 0,
                            position: *p0,
                            kind: Endpoint {
                                handle: Some(((contour_index, curve_index), p1.uuid)),
                            },
                        },
                    );
                    points.insert(
                        ((contour_index, curve_index), p1.uuid),
                        ControlPoint {
                            contour_index,
                            curve_index,
                            point_index: 1,
                            position: *p1,
                            kind: Handle {
                                end_points: vec![
                                    ((contour_index, curve_index), p0.uuid),
                                    ((contour_index, curve_index), p2.uuid),
                                ],
                            },
                        },
                    );
                    points.insert(
                        ((contour_index, curve_index), p2.uuid),
                        ControlPoint {
                            contour_index,
                            curve_index,
                            point_index: 2,
                            position: *p2,
                            kind: Endpoint {
                                handle: Some(((contour_index, curve_index), p1.uuid)),
                            },
                        },
                    );
                    for p in [p0, p1, p2] {
                        kd_tree.add(((contour_index, curve_index), p.uuid), *p);
                    }
                }
                2 => {
                    for endpoint in 0..=1 {
                        let p = &curve.points().borrow()[endpoint];
                        points.insert(
                            ((contour_index, curve_index), p.uuid),
                            ControlPoint {
                                contour_index,
                                curve_index,
                                point_index: endpoint,
                                position: *p,
                                kind: Endpoint { handle: None },
                            },
                        );
                        kd_tree.add(((contour_index, curve_index), p.uuid), *p);
                    }
                }
                1 => {}
                0 => {}
                _ => unreachable!(), //FIXME
            }
        }
    }

    fn update_positions(&mut self, new_pos: Point) {
        let mut action = self.update_point(&self.selection, new_pos);
        (action.redo)();
        let app: &crate::Application =
            crate::Application::from_instance(&self.app.downcast_ref::<crate::GerbApp>().unwrap());
        let undo_db = app.undo_db.borrow_mut();
        undo_db.event(action);
    }

    fn new_guideline(&self, angle: f64, p: Point) -> crate::Action {
        let (x, y) = (p.x, p.y);
        let viewport = self.viewport.clone();
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "guideline",
                id: Box::new([]),
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow_mut().guidelines.push(Guideline::builder().angle(angle).x(x).y(y).build());
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
            { serde_json::to_value(&self.glyph.borrow().guidelines[idx].imp()).unwrap() };
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

    fn update_point(&self, idxs: &[((usize, usize), Uuid)], new_pos: Point) -> crate::Action {
        let viewport = self.viewport.clone();
        let old_positions = {
            let mut v = Vec::with_capacity(idxs.len());
            for idx in idxs {
                v.push(if let Some(p) = self.points.borrow().get(idx) {
                    (*idx, p.position)
                } else {
                    (*idx, Point::from((0.0, 0.0)))
                });
            }
            Rc::new(v)
        };
        let idxs = Rc::new(idxs.to_vec());
        crate::Action {
            stamp: crate::EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: "point",
                id: unsafe {
                    std::mem::transmute::<&[((usize, usize), Uuid)], &[u8]>(&idxs).into()
                },
            },
            compress: true,
            redo: Box::new(
                clone!(@strong old_positions, @strong idxs, @weak self.points as points, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak viewport => move || {
                    let mut points = points.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for idx in idxs.iter() {
                        if let Some(p) = points.get_mut(idx) {
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
                clone!(@strong old_positions, @strong idxs, @weak self.points as points, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak viewport => move || {
                    let mut points = points.borrow_mut();
                    let mut kd_tree = kd_tree.borrow_mut();
                    for (&idx, &old_position) in idxs.iter().zip(old_positions.iter()) {
                        if let Some(ref mut p) = points.get_mut(&old_position.0) {
                            /* update kd_tree */
                            assert!(kd_tree.remove(idx, new_pos));
                            kd_tree.add(idx, old_position.1);

                            /* finally update actual point */
                            p.position = old_position.1;
                            let glyph = glyph.borrow();
                            let curves = glyph.contours[p.contour_index].curves().borrow_mut();
                            curves[p.curve_index]
                                .points()
                                .borrow_mut()[p.point_index] = old_position.1;
                        }
                    }
                    viewport.queue_draw();
                }),
            ),
        }
    }

    fn set_selection(&mut self, selection: &[(((usize, usize), Uuid), crate::utils::IPoint)]) {
        self.selection.clear();
        self.selection.extend(selection.iter().map(|(u, _)| u));
    }
}

#[derive(Debug, Default)]
pub struct GlyphEditViewInner {
    app: OnceCell<gtk::Application>,
    glyph: OnceCell<Rc<RefCell<Glyph>>>,
    glyph_state: OnceCell<Rc<RefCell<GlyphState>>>,
    viewport: Canvas,
    statusbar_context_id: Cell<Option<u32>>,
    overlay: super::Overlay,
    hovering: Cell<Option<(usize, usize)>>,
    pub toolbar_box: gtk::Box,
    pub viewhidebox: OnceCell<visibility_toggles::ViewHideBox>,
    zoom_percent_label: gtk::Label,
    units_per_em: Cell<f64>,
    descender: Cell<f64>,
    x_height: Cell<f64>,
    cap_height: Cell<f64>,
    ascender: Cell<f64>,
    settings: OnceCell<Settings>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphEditViewInner {
    const NAME: &'static str = "GlyphEditView";
    type Type = GlyphEditView;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GlyphEditViewInner {
    // Here we are overriding the glib::Object::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.statusbar_context_id.set(None);
        self.viewport.set_mouse(ViewPoint((0.0, 0.0).into()));

        self.viewport.connect_scroll_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                if event.state().contains(gtk::gdk::ModifierType::SHIFT_MASK) {
                    let (mut dx, mut dy) = event.delta();
                    dbg!(event.delta());
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
                viewport.set_mouse(ViewPoint(event.position().into()));
                retval
            }),
        );

        self.viewport.connect_button_release_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let retval = Tool::on_button_release_event(obj, viewport, event);
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
        self.toolbar_box
            .set_orientation(gtk::Orientation::Horizontal);
        self.toolbar_box.set_expand(false);
        self.toolbar_box.set_halign(gtk::Align::Center);
        self.toolbar_box.set_valign(gtk::Align::Start);
        self.toolbar_box.set_spacing(5);
        self.toolbar_box.set_visible(true);
        self.toolbar_box.set_can_focus(true);
        let toolbar = gtk::Toolbar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            //.toolbar_style(gtk::ToolbarStyle::Both)
            .visible(true)
            .can_focus(true)
            .build();

        let panning_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::GRAB_ICON_SVG,
            )),
            Some("panning"),
        );
        panning_button.set_visible(true);
        // FIXME: doesn't seem to work?
        panning_button.set_tooltip_text(Some("Pan"));
        panning_button.connect_clicked(clone!(@weak obj => move |_self| {
        }));

        let manipulate_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::SELECT_ICON_SVG,
            )),
            Some("Manipulate"),
        );
        manipulate_button.set_visible(true);
        // FIXME: doesn't seem to work?
        manipulate_button.set_tooltip_text(Some("Pan"));
        manipulate_button.connect_clicked(clone!(@weak obj => move |_self| {
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
            let t = &obj.imp().viewport.imp().transformation;
            t.zoom_in();
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
            let t = &obj.imp().viewport.imp().transformation;
            t.zoom_out();
        }));

        self.zoom_percent_label.set_label("100%");
        self.zoom_percent_label.set_visible(true);
        self.zoom_percent_label.set_selectable(true); // So that the widget can receive the button-press event
        self.zoom_percent_label.set_width_chars(5); // So that if 2 digit zoom (<100%) has the same length as a widget with a three digit zoom value. For example 75% and 125% should result in the same width
        self.zoom_percent_label
            .set_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK);
        self.zoom_percent_label
            .set_tooltip_text(Some("Interface zoom percentage"));

        self.zoom_percent_label.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, _event| {
                let t = &obj.imp().viewport.imp().transformation;
                t.reset_zoom();
                Inhibit(false)
            }),
        );
        self.viewport
            .imp()
            .transformation
            .bind_property(Transformation::SCALE, &self.zoom_percent_label, "label")
            .transform_to(|_, scale: &Value| {
                let scale: f64 = scale.get().ok()?;
                Some(format!("{:.0}%", scale * 100.).to_value())
            })
            .build();
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
        toolbar.add(&panning_button);
        toolbar.set_item_homogeneous(&panning_button, false);
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
        self.toolbar_box.pack_start(&toolbar, false, false, 0);
        self.toolbar_box
            .pack_start(&self.zoom_percent_label, false, false, 0);
        self.toolbar_box.pack_start(&debug_button, false, false, 0);
        self.toolbar_box
            .style_context()
            .add_class("glyph-edit-toolbox");
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

    pub fn new(app: gtk::Application, project: Project, glyph: Rc<RefCell<Glyph>>) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp().glyph.set(glyph.clone()).unwrap();
        ret.imp().app.set(app.clone()).unwrap();
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
        ret.imp()
            .viewport
            .imp()
            .transformation
            .set_property::<f64>(Transformation::PIXELS_PER_UNIT, {
                EM_SQUARE_PIXELS / project.property::<f64>(Project::UNITS_PER_EM)
            });
        ret.imp()
            .viewport
            .imp()
            .transformation
            .bind_property(
                Transformation::PIXELS_PER_UNIT,
                &project,
                Project::UNITS_PER_EM,
            )
            .transform_from(|_, units_per_em: &Value| {
                let units_per_em: f64 = units_per_em.get().ok()?;
                Some((EM_SQUARE_PIXELS / units_per_em).to_value())
            })
            .build();
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
        ret
    }
}
