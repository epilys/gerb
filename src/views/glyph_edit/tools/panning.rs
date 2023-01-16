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

use super::tool_impl::*;

use crate::{
    utils::points::Point,
    views::{Canvas, GlyphEditView, UnitPoint, ViewPoint},
};
use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::Inhibit;
use gtk::{
    glib::{self},
    prelude::*,
    subclass::prelude::*,
};
use std::cell::Cell;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ControlPointMode {
    None,
    Drag,
    DragGuideline(usize),
    Select,
}

impl Default for ControlPointMode {
    fn default() -> ControlPointMode {
        ControlPointMode::None
    }
}

#[derive(Default)]
pub struct PanningToolInner {
    pub active: Cell<bool>,
    pub mode: Cell<ControlPointMode>,
    pub is_selection_empty: Cell<bool>,
    pub is_selection_active: Cell<bool>,
    pub selection_upper_left: Cell<UnitPoint>,
    pub selection_bottom_right: Cell<UnitPoint>,
}

#[glib::object_subclass]
impl ObjectSubclass for PanningToolInner {
    const NAME: &'static str = "PanningTool";
    type ParentType = ToolImpl;
    type Type = PanningTool;
}

impl ObjectImpl for PanningToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.active.set(false);
        self.is_selection_empty.set(true);
        self.is_selection_active.set(false);
        obj.set_property::<String>(ToolImpl::NAME, "Panning".to_string());
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::svg_to_image_widget(crate::resources::GRAB_ICON_SVG),
        );
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    PanningTool::ACTIVE,
                    PanningTool::ACTIVE,
                    PanningTool::ACTIVE,
                    false,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            PanningTool::ACTIVE => self.active.get().to_value(),
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
            PanningTool::ACTIVE => self.active.set(value.get().unwrap()),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl ToolImplImpl for PanningToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
        match event.button() {
            gtk::gdk::BUTTON_MIDDLE => {
                self.mode.set(ControlPointMode::None);
                glyph_state.active_tool = PanningTool::static_type();
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, true);
                viewport.set_cursor("crosshair");
                Inhibit(true)
            }
            gtk::gdk::BUTTON_PRIMARY => {
                match self.mode.get() {
                    ControlPointMode::Drag | ControlPointMode::DragGuideline(_) => {
                        self.mode.set(ControlPointMode::None);
                        view.imp().hovering.set(None);
                        viewport.queue_draw();
                        glyph_state.active_tool = glib::types::Type::INVALID;
                        self.instance()
                            .set_property::<bool>(PanningTool::ACTIVE, false);
                        viewport.set_cursor("default");
                    }
                    ControlPointMode::None => {
                        let event_position = event.position();
                        let UnitPoint(position) =
                            viewport.view_to_unit_point(ViewPoint(event_position.into()));
                        if viewport.property::<bool>(Canvas::SHOW_RULERS) {
                            let ruler_breadth =
                                viewport.property::<f64>(Canvas::RULER_BREADTH_PIXELS);
                            if event_position.0 < ruler_breadth || event_position.1 < ruler_breadth
                            {
                                let angle = if event_position.0 < ruler_breadth
                                    && event_position.1 < ruler_breadth
                                {
                                    -45.0
                                } else if event_position.0 < ruler_breadth {
                                    90.0
                                } else {
                                    0.0
                                };
                                let mut action = glyph_state.new_guideline(angle, position);
                                (action.redo)();
                                let app: &crate::Application = crate::Application::from_instance(
                                    view.imp()
                                        .app
                                        .get()
                                        .unwrap()
                                        .downcast_ref::<crate::GerbApp>()
                                        .unwrap(),
                                );
                                let undo_db = app.undo_db.borrow_mut();
                                undo_db.event(action);
                            }
                        }
                        let mut is_guideline: bool = false;
                        for (i, g) in glyph_state.glyph.borrow().guidelines.iter().enumerate() {
                            if g.imp().on_line_query(position, None) {
                                view.imp()
                                    .select_object(Some(g.clone().upcast::<gtk::glib::Object>()));
                                self.mode.set(ControlPointMode::DragGuideline(i));
                                self.instance()
                                    .set_property::<bool>(PanningTool::ACTIVE, true);
                                is_guideline = true;
                                viewport.set_cursor("grab");
                                break;
                            }
                        }
                        if !is_guideline {
                            let pts = glyph_state.kd_tree.borrow().query_point(position, 10);
                            glyph_state.set_selection(&pts);
                            if pts.is_empty() {
                                view.imp().hovering.set(None);
                                viewport.queue_draw();
                                glyph_state.active_tool = glib::types::Type::INVALID;
                                self.instance()
                                    .set_property::<bool>(PanningTool::ACTIVE, false);
                                viewport.set_cursor("default");
                            } else {
                                self.instance()
                                    .set_property::<bool>(PanningTool::ACTIVE, true);
                                glyph_state.active_tool = PanningTool::static_type();
                                self.mode.set(ControlPointMode::Drag);
                                viewport.set_cursor("grab");
                            }
                        }

                        if !self.instance().property::<bool>(PanningTool::ACTIVE) {
                            self.instance()
                                .set_property::<bool>(PanningTool::ACTIVE, true);
                            glyph_state.active_tool = PanningTool::static_type();
                            self.mode.set(ControlPointMode::Select);
                            viewport.set_cursor("default");
                        }
                    }
                    ControlPointMode::Select => {
                        match (
                            self.is_selection_active.get(),
                            self.is_selection_empty.get(),
                        ) {
                            (_, true) | (false, false) => {
                                let event_position = event.position();
                                let position =
                                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                                self.selection_upper_left.set(position);
                                self.selection_bottom_right.set(position);
                                self.is_selection_empty.set(false);
                                self.is_selection_active.set(true);
                                glyph_state.set_selection(&[]);
                            }
                            (true, false) => {
                                self.is_selection_empty.set(true);
                            }
                        }
                    }
                }
                Inhibit(true)
            }
            gtk::gdk::BUTTON_SECONDARY => {
                if self.mode.get() == ControlPointMode::None {
                    self.instance()
                        .set_property::<bool>(PanningTool::ACTIVE, false);
                    viewport.queue_draw();
                    viewport.set_cursor("default");
                    glyph_state.active_tool = glib::types::Type::INVALID;
                    Inhibit(true)
                } else if self.mode.get() == ControlPointMode::Select {
                    self.is_selection_empty.set(true);
                    self.is_selection_active.set(false);
                    glyph_state.set_selection(&[]);
                    self.instance()
                        .set_property::<bool>(PanningTool::ACTIVE, false);
                    viewport.queue_draw();
                    viewport.set_cursor("default");
                    self.mode.set(ControlPointMode::None);
                    glyph_state.active_tool = glib::types::Type::INVALID;
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }
            _ => Inhibit(false),
        }
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mode = self.mode.get();
        if mode == ControlPointMode::Select {
            return match (
                self.is_selection_active.get(),
                self.is_selection_empty.get(),
            ) {
                (_, true) => Inhibit(false),
                (true, false) => {
                    let event_position = event.position();
                    let upper_left = self.selection_upper_left.get();
                    let bottom_right =
                        viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    self.is_selection_active.set(false);
                    self.selection_bottom_right.set(bottom_right);
                    let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
                    let pts = glyph_state
                        .kd_tree
                        .borrow()
                        .query_region((upper_left.0, bottom_right.0));
                    glyph_state.set_selection(&pts);
                    Inhibit(true)
                }
                (false, _) => Inhibit(false),
            };
        }
        let active = self.instance().property::<bool>(PanningTool::ACTIVE);
        if !active {
            return Inhibit(false);
        }
        match event.button() {
            gtk::gdk::BUTTON_PRIMARY => {
                if mode == ControlPointMode::None {
                    return Inhibit(false);
                }
                view.imp()
                    .glyph_state
                    .get()
                    .unwrap()
                    .borrow_mut()
                    .active_tool = glib::types::Type::INVALID;
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                viewport.set_cursor("default");
            }
            gtk::gdk::BUTTON_MIDDLE => {
                view.imp()
                    .glyph_state
                    .get()
                    .unwrap()
                    .borrow_mut()
                    .active_tool = glib::types::Type::INVALID;
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                viewport.set_cursor("default");
            }
            _ => return Inhibit(false),
        }
        Inhibit(true)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
        let UnitPoint(position) = viewport.view_to_unit_point(ViewPoint(event.position().into()));
        if !self.instance().property::<bool>(PanningTool::ACTIVE) {
            let glyph = glyph_state.glyph.borrow();
            let pts = glyph_state.kd_tree.borrow().query_point(position, 10);
            if pts.is_empty() {
                view.imp().hovering.set(None);
                viewport.queue_draw();
            }
            if let Some(((i, j), curve)) = glyph.on_curve_query(position, &pts) {
                view.imp().new_statusbar_message(&format!("{:?}", curve));
                view.imp().hovering.set(Some((i, j)));
                viewport.set_cursor("grab");
                viewport.queue_draw();
            }
            return Inhibit(false);
        }
        match self.mode.get() {
            ControlPointMode::Drag => {
                glyph_state.update_positions(position);
            }
            ControlPointMode::DragGuideline(idx) => {
                let mut action = glyph_state.update_guideline(idx, position);
                (action.redo)();
                let app: &crate::Application = crate::Application::from_instance(
                    view.imp()
                        .app
                        .get()
                        .unwrap()
                        .downcast_ref::<crate::GerbApp>()
                        .unwrap(),
                );
                let undo_db = app.undo_db.borrow_mut();
                undo_db.event(action);
            }
            ControlPointMode::None => {
                let mouse: ViewPoint = viewport.get_mouse();
                let delta = <_ as Into<Point>>::into(event.position()) - mouse.0;
                viewport
                    .imp()
                    .transformation
                    .move_camera_by_delta(ViewPoint(delta));
            }
            ControlPointMode::Select => {
                return match (
                    self.is_selection_active.get(),
                    self.is_selection_empty.get(),
                ) {
                    (_, true) => Inhibit(false),
                    (true, false) => {
                        let event_position = event.position();
                        let bottom_right =
                            viewport.view_to_unit_point(ViewPoint(event_position.into()));
                        self.selection_bottom_right.set(bottom_right);
                        Inhibit(true)
                    }
                    (false, _) => Inhibit(false),
                };
            }
        }
        Inhibit(true)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        obj.set_property::<bool>(PanningTool::ACTIVE, true);
        view.imp().viewport.set_cursor("grab");
        self.parent_on_activate(obj, view);
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        obj.set_property::<bool>(PanningTool::ACTIVE, false);
        view.imp().viewport.set_cursor("default");
        self.parent_on_deactivate(obj, view);
    }
}

impl PanningToolInner {}

glib::wrapper! {
    pub struct PanningTool(ObjectSubclass<PanningToolInner>)
        @extends ToolImpl;
}

impl PanningTool {
    pub const ACTIVE: &str = "active";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}
