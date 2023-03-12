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

use super::{constraints::*, tool_impl::*, MoveDirection, SelectionAction, SelectionModifier};

use gtk::cairo::Matrix;
use gtk::Inhibit;
use std::collections::HashSet;

use crate::glyphs::Contour;
use crate::prelude::*;
use crate::views::canvas::{Layer, LayerBuilder};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum Mode {
    #[default]
    None,
    Pan,
    Drag,
    DragGuideline(usize),
    ResizeDimensions {
        previous_value: Option<f64>,
    },
    Select,
}

#[derive(Default)]
pub struct PanningToolInner {
    pub active: Cell<bool>,
    pub mode: Cell<Mode>,
    pub is_selection_empty: Cell<bool>,
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
        view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let event_button = event.button();
        match self.mode.get() {
            Mode::Pan => {
                self.mode.set(Mode::None);
                view.hovering.set(None);
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
                Lock::clear(&view);
                Precision::clear(&view);
                self.mode.set(Mode::None);
                if matches!(m, Mode::Drag) {
                    view.set_property(Editor::MODIFYING_IN_PROCESS, false);
                }
                view.hovering.set(None);
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
                    let state = view.state().borrow();
                    let curve_query = {
                        let glyph = state.glyph.borrow();
                        glyph.on_curve_query(position, &[])
                    };
                    if let Some(((i, _), _)) = curve_query {
                        let pts = {
                            let glyph = state.glyph.borrow();
                            let tmp = glyph.contours[i]
                                .curves()
                                .iter()
                                .enumerate()
                                .flat_map(|(j, c)| {
                                    c.points()
                                        .iter()
                                        .map(move |cp| cp.glyph_index(i, j))
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                })
                                .collect::<Vec<_>>();
                            tmp
                        };
                        drop(state);
                        view.set_selection(&pts, SelectionModifier::Replace);
                        self.instance()
                            .set_property::<bool>(PanningTool::ACTIVE, true);
                        self.mode.set(Mode::Drag);
                        view.set_property(Editor::MODIFYING_IN_PROCESS, true);
                        viewport.set_cursor("grab");
                    }
                }
            }
            Mode::None if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let event_position = event.position();
                let uposition @ UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                let lock_guidelines = view.property::<bool>(Editor::LOCK_GUIDELINES);
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
                        let mut action = view.state().borrow().new_guideline(angle, position);
                        (action.redo)();
                        let app: &Application = view.app();
                        let undo_db = app.undo_db.borrow();
                        undo_db.event(action);
                    }
                }
                let mut is_guideline: bool = false;
                for (i, g) in view
                    .state()
                    .borrow()
                    .glyph
                    .borrow()
                    .guidelines()
                    .iter()
                    .enumerate()
                {
                    if lock_guidelines {
                        break;
                    }
                    if g.on_line_query(position, None) {
                        g.set_property(Guideline::X, position.x);
                        g.set_property(Guideline::Y, position.y);
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
                        let state = view.state().borrow();
                        let glyph = state.glyph.borrow();
                        glyph.on_curve_query(position, &[])
                    };
                    let pts = view
                        .state()
                        .borrow()
                        .kd_tree
                        .borrow()
                        .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
                    let is_empty = {
                        let state = view.state().borrow();
                        let current_selection = state.get_selection_set();
                        if current_selection.is_empty()
                            || !pts.iter().any(|i| current_selection.contains(&i.uuid))
                        {
                            drop(state);
                            view.set_selection(&pts, event.state().into());
                            pts.is_empty()
                        } else {
                            current_selection.is_empty()
                        }
                    };
                    if is_empty {
                        if let Some(((i, j), curve)) = curve_query {
                            let pts = curve
                                .points()
                                .iter()
                                .map(|cp| cp.uuid)
                                .collect::<HashSet<_>>();
                            if !pts.is_empty() {
                                self.is_selection_empty.set(false);
                                if !view.state().borrow().get_selection_set().is_superset(&pts) {
                                    let pts = curve
                                        .points()
                                        .iter()
                                        .map(|cp| cp.glyph_index(i, j))
                                        .collect::<Vec<_>>();
                                    view.set_selection(&pts, event.state().into());
                                }
                                self.instance()
                                    .set_property::<bool>(PanningTool::ACTIVE, true);
                                self.mode.set(Mode::Drag);
                                view.set_property(Editor::MODIFYING_IN_PROCESS, true);
                                view.hovering.set(Some((i, j)));
                                viewport.set_cursor("grab");
                                return Inhibit(true);
                            }
                        }
                        view.hovering.set(None);
                        self.instance()
                            .set_property::<bool>(PanningTool::ACTIVE, true);
                        if viewport.property::<bool>(Canvas::SHOW_TOTAL_AREA) {
                            let previous_value = view.state().borrow().glyph.borrow().width();
                            let glyph_width = previous_value.unwrap_or(0.0);
                            let (x, y) = (position.x, position.y);
                            let units_per_em = viewport
                                .transformation
                                .property::<f64>(Transformation::UNITS_PER_EM);

                            if (x - glyph_width).abs() <= 6.0 && (0.0..=units_per_em).contains(&y) {
                                /* resize glyph width box */
                                self.instance()
                                    .set_property::<bool>(PanningTool::ACTIVE, true);
                                // [ref:needs_user_doc]
                                self.mode.set(Mode::ResizeDimensions { previous_value });
                                viewport.set_cursor("grab");
                                return Inhibit(true);
                            }
                        }
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
                        view.set_property(Editor::MODIFYING_IN_PROCESS, true);
                        viewport.set_cursor("grab");
                    }
                }
            }
            Mode::Select if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let event_position = event.position();
                let position = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                self.selection_upper_left.set(position);
                self.selection_bottom_right.set(position);
                self.is_selection_empty.set(true);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, true);
                view.set_selection(&[], SelectionModifier::Replace);
            }
            Mode::None if event_button == gtk::gdk::BUTTON_SECONDARY => {
                let event_position = event.position();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event_position.into()));
                for (i, g) in view
                    .state()
                    .borrow()
                    .glyph
                    .borrow()
                    .guidelines()
                    .iter()
                    .enumerate()
                {
                    if g.on_line_query(position, None) {
                        let menu = crate::utils::menu::Menu::new()
                            .title(Some(std::borrow::Cow::from(format!(
                                "{} - {}",
                                g.name().as_deref().unwrap_or("Anonymous guideline"),
                                g.identifier().as_deref().unwrap_or("No identifier")
                            ))))
                            .separator()
                            .add_button_cb(
                                "Edit",
                                clone!(@weak g, @weak view =>  move |_| {
                                    let w = g.new_property_window(view.app(), false);
                                    w.present();
                                }),
                            )
                            .add_button_cb(
                                "Delete",
                                clone!(@weak view as obj, @weak viewport =>  move |_| {
                                    let state = obj.state().borrow();
                                    if state.glyph.borrow().guidelines().get(i).is_some() { // Prevent panic if `i` out of bounds
                                        let mut action = state.delete_guideline(i);
                                        (action.redo)();
                                        state.add_undo_action(action);
                                        viewport.queue_draw();
                                    }
                                }),
                            );
                        menu.popup(event.time());
                        return Inhibit(true);
                    }
                }
                self.is_selection_empty.set(true);
                view.set_selection(&[], SelectionModifier::Replace);

                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                viewport.queue_draw();
                self.set_default_cursor(&view);
            }
            Mode::Select if event_button == gtk::gdk::BUTTON_SECONDARY => {
                self.is_selection_empty.set(true);
                view.set_selection(&[], SelectionModifier::Replace);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                viewport.queue_draw();
                self.set_default_cursor(&view);
                self.mode.set(Mode::None);
            }
            Mode::ResizeDimensions { previous_value }
                if event_button == gtk::gdk::BUTTON_PRIMARY =>
            {
                let (metadata, new_value) = {
                    let state = view.state().borrow();
                    let glyph = state.glyph.borrow();
                    (glyph.metadata.clone(), glyph.width())
                };
                let action = Action {
                    stamp: EventStamp {
                        t: std::any::TypeId::of::<Editor>(),
                        property: GlyphMetadata::static_type().name(),
                        id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[]).into() },
                    },
                    compress: false,
                    redo: Box::new(clone!(@weak metadata => move || {
                        metadata.width.set(new_value);
                    })),
                    undo: Box::new(clone!(@weak metadata => move || {
                        metadata.width.set(previous_value);
                    })),
                };
                let app: &Application = view.app();
                let undo_db = app.undo_db.borrow();
                undo_db.event(action);

                self.mode.set(Mode::None);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            _ => return Inhibit(false),
        }
        Inhibit(true)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mode = self.mode.get();
        if mode == Mode::Select && self.is_selection_empty.get() {
            let event_position = event.position();
            let upper_left = self.selection_upper_left.get();
            let bottom_right = viewport.view_to_unit_point(ViewPoint(event_position.into()));
            self.instance()
                .set_property::<bool>(PanningTool::ACTIVE, false);
            self.selection_bottom_right.set(bottom_right);
            let pts = view
                .state()
                .borrow()
                .kd_tree
                .borrow()
                .query_region((upper_left.0, bottom_right.0));
            if !pts.is_empty() {
                self.is_selection_empty.set(false);
            }
            view.set_selection(&pts, event.state().into());
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
            Mode::Select if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let event_position = event.position();
                let bottom_right = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                self.selection_bottom_right.set(bottom_right);
                self.is_selection_empty.set(true);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.mode.set(Mode::None);
                self.set_default_cursor(&view);
                viewport.queue_draw();
            }
            Mode::Drag if event_button == gtk::gdk::BUTTON_PRIMARY => {
                view.action_group
                    .change_action_state(Editor::LOCK_ACTION, &Lock::empty().to_variant());
                self.mode.set(Mode::None);
                view.set_property(Editor::MODIFYING_IN_PROCESS, false);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            Mode::None if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let state = view.state().borrow();
                let glyph = state.glyph.borrow();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                if let Some(((i, j), curve)) = glyph.on_curve_query(position, &[]) {
                    drop(glyph);
                    drop(state);
                    self.is_selection_empty.set(false);
                    let pts = curve
                        .points()
                        .iter()
                        .map(|cp| cp.glyph_index(i, j))
                        .collect::<Vec<_>>();
                    view.set_selection(&pts, event.state().into());
                    self.mode.set(Mode::None);
                } else {
                    return Inhibit(false);
                }
            }
            Mode::ResizeDimensions { previous_value }
                if event_button == gtk::gdk::BUTTON_SECONDARY =>
            {
                let state = view.state().borrow();
                let glyph = state.glyph.borrow();
                glyph.width.set(previous_value);
                self.mode.set(Mode::None);
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            Mode::DragGuideline(idx) if event_button == gtk::gdk::BUTTON_PRIMARY => {
                let event_position = event.position();
                let ruler_breadth = viewport.property::<f64>(Canvas::RULER_BREADTH_PIXELS);
                if event_position.0 < ruler_breadth || event_position.1 < ruler_breadth {
                    let mut action = view.state().borrow().delete_guideline(idx);
                    (action.redo)();
                    let app: &Application = view.app();
                    let undo_db = app.undo_db.borrow();
                    undo_db.event(action);
                    self.mode.set(Mode::None);
                    view.hovering.set(None);
                    viewport.queue_draw();
                    self.instance()
                        .set_property::<bool>(PanningTool::ACTIVE, false);
                    self.set_default_cursor(&view);
                    return Inhibit(true);
                }
                return Inhibit(false);
            }
            _ if event_button == gtk::gdk::BUTTON_MIDDLE => {
                self.instance()
                    .set_property::<bool>(PanningTool::ACTIVE, false);
                self.set_default_cursor(&view);
            }
            _ if event_button == gtk::gdk::BUTTON_SECONDARY => {
                self.set_default_cursor(&view);
                let (is_corner, on_curve_query) = {
                    let scale: f64 = viewport
                        .transformation
                        .property::<f64>(Transformation::SCALE);
                    let ppu: f64 = viewport
                        .transformation
                        .property::<f64>(Transformation::PIXELS_PER_UNIT);
                    let state = view.state().borrow();
                    let glyph = state.glyph.borrow();
                    let UnitPoint(position) =
                        viewport.view_to_unit_point(ViewPoint(event.position().into()));
                    let pts = state
                        .kd_tree
                        .borrow()
                        .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
                    let is_corner = if pts.is_empty() {
                        None
                    } else {
                        pts.iter().find_map(|p| {
                            state.glyph.borrow().contours[p.contour_index].curves()[p.curve_index]
                                .points()
                                .iter()
                                .find(|cp| cp.uuid == p.uuid && cp.continuity.is_some())
                                .and_then(|cp| Some((*p, cp.continuity?)))
                        })
                    };
                    let on_curve_query = glyph.on_curve_query(position, &[]);
                    (is_corner, on_curve_query)
                };
                if let Some((idx, corner_continuity)) = is_corner {
                    let mut menu = crate::utils::menu::Menu::new()
                        .title(Some(format!("{:?}", corner_continuity).into()))
                        .separator();
                    menu = if !matches!(corner_continuity, Continuity::Velocity) {
                        menu.add_button_cb(
                            "make smooth",
                            clone!(@strong view => move |_| {
                                let new_val = Continuity::Velocity;
                                let mut action = change_continuity(&view, idx,corner_continuity, new_val);
                                (action.redo)();
                                let app: &Application = view
                                    .app();
                                let undo_db = app.undo_db.borrow();
                                undo_db.event(action);
                            }),
                        )
                    } else {
                        menu.add_button("make smooth")
                    };
                    menu = if !matches!(corner_continuity, Continuity::Positional) {
                        menu.add_button_cb(
                            "make corner",
                            clone!(@strong view => move |_| {
                                let new_val = Continuity::Positional;
                                let mut action = change_continuity(&view, idx,corner_continuity, new_val);
                                (action.redo)();
                                let app: &Application = view
                                    .app();
                                let undo_db = app.undo_db.borrow();
                                undo_db.event(action);
                            }),
                        )
                    } else {
                        menu.add_button("make corner")
                    };
                    menu = if !matches!(corner_continuity, Continuity::Tangent { .. }) {
                        menu.add_button_cb(
                            "make tangent",
                            clone!(@strong view => move |_| {
                                let new_val = Continuity::Tangent { beta: 1.00 };
                                let mut action = change_continuity(&view, idx,corner_continuity, new_val);
                                (action.redo)();
                                let app: &Application = view
                                    .app();
                                let undo_db = app.undo_db.borrow();
                                undo_db.event(action);
                            }),
                        )
                    } else {
                        menu.add_button("make tangent")
                    };
                    menu.popup(event.time());
                    return Inhibit(true);
                } else if let Some(((contour_index, _), _curve)) = on_curve_query {
                    crate::utils::menu::Menu::new()
                        .add_button_cb(
                            "reverse",
                            clone!(@strong view => move |_| {
                                let state = view.state().borrow();
                                let mut action = state
                                    .reverse_contour(&state.glyph.borrow().contours[contour_index], contour_index);
                                (action.redo)();
                                let app: &Application = view.app();
                                let undo_db = app.undo_db.borrow();
                                undo_db.event(action);
                            }),
                        )
                        .popup(event.time());
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
        view: Editor,
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
                .transformation
                .move_camera_by_delta(ViewPoint(<_ as Into<Point>>::into((5.0 * dx, 5.0 * dy))));
            viewport.queue_draw();
            return Inhibit(true);
        } else if event.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            if let Mode::DragGuideline(idx) = self.mode.get() {
                // [ref:needs_user_doc]
                /* rotate guideline that is currently being dragged */
                let (_dx, dy) = event.delta();
                let state = view.state().borrow();
                if let Some(g) = state.glyph.borrow().guidelines().get(idx) {
                    let (x, y) = (g.x(), g.y());
                    let UnitPoint(u_p) =
                        viewport.view_to_unit_point(ViewPoint(event.position().into()));
                    let mut delta = u_p - (x, y).into();
                    delta.y *= -1.0;
                    let mut m = Matrix::identity();
                    m.translate(delta.x, delta.y);
                    state.transform_guideline(idx, Matrix::identity(), 1.5 * dy);
                    let mut m = Matrix::identity();
                    m.translate(-delta.x, -delta.y);
                    return Inhibit(true);
                };
            }
        } else {
            /* Grow/shrink control points when scrolling above an on-curve point.*/
            // [ref:needs_user_doc]
            // [ref:needs_unit_test]
            let (dx, dy) = event.delta();
            if !((dx != 0.0) ^ (dy != 0.0)) {
                return Inhibit(true);
            }
            let norm = if dx != 0.0 { dx } else { dy };
            let is_corner = {
                let scale: f64 = viewport
                    .transformation
                    .property::<f64>(Transformation::SCALE);
                let ppu: f64 = viewport
                    .transformation
                    .property::<f64>(Transformation::PIXELS_PER_UNIT);
                let state = view.state().borrow();
                let UnitPoint(position) =
                    viewport.view_to_unit_point(ViewPoint(event.position().into()));
                let pts = state
                    .kd_tree
                    .borrow()
                    .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
                let is_corner = if pts.is_empty() {
                    None
                } else {
                    let glyph = state.glyph.borrow();

                    pts.iter().find_map(|p| {
                        let contour = &glyph.contours[p.contour_index];
                        let curves = contour.curves();
                        let ret = curves[p.curve_index]
                            .points()
                            .iter()
                            .find(|cp| cp.uuid == p.uuid && cp.continuity.is_some())
                            .map(|cp| {
                                (
                                    cp.position,
                                    contour.get_control_point(
                                        cp.glyph_index(p.contour_index, p.curve_index),
                                    ),
                                )
                            });
                        ret
                    })
                };

                is_corner
            };
            if let Some((pos, (Some((neighbor, npos)), _)))
            | Some((pos, (None, Some((neighbor, npos))))) = is_corner
            {
                let delta_vector = (npos - pos).unit() * norm;
                let state = view.state().borrow();
                let mut m = Matrix::identity();
                m.translate(delta_vector.x, delta_vector.y);
                let mut action = state.transform_points(&[neighbor], m);
                action.compress = true;
                (action.redo)();
                state.add_undo_action(action);
            }
        }
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        view: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let warp_cursor = viewport.property::<bool>(Canvas::WARP_CURSOR);
        let state = view.state().borrow();
        let UnitPoint(position) = viewport.view_to_unit_point(ViewPoint(event.position().into()));
        if !self.instance().property::<bool>(PanningTool::ACTIVE) {
            let glyph = state.glyph.borrow();
            let pts = state
                .kd_tree
                .borrow()
                .query_point(position, (10.0 / (scale * ppu)).ceil() as i64);
            if pts.is_empty() {
                view.hovering.set(None);
                viewport.queue_draw();
            }
            if let Some(((i, j), _curve)) = glyph.on_curve_query(position, &pts) {
                viewport.set_cursor("grab");
                view.hovering.set(Some((i, j)));
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
                // [ref:needs_user_doc]
                match Lock::from_bits(view.property(Editor::LOCK)) {
                    Some(Lock::X) => {
                        delta.y = 0.0;
                    }
                    Some(Lock::Y) => {
                        delta.x = 0.0;
                    }
                    Some(Lock::LOCAL) => {
                        // [ref:FIXME] ugly, wobbly but mostly works
                        let selection = state.get_selection();
                        if !selection.is_empty() {
                            let UnitPoint(upos) = viewport
                                .view_to_unit_point(ViewPoint(Point::from(event.position())));
                            let lock_delta = {
                                let glyph = state.glyph.borrow();
                                let curves = glyph.contours[selection[0].contour_index].curves();
                                let curv = &curves[selection[0].curve_index];
                                let degree = curv.degree().unwrap();
                                let points = curv.points();
                                let p1 =
                                    &points.iter().find(|p| p.uuid == selection[0].uuid).unwrap();
                                let p2 = match degree {
                                    0 => unimplemented!(),
                                    1 => {
                                        if points[0].uuid == p1.uuid {
                                            &points[1]
                                        } else {
                                            &points[0]
                                        }
                                    }
                                    2 => {
                                        if points[0].uuid == p1.uuid {
                                            &points[1]
                                        } else if points[1].uuid == p1.uuid {
                                            &points[2]
                                        } else {
                                            &points[1]
                                        }
                                    }
                                    3 => {
                                        if points[0].uuid == p1.uuid {
                                            &points[1]
                                        } else if points[1].uuid == p1.uuid {
                                            &points[0]
                                        } else if points[2].uuid == p1.uuid {
                                            &points[3]
                                        } else {
                                            &points[2]
                                        }
                                    }
                                    _ => unimplemented!(),
                                };

                                let alpha = upos - p1.position;
                                let beta = p2.position - p1.position;
                                let bunit = beta / beta.norm();
                                let scalar = alpha.dot(beta) / beta.norm();
                                (scalar * bunit + p1.position) - p1.position
                            };
                            let new_mouse =
                                viewport.unit_to_view_point(UnitPoint(upos + lock_delta));
                            let delta = new_mouse.0 - ViewPoint(Point::from(event.position())).0;
                            view.app()
                                .warp_cursor(event.device(), (delta.x as i32, delta.y as i32))
                                .unwrap();
                            viewport.set_mouse(new_mouse);
                            let mut m = Matrix::identity();
                            m.translate(lock_delta.x, lock_delta.y);
                            state.transform_selection(m, true);
                            return Inhibit(true);
                        }
                    }
                    _ => {}
                }
                let mut m = Matrix::identity();
                if let Some(snap_delta) = Snap::from_bits(view.property(Editor::SNAP))
                    .filter(|s| !s.is_empty())
                    .and_then(|snap| {
                        snap_to_closest_anchor(
                            &view,
                            UnitPoint(delta),
                            viewport.view_to_unit_point(mouse),
                            snap,
                        )
                    })
                {
                    let UnitPoint(position) = viewport.view_to_unit_point(mouse);
                    let new_mouse = viewport.unit_to_view_point(UnitPoint(position + snap_delta));
                    let delta = new_mouse.0 - mouse.0;
                    view.app()
                        .warp_cursor(event.device(), (delta.x as i32, delta.y as i32))
                        .unwrap();
                    viewport.set_mouse(new_mouse);
                    m.translate(snap_delta.x, snap_delta.y);
                } else {
                    m.translate(delta.x, delta.y);
                }
                state.transform_selection(m, true);
            }
            Mode::DragGuideline(idx) => {
                let mouse: ViewPoint = viewport.get_mouse();
                let mut delta =
                    (<_ as Into<Point>>::into(event.position()) - mouse.0) / (scale * ppu);
                delta.y *= -1.0;
                match Lock::from_bits(view.property(Editor::LOCK)) {
                    Some(Lock::X) => {
                        delta.y = 0.0;
                    }
                    Some(Lock::Y) => {
                        delta.x = 0.0;
                    }
                    _ => {}
                }
                let mut m = gtk::cairo::Matrix::identity();
                m.translate(delta.x, delta.y);
                state.transform_guideline(idx, m, 0.0);
            }
            Mode::Pan => {
                if warp_cursor {
                    let (width, height) = (
                        f64::from(viewport.allocated_width()),
                        f64::from(viewport.allocated_height()),
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
                    .transformation
                    .move_camera_by_delta(ViewPoint(delta));
            }
            Mode::Select => {
                if let Some(pixbuf) = match event.state().into() {
                    SelectionModifier::Add => self.cursor_plus.get().unwrap().clone(),
                    SelectionModifier::Remove => self.cursor_minus.get().unwrap().clone(),
                    SelectionModifier::Replace => self.cursor.get().unwrap().clone(),
                } {
                    view.viewport.set_cursor_from_pixbuf(pixbuf);
                }
                let event_position = event.position();
                let bottom_right = viewport.view_to_unit_point(ViewPoint(event_position.into()));
                self.selection_bottom_right.set(bottom_right);
                return Inhibit(true);
            }
            Mode::ResizeDimensions { previous_value: _ } => {
                let mouse: ViewPoint = viewport.get_mouse();

                if viewport
                    .view_to_unit_point(ViewPoint(event.position().into()))
                    .0
                    .x
                    > 0.0
                {
                    let delta =
                        (<_ as Into<Point>>::into(event.position()) - mouse.0) / (scale * ppu);
                    let width = {
                        let glyph = state.glyph.borrow();
                        glyph.width().unwrap_or(0.0)
                    };
                    if width + delta.x >= 0.0 {
                        let glyph = state.glyph.borrow();
                        glyph.width.set(Some(width + delta.x));
                    }
                } else {
                    let glyph = state.glyph.borrow();
                    glyph.width.set(Some(0.0));
                }
            }
        }
        Inhibit(true)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &Editor) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("selection box"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: ContextRef| {
                    PanningTool::draw_select_box(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(PanningTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.viewport.add_post_layer(layer);
        self.setup_actions(view);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &Editor) {
        obj.set_property::<bool>(PanningTool::ACTIVE, true);
        self.set_default_cursor(view);
        self.parent_on_activate(obj, view);
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &Editor) {
        obj.set_property::<bool>(PanningTool::ACTIVE, false);
        self.set_default_cursor(view);
        self.parent_on_deactivate(obj, view);
    }
}

impl PanningToolInner {
    fn set_default_cursor(&self, view: &Editor) {
        if let Some(pixbuf) = self.cursor.get().unwrap().clone() {
            view.viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.viewport.set_cursor("default");
        }
    }

    fn setup_actions(&self, view: &Editor) {
        let obj = self.instance();
        let move_action = gio::SimpleAction::new(PanningTool::MOVE_ACTION, None);
        move_action.connect_activate(glib::clone!(@weak view, @weak obj => move |_, _| {
            let state = view.state().borrow();
            match obj.imp().mode.get() {
                Mode::None if !state.get_selection_set().is_empty() => {
                    obj.set_property::<bool>(PanningTool::ACTIVE, true);
                    obj.imp().mode.set(Mode::Drag);
                    view.set_property(Editor::MODIFYING_IN_PROCESS, true);
                    view.viewport.set_cursor("grab");
                }
                m @ Mode::Drag | m @ Mode::DragGuideline(_) => {
                    obj.set_property::<bool>(PanningTool::ACTIVE, false);
                    obj.imp().mode.set(Mode::Drag);
                    if matches!(m, Mode::Drag) {
                        view.set_property(Editor::MODIFYING_IN_PROCESS, true);
                    }
                    view.viewport.set_cursor("grab");
                }
                _ => {}
            }
        }));
        view.action_group.add_action(&move_action);
        let scale_action = gio::SimpleAction::new(PanningTool::SCALE_ACTION, None);
        scale_action.connect_activate(|_, _| {});
        view.action_group.add_action(&scale_action);
        let rotate_action = gio::SimpleAction::new(PanningTool::ROTATE_ACTION, None);
        rotate_action.connect_activate(|_, _| {});
        view.action_group.add_action(&rotate_action);
        let mut sh = view.shortcuts.borrow_mut();
        sh.push(ShortcutAction::new(
            "move".into(),
            Shortcut::empty().shift().char('G'),
            Box::new(|group| {
                group.activate_action(PanningTool::MOVE_ACTION, None);
                true
            }),
            None,
        ));
        sh.push(ShortcutAction::new(
            "scale".into(),
            Shortcut::empty().shift().char('S'),
            Box::new(|group| {
                group.activate_action(PanningTool::SCALE_ACTION, None);
                true
            }),
            None,
        ));
        sh.push(ShortcutAction::new(
            "rotate".into(),
            Shortcut::empty().shift().char('R'),
            Box::new(|group| {
                group.activate_action(PanningTool::ROTATE_ACTION, None);
                true
            }),
            None,
        ));
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
    pub const MOVE_ACTION: &str = "move.selection";
    pub const SCALE_ACTION: &str = "scale.selection";
    pub const ROTATE_ACTION: &str = "rotate.selection";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn draw_select_box(viewport: &Canvas, mut cr: ContextRef, obj: Editor) -> Inhibit {
        let state = obj.state().borrow();
        let t = state.tools[&Self::static_type()]
            .clone()
            .downcast::<Self>()
            .unwrap();
        if !t.imp().active.get()
            || !matches!(
                t.imp().mode.get(),
                Mode::Select | Mode::ResizeDimensions { .. } | Mode::Drag
            )
        {
            return Inhibit(false);
        }
        let resize = matches!(t.imp().mode.get(), Mode::ResizeDimensions { .. });
        let select = matches!(t.imp().mode.get(), Mode::Select);
        let drag = matches!(t.imp().mode.get(), Mode::Drag);
        let empty = t.imp().is_selection_empty.get();
        if empty && !select && !drag && !resize {
            return Inhibit(false);
        }

        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);

        /* Calculate how much we need to multiply a pixel value to scale it back after performing
         * the matrix transformation */
        let f = 1.0 / (scale * ppu);

        let line_width = if select { 1.0 } else { 0.5 } * f;

        let matrix = viewport.transformation.matrix();

        let mut cr1 = cr.push();

        cr1.set_line_width(line_width);
        cr1.transform(matrix);

        if resize {
            let units_per_em = viewport
                .transformation
                .property::<f64>(Transformation::UNITS_PER_EM);
            let glyph_width = state.glyph.borrow().width().unwrap_or(0.0);

            cr1.set_source_color(Color::BLACK);
            cr1.set_line_width(1.0);
            cr1.rectangle(0.0, 0.0, glyph_width, units_per_em);
            cr1.stroke().unwrap();
            cr1.set_line_width(2.0);
            cr1.move_to(glyph_width, 0.0);
            cr1.line_to(glyph_width, units_per_em);
            cr1.stroke().unwrap();

            drop(cr1);
            let cr2 = cr.push();

            let extents = cr2.text_extents("Cancel").unwrap();
            let ViewPoint(mouse) = viewport.get_mouse();
            let scale_factor = viewport.scale_factor();
            // [ref:FIXME] remove unwraps
            // [ref:FIXME] don't allocate a pixbuf in every draw call
            /*
            let esc = crate::resources::icons::ESC_BUTTON
                .to_pixbuf()
                .unwrap()
                .scale_simple(64, 64, gtk::gdk_pixbuf::InterpType::Bilinear)
                .unwrap();
            */
            let rmb = crate::resources::icons::RIGHT_MOUSE_BUTTON
                .to_pixbuf()
                .unwrap()
                .scale_simple(64, 64, gtk::gdk_pixbuf::InterpType::Bilinear)
                .unwrap();
            let mut x = (f64::from(rmb.width())).mul_add(0.1, mouse.x);
            let mut y = mouse.y;
            let mut row_height = 0.0;
            let (h, w) = (
                (f64::from(rmb.height())) / (f64::from(scale_factor)),
                (f64::from(rmb.width())) / (f64::from(scale_factor)),
            );
            if h > row_height {
                row_height = h;
            }
            cr2.set_source_surface(
                &rmb.create_surface(scale_factor, viewport.window().as_ref())
                    .unwrap(),
                x + 0.5,
                y + 0.5,
            )
            .unwrap();
            cr2.paint().unwrap();
            x += w;
            cr2.set_source_color(Color::BLACK);
            cr2.move_to(
                (f64::from(rmb.width())).mul_add(0.1, x) + 0.5,
                extents
                    .height
                    .mul_add(0.5, row_height.mul_add(0.5, mouse.y))
                    + 0.5,
            );
            cr2.show_text("Cancel").unwrap();

            y += row_height * 1.1;

            let i = crate::resources::icons::LEFT_MOUSE_BUTTON
                .to_pixbuf()
                .unwrap()
                .scale_simple(64, 64, gtk::gdk_pixbuf::InterpType::Bilinear)
                .unwrap();
            let (h, w) = (
                (f64::from(i.height())) / (f64::from(scale_factor)),
                (f64::from(i.width())) / (f64::from(scale_factor)),
            );
            cr2.set_source_surface(
                &i.create_surface(scale_factor, viewport.window().as_ref())
                    .unwrap(),
                x - w + 0.5,
                y + 0.5,
            )
            .unwrap();
            cr2.paint().unwrap();

            cr2.set_source_color(Color::BLACK);
            cr2.move_to(
                (f64::from(rmb.width())).mul_add(0.1, x) + 0.5,
                extents.height.mul_add(0.5, h.mul_add(0.5, y)) + 0.5,
            );
            cr2.show_text("Apply").unwrap();

            return Inhibit(true);
        }

        if drag {
            let cr2 = cr1.push();
            cr2.set_source_color(Color::BLACK);
            cr2.set_line_width(3.0);
            cr2.set_dash(&[2.0 * f, 2.0 * f], 0.5 * f);
            let selection = state.get_selection();
            let contours = &state.glyph.borrow().contours;
            let points = selection
                .iter()
                .map(|g| contours[g.contour_index].get_point(*g).unwrap())
                .collect::<Vec<Point>>();
            let kd = state.kd_tree.borrow();
            let mut x_set =
                HashSet::<i64>::from_iter(points.iter().map(|pos| pos.x.round() as i64));
            let mut y_set =
                HashSet::<i64>::from_iter(points.iter().map(|pos| pos.y.round() as i64));
            for pos in points {
                let query_x = kd.query_on_axis(Coordinate::X, pos, 1.0);
                let query_y = kd.query_on_axis(Coordinate::Y, pos, 1.0);

                for (p, ax) in query_x
                    .into_iter()
                    .map(|p| (p, Coordinate::X))
                    .chain(query_y.into_iter().map(|p| (p, Coordinate::Y)))
                {
                    let p = contours[p.contour_index].get_point(p).unwrap();

                    match ax {
                        Coordinate::X => {
                            if x_set.contains(&(p.x.round() as i64)) {
                                continue;
                            }
                            x_set.insert(p.x.round() as i64);
                        }
                        Coordinate::Y => {
                            if y_set.contains(&(p.y.round() as i64)) {
                                continue;
                            }
                            y_set.insert(p.y.round() as i64);
                        }
                    }

                    cr2.move_to(pos.x, pos.y);
                    cr2.line_to(p.x, p.y);
                    cr2.stroke().unwrap();
                }
            }
            return Inhibit(true);
        }

        let UnitPoint(upper_left) = t.imp().selection_upper_left.get();
        let UnitPoint(bottom_right) = t.imp().selection_bottom_right.get();
        let (width, height) = ((bottom_right - upper_left).x, (bottom_right - upper_left).y);
        if width == 0.0 || height == 0.0 {
            return Inhibit(false);
        }

        cr1.set_source_rgba(0.0, 0.0, 0.0, 0.9);
        cr1.rectangle(upper_left.x, upper_left.y, width, height);
        if select {
            cr1.stroke_preserve().unwrap();
            // turqoise, #278cac
            cr1.set_source_rgba(39.0 / 255.0, 140.0 / 255.0, 172.0 / 255.0, 0.1);
            cr1.fill().unwrap();
        } else {
            cr1.stroke().unwrap();
        }
        drop(cr1);

        if !select {
            let rectangle_dim = 5.0 * f;

            let cr2 = cr.push();
            cr2.set_line_width(line_width);
            cr2.transform(matrix);
            for p in [
                upper_left,
                bottom_right,
                upper_left + (width, 0.0).into(),
                upper_left + (0.0, height).into(),
            ] {
                cr2.set_source_rgba(0.0, 0.0, 0.0, 0.9);
                cr2.rectangle(
                    p.x - rectangle_dim / 2.0,
                    p.y - rectangle_dim / 2.0,
                    rectangle_dim,
                    rectangle_dim,
                );
                cr2.stroke_preserve().unwrap();
                cr2.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr2.fill().unwrap();
            }
        }

        Inhibit(true)
    }

    pub fn move_action(&self, view: &Editor, direction: MoveDirection) {
        let mut m = Matrix::identity();
        let step = match Precision::from_bits(view.property(Editor::PRECISION)) {
            Some(v) if v == Precision::_1 => 1.0,
            Some(v) if v == Precision::_05 => 0.5,
            Some(v) if v == Precision::_01 => 0.1,
            _ => 5.0,
        };

        match direction {
            MoveDirection::Up => {
                m.translate(0.0, step);
            }
            MoveDirection::Down => {
                m.translate(0.0, -step);
            }
            MoveDirection::Right => {
                m.translate(step, 0.0);
            }
            MoveDirection::Left => {
                m.translate(-step, 0.0);
            }
        }
        view.state().borrow().transform_selection(m, true);
        view.queue_draw();
    }

    pub fn selection_action(&self, view: &Editor, action: SelectionAction) {
        match action {
            SelectionAction::All => {
                let pts = view.state().borrow().kd_tree.borrow().all();
                view.set_selection(&pts, SelectionModifier::Replace);
            }
            SelectionAction::None => {
                view.set_selection(&[], SelectionModifier::Replace);
            }
        }
        view.queue_draw();
    }
}

fn snap_to_closest_anchor(
    obj: &Editor,
    UnitPoint(delta): UnitPoint,
    UnitPoint(mouse): UnitPoint,
    snap: Snap,
) -> Option<Point> {
    let point = mouse + delta;
    type Distance = f64;

    if snap == Snap::EMPTY {
        return None;
    }

    let mut candidates: Vec<(Point, Distance)> = vec![];
    if snap.intersects(Snap::ANGLE) {
        //todo
    }
    if snap.intersects(Snap::GRID) {
        //todo
    }

    let (guidelines, metrics) = (
        snap.intersects(Snap::GUIDELINES),
        snap.intersects(Snap::METRICS),
    );
    if guidelines || metrics {
        let state = obj.state().borrow();
        for g in state
            .glyph
            .borrow()
            .guidelines()
            .iter()
            .filter(|_| guidelines)
            .chain(
                obj.project
                    .get()
                    .unwrap()
                    .guidelines
                    .borrow()
                    .iter()
                    .filter(|_| guidelines),
            )
            .chain(
                obj.project
                    .get()
                    .unwrap()
                    .metric_guidelines
                    .borrow()
                    .iter()
                    .filter(|_| metrics),
            )
        {
            let d = g.distance_from_point(mouse);
            if d <= 25.0 && g.distance_from_point(point) < d {
                candidates.push((g.project_point(mouse) - mouse, d));
            }
        }
    }
    if snap.intersects(Snap::METRICS) {
        //todo
    }
    candidates.sort_by(|(_, a), (_, b)| a.total_cmp(b));
    candidates.get(0).map(|(p, _)| *p)
}

fn change_continuity(
    view: &Editor,
    index: GlyphPointIndex,
    prev_val: Continuity,
    new_val: Continuity,
) -> Action {
    let contour: Contour = {
        let c = view.state().borrow().glyph.borrow().contours[index.contour_index].clone();
        c
    };
    Action {
        stamp: EventStamp {
            t: std::any::TypeId::of::<Editor>(),
            property: Contour::static_type().name(),
            id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[index.contour_index]).into() },
        },
        compress: false,
        redo: Box::new(clone!(@weak contour, @weak view => move || {
            contour.change_continuity(index, new_val);
        })),
        undo: Box::new(clone!(@weak contour, @weak view => move || {
            contour.change_continuity(index, prev_val);
        })),
    }
}
