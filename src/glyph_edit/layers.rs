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
use crate::utils::colors::*;

pub fn draw_glyph_layer(
    viewport: &Canvas,
    cr: &gtk::cairo::Context,
    obj: GlyphEditView,
) -> Inhibit {
    let inner_fill = viewport.property::<bool>(Canvas::INNER_FILL);
    let scale: f64 = viewport
        .imp()
        .transformation
        .property::<f64>(Transformation::SCALE);
    let ppu: f64 = viewport
        .imp()
        .transformation
        .property::<f64>(Transformation::PIXELS_PER_UNIT);
    let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
    let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
    let units_per_em = obj.property::<f64>(GlyphEditView::UNITS_PER_EM);
    let matrix = viewport.imp().transformation.matrix();

    let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
    let mouse = viewport.get_mouse();
    let unit_mouse = viewport.view_to_unit_point(mouse);
    let ViewPoint(view_camera) = viewport.imp().transformation.camera();
    let UnitPoint(camera) = viewport.view_to_unit_point(viewport.imp().transformation.camera());

    cr.save().unwrap();

    cr.transform(matrix);
    cr.save().unwrap();
    cr.set_line_width(2.5);
    cr.arc(
        camera.x,
        camera.y,
        5.0 + 1.0,
        0.,
        2.0 * std::f64::consts::PI,
    );
    cr.stroke().unwrap();

    obj.imp().new_statusbar_message(&format!("Mouse: ({:.2}, {:.2}), Unit mouse: ({:.2}, {:.2}), Camera: ({:.2}, {:.2}), Unit Camera: ({:.2}, {:.2}), Size: ({width:.2}, {height:.2}), Scale: {scale:.2}", mouse.0.x, mouse.0.y, unit_mouse.0.x, unit_mouse.0.y, view_camera.x, view_camera.y, camera.x, camera.y));

    cr.restore().unwrap();

    /* Draw the glyph */

    {
        let line_width = obj
            .imp()
            .settings
            .get()
            .unwrap()
            .property::<f64>(Settings::LINE_WIDTH)
            / (scale * ppu);
        let (handle, handle_connection) = if viewport.property::<bool>(Canvas::SHOW_HANDLES) {
            (
                Some(
                    viewport
                        .property::<DrawOptions>(Canvas::HANDLE_OPTIONS)
                        .scale(scale * ppu),
                ),
                Some(
                    viewport
                        .property::<DrawOptions>(Canvas::HANDLE_CONNECTION_OPTIONS)
                        .scale(scale * ppu),
                ),
            )
        } else {
            (None, None)
        };
        let direction_options = if viewport.property::<bool>(Canvas::SHOW_DIRECTION) {
            Some(
                viewport
                    .property::<DrawOptions>(Canvas::DIRECTION_OPTIONS)
                    .scale(scale * ppu),
            )
        } else {
            None
        };

        let options = GlyphDrawingOptions {
            outline: viewport
                .property::<DrawOptions>(Canvas::OUTLINE_OPTIONS)
                .scale(scale * ppu),
            inner_fill: Some(
                <DrawOptions>::from((
                    if inner_fill {
                        Color::BLACK
                    } else {
                        viewport.property::<Color>(Canvas::GLYPH_INNER_FILL_COLOR)
                    },
                    line_width,
                ))
                .scale(scale * ppu),
            ),
            highlight: obj.imp().hovering.get(),
            matrix: Matrix::identity(),
            units_per_em,
            handle_connection,
            handle,
            corner: handle,
            smooth_corner: handle,
            direction_arrow: direction_options,
            selection: Some(glyph_state.get_selection()),
        };
        glyph_state.glyph.borrow().draw(cr, options);
    }

    cr.restore().unwrap();

    Inhibit(false)
}

