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
mod panning;
mod shapes;
mod tool_impl;
mod zoom;
pub use bezier::*;
pub use bspline::*;
pub use panning::*;
pub use shapes::*;
pub use tool_impl::*;
pub use zoom::*;

pub struct Tool;

impl Tool {
    pub fn on_button_press_event(
        obj: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
        let active_tools = glyph_state
            .tools
            .get(&glyph_state.active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                glyph_state
                    .tools
                    .get(&glyph_state.panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            );
        drop(glyph_state);
        for t in active_tools {
            if t.on_button_press_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn on_button_release_event(
        obj: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
        let active_tools = glyph_state
            .tools
            .get(&glyph_state.active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                glyph_state
                    .tools
                    .get(&glyph_state.panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            );
        drop(glyph_state);
        for t in active_tools {
            if t.on_button_release_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn on_scroll_event(
        obj: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
        let (panning_tool, active_tool) = (glyph_state.panning_tool, glyph_state.active_tool);
        let active_tools = glyph_state
            .tools
            .get(&active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                glyph_state
                    .tools
                    .get(&panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            )
            .chain(glyph_state.tools.clone().into_iter().filter_map(|(k, v)| {
                if [panning_tool, active_tool].contains(&k) {
                    None
                } else {
                    Some(v)
                }
            }));
        drop(glyph_state);
        for t in active_tools {
            if t.on_scroll_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn on_motion_notify_event(
        obj: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
        let active_tools = glyph_state
            .tools
            .get(&glyph_state.active_tool)
            .map(Clone::clone)
            .into_iter()
            .chain(
                glyph_state
                    .tools
                    .get(&glyph_state.panning_tool)
                    .map(Clone::clone)
                    .into_iter(),
            );
        drop(glyph_state);
        for t in active_tools {
            if t.on_motion_notify_event(obj.clone(), viewport, event) == Inhibit(true) {
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    pub fn setup_toolbox(obj: &GlyphEditView) {
        obj.imp()
            .toolbar_box
            .set_orientation(gtk::Orientation::Vertical);
        obj.imp().toolbar_box.set_expand(false);
        obj.imp().toolbar_box.set_halign(gtk::Align::Start);
        obj.imp().toolbar_box.set_valign(gtk::Align::Start);
        obj.imp().toolbar_box.set_spacing(5);
        obj.imp().toolbar_box.set_border_width(0);
        obj.imp().toolbar_box.set_visible(true);
        obj.imp().toolbar_box.set_tooltip_text(Some("Tools"));
        obj.imp().toolbar_box.set_can_focus(true);
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
        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
        for t in [
            PanningTool::new().upcast::<ToolImpl>(),
            BezierTool::new().upcast::<ToolImpl>(),
            BSplineTool::new().upcast::<ToolImpl>(),
            QuadrilateralTool::new().upcast::<ToolImpl>(),
            EllipseTool::new().upcast::<ToolImpl>(),
            ZoomInTool::new().upcast::<ToolImpl>(),
            ZoomOutTool::new().upcast::<ToolImpl>(),
        ] {
            t.setup_toolbox(&toolbar, obj);
            glyph_state.tools.insert(t.type_(), t);
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
                                    let t = &obj.imp().viewport.imp().transformation;
                                    t.reset_zoom();
                                }),
                            )
                            .add_button_cb(
                                "set zoom value",
                                clone!(@strong obj, @weak _self => move |_| {
                                    let t = &obj.imp().viewport.imp().transformation;
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
                                    let t = &obj.imp().viewport.imp().transformation;
                                    t.set_property(Transformation::FIT_VIEW, true);
                                }),
                            )
                            .add_button_cb(
                                "reset camera",
                                clone!(@strong obj => move |_| {
                                    let t = &obj.imp().viewport.imp().transformation;
                                    t.set_property(Transformation::CENTERED, true);
                                }),
                            ).popup(event.time());
                        Inhibit(true)
                    }
                    gtk::gdk::BUTTON_PRIMARY => {
                        let t = &obj.imp().viewport.imp().transformation;
                        t.reset_zoom();
                        Inhibit(true)
                    }
                    _ => Inhibit(false),
                }
            }),
        );
        obj.imp()
            .viewport
            .imp()
            .transformation
            .bind_property(Transformation::SCALE, &zoom_percent_label, "label")
            .transform_to(|_, scale: &Value| {
                let scale: f64 = scale.get().ok()?;
                Some(format!("{:.0}%", scale * 100.).to_value())
            })
            .build();
        obj.imp().toolbar_box.pack_start(&toolbar, false, false, 0);
        obj.imp()
            .toolbar_box
            .pack_start(&zoom_percent_label, false, false, 0);
        obj.imp()
            .toolbar_box
            .style_context()
            .add_class("glyph-edit-toolbox");
    }
}

pub fn new_contour_action(
    glyph: Rc<RefCell<Glyph>>,
    contour: Contour,
    subaction: crate::Action,
) -> crate::Action {
    let subaction = Rc::new(RefCell::new(subaction));
    crate::Action {
        stamp: crate::EventStamp {
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
                glyph.borrow_mut().contours.pop();
                (subaction.borrow_mut().undo)();
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

pub mod constraints {
    use crate::GlyphEditView;

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
        const ACTION_NAME: &str = GlyphEditView::LOCK_ACTION;

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
                other => unreachable!("{other:?}"),
            }
        }
    }

    #[glib::flags(name = "Precision")]
    pub enum Precision {
        //const _01           = 0b00000001;
        //const _05           = 0b00000010;
        //const _1            = 0b00000100;
        #[flags_value(name = "Limit transformations to 0.5 units step size.", nick = "5")]
        _5 = 0b00000001,
    }

    impl Precision {
        const ACTION_NAME: &str = GlyphEditView::PRECISION_ACTION;
    }

    #[glib::flags(name = "Snap")]
    pub enum Snap {
        #[flags_value(name = "Snap to angle", nick = "angle")]
        ANGLE = 0b00000001,
    }

    impl Snap {
        const ACTION_NAME: &str = GlyphEditView::SNAP_ACTION;
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
                    pub fn clear(view: &GlyphEditView) {
                        use gtk::prelude::ActionGroupExt;
                        use gtk::glib::subclass::types::ObjectSubclassIsExt;
                        use crate::glib::ToVariant;

                        view.imp()
                            .action_group
                            .change_action_state(Self::ACTION_NAME, &<$ty>::empty().to_variant());
                    }
                }
            )*
        };
    }

    impl_methods!(Lock, Precision, Snap);

    pub fn create_constraint_actions(obj: &GlyphEditView) {
        use gtk::gio;
        use gtk::prelude::*;
        use gtk::subclass::prelude::*;

        let lock = gio::PropertyAction::new(GlyphEditView::LOCK_ACTION, obj, GlyphEditView::LOCK);
        for (name, (axis, complement)) in [
            (GlyphEditView::LOCK_X_ACTION, (Lock::X, Lock::Y)),
            (GlyphEditView::LOCK_Y_ACTION, (Lock::Y, Lock::X)),
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
            obj.imp().action_group.add_action(&toggle_axis);
        }
        for (name, opt) in [
            (GlyphEditView::LOCK_LOCAL_ACTION, Lock::LOCAL),
            (GlyphEditView::LOCK_CONTROLS_ACTION, Lock::CONTROLS),
        ] {
            let change_opt = gio::SimpleAction::new(name, None);
            change_opt.connect_activate(glib::clone!(@weak obj, @weak lock => move |_, _| {
                let Some(state) = lock.state() else { return; };
                let Some(mut lock_flags) = state.get::<Lock>() else { return; };
                if lock_flags.intersection(Lock::X | Lock::Y).is_empty() {
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
            obj.imp().action_group.add_action(&change_opt);
        }
        obj.imp().action_group.add_action(&lock);
    }
}
