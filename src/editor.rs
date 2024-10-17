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

use gtk::cairo::Matrix;
use indexmap::IndexMap;
use once_cell::unsync::OnceCell;
use std::collections::HashSet;

use crate::app::settings::types::ShowMinimap;
use crate::glyphs::{Contour, Glyph, GlyphDrawingOptions, GlyphPointIndex, Guideline};
use crate::prelude::*;
use crate::views::{
    canvas::{Layer, LayerBuilder},
    overlay::Child,
};

mod layers;
mod menu;
mod settings;
mod shortcuts;
mod state;
mod tools;
pub use settings::EditorSettings;
pub use state::State;

use tools::{PanningTool, SelectionModifier, Tool, ToolImpl};

type StatusBarMessage = u32;

#[derive(Debug, Default)]
pub struct EditorInner {
    app: OnceCell<Application>,
    project: OnceCell<Project>,
    glyph: OnceCell<Rc<RefCell<Glyph>>>,
    state: OnceCell<Rc<RefCell<State>>>,
    viewport: Canvas,
    statusbar_context_id: Cell<Option<u32>>,
    overlay: Overlay,
    hovering: Cell<Option<(usize, usize)>>,
    pub toolbar_box: gtk::Box,
    units_per_em: Cell<f64>,
    descender: Cell<f64>,
    x_height: Cell<f64>,
    cap_height: Cell<f64>,
    ascender: Cell<f64>,
    lock_guidelines: Cell<bool>,
    show_glyph_guidelines: Cell<bool>,
    show_project_guidelines: Cell<bool>,
    show_metrics_guidelines: Cell<bool>,
    modifying_in_process: Cell<bool>,
    show_minimap: Cell<ShowMinimap>,
    settings: OnceCell<Settings>,
    menubar: gtk::MenuBar,
    preview: Cell<Option<StatusBarMessage>>,
    ctrl: OnceCell<gtk::EventControllerKey>,
    action_group: gio::SimpleActionGroup,
    lock: Cell<(Option<StatusBarMessage>, tools::constraints::Lock)>,
    snap: Cell<(Option<StatusBarMessage>, tools::constraints::Snap)>,
    precision: Cell<(Option<StatusBarMessage>, tools::constraints::Precision)>,
    shortcuts: shortcuts::Shortcuts,
    shortcut_status: gtk::Box,
}

#[glib::object_subclass]
impl ObjectSubclass for EditorInner {
    const NAME: &'static str = "Editor";
    type Type = Editor;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for EditorInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.lock_guidelines.set(false);
        self.setup_shortcuts(obj);
        self.shortcuts.rebuild();
        self.show_glyph_guidelines.set(true);
        self.show_project_guidelines.set(true);
        self.show_metrics_guidelines.set(true);
        self.statusbar_context_id.set(None);
        self.viewport.set_mouse(ViewPoint((0.0, 0.0).into()));

