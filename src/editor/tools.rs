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

use super::*;

mod bezier;
mod bspline;
mod image;
mod panning;
mod shapes;
mod tool_impl;
mod zoom;
pub use bezier::*;
pub use bspline::*;
pub use image::*;
pub use panning::*;
pub use shapes::*;
pub use tool_impl::*;
pub use zoom::*;

pub struct Tool;

impl Tool {
    pub fn on_button_press_event(
        obj: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let state = obj.state().borrow();
        let active_tools = state
            .tools
            .get(&state.active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                state
                    .tools
                    .get(&state.panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            );
        drop(state);
        for t in active_tools {
            if t.on_button_press_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn on_button_release_event(
        obj: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let state = obj.state().borrow();
        let active_tools = state
            .tools
            .get(&state.active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                state
                    .tools
                    .get(&state.panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            );
        drop(state);
        for t in active_tools {
            if t.on_button_release_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn on_scroll_event(
        obj: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        let state = obj.state().borrow();
        let (panning_tool, active_tool) = (state.panning_tool, state.active_tool);
        let active_tools = state
            .tools
            .get(&active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(state.tools.get(&panning_tool).map(Clone::clone).into_iter())
            .chain(state.tools.clone().into_iter().filter_map(|(k, v)| {
                if [panning_tool, active_tool].contains(&k) {
                    None
                } else {
                    Some(v)
                }
            }));
        drop(state);
        for t in active_tools {
            if t.on_scroll_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn on_motion_notify_event(
        obj: Editor,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let state = obj.state().borrow();
        let active_tools = state
            .tools
            .get(&state.active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                state
                    .tools
                    .get(&state.panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            );
        drop(state);
        for t in active_tools {
            if t.on_motion_notify_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn setup_toolbox(obj: &Editor, glyph: Rc<RefCell<Glyph>>) {
        obj.toolbar_box.set_orientation(gtk::Orientation::Vertical);
        obj.toolbar_box.set_expand(false);
        obj.toolbar_box.set_halign(gtk::Align::Start);
        obj.toolbar_box.set_valign(gtk::Align::Start);
        obj.toolbar_box.set_spacing(5);
        obj.toolbar_box.set_border_width(0);
        obj.toolbar_box.set_visible(true);
        obj.toolbar_box.set_tooltip_text(Some("Tools"));
        obj.toolbar_box.set_can_focus(true);
        let toolbar = gtk::Toolbar::builder()
            .orientation(gtk::Orientation::Vertical)
            .expand(false)
            .show_arrow(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .toolbar_style(gtk::ToolbarStyle::Icons)
            .visible(true)
            .can_focus(true)
            .build();
        let mut state = obj.state().borrow_mut();
        for t in [
            PanningTool::new().upcast::<ToolImpl>(),
            BezierTool::new().upcast::<ToolImpl>(),
            //BSplineTool::new().upcast::<ToolImpl>(),
            QuadrilateralTool::new().upcast::<ToolImpl>(),
            EllipseTool::new().upcast::<ToolImpl>(),
            ImageTool::new(glyph, obj.project.get().unwrap().clone()).upcast::<ToolImpl>(),
            ZoomInTool::new().upcast::<ToolImpl>(),
            ZoomOutTool::new().upcast::<ToolImpl>(),
        ] {
            t.setup_toolbox(&toolbar, obj);
            state.tools.insert(t.type_(), t);
        }

        let zoom_percent_label = gtk::Label::builder()
            .label("100%")
            .visible(true)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .hexpand(false)
            .vexpand(false)
            .selectable(true) // So that the widget can receive the button-press event
            .width_chars(5) // So that if 2 digit zoom (<100%) has the same length as a widget with a three digit zoom value. For example 75% and 125% should result in the same width
            .events(gtk::gdk::EventMask::BUTTON_PRESS_MASK)
            .tooltip_text("Interface zoom percentage")
            .build();
        zoom_percent_label.style_context().add_class("zoom-label");

        zoom_percent_label.connect_button_press_event(
            clone!(@strong obj => @default-return Inhibit(false), move |_self, event| {
                match event.button() {
                    gtk::gdk::BUTTON_SECONDARY => {
                        crate::utils::menu::Menu::new()
                            .add_button_cb(
                                "reset zoom",
                                clone!(@strong obj => move |_| {
                                    let t = &obj.viewport.transformation;
                                    t.reset_zoom();
                                }),
                            )
                            .add_button_cb(
                                "set zoom value",
                                clone!(@strong obj, @weak _self => move |_| {
                                    let t = &obj.viewport.transformation;
                                    let dialog = gtk::Dialog::with_buttons(
                                        Some("set zoom value"),
                                        gtk::Window::NONE,
                                        gtk::DialogFlags::MODAL,
                                        &[
                                        ("Cancel", gtk::ResponseType::No),
                                        ("Save", gtk::ResponseType::Yes),
                                        ],
                                    );
                                    let content_box: gtk::Box = dialog.content_area();
                                    content_box.set_margin(5);
                                    let scale: f64 = t.property::<f64>(Transformation::SCALE);
                                    let error = gtk::Label::new(None);
                                    error.set_visible(false);
                                    let entry = gtk::Entry::builder()
                                        .input_purpose(gtk::InputPurpose::Number)
                                        .text(&format!("{:.2}", scale * 100.0))
                                        .margin(5)
                                        .build();
                                    content_box.add(&error);
                                    content_box.add(&entry);

                                    dialog.connect_response(
                                        clone!(@weak entry, @weak t, @weak error => move |dialog, response| {
                                            match response {
                                                gtk::ResponseType::No => {
                                                    /* cancel */
                                                    dialog.close();
                                                }
                                                gtk::ResponseType::Yes => {
                                                    /* Save */
                                                    if let Some(v) = entry.buffer().text().parse::<f64>()
                                                        .map_err(|err| {
                                                            error.set_text(&err.to_string());
                                                            error.set_visible(true);
                                                            err
                                                        })
                                                    .ok()
                                                        .and_then(|v| {
                                                            if !v.is_finite() || !(0.1..=1000.0).contains(&v) {
                                                                error.set_text(
                                                                    "Value out of range, must be at least 0.1 and at most 1000.0",
                                                                );
                                                                error.set_visible(true);
                                                                None
                                                            } else {
                                                                Some(v / 100.0)
                                                            }
                                                        })
                                                    {
                                                        t.set_property::<f64>(Transformation::SCALE, v);
                                                        dialog.close();
                                                    }
                                                }
                                                _ => { /* ignore */ }
                                            }
                                        }),
                                    );
                                    dialog.show_all();
                                }),
                            )
                            .add_button_cb(
                                "fit to page",
                                clone!(@strong obj => move |_| {
                                    let t = &obj.viewport.transformation;
                                    t.set_property(Transformation::FIT_VIEW, true);
                                }),
                            )
                            .add_button_cb(
                                "reset camera",
                                clone!(@strong obj => move |_| {
                                    let t = &obj.viewport.transformation;
                                    t.set_property(Transformation::CENTERED, true);
                                }),
                            ).popup(event.time());
                        Inhibit(true)
                    }
                    gtk::gdk::BUTTON_PRIMARY => {
                        let t = &obj.viewport.transformation;
                        t.reset_zoom();
                        Inhibit(true)
                    }
                    _ => Inhibit(false),
                }
            }),
        );
        obj.viewport
            .transformation
            .bind_property(Transformation::SCALE, &zoom_percent_label, "label")
            .transform_to(|_, scale: &Value| {
                let scale: f64 = scale.get().ok()?;
                Some(format!("{:.0}%", scale * 100.).to_value())
            })
            .build();
        obj.toolbar_box.pack_start(&toolbar, false, false, 0);
        obj.toolbar_box
            .pack_start(&zoom_percent_label, false, false, 0);
        obj.toolbar_box
            .style_context()
            .add_class("glyph-edit-toolbox");
    }
}

pub fn new_contour_action(
    glyph: Rc<RefCell<Glyph>>,
    contour: Contour,
    subaction: Action,
) -> Action {
    let subaction = Rc::new(RefCell::new(subaction));
    Action {
        stamp: EventStamp {
            t: std::any::TypeId::of::<Contour>(),
            property: "create contour",
            id: Box::new([]),
        },
        compress: false,
        redo: Box::new(
            clone!(@strong glyph, @strong contour, @strong subaction => move || {
                glyph.borrow_mut().contours.push(contour.clone());
                (subaction.borrow_mut().redo)();
            }),
        ),
        undo: Box::new(
            clone!(@strong glyph, @strong contour, @strong subaction => move || {
                (subaction.borrow_mut().undo)();
                glyph.borrow_mut().contours.pop();
            }),
        ),
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum SelectionModifier {
    #[default]
    Replace,
    Add,
    Remove,
}

impl From<gtk::gdk::ModifierType> for SelectionModifier {
    fn from(modifier: gtk::gdk::ModifierType) -> SelectionModifier {
        if modifier.contains(gtk::gdk::ModifierType::SHIFT_MASK) {
            if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                SelectionModifier::Remove
            } else {
                SelectionModifier::Add
            }
        } else {
            SelectionModifier::default()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectionAction {
    All,
    None,
}

pub mod constraints {
    use super::*;

    #[derive(Default)]
    #[glib::flags(name = "Lock")]
    pub enum Lock {
        #[default]
        #[flags_skip]
        EMPTY = 0,
        #[flags_value(name = "Lock transformation to X axis.", nick = "x")]
        X = 0b00000001,
        #[flags_value(name = "Lock transformation to Y axis.", nick = "y")]
        Y = 0b00000010,
        #[flags_value(name = "Lock transformation to local coordinates.", nick = "local")]
        LOCAL = 0b00000100,
        #[flags_value(name = "Lock transformation to control point axis.", nick = "controls")]
        CONTROLS = 0b00001000,
    }

    impl Lock {
        const ACTION_NAME: &str = Editor::LOCK_ACTION;

        pub fn as_str(&self) -> &'static str {
            match *self {
                Lock::EMPTY => "",
                v if v == (Lock::X | Lock::LOCAL) => "Locked X axis. | Local coordinates.",
                v if v == (Lock::X | Lock::CONTROLS) => {
                    "Locked X axis. | Control point axis coordinates."
                }
                v if v == Lock::X => "Locked X axis.",
                v if v == (Lock::Y | Lock::LOCAL) => "Locked Y axis. | Local coordinates.",
                v if v == (Lock::Y | Lock::CONTROLS) => {
                    "Locked Y axis. | Control point axis coordinates."
                }
                v if v == Lock::Y => "Locked Y axis.",
                v if v == Lock::LOCAL => "Local coordinates.",
                v if v == Lock::CONTROLS => "Control point axis coordinates.",
                other => unreachable!("{other:?}"),
            }
        }
    }

    #[derive(Default)]
    #[glib::flags(name = "Precision")]
    pub enum Precision {
        #[default]
        #[flags_skip]
        EMPTY = 0,
        #[flags_value(name = "Limit transformations to 1 units step size.", nick = "1")]
        _1 = 0b00000100,
        #[flags_value(name = "Limit transformations to 0.5 units step size.", nick = "0.5")]
        _05 = 0b00000010,
        #[flags_value(name = "Limit transformations to 0.1 units step size.", nick = "0.1")]
        _01 = 0b00000001,
    }

    impl Precision {
        const ACTION_NAME: &str = Editor::PRECISION_ACTION;

        pub fn as_str(&self) -> &'static str {
            match *self {
                Self::EMPTY => "",
                v if v == Self::_1 => "1.",
                v if v == Self::_05 => ".5",
                v if v == Self::_01 => ".1",
                other => unreachable!("{other:?}"),
            }
        }
    }

    #[derive(Default)]
    #[glib::flags(name = "Snap")]
    pub enum Snap {
        #[default]
        #[flags_skip]
        EMPTY = 0,
        #[flags_value(name = "Snap to angle", nick = "angle")]
        ANGLE = 0b00000001,
        #[flags_value(name = "Snap to grid", nick = "grid")]
        GRID = 0b00000010,
        #[flags_value(name = "Snap to guidelines", nick = "guideline")]
        GUIDELINES = 0b00000100,
        #[flags_value(name = "Snap to metrics", nick = "metrics")]
        METRICS = 0b00001000,
    }

    impl Snap {
        const ACTION_NAME: &str = Editor::SNAP_ACTION;

        pub fn as_str(&self) -> &'static str {
            match *self {
                Self::EMPTY => "",
                Self::ANGLE => "Snap to angles.",
                v if v == Self::ANGLE | Self::GRID => "Snap to angles, grid.",
                v if v == Self::ANGLE | Self::GUIDELINES => "Snap to angles, guidelines.",
                v if v == Self::ANGLE | Self::GRID | Self::GUIDELINES => {
                    "Snap to angles, grid, guidelines."
                }
                v if v == Self::ANGLE | Self::GRID | Self::GUIDELINES | Self::METRICS => {
                    "Snap to angles, grid, guidelines, glyph metrics."
                }
                v if v == Self::ANGLE | Self::GRID | Self::METRICS => {
                    "Snap to angles, grid, glyph metrics."
                }
                v if v == Self::ANGLE | Self::GUIDELINES | Self::METRICS => {
                    "Snap to angles, guidelines, glyph metrics."
                }
                v if v == Self::ANGLE | Self::METRICS => "Snap to angles, glyph metrics.",
                v if v == Self::GRID | Self::GUIDELINES | Self::METRICS => {
                    "Snap to grid, guidelines, glyph metrics."
                }
                v if v == Self::GRID | Self::GUIDELINES => "Snap to grid, guidelines.",
                v if v == Self::GRID | Self::METRICS => "Snap to grid, glyph metrics.",
                v if v == Self::GRID => "Snap to grid.",
                v if v == Self::GUIDELINES | Self::METRICS => "Snap to guidelines, glyph metrics.",
                v if v == Self::GUIDELINES => "Snap to guidelines.",
                v if v == Self::METRICS => "Snap to glyph metrics.",
                other => unreachable!("{other:?}"),
            }
        }
    }

    macro_rules! impl_methods {
        ($($ty:ty),*) => {
            $(
                impl glib::variant::StaticVariantType for $ty {
                    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
                        u32::static_variant_type()
                    }
                }

                impl glib::variant::ToVariant for $ty {
                    fn to_variant(&self) -> glib::Variant {
                        self.bits().to_variant()
                    }
                }

                impl glib::variant::FromVariant for $ty {
                    fn from_variant(variant: &glib::Variant) -> Option<$ty> {
                        <$ty>::from_bits(variant.get::<u32>()?)
                    }
                }

                impl $ty {
                    pub fn clear(view: &Editor) {
                        use gtk::prelude::ActionGroupExt;
                        use crate::glib::ToVariant;

                        view
                            .action_group
                            .change_action_state(Self::ACTION_NAME, &<$ty>::empty().to_variant());
                    }
                }
            )*
        };
    }

    impl_methods!(Lock, Precision, Snap);

    pub fn create_constraint_actions(obj: &Editor) {
        use gtk::prelude::*;

        let lock = gio::PropertyAction::new(Editor::LOCK_ACTION, obj, Editor::LOCK);
        for (name, (axis, complement)) in [
            (Editor::LOCK_X_ACTION, (Lock::X, Lock::Y)),
            (Editor::LOCK_Y_ACTION, (Lock::Y, Lock::X)),
        ] {
            let toggle_axis = gio::SimpleAction::new(name, None);
            toggle_axis.connect_activate(glib::clone!(@weak obj, @weak lock => move |_, _| {
                let Some(state) = lock.state() else { return; };
                let Some(mut lock_flags) = state.get::<Lock>() else { return; };
                if lock_flags.intersects(axis) {
                    lock.change_state(&Lock::empty().to_variant());
                } else {
                    lock_flags.set(complement, false);
                    lock_flags.set(axis, true);
                    lock.change_state(&lock_flags.to_variant());
                }
            }));
            obj.action_group.add_action(&toggle_axis);
        }
        for (name, opt) in [
            (Editor::LOCK_LOCAL_ACTION, Lock::LOCAL),
            (Editor::LOCK_CONTROLS_ACTION, Lock::CONTROLS),
        ] {
            let change_opt = gio::SimpleAction::new(name, None);
            change_opt.connect_activate(glib::clone!(@weak obj, @weak lock => move |_, _| {
                let Some(state) = lock.state() else { return; };
                let Some(mut lock_flags) = state.get::<Lock>() else { return; };
                if !lock_flags.intersection(Lock::X | Lock::Y).is_empty() {
                    return;
                }
                if lock_flags.intersects(opt) {
                    lock_flags.set(opt, false);
                    lock.change_state(&lock_flags.to_variant());
                } else {
                    lock_flags.set(Lock::LOCAL|Lock::CONTROLS, false);
                    lock_flags.set(opt, true);
                    lock.change_state(&lock_flags.to_variant());
                }
            }));
            obj.action_group.add_action(&change_opt);
        }
        obj.action_group.add_action(&lock);

        let snap = gio::PropertyAction::new(Editor::SNAP_ACTION, obj, Editor::SNAP);
        for (name, anchor) in [
            (Editor::SNAP_ANGLE_ACTION, Snap::ANGLE),
            (Editor::SNAP_GRID_ACTION, Snap::GRID),
            (Editor::SNAP_GUIDELINES_ACTION, Snap::GUIDELINES),
            (Editor::SNAP_METRICS_ACTION, Snap::METRICS),
        ] {
            let toggle_anchor = gio::SimpleAction::new(name, None);
            toggle_anchor.connect_activate(glib::clone!(@weak obj, @weak snap => move |_, _| {
                let Some(state) = snap.state() else { return; };
                let Some(mut snap_flags) = state.get::<Snap>() else { return; };
                snap_flags.toggle(anchor);
                snap.change_state(&snap_flags.to_variant());
            }));
            obj.action_group.add_action(&toggle_anchor);
        }
        let clear_snap = gio::SimpleAction::new(Editor::SNAP_CLEAR_ACTION, None);
        clear_snap.connect_activate(glib::clone!(@weak obj => move |_, _| {
            Snap::clear(&obj);
        }));
        obj.action_group.add_action(&clear_snap);
        obj.action_group.add_action(&snap);
        {
            let a = gio::PropertyAction::new(Editor::PRECISION_ACTION, obj, Editor::PRECISION);
            obj.action_group.add_action(&a);
        }
    }
}