pub fn draw_guidelines(viewport: &Canvas, cr: &gtk::cairo::Context, obj: GlyphEditView) -> Inhibit {
    let glyph_state = obj.imp().glyph_state.get().unwrap();
    let matrix = viewport.imp().transformation.matrix();
    let units_per_em = obj.property::<f64>(GlyphEditView::UNITS_PER_EM);
    cr.save().unwrap();

    if viewport.property::<bool>(Canvas::SHOW_TOTAL_AREA) {
        cr.save().unwrap();
        cr.transform(matrix);

        /* Draw em square of units_per_em units: */
        cr.set_source_color_alpha(viewport.property::<Color>(Canvas::GLYPH_BBOX_BG_COLOR));
        cr.rectangle(
            0.0,
            0.0,
            glyph_state
                .borrow()
                .glyph
                .borrow()
                .width
                .unwrap_or(units_per_em),
            1000.0,
        );
        cr.fill().unwrap();
        cr.restore().unwrap();
    }

    if viewport.property::<bool>(Canvas::SHOW_GUIDELINES) {
        let show_glyph_guidelines = obj.property::<bool>(GlyphEditView::SHOW_GLYPH_GUIDELINES);
        let show_project_guidelines = obj.property::<bool>(GlyphEditView::SHOW_PROJECT_GUIDELINES);
        let show_metrics_guidelines = obj.property::<bool>(GlyphEditView::SHOW_METRICS_GUIDELINES);
        let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
        let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
        let mouse = viewport.get_mouse();
        let UnitPoint(unit_mouse) = viewport.view_to_unit_point(mouse);
        cr.set_line_width(
            obj.imp()
                .settings
                .get()
                .unwrap()
                .property(Settings::GUIDELINE_WIDTH),
        );
        let glyph_state_ref = glyph_state.borrow();
        for g in glyph_state_ref
            .glyph
            .borrow()
            .guidelines
            .iter()
            .filter(|_| show_glyph_guidelines)
            .chain(
                obj.imp()
                    .project
                    .get()
                    .unwrap()
                    .imp()
                    .guidelines
                    .borrow()
                    .iter()
                    .filter(|_| show_project_guidelines),
            )
            .chain(
                obj.imp()
                    .project
                    .get()
                    .unwrap()
                    .imp()
                    .metric_guidelines
                    .borrow()
                    .iter()
                    .filter(|_| show_metrics_guidelines),
            )
        {
            let highlight = g.imp().on_line_query(unit_mouse, None);
            g.imp().draw(cr, matrix, (width, height), highlight);
            if highlight {
                cr.move_to(mouse.0.x, mouse.0.y);
                let line_height = cr.text_extents("Guideline").unwrap().height * 1.5;
                cr.show_text("Guideline").unwrap();
                for (i, line) in [
                    format!("Name: {}", g.name().as_deref().unwrap_or("-")),
                    format!("Identifier: {}", g.identifier().as_deref().unwrap_or("-")),
                    format!("Point: ({:.2}, {:.2})", g.x(), g.y()),
                    format!("Angle: {:01}deg", g.angle()),
                ]
                .into_iter()
                .enumerate()
                {
                    cr.move_to(mouse.0.x, mouse.0.y + (i + 1) as f64 * line_height);
                    cr.show_text(&line).unwrap();
                }
            } else if g.angle() == 0.0 {
                cr.save().unwrap();
                cr.set_source_color_alpha(Color::try_from_hex("#bbbaae").unwrap());
                let ViewPoint(Point { y, .. }) =
                    viewport.unit_to_view_point(UnitPoint((0.0, g.y()).into()));
                let label = if let Some(name) = g.name().as_deref() {
                    format!("{name} ({:00})", g.y().ceil())
                } else {
                    format!("{}", g.y().ceil())
                };
                let extents = cr.text_extents(&label).unwrap();
                cr.move_to(width - 2.5 - extents.width, y - 0.5);
                cr.show_text(&label).unwrap();
                cr.restore().unwrap();
            } else if g.angle() == 90.0 {
            }
            cr.stroke().unwrap();
        }
    }
    cr.restore().unwrap();
    Inhibit(false)
}

impl GlyphEditViewInner {
    pub fn create_layer_widget(&self) -> gtk::ListBox {
        let listbox = gtk::ListBox::builder()
            .name("layers")
            .expand(false)
            .visible(true)
            .can_focus(true)
            .tooltip_text("layers")
            .halign(gtk::Align::Start)
            .valign(gtk::Align::End)
            .selection_mode(gtk::SelectionMode::None)
            .build();
        let label = gtk::Label::new(Some("layers"));
        label.set_visible(true);
        label.set_sensitive(false);
        listbox.add(&label);
        for layer in self
            .viewport
            .imp()
            .pre_layers
            .borrow()
            .iter()
            .chain(self.viewport.imp().layers.borrow().iter())
            .chain(self.viewport.imp().post_layers.borrow().iter())
        {
            let label = gtk::Label::new(Some(&layer.property::<String>(Layer::NAME)));
            label.set_visible(true);
            label.set_margin(3);
            let button = gtk::CheckButton::builder()
                .visible(true)
                .active(true)
                .build();
            layer
                .bind_property(Layer::ACTIVE, &button, "active")
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();
            button.connect_toggled(clone!(@strong self.viewport as viewport => move |button| {
                if button.is_active() {
                    button.style_context().add_class("active");
                } else {
                    button.style_context().remove_class("active");
                }
                viewport.queue_draw();
            }));
            button.toggled();
            let row = gtk::Box::builder().visible(true).expand(true).build();
            row.pack_start(&label, true, true, 0);
            row.pack_start(&button, false, false, 0);
            listbox.add(&row);
        }
        listbox.show_all();
        listbox
    }
}
