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

use super::super::SelectionModifier;
use super::tool_impl::*;

use crate::GlyphEditView;
use crate::{
    utils::points::Point,
    views::{
        canvas::{Layer, LayerBuilder},
        Canvas, Transformation, UnitPoint, ViewPoint,
    },
};
use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::Inhibit;
use gtk::{cairo::Matrix, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    None,
    Pan,
    Drag,
    DragGuideline(usize),
    Select,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::None
    }
}

#[derive(Default)]
pub struct PanningToolInner {
    pub active: Cell<bool>,
    pub mode: Cell<Mode>,
    pub is_selection_empty: Cell<bool>,
    pub is_selection_active: Cell<bool>,
    pub selection_upper_left: Cell<UnitPoint>,
    pub selection_bottom_right: Cell<UnitPoint>,
    cursor: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
    cursor_plus: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
    cursor_minus: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
    layer: OnceCell<Layer>,
}

impl std::fmt::Debug for PanningToolInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("PanningTool")
            .field("mode", &self.mode.get())
            .field("active", &self.active.get())
            .field("is_selection_empty", &self.is_selection_empty.get())
            .field("is_selection_active", &self.is_selection_active.get())
            .finish()
    }
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
            crate::resources::icons::GRAB_ICON.to_image_widget(),
        );
        for (field, resource) in [
            (&self.cursor, crate::resources::cursors::ARROW_CURSOR),
            (
                &self.cursor_plus,
                crate::resources::cursors::ARROW_PLUS_CURSOR,
            ),
            (
                &self.cursor_minus,
                crate::resources::cursors::ARROW_MINUS_CURSOR,
            ),
        ] {
            field.set(resource.to_pixbuf()).unwrap();
        }
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
        let scale: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
        let event_button = event.button();
        match self.mode.get() {
            Mode::Pan => {
                self.mode.set(Mode::None);
                view.imp().hovering.set(None);
                viewport.queue_draw();
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            Mode::None if event_button == gtk::gdk::BUTTON_MIDDLE => {
                self.mode.set(Mode::Pan);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, true);
                viewport.set_cursor("crosshair");
            }
            m @ Mode::Drag | m @ Mode::DragGuideline(_)
                if event_button == gtk::gdk::BUTTON_PRIMARY =>
            {
                self.mode.set(Mode::None);
                view.imp().hovering.set(None);
                viewport.queue_draw();
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
                if matches!(
                    (m, event.event_type()),
                    (Mode::Drag, gtk::gdk::EventType::DoubleButtonPress)
                ) {
                    let UnitPoint(position) =
                        viewport.view_to_unit_point(ViewPoint(event.position().into()));
                    let pts = glyph_state
                        .kd_tree
                        .borrow()
                        .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
                    if !pts.is_empty() {
                        let menu = crate::utils::menu::Menu::new()
                            .title(Some("Point".into()))
                            .separator()
                            .add_button_cb("Dissolve point", move |_| {})
                            .add_button_cb("Make smooth", move |_| {})
                            .add_button_cb("Make corner", move |_| {});
                        menu.popup(event.time());
                    }
                }
            }
            Mode::None if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let event_position = event.position();
                let uposition @ UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                let lock_guidelines = view.property::<bool>(GlyphEditView::LOCK_GUIDELINES);
                if viewport.property::<bool>(Canvas::SHOW_RULERS) && !lock_guidelines {
                    let ruler_breadth = viewport.property::<f64>(Canvas::RULER_BREADTH_PIXELS);
                    if event_position.0 < ruler_breadth || event_position.1 < ruler_breadth {
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
                    if lock_guidelines {
                        break;
                    }
                    if g.imp().on_line_query(position, None) {
                        view.imp()
                            .select_object(Some(g.clone().upcast::<gtk::glib::Object>()));
                        self.mode.set(Mode::DragGuideline(i));
                        self.instance()
                            .set_property::<bool>(PanningTool::ACTIVE, true);
                        is_guideline = true;
                        viewport.set_cursor("grab");
                        break;
                    }
                }
                if !is_guideline {
                    let curve_query = {
                        let glyph = glyph_state.glyph.borrow();
                        glyph.on_curve_query(position, &[])
                    };
                    let pts = glyph_state
                        .kd_tree
                        .borrow()
                        .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
                    let current_selection = glyph_state.get_selection();
                    let is_empty = if current_selection.is_empty()
                        || !pts.iter().any(|i| current_selection.contains(&i.uuid))
                    {
                        glyph_state.set_selection(&pts, event.state().into());
                        pts.is_empty()
                    } else {
                        current_selection.is_empty()
                    };
                    if is_empty {
                        if let Some(((i, j), curve)) = curve_query {
                            let pts = curve
                                .points()
                                .borrow()
                                .iter()
                                .map(|cp| cp.uuid)
                                .collect::<HashSet<_>>();
                            if !pts.is_empty() {
                                self.is_selection_empty.set(false);
                                if !glyph_state.get_selection().is_superset(&pts) {
                                    let pts = curve
                                        .points()
                                        .borrow()
                                        .iter()
                                        .map(|cp| cp.glyph_index(i, j))
                                        .collect::<Vec<_>>();
                                    glyph_state.set_selection(&pts, event.state().into());
                                }
                                self.instance()
                                    .set_property::<bool>(PanningTool::ACTIVE, true);
                                self.mode.set(Mode::Drag);
                                view.imp().hovering.set(Some((i, j)));
                                viewport.set_cursor("grab");
                                return Inhibit(true);
                            }
                        }
                        view.imp().hovering.set(None);
                        self.instance()
                            .set_property::<bool>(PanningTool::ACTIVE, true);
                        self.is_selection_active.set(true);
                        self.is_selection_empty.set(true);
                        self.selection_upper_left.set(uposition);
                        self.selection_bottom_right.set(uposition);
                        self.mode.set(Mode::Select);
                        self.set_default_cursor(&view);
                        viewport.queue_draw();
                    } else {
                        self.instance()
                            .set_property::<bool>(PanningTool::ACTIVE, true);
                        self.mode.set(Mode::Drag);
                        viewport.set_cursor("grab");
                    }
                }
            }
            Mode::Select if event_button == gtk::gdk::BUTTON_PRIMARY => {
                if self.is_selection_active.get() {
                    self.is_selection_empty.set(true);
                } else {
                    let event_position = event.position();
                    let position = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    self.selection_upper_left.set(position);
                    self.selection_bottom_right.set(position);
                    self.is_selection_empty.set(true);
                    self.is_selection_active.set(true);
                    self.instance()
                        .set_property::<bool>(PanningTool::ACTIVE, true);
                    glyph_state.set_selection(&[], SelectionModifier::Replace);
                }
            }
            Mode::None if event_button == gtk::gdk::BUTTON_SECONDARY => {
                let event_position = event.position();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                for (i, g) in glyph_state.glyph.borrow().guidelines.iter().enumerate() {
                    if g.imp().on_line_query(position, None) {
                        let menu = crate::utils::menu::Menu::new()
                            .title(Some(std::borrow::Cow::from(format!(
                                    "{} - {}",
                                    g.name().as_deref()
                                        .unwrap_or("Anonymous guideline"),
                                    g.identifier().as_deref()
                                        .unwrap_or("No identifier")
                                ))))
                            .separator()
                            .add_button_cb("Edit", clone!(@weak g =>  move |_| {
                                let obj = g.upcast::<gtk::glib::Object>();
                                let w = crate::utils::new_property_window(obj, "Settings");
                                w.present();
                            }))
                            .add_button_cb("Delete", clone!(@weak view as obj, @weak viewport =>  move |_| {
                                let glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                                if glyph_state.glyph.borrow().guidelines.get(i).is_some() { // Prevent panic if `i` out of bounds
                                    let mut action = glyph_state.delete_guideline(i);
                                    (action.redo)();
                                    glyph_state.add_undo_action(action);
                                    viewport.queue_draw();
                                }
                            }));
                        menu.popup(event.time());
                        return Inhibit(true);
                    }
                }
                self.is_selection_empty.set(true);
                self.is_selection_active.set(false);
                glyph_state.set_selection(&[], SelectionModifier::Replace);

                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                viewport.queue_draw();
                self.set_default_cursor(&view);
            }
            Mode::Select if event_button == gtk::gdk::BUTTON_SECONDARY => {
                self.is_selection_empty.set(true);
                self.is_selection_active.set(false);
                glyph_state.set_selection(&[], SelectionModifier::Replace);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                viewport.queue_draw();
                self.set_default_cursor(&view);
                self.mode.set(Mode::None);
            }
            _ => return Inhibit(false),
        }
        Inhibit(true)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mode = self.mode.get();
        if mode == Mode::Select && self.is_selection_active.get() && self.is_selection_empty.get() {
            let event_position = event.position();
            let upper_left = self.selection_upper_left.get();
            let bottom_right = viewport.view_to_unit_point(ViewPoint(event_position.into()));
            self.is_selection_active.set(false);
            self.instance()
                .set_property::<bool>(PanningTool::ACTIVE, false);
            self.selection_bottom_right.set(bottom_right);
            let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
            let pts = glyph_state
                .kd_tree
                .borrow()
                .query_region((upper_left.0, bottom_right.0));
            if !pts.is_empty() {
                self.is_selection_empty.set(false);
            }
            glyph_state.set_selection(&pts, event.state().into());
            self.mode.set(Mode::None);
            self.set_default_cursor(&view);
            return Inhibit(true);
        }
        let event_button = event.button();
        match mode {
            Mode::Pan if event_button == gtk::gdk::BUTTON_MIDDLE => {
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.mode.set(Mode::None);
                viewport.queue_draw();
                self.set_default_cursor(&view);
            }
            Mode::Select
                if event_button == gtk::gdk::BUTTON_PRIMARY && self.is_selection_active.get() =>
            {
                let event_position = event.position();
                let bottom_right = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                self.selection_bottom_right.set(bottom_right);
                self.is_selection_active.set(false);
                self.is_selection_empty.set(true);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.mode.set(Mode::None);
                self.set_default_cursor(&view);
                viewport.queue_draw();
            }
            Mode::Drag if event_button == gtk::gdk::BUTTON_PRIMARY => {
                self.mode.set(Mode::None);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            Mode::None if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let mut glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
                let glyph = glyph_state.glyph.borrow();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                if let Some(((i, j), curve)) = glyph.on_curve_query(position, &[]) {
                    drop(glyph);
                    self.is_selection_empty.set(false);
                    let pts = curve
                        .points()
                        .borrow()
                        .iter()
                        .map(|cp| cp.glyph_index(i, j))
                        .collect::<Vec<_>>();
                    glyph_state.set_selection(&pts, event.state().into());
                    self.mode.set(Mode::None);
                } else {
                    return Inhibit(false);
                }
            }
            _ if event_button == gtk::gdk::BUTTON_MIDDLE => {
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            _ if event_button == gtk::gdk::BUTTON_SECONDARY => {
                let glyph_state = view.imp().glyph_state.get().unwrap().borrow();
                self.set_default_cursor(&view);
                let glyph = glyph_state.glyph.borrow();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                if let Some(((i, _), _curve)) = glyph.on_curve_query(position, &[]) {
                    crate::utils::menu::Menu::new()
                            .add_button_cb(
                                "reverse",
                                clone!(@strong view => move |_| {
                                    let glyph_state = view.imp().glyph_state.get().unwrap().borrow_mut();
                                    glyph_state.glyph.borrow().contours[i].reverse_direction();
                                }),
                            ).popup(event.time());
                    return Inhibit(true);
                }
                return Inhibit(false);
            }
            Mode::None => return Inhibit(false),
            _ => return Inhibit(false),
        }
        Inhibit(true)
    }

    fn on_scroll_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        if event.state().contains(gtk::gdk::ModifierType::SHIFT_MASK) {
            /* pan with middle mouse button */
            let (mut dx, mut dy) = event.delta();
            if event.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                if dy.abs() > dx.abs() {
                    dx = dy;
                }
                dy = 0.0;
            }
            viewport
                .imp()
                .transformation
                .move_camera_by_delta(ViewPoint(<_ as Into<Point>>::into((5.0 * dx, 5.0 * dy))));
            viewport.queue_draw();
            return Inhibit(true);
        } else if event.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            if let Mode::DragGuideline(idx) = self.mode.get() {
                /* rotate guideline that is currently being dragged */
                let (_dx, dy) = event.delta();
                let glyph_state = view.imp().glyph_state.get().unwrap().borrow();
                glyph_state.transform_guideline(idx, Matrix::identity(), 1.5 * dy);
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let scale: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let warp_cursor = viewport.property::<bool>(Canvas::WARP_CURSOR);
        let glyph_state = view.imp().glyph_state.get().unwrap().borrow();
        let UnitPoint(position) = viewport.view_to_unit_point(ViewPoint(event.position().into()));
        if !self.instance().property::<bool>(PanningTool::ACTIVE) {
            let glyph = glyph_state.glyph.borrow();
            let pts = glyph_state
                .kd_tree
                .borrow()
                .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
            if pts.is_empty() {
                view.imp().hovering.set(None);
                viewport.queue_draw();
            }
            if let Some(((i, j), curve)) = glyph.on_curve_query(position, &pts) {
                viewport.set_cursor("grab");
                view.imp().new_statusbar_message(&format!("{:?}", curve));
                view.imp().hovering.set(Some((i, j)));
                viewport.queue_draw();
            } else {
                self.set_default_cursor(&view);
            }
            return Inhibit(false);
        }

        match self.mode.get() {
            Mode::None => {}
            Mode::Drag => {
                let mouse: ViewPoint = viewport.get_mouse();
                let mut delta =
                    (<_ as Into<Point>>::into(event.position()) - mouse.0) / (scale * ppu);
                delta.y *= -1.0;
                let mut m = Matrix::identity();
                m.translate(delta.x, delta.y);
                glyph_state.transform_selection(m, true);
            }
            Mode::DragGuideline(idx) => {
                let mouse: ViewPoint = viewport.get_mouse();
                let mut delta =
                    (<_ as Into<Point>>::into(event.position()) - mouse.0) / (scale * ppu);
                delta.y *= -1.0;
                let mut m = gtk::cairo::Matrix::identity();
                m.translate(delta.x, delta.y);
                glyph_state.transform_guideline(idx, m, 0.0);
            }
            Mode::Pan => {
                if warp_cursor {
                    let (width, height) = (
                        viewport.allocated_width() as f64,
                        viewport.allocated_height() as f64,
                    );
                    let ruler_breadth = viewport.property::<f64>(Canvas::RULER_BREADTH_PIXELS);
                    let (x, y) = event.position();
                    if x + ruler_breadth >= width
                        || y + ruler_breadth >= height
                        || x <= ruler_breadth
                        || y <= ruler_breadth
                    {
                        let ViewPoint(mouse) = viewport.get_mouse();
                        if let Some(device) = event.device() {
                            let (screen, root_x, root_y) = device.position();
                            let move_: Option<(i32, i32)> = if x + ruler_breadth >= width {
                                viewport.set_mouse(ViewPoint(mouse - (width, 0.0).into()));
                                (root_x - width as i32 + 3 * ruler_breadth as i32, root_y).into()
                            } else if y + ruler_breadth >= height {
                                viewport.set_mouse(ViewPoint(mouse - (0.0, height).into()));
                                (root_x, root_y - height as i32 - ruler_breadth as i32).into()
                            } else if x <= ruler_breadth {
                                viewport.set_mouse(ViewPoint(mouse + (width, 0.0).into()));
                                (root_x + width as i32 - 3 * ruler_breadth as i32, root_y).into()
                            } else if y <= ruler_breadth {
                                viewport.set_mouse(ViewPoint(mouse + (0.0, height).into()));
                                (root_x, root_y + height as i32 - 3 * ruler_breadth as i32).into()
                            } else {
                                None
                            };
                            if let Some((move_x, move_y)) = move_ {
                                device.warp(&screen, move_x, move_y);
                            }
                        }
                    }
                }
                let mouse: ViewPoint = viewport.get_mouse();
                let delta = <_ as Into<Point>>::into(event.position()) - mouse.0;
                viewport
                    .imp()
                    .transformation
                    .move_camera_by_delta(ViewPoint(delta));
            }
            Mode::Select => {
                if let Some(pixbuf) = match event.state().into() {
                    SelectionModifier::Add => self.cursor_plus.get().unwrap().clone(),
                    SelectionModifier::Remove => self.cursor_minus.get().unwrap().clone(),
                    SelectionModifier::Replace => self.cursor.get().unwrap().clone(),
                } {
                    view.imp().viewport.set_cursor_from_pixbuf(pixbuf);
                }
                return if self.is_selection_active.get() {
                    let event_position = event.position();
                    let bottom_right =
                        viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    self.selection_bottom_right.set(bottom_right);
                    Inhibit(true)
                } else {
                    Inhibit(false)
                };
            }
        }
        Inhibit(true)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("selection box"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: &gtk::cairo::Context| {
                    PanningTool::draw_select_box(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(PanningTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.imp().viewport.add_post_layer(layer);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        obj.set_property::<bool>(PanningTool::ACTIVE, true);
        self.set_default_cursor(view);
        self.parent_on_activate(obj, view);
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        obj.set_property::<bool>(PanningTool::ACTIVE, false);
        self.set_default_cursor(view);
        self.parent_on_deactivate(obj, view);
    }
}

impl PanningToolInner {
    fn set_default_cursor(&self, view: &GlyphEditView) {
        if let Some(pixbuf) = self.cursor.get().unwrap().clone() {
            view.imp().viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.imp().viewport.set_cursor("default");
        }
    }
}

glib::wrapper! {
    pub struct PanningTool(ObjectSubclass<PanningToolInner>)
        @extends ToolImpl;
}

impl Default for PanningTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PanningTool {
    pub const ACTIVE: &str = "active";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn draw_select_box(
        viewport: &Canvas,
        cr: &gtk::cairo::Context,
        obj: GlyphEditView,
    ) -> Inhibit {
        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
        let t = glyph_state.tools[&Self::static_type()]
            .clone()
            .downcast::<PanningTool>()
            .unwrap();
        if !t.imp().active.get() || t.imp().mode.get() != Mode::Select {
            return Inhibit(false);
        }
        let active = t.imp().is_selection_active.get();
        let empty = t.imp().is_selection_empty.get();
        if empty && !active {
            return Inhibit(false);
        }
        let UnitPoint(upper_left) = t.imp().selection_upper_left.get();
        let UnitPoint(bottom_right) = t.imp().selection_bottom_right.get();

        let scale: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);

        /* Calculate how much we need to multiply a pixel value to scale it back after performing
         * the matrix transformation */
        let f = 1.0 / (scale * ppu);

        let line_width = if active { 2.0 } else { 1.5 } * f;

        let matrix = viewport.imp().transformation.matrix();
        let (width, height) = ((bottom_right - upper_left).x, (bottom_right - upper_left).y);
        if width == 0.0 || height == 0.0 {
            return Inhibit(false);
        }

        cr.save().unwrap();

        cr.set_line_width(line_width);
        cr.set_dash(&[4.0 * f, 2.0 * f], 0.5 * f);
        cr.transform(matrix);

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.9);
        cr.rectangle(upper_left.x, upper_left.y, width, height);
        if active {
            cr.stroke_preserve().unwrap();
            // turqoise, #278cac
            cr.set_source_rgba(39.0 / 255.0, 140.0 / 255.0, 172.0 / 255.0, 0.1);
            cr.fill().unwrap();
        } else {
            cr.stroke().unwrap();
        }
        cr.restore().unwrap();

        if !active {
            let rectangle_dim = 5.0 * f;

            cr.save().unwrap();
            cr.set_line_width(line_width);
            cr.transform(matrix);
            for p in [
                upper_left,
                bottom_right,
                upper_left + (width, 0.0).into(),
                upper_left + (0.0, height).into(),
            ] {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.9);
                cr.rectangle(
                    p.x - rectangle_dim / 2.0,
                    p.y - rectangle_dim / 2.0,
                    rectangle_dim,
                    rectangle_dim,
                );
                cr.stroke_preserve().unwrap();
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr.fill().unwrap();
            }

            cr.restore().unwrap();
        }

        Inhibit(true)
    }
}
