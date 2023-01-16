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
mod tool_impl;
mod zoom;
pub use bezier::*;
pub use bspline::*;
pub use panning::*;
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
                    .get(&glyph_state.default_tool)
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
                    .get(&glyph_state.default_tool)
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
                    .get(&glyph_state.default_tool)
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
            .set_orientation(gtk::Orientation::Horizontal);
        obj.imp().toolbar_box.set_expand(false);
        obj.imp().toolbar_box.set_halign(gtk::Align::Center);
        obj.imp().toolbar_box.set_valign(gtk::Align::Start);
        obj.imp().toolbar_box.set_spacing(5);
        obj.imp().toolbar_box.set_visible(true);
        obj.imp().toolbar_box.set_can_focus(true);
        let toolbar = gtk::Toolbar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            //.toolbar_style(gtk::ToolbarStyle::Both)
            .visible(true)
            .can_focus(true)
            .build();
        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
        for t in [
            PanningTool::new().upcast::<ToolImpl>(),
            BezierTool::new().upcast::<ToolImpl>(),
            BSplineTool::new().upcast::<ToolImpl>(),
            ZoomInTool::new().upcast::<ToolImpl>(),
            ZoomOutTool::new().upcast::<ToolImpl>(),
        ] {
            t.setup_toolbox(&toolbar, obj);
            glyph_state.tools.insert(t.type_(), t);
        }

        let zoom_percent_label = gtk::Label::builder()
            .label("100%")
            .visible(true)
            .selectable(true) // So that the widget can receive the button-press event
            .width_chars(5) // So that if 2 digit zoom (<100%) has the same length as a widget with a three digit zoom value. For example 75% and 125% should result in the same width
            .events(gtk::gdk::EventMask::BUTTON_PRESS_MASK)
            .tooltip_text("Interface zoom percentage")
            .build();

        zoom_percent_label.connect_button_press_event(
            clone!(@strong obj => @default-return Inhibit(false), move |_self, event| {
                match event.button() {
                    gtk::gdk::BUTTON_SECONDARY => {
                        crate::utils::menu::Menu::new()
                            .add_button_cb(
                                "reset zoom",
                                clone!(@strong obj => @default-return Inhibit(false), move |_, _| {
                                    let t = &obj.imp().viewport.imp().transformation;
                                    t.reset_zoom();
                                    Inhibit(true)
                                }),
                            )
                            .add_button_cb(
                                "reset camera",
                                clone!(@strong obj => @default-return Inhibit(false), move |_, _| {
                                    let t = &obj.imp().viewport.imp().transformation;
                                    t.center_camera();
                                    Inhibit(true)
                                }),
                            )
                            .add_button_cb(
                                "set zoom value",
                                clone!(@strong obj, @weak _self => @default-return Inhibit(false), move |_, _| {
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
                                    Inhibit(true)
                                }),
                            ).popup();
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
        let debug_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Debug info"));
        debug_button.set_visible(true);
        debug_button.set_tooltip_text(Some("Debug info"));
        debug_button.connect_clicked(clone!(@strong obj => move |_| {
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
        obj.imp().toolbar_box.pack_start(&toolbar, false, false, 0);
        obj.imp()
            .toolbar_box
            .pack_start(&zoom_percent_label, false, false, 0);
        obj.imp()
            .toolbar_box
            .pack_start(&debug_button, false, false, 0);
        obj.imp()
            .toolbar_box
            .style_context()
            .add_class("glyph-edit-toolbox");
    }
}
