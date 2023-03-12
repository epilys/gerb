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

use super::{Editor, EditorInner};
use crate::glyphs::Contour;
use crate::prelude::*;
use crate::views::Canvas;
use gtk::{gio, glib::subclass::prelude::*, prelude::*};

fn new_accel_item(menu: &gio::Menu, app: &Application, label: &str, detailed_action_name: &str) {
    let item = gio::MenuItem::new(Some(label), Some(detailed_action_name));
    // https://stackoverflow.com/a/16860754
    if let Some(accel) = app
        .accels_for_action(detailed_action_name)
        .into_iter()
        .next()
    {
        item.set_attribute_value("accel", Some(&accel.to_variant()));
    }
    menu.append_item(&item);
}

impl EditorInner {
    pub fn setup_menu(&self, obj: &Editor) {
        let app = self.app();
        let menumodel = gio::Menu::new();
        let action_group = gtk::gio::SimpleActionGroup::new();
        {
            let glyph_menu = gio::Menu::new();
            new_accel_item(&glyph_menu, app, "Preview", "view.preview");
            new_accel_item(&glyph_menu, app, "Save", "glyph.save");
            new_accel_item(&glyph_menu, app, "Properties", "glyph.properties");
            new_accel_item(&glyph_menu, app, "Inspect", "glyph.inspect");
            new_accel_item(&glyph_menu, app, "Export to SVG", "glyph.export.svg");
            {
                let view_glyph_menu = gio::Menu::new();
                new_accel_item(&view_glyph_menu, app, "Show grid", "glyph.show.grid");
                new_accel_item(&view_glyph_menu, app, "Show handles", "glyph.show.handles");
                new_accel_item(&view_glyph_menu, app, "Inner fill", "glyph.show.inner-fill");
                new_accel_item(
                    &view_glyph_menu,
                    app,
                    "Show total area",
                    "glyph.show.total-area",
                );
                new_accel_item(
                    &view_glyph_menu,
                    app,
                    "View settings",
                    "glyph.open.settings",
                );
                glyph_menu.append_section(None, &view_glyph_menu);
            }
            menumodel.append_submenu(Some("_Glyph"), &glyph_menu);
            for (action_name, property) in [
                ("show.grid", Canvas::SHOW_GRID),
                ("show.inner-fill", Canvas::INNER_FILL),
                ("show.handles", Canvas::SHOW_HANDLES),
                ("show.total-area", Canvas::SHOW_TOTAL_AREA),
            ] {
                let prop_action =
                    gtk::gio::PropertyAction::new(action_name, &obj.imp().viewport, property);
                action_group.add_action(&prop_action);
            }
            let settings = gtk::gio::SimpleAction::new("open.settings", None);
            settings.connect_activate(
                glib::clone!(@weak self.viewport as viewport, @weak app => move |_, _| {
                    let w = viewport.new_property_window(&app, false);
                    w.present();
                }),
            );
            action_group.add_action(&settings);
        }
        {
            let curve_menu = gio::Menu::new();
            new_accel_item(&curve_menu, app, "Properties", "glyph.curve.properties");
            new_accel_item(&curve_menu, app, "Make cubic", "glyph.curve.make_cubic");
            new_accel_item(
                &curve_menu,
                app,
                "Make quadratic",
                "glyph.curve.make_quadratic",
            );
            menumodel.append_submenu(Some("_Curve"), &curve_menu);
        }
        {
            let contour_menu = gio::Menu::new();
            new_accel_item(&contour_menu, app, "Properties", "glyph.contour.properties");
            new_accel_item(&contour_menu, app, "Reverse", "glyph.contour.reverse");
            menumodel.append_submenu(Some("_Contour"), &contour_menu);
        }
        {
            let guideline_menu = gio::Menu::new();
            new_accel_item(
                &guideline_menu,
                app,
                "Properties",
                "glyph.guideline.properties",
            );
            {
                let view_guideline_menu = gio::Menu::new();
                new_accel_item(
                    &view_guideline_menu,
                    app,
                    "Show guidelines",
                    "glyph.show.guideline",
                );
                new_accel_item(
                    &view_guideline_menu,
                    app,
                    "Show glyph guidelines",
                    "glyph.show.guideline.glyph",
                );
                new_accel_item(
                    &view_guideline_menu,
                    app,
                    "Show project guidelines",
                    "glyph.show.guideline.project",
                );
                new_accel_item(
                    &view_guideline_menu,
                    app,
                    "Show metrics guidelines",
                    "glyph.show.guideline.metrics",
                );
                new_accel_item(
                    &view_guideline_menu,
                    app,
                    "Lock guidelines",
                    "glyph.guideline.lock",
                );
                guideline_menu.append_section(None, &view_guideline_menu);
            }
            menumodel.append_submenu(Some("_Guidelines"), &guideline_menu);
            let show_action = gtk::gio::PropertyAction::new(
                "show.guideline",
                &obj.imp().viewport,
                Canvas::SHOW_GUIDELINES,
            );
            action_group.add_action(&show_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "show.guideline.glyph",
                obj,
                Editor::SHOW_GLYPH_GUIDELINES,
            );
            action_group.add_action(&prop_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "show.guideline.project",
                obj,
                Editor::SHOW_PROJECT_GUIDELINES,
            );
            action_group.add_action(&prop_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "show.guideline.metrics",
                obj,
                Editor::SHOW_METRICS_GUIDELINES,
            );
            action_group.add_action(&prop_action);
            let prop_action =
                gtk::gio::PropertyAction::new("guideline.lock", obj, Editor::LOCK_GUIDELINES);
            action_group.add_action(&prop_action);
        }
        {
            let layer_menu = gio::Menu::new();
            new_accel_item(&layer_menu, app, "Properties", "glyph.layer.properties");
            menumodel.append_submenu(Some("_Layers"), &layer_menu);
        }
        {
            let save = gtk::gio::SimpleAction::new("save", None);
            save.connect_activate(glib::clone!(@weak obj => move |_, _| {
                let project = obj.project();
                let path = project.path.borrow();
                if let Err(err) = obj.state().borrow().glyph.borrow().save(&path.join("glyphs")) {
                    let dialog = crate::utils::widgets::new_simple_error_dialog(
                        Some("Error: Could not save glyph."),
                        &err.to_string(),
                        None,
                        obj.app().window.upcast_ref(),
                    );
                    dialog.run();
                    dialog.emit_close();
                };
            }));
            action_group.add_action(&save);
            let properties = gtk::gio::SimpleAction::new("properties", None);
            properties.connect_activate(glib::clone!(@weak obj, @weak app => move |_, _| {
                let w = obj.glyph().borrow().metadata.new_property_window(&app, false);
                w.present();
            }));
            action_group.add_action(&properties);
            let inspect = gtk::gio::SimpleAction::new("inspect", None);
            inspect.connect_activate(glib::clone!(@weak obj => move |_, _| {
                obj.make_debug_window();
            }));
            action_group.add_action(&inspect);
            let export_svg = gtk::gio::SimpleAction::new("export.svg", None);
            export_svg.connect_activate(clone!(@weak obj => move |_, _| {
                let dialog = gtk::FileChooserDialog::builder()
                    .create_folders(true)
                    .do_overwrite_confirmation(true)
                    .action(gtk::FileChooserAction::Save)
                    .visible(true)
                    .sensitive(true)
                    .build();
                dialog.add_button("Save", gtk::ResponseType::Ok);
                dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                let glyph = obj.state().borrow().glyph.clone();
                dialog.set_current_name(&format!("{}.svg", glyph.borrow().name()));
                if dialog.run() == gtk::ResponseType::Ok {
                    if let Some(f) = dialog.filename() {
                        if let Err(err) = glyph.borrow().save_to_svg(f) {
                            let dialog = crate::utils::widgets::new_simple_error_dialog(
                                Some("Error: Could not generate SVG file"),
                                &err.to_string(),
                                None,
                                obj.app().window.upcast_ref(),
                            );
                            dialog.run();
                            dialog.emit_close();
                        }
                    }
                }
                dialog.emit_close();
            }));
            action_group.add_action(&export_svg);
            self.menubar
                .insert_action_group("glyph", Some(&action_group));
            obj.insert_action_group("glyph", Some(&action_group));
            self.menubar.bind_model(Some(&menumodel), None, true);
        }
    }
}

impl Editor {
    pub fn make_debug_window(&self) {
        let state = self.state().borrow();
        let glyph = state.glyph.borrow();
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_attached_to(Some(self));
        window.set_default_size(640, 480);
        window.insert_action_group("glyph", self.imp().menubar.action_group("glyph").as_ref());
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
        let glif_info = gtk::Label::new(Some(&glyph.glif_source.borrow()));
        glif_info.set_halign(gtk::Align::Start);
        scrolled_window.set_child(Some(&glif_info));
        hbox.pack_start(&scrolled_window, true, true, 0);
        {
            let save_to_svg = gtk::Button::builder()
                .label("Save to SVG")
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .visible(true)
                .action_name("glyph.export.svg")
                .build();
            hbox.pack_start(&save_to_svg, false, true, 5);
        }

        window.add(&hbox);
        window.show_all();
    }
}