        self.viewport.connect_scroll_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let retval = Tool::on_scroll_event(obj, viewport, event);
                if retval == Inhibit(true) {
                    viewport.queue_draw();
                }
                retval
            }),
        );

        self.viewport.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let prev_mouse = viewport.get_mouse();
                let retval = Tool::on_button_press_event(obj, viewport, event);
                if retval == Inhibit(true) {
                    viewport.queue_draw();
                }
                if prev_mouse == viewport.get_mouse() {
                    // Tool didn't update mouse position, so let's do it now. This is to prevent
                    // overwriting any modifications to the mouse done by tools.
                    viewport.set_mouse(ViewPoint(event.position().into()));
                }
                retval
            }),
        );

        self.viewport.connect_button_release_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let prev_mouse = viewport.get_mouse();
                let retval = Tool::on_button_release_event(obj, viewport, event);
                if retval == Inhibit(true) {
                    viewport.queue_draw();
                }
                if prev_mouse == viewport.get_mouse() {
                    // Tool didn't update mouse position, so let's do it now. This is to prevent
                    // overwriting any modifications to the mouse done by tools.
                    viewport.set_mouse(ViewPoint(event.position().into()));
                }
                retval
            }),
        );

        self.viewport.connect_motion_notify_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let prev_mouse = viewport.get_mouse();
                let retval = Tool::on_motion_notify_event(obj, viewport, event);
                if retval == Inhibit(true) {
                    viewport.queue_draw();
                }
                if prev_mouse == viewport.get_mouse() {
                    // Tool didn't update mouse position, so let's do it now. This is to prevent
                    // overwriting any modifications to the mouse done by tools.
                    viewport.set_mouse(ViewPoint(event.position().into()));
                }
                retval
            }),
        );

        self.viewport.add_layer(
            LayerBuilder::new()
                .set_name(Some("glyph"))
                .set_active(true)
                .set_hidden(false)
                .set_callback(Some(Box::new(clone!(@weak obj => @default-return Inhibit(false), move |viewport: &Canvas, mut cr: ContextRef<'_, '_>| {
                    layers::draw_glyph_layer(viewport, cr.push(), obj)
                }))))
                .build(),
        );
        self.viewport.add_pre_layer(
            LayerBuilder::new()
                .set_name(Some("guidelines"))
                .set_active(true)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak obj => @default-return Inhibit(false), move |viewport: &Canvas, mut cr: ContextRef<'_, '_>| {
                    layers::draw_guidelines(viewport, cr.push(), obj)
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
        self.overlay.set_child(&self.viewport);
        self.overlay
            .add_overlay(Child::new(self.toolbar_box.clone()));
        self.overlay
            .add_overlay(Child::new(self.create_layer_widget()).expanded(false));
        obj.add(&self.overlay);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        Editor::TITLE,
                        Editor::TITLE,
                        Editor::TITLE,
                        Some("edit glyph"),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::CLOSEABLE,
                        Editor::CLOSEABLE,
                        Editor::CLOSEABLE,
                        true,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::PREVIEW,
                        Editor::PREVIEW,
                        Editor::PREVIEW,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::IS_MENU_VISIBLE,
                        Editor::IS_MENU_VISIBLE,
                        Editor::IS_MENU_VISIBLE,
                        true,
                        glib::ParamFlags::READABLE,
                    ),
                    def_param!(f64 Editor::UNITS_PER_EM, 1.0, ufo::constants::UNITS_PER_EM),
                    def_param!(f64 Editor::X_HEIGHT, 1.0, ufo::constants::X_HEIGHT),
                    def_param!(f64 Editor::ASCENDER, f64::MIN, ufo::constants::ASCENDER),
                    def_param!(f64 Editor::DESCENDER, f64::MIN, ufo::constants::DESCENDER),
                    def_param!(f64 Editor::CAP_HEIGHT, f64::MIN, ufo::constants::CAP_HEIGHT),
                    glib::ParamSpecBoolean::new(
                        Editor::LOCK_GUIDELINES,
                        Editor::LOCK_GUIDELINES,
                        Editor::LOCK_GUIDELINES,
                        false,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::SHOW_GLYPH_GUIDELINES,
                        Editor::SHOW_GLYPH_GUIDELINES,
                        Editor::SHOW_GLYPH_GUIDELINES,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::SHOW_PROJECT_GUIDELINES,
                        Editor::SHOW_PROJECT_GUIDELINES,
                        Editor::SHOW_PROJECT_GUIDELINES,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::SHOW_METRICS_GUIDELINES,
                        Editor::SHOW_METRICS_GUIDELINES,
                        Editor::SHOW_METRICS_GUIDELINES,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Editor::MODIFYING_IN_PROCESS,
                        Editor::MODIFYING_IN_PROCESS,
                        Editor::MODIFYING_IN_PROCESS,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecEnum::new(
                        Editor::SHOW_MINIMAP,
                        Editor::SHOW_MINIMAP,
                        Editor::SHOW_MINIMAP,
                        ShowMinimap::static_type(),
                        ShowMinimap::WhenManipulating as i32,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecObject::new(
                        Editor::ACTIVE_TOOL,
                        Editor::ACTIVE_TOOL,
                        Editor::ACTIVE_TOOL,
                        ToolImpl::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Editor::PANNING_TOOL,
                        Editor::PANNING_TOOL,
                        Editor::PANNING_TOOL,
                        PanningTool::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Workspace::MENUBAR,
                        Workspace::MENUBAR,
                        Workspace::MENUBAR,
                        gtk::MenuBar::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecUInt::new(
                        Editor::LOCK,
                        Editor::LOCK,
                        "Lock transformation movement to specific axes.",
                        0,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecUInt::new(
                        Editor::SNAP,
                        Editor::SNAP,
                        "Snap transformation movement to specific references.",
                        0,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecUInt::new(
                        Editor::PRECISION,
                        Editor::PRECISION,
                        "Increase accuracy of transformations.",
                        0,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Editor::TITLE => {
                if let Some(name) = obj
                    .state
                    .get()
                    .map(|s| s.borrow().glyph.borrow().name_markup())
                {
                    format!("edit <i>{}</i>", name).to_value()
                } else {
                    "edit glyph".to_value()
                }
            }
            Editor::CLOSEABLE => true.to_value(),
            Editor::PREVIEW => self.preview.get().is_some().to_value(),
            Editor::IS_MENU_VISIBLE => true.to_value(),
            Editor::UNITS_PER_EM => self.units_per_em.get().to_value(),
            Editor::X_HEIGHT => self.x_height.get().to_value(),
            Editor::ASCENDER => self.ascender.get().to_value(),
            Editor::DESCENDER => self.descender.get().to_value(),
            Editor::CAP_HEIGHT => self.cap_height.get().to_value(),
            Editor::LOCK_GUIDELINES => self.lock_guidelines.get().to_value(),
            Editor::SHOW_GLYPH_GUIDELINES => self.show_glyph_guidelines.get().to_value(),
            Editor::SHOW_PROJECT_GUIDELINES => self.show_project_guidelines.get().to_value(),
            Editor::SHOW_METRICS_GUIDELINES => self.show_metrics_guidelines.get().to_value(),
            Editor::MODIFYING_IN_PROCESS => self.modifying_in_process.get().to_value(),
            Editor::SHOW_MINIMAP => self.show_minimap.get().to_value(),
            Editor::ACTIVE_TOOL => {
                let state = self.state.get().unwrap().borrow();
                let active_tool = state.active_tool;
                state.tools.get(&active_tool).cloned().to_value()
            }
            Editor::PANNING_TOOL => {
                let state = self.state.get().unwrap().borrow();
                let panning_tool = state.panning_tool;
                state.tools.get(&panning_tool).cloned().to_value()
            }
            Editor::MENUBAR => Some(self.menubar.clone()).to_value(),
            Editor::LOCK => self.lock.get().1.bits().to_value(),
            Editor::SNAP => self.snap.get().1.bits().to_value(),
            Editor::PRECISION => self.precision.get().1.bits().to_value(),
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
        self.instance().queue_draw();
        match pspec.name() {
            Editor::UNITS_PER_EM => {
                self.units_per_em.set(value.get().unwrap());
            }
            Editor::X_HEIGHT => {
                self.x_height.set(value.get().unwrap());
            }
            Editor::ASCENDER => {
                self.ascender.set(value.get().unwrap());
            }
            Editor::DESCENDER => {
                self.descender.set(value.get().unwrap());
            }
            Editor::CAP_HEIGHT => {
                self.cap_height.set(value.get().unwrap());
            }
            Editor::LOCK_GUIDELINES => {
                self.lock_guidelines.set(value.get().unwrap());
            }
            Editor::SHOW_GLYPH_GUIDELINES => {
                self.show_glyph_guidelines.set(value.get().unwrap());
            }
            Editor::SHOW_PROJECT_GUIDELINES => {
                self.show_project_guidelines.set(value.get().unwrap());
            }
            Editor::SHOW_METRICS_GUIDELINES => {
                self.show_metrics_guidelines.set(value.get().unwrap());
            }
            Editor::MODIFYING_IN_PROCESS => {
                self.modifying_in_process.set(value.get().unwrap());
            }
            Editor::SHOW_MINIMAP => {
                self.show_minimap.set(value.get().unwrap());
            }
            Editor::PREVIEW => {
                let v: bool = value.get().unwrap();
                if let Some(mid) = self.preview.get() {
                    if v {
                        return;
                    }
                    self.pop_statusbar_message(Some(mid));
                    self.preview.set(None);
                } else {
                    if !v {
                        return;
                    }
                    self.preview.set(self.new_statusbar_message("Preview."));
                }
                self.viewport.queue_draw();
            }
            Editor::LOCK => {
                if let Some(v) = value
                    .get::<u32>()
                    .ok()
                    .and_then(tools::constraints::Lock::from_bits)
                {
                    let (msg, _) = self.lock.get();
                    if msg.is_some() {
                        self.pop_statusbar_message(msg);
                    }
                    let new_msg = if v.is_empty() {
                        None
                    } else {
                        self.new_statusbar_message(v.as_str())
                    };
                    self.lock.set((new_msg, v));
                    self.viewport.queue_draw();
                }
            }
            Editor::SNAP => {
                if let Some(v) = value
                    .get::<u32>()
                    .ok()
                    .and_then(tools::constraints::Snap::from_bits)
                {
                    let (msg, _) = self.snap.get();
                    if msg.is_some() {
                        self.pop_statusbar_message(msg);
                    }
                    let new_msg = if v.is_empty() {
                        None
                    } else {
                        self.new_statusbar_message(v.as_str())
                    };
                    self.snap.set((new_msg, v));
                    self.viewport.queue_draw();
                }
            }
            Editor::PRECISION => {
                if let Some(v) = value
                    .get::<u32>()
                    .ok()
                    .and_then(tools::constraints::Precision::from_bits)
                {
                    let (msg, _) = self.precision.get();
                    if msg.is_some() {
                        self.pop_statusbar_message(msg);
                    }
                    let new_msg = if v.is_empty() {
                        None
                    } else {
                        self.new_statusbar_message(v.as_str())
                    };
                    self.precision.set((new_msg, v));
                    self.viewport.queue_draw();
                }
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl WidgetImpl for EditorInner {}
impl ContainerImpl for EditorInner {}
impl BinImpl for EditorInner {}

impl EditorInner {
    pub fn app(&self) -> &Application {
        self.app.get().unwrap()
    }

    pub fn project(&self) -> &Project {
        self.project.get().unwrap()
    }

    pub fn glyph(&self) -> &Rc<RefCell<Glyph>> {
        self.glyph.get().unwrap()
    }

    pub fn app_settings(&self) -> &Settings {
        self.settings.get().unwrap()
    }

    fn new_statusbar_message(&self, msg: &str) -> Option<StatusBarMessage> {
        let statusbar = self.app().statusbar();
        if self.statusbar_context_id.get().is_none() {
            self.statusbar_context_id.set(Some(
                statusbar.context_id(&format!("Editor-{:?}", &self.glyph.get().unwrap())),
            ));
        }
        if let Some(cid) = self.statusbar_context_id.get().as_ref() {
            return Some(statusbar.push(*cid, msg));
        }
        None
    }

    fn pop_statusbar_message(&self, msg: Option<StatusBarMessage>) {
        let statusbar = self.app().statusbar();
        if let Some(cid) = self.statusbar_context_id.get().as_ref() {
            if let Some(mid) = msg {
                gtk::prelude::StatusbarExt::remove(&statusbar, *cid, mid);
            } else {
                statusbar.pop(*cid);
            }
        }
    }
}

glib::wrapper! {
    pub struct Editor(ObjectSubclass<EditorInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl_deref!(Editor, EditorInner);
impl_friendly_name!(Editor);

impl Editor {
    pub const CLOSEABLE: &'static str = Workspace::CLOSEABLE;
    pub const TITLE: &'static str = Workspace::TITLE;
    pub const IS_MENU_VISIBLE: &'static str = Workspace::IS_MENU_VISIBLE;
    pub const MENUBAR: &'static str = Workspace::MENUBAR;
    pub const PREVIEW: &'static str = "preview";
    inherit_property!(
        FontInfo,
        ASCENDER,
        CAP_HEIGHT,
        DESCENDER,
        UNITS_PER_EM,
        X_HEIGHT
    );
    pub const MODIFYING_IN_PROCESS: &'static str = "modifying-in-process";
    pub const ACTIVE_TOOL: &'static str = "active-tool";
    pub const PANNING_TOOL: &'static str = "panning-tool";
    inherit_property!(
        EditorSettings,
        SHOW_MINIMAP,
        LOCK_GUIDELINES,
        SHOW_GLYPH_GUIDELINES,
        SHOW_PROJECT_GUIDELINES,
        SHOW_METRICS_GUIDELINES
    );

    pub fn new(app: Application, glyph: Rc<RefCell<Glyph>>) -> Self {
        let ret: Self = glib::Object::new(&[]).unwrap();
        ret.glyph.set(glyph.clone()).unwrap();
        ret.app.set(app.clone()).unwrap();
        let project = app.runtime.project.borrow().clone();
        ret.connect_map(|self_| {
            let status = self_.app().statusbar().message_area().unwrap();
            status.pack_end(&self_.shortcut_status, false, false, 1);
        });
        ret.connect_unmap(|self_| {
            let status = self_.app().statusbar().message_area().unwrap();
            status.remove(&self_.shortcut_status);
        });
        {
            let property = Self::UNITS_PER_EM;
            ret.bind_property(property, &ret.viewport.transformation, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        ret.viewport.transformation.set_property(
            Transformation::CONTENT_WIDTH,
            glyph
                .borrow()
                .width()
                .unwrap_or_else(|| ret.property::<f64>(Self::UNITS_PER_EM)),
        );
        for property in [
            Self::ASCENDER,
            Self::CAP_HEIGHT,
            Self::DESCENDER,
            Self::UNITS_PER_EM,
            Self::X_HEIGHT,
        ] {
            project
                .fontinfo()
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        let settings = app.runtime.settings.clone();
        settings.register_obj(ret.viewport.upcast_ref(), CanvasSettings::new());
        settings.register_obj(ret.upcast_ref(), EditorSettings::new());
        settings
            .bind_property(Canvas::WARP_CURSOR, &ret.viewport, Canvas::WARP_CURSOR)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        for prop in [Settings::HANDLE_SIZE, Settings::LINE_WIDTH] {
            settings.connect_notify_local(
                Some(prop),
                clone!(@strong ret => move |_self, _| {
                    ret.viewport.queue_draw();
                }),
            );
        }
        ret.settings.set(settings).unwrap();
        for prop in [
            Canvas::SHOW_GRID,
            Canvas::SHOW_GUIDELINES,
            Canvas::SHOW_HANDLES,
            Canvas::INNER_FILL,
            Canvas::SHOW_TOTAL_AREA,
        ] {
            let prop_action = gio::PropertyAction::new(prop, &ret.viewport, prop);
            ret.action_group.add_action(&prop_action);
        }
        {
            let prop_action = gio::PropertyAction::new(Self::PREVIEW_ACTION, &ret, Self::PREVIEW);
            ret.action_group.add_action(&prop_action);
        }
        for (zoom_action, tool_func) in [
            (
                Self::ZOOM_IN_ACTION,
                &Transformation::zoom_in as &dyn Fn(&Transformation) -> bool,
            ),
            (Self::ZOOM_OUT_ACTION, &Transformation::zoom_out),
        ] {
            let action = gio::SimpleAction::new(zoom_action, None);
            action.connect_activate(glib::clone!(@weak ret as obj => move |_, _| {
                let t = &obj.viewport.transformation;
                tool_func(t);
            }));
            ret.action_group.add_action(&action);
        }
        tools::constraints::create_constraint_actions(&ret);
        ret.insert_action_group("view", Some(&ret.action_group));
        ret.menubar
            .insert_action_group("view", Some(&ret.action_group));
        ret.state
            .set(Rc::new(RefCell::new(State::new(
                &glyph,
                app,
                ret.viewport.clone(),
            ))))
            .expect("Failed to create glyph state");
        ret.project.set(project).unwrap();
        Tool::setup_toolbox(&ret, glyph);
        ret.setup_menu(&ret);
        ret
    }

    pub fn set_selection(&self, selection: &[GlyphPointIndex], modifier: SelectionModifier) {
        use SelectionModifier::*;
        {
            let state = self.state().borrow();
            match modifier {
                Replace if selection.is_empty() && state.selection.is_empty() => {
                    return;
                }
                Add if selection.is_empty() => {
                    return;
                }
                Remove if selection.is_empty() => {
                    return;
                }
                Add if selection
                    .iter()
                    .all(|p| state.selection_set.contains(&p.uuid)) =>
                {
                    return;
                }
                Remove
                    if !selection
                        .iter()
                        .any(|p| state.selection_set.contains(&p.uuid)) =>
                {
                    return;
                }
                _ => {}
            }
        }

        let new = Rc::new(selection.to_vec());
        let old = Rc::new(self.state().borrow().selection.clone());
        let mut action = Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Self::static_type().name(),
                id: unsafe { std::mem::transmute::<&[GlyphPointIndex], &[u8]>(selection).into() },
            },
            compress: true,
            redo: Box::new(
                clone!(@weak self as obj, @strong new, @strong old => move || {
                    let State {
                        ref mut selection,
                        ref mut selection_set,
                        ..
                    } = &mut *obj.state().borrow_mut();
                    match modifier {
                        Replace => {
                            selection.clear();
                            selection_set.clear();
                            selection.extend(new.iter());
                            for v in selection.iter() {
                                selection_set.insert(v.uuid);
                            }
                        }
                        Add => {
                            selection.extend(new.iter().filter(|p| !selection_set.contains(&p.uuid)));
                            for v in selection.iter() {
                                selection_set.insert(v.uuid);
                            }
                        }
                        Remove => {
                            selection.retain(|e| !new.contains(e));
                            for v in new.iter() {
                                selection_set.remove(&v.uuid);
                            }
                        }
                    }
                    obj.viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self as obj, @strong new, @strong old => move || {
                    let State {
                        ref mut selection,
                        ref mut selection_set,
                        ..
                    } = &mut *obj.state().borrow_mut();
                    selection.clear();
                    selection_set.clear();
                    selection.extend(old.iter());
                    for v in selection.iter() {
                        selection_set.insert(v.uuid);
                    }
                    obj.viewport.queue_draw();
                }),
            ),
        };
        (action.redo)();
        self.state().borrow().add_undo_action(action);
    }

    pub fn state(&self) -> &Rc<RefCell<State>> {
        self.state.get().unwrap()
    }
}
