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

use super::{GlyphEditView, GlyphEditViewInner};
use crate::glyphs::Contour;
use crate::views::Canvas;
use gtk::{gio, glib::subclass::prelude::*, prelude::*};

impl GlyphEditViewInner {
    pub fn setup_menu(&self, obj: &GlyphEditView) {
        let menumodel = gio::Menu::new();
        let action_map = gtk::gio::SimpleActionGroup::new();
        {
            let glyph_menu = gio::Menu::new();
            glyph_menu.append(Some("Properties"), Some("glyph.properties"));
            glyph_menu.append(Some("Inspect"), Some("glyph.inspect"));
            glyph_menu.append(Some("Export to SVG"), Some("glyph.export.svg"));
            {
                let view_glyph_menu = gio::Menu::new();
                view_glyph_menu.append(Some("Show grid"), Some("glyph.show.grid"));
                view_glyph_menu.append(Some("Show handles"), Some("glyph.show.handles"));
                view_glyph_menu.append(Some("Inner fill"), Some("glyph.show.inner-fill"));
                view_glyph_menu.append(Some("Show total area"), Some("glyph.show.total-area"));
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
                action_map.add_action(&prop_action);
            }
        }
        {
            let curve_menu = gio::Menu::new();
            curve_menu.append(Some("Properties"), Some("glyph.curve.properties"));
            curve_menu.append(Some("Make cubic"), Some("glyph.curve.make_cubic"));
            curve_menu.append(Some("Make quadratic"), Some("glyph.curve.make_quadratic"));
            menumodel.append_submenu(Some("_Curve"), &curve_menu);
        }
        {
            let contour_menu = gio::Menu::new();
            contour_menu.append(Some("Properties"), Some("glyph.contour.properties"));
            contour_menu.append(Some("Reverse"), Some("glyph.contour.reverse"));
            menumodel.append_submenu(Some("_Contour"), &contour_menu);
        }
        {
            let guideline_menu = gio::Menu::new();
            guideline_menu.append(Some("Properties"), Some("glyph.guideline.properties"));
            {
                let view_guideline_menu = gio::Menu::new();
                view_guideline_menu.append(Some("Show guidelines"), Some("glyph.guideline.show"));
                view_guideline_menu.append(
                    Some("Show glyph guidelines"),
                    Some("glyph.guideline.show.glyph"),
                );
                view_guideline_menu.append(
                    Some("Show project guidelines"),
                    Some("glyph.guideline.show.project"),
                );
                view_guideline_menu.append(
                    Some("Show metrics guidelines"),
                    Some("glyph.guideline.show.metrics"),
                );
                view_guideline_menu.append(Some("Lock guidelines"), Some("glyph.guideline.lock"));
                guideline_menu.append_section(None, &view_guideline_menu);
            }
            menumodel.append_submenu(Some("_Guidelines"), &guideline_menu);
            let show_action = gtk::gio::PropertyAction::new(
                "guideline.show",
                &obj.imp().viewport,
                Canvas::SHOW_GUIDELINES,
            );
            action_map.add_action(&show_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "guideline.show.glyph",
                obj,
                GlyphEditView::SHOW_GLYPH_GUIDELINES,
            );
            action_map.add_action(&prop_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "guideline.show.project",
                obj,
                GlyphEditView::SHOW_PROJECT_GUIDELINES,
            );
            action_map.add_action(&prop_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "guideline.show.metrics",
                obj,
                GlyphEditView::SHOW_METRICS_GUIDELINES,
            );
            action_map.add_action(&prop_action);
            let prop_action = gtk::gio::PropertyAction::new(
                "guideline.lock",
                obj,
                GlyphEditView::LOCK_GUIDELINES,
            );
            action_map.add_action(&prop_action);
        }
        {
            let layer_menu = gio::Menu::new();
            layer_menu.append(Some("Properties"), Some("glyph.layer.properties"));
            menumodel.append_submenu(Some("_Layers"), &layer_menu);
        }
        {
            let properties = gtk::gio::SimpleAction::new("properties", None);
            properties.connect_activate(glib::clone!(@weak obj => move |_, _| {
                /*
                 * TODO: Glyph struct is not a GObject yet
                let gapp = obj.imp().app.get().unwrap().downcast_ref::<crate::GerbApp>().unwrap();
                let obj: glib::Object = obj.imp().glyph.get().unwrap().borrow().clone().upcast();
                let w = crate::utils::new_property_window(obj, "Glyph properties");
                w.present();
                */
            }));
            action_map.add_action(&properties);
            let inspect = gtk::gio::SimpleAction::new("inspect", None);
            inspect.connect_activate(glib::clone!(@weak obj => move |_, _| {
                obj.make_debug_window();
            }));
            action_map.add_action(&inspect);
            #[cfg(feature = "svg")]
            let export_svg = gtk::gio::SimpleAction::new("export.svg", None);
            #[cfg(feature = "svg")]
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
                let glyph = obj.imp().glyph_state.get().unwrap().borrow().glyph.clone();
                dialog.set_current_name(&format!("{}.svg", glyph.borrow().name.as_ref()));
                if dialog.run() == gtk::ResponseType::Ok {
                    if let Some(f) = dialog.filename() {
                        if let Err(err) = glyph.borrow().save_to_svg(f) {
                            let dialog = gtk::MessageDialog::new(
                                gtk::Window::NONE,
                                gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                                gtk::MessageType::Error,
                                gtk::ButtonsType::Close,
                                &err.to_string(),
                            );
                            dialog.set_title("Error: Could not svg file");
                            dialog.set_use_markup(true);
                            dialog.run();
                            dialog.emit_close();
                        }
                    }
                }
                dialog.emit_close();
            }));
            #[cfg(feature = "svg")]
            action_map.add_action(&export_svg);
            self.menubar.insert_action_group("glyph", Some(&action_map));
            self.menubar.bind_model(Some(&menumodel), None, true);
        }
    }
}

impl GlyphEditView {
    pub fn make_debug_window(&self) {
        let glyph_state = self.imp().glyph_state.get().unwrap().borrow();
        let glyph = glyph_state.glyph.borrow();
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
        let glif_info = gtk::Label::new(Some(&glyph.glif_source));
        glif_info.set_halign(gtk::Align::Start);
        scrolled_window.set_child(Some(&glif_info));
        hbox.pack_start(&scrolled_window, true, true, 0);
        #[cfg(feature = "svg")]
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
