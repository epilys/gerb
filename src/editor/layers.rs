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

pub fn draw_glyph_layer(viewport: &Canvas, mut cr: ContextRef, obj: Editor) -> Inhibit {
    let inner_fill = viewport.property::<bool>(Canvas::INNER_FILL);
    let scale: f64 = viewport
        .transformation
        .property::<f64>(Transformation::SCALE);
    let ppu: f64 = viewport
        .transformation
        .property::<f64>(Transformation::PIXELS_PER_UNIT);
    let units_per_em = obj.property::<f64>(Editor::UNITS_PER_EM);
    let preview = obj.property::<bool>(Editor::PREVIEW);
    let matrix = viewport.transformation.matrix();

    let state = obj.state().borrow();
    let UnitPoint(camera) = viewport.view_to_unit_point(viewport.transformation.camera());

    cr.transform(matrix);
    {
        let cr1 = cr.push();

        cr1.set_line_width(2.5);
        if !preview {
            cr1.arc(
                camera.x,
                camera.y,
                5.0 + 1.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cr1.stroke().unwrap();
        }
    }
    /*
        {
            let mouse = viewport.get_mouse();
            let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
            let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
            let unit_mouse = viewport.view_to_unit_point(mouse);
            let ViewPoint(view_camera) = viewport.transformation.camera();
            obj.new_statusbar_message(&format!("Mouse: ({:.2}, {:.2}), Unit mouse: ({:.2}, {:.2}), Camera: ({:.2}, {:.2}), Unit Camera: ({:.2}, {:.2}), Size: ({width:.2}, {height:.2}), Scale: {scale:.2}", mouse.0.x, mouse.0.y, unit_mouse.0.x, unit_mouse.0.y, view_camera.x, view_camera.y, camera.x, camera.y));
        }
    */

    /* Draw the glyph */

    {
        let line_width = obj.app_settings().property::<f64>(Settings::LINE_WIDTH) / (scale * ppu);
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
        let direction_arrow = if viewport.property::<bool>(Canvas::SHOW_DIRECTION) {
            Some(viewport.property::<DrawOptions>(Canvas::DIRECTION_OPTIONS))
        } else {
            None
        };

        let options = if preview {
            GlyphDrawingOptions {
                outline: (Color::BLACK, 0.0).into(),
                inner_fill: Some((Color::BLACK, line_width).into()),
                highlight: None,
                matrix: Matrix::identity(),
                units_per_em,
                ..Default::default()
            }
        } else {
            GlyphDrawingOptions {
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
                highlight: obj.hovering.get(),
                matrix: Matrix::identity(),
                units_per_em,
                handle_connection,
                handle,
                corner: handle,
                smooth_corner: handle,
                direction_arrow,
                selection: Some(state.get_selection_set()),
            }
        };
        state.glyph.borrow().draw(cr.push(), options);
    }

    Inhibit(false)
}

pub fn draw_guidelines(viewport: &Canvas, mut cr: ContextRef, obj: Editor) -> Inhibit {
    let state = obj.state();
    let matrix = viewport.transformation.matrix();
    let units_per_em = obj.property::<f64>(Editor::UNITS_PER_EM);
    let preview = obj.property::<bool>(Editor::PREVIEW);
    if preview {
        return Inhibit(false);
    }
    let mut cr1 = cr.push();

    if viewport.property::<bool>(Canvas::SHOW_TOTAL_AREA) {
        let cr2 = cr1.push();
        cr2.transform(matrix);

        /* Draw em square of units_per_em units: */
        cr2.set_source_color_alpha(viewport.property::<Color>(Canvas::GLYPH_BBOX_BG_COLOR));
        cr2.rectangle(
            0.0,
            0.0,
            state.borrow().glyph.borrow().width.unwrap_or(units_per_em),
            1000.0,
        );
        cr2.fill().unwrap();
    }

    if viewport.property::<bool>(Canvas::SHOW_GUIDELINES) {
        let scale: f64 = viewport
            .transformation
            .property::<f64>(Transformation::SCALE);
        let ppu: f64 = viewport
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let show_glyph_guidelines = obj.property::<bool>(Editor::SHOW_GLYPH_GUIDELINES);
        let show_project_guidelines = obj.property::<bool>(Editor::SHOW_PROJECT_GUIDELINES);
        let show_metrics_guidelines = obj.property::<bool>(Editor::SHOW_METRICS_GUIDELINES);
        let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
        let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
        let mouse = viewport.get_mouse();
        let UnitPoint(unit_mouse) = viewport.view_to_unit_point(mouse);
        cr1.set_line_width(
            obj.app_settings()
                .property::<f64>(Settings::GUIDELINE_WIDTH)
                / (scale * ppu),
        );
        let state_ref = state.borrow();
        for (show_origin, g) in state_ref
            .glyph
            .borrow()
            .guidelines
            .iter()
            .filter(|_| show_glyph_guidelines)
            .map(|g| (true, g))
            .chain(
                obj.project()
                    .guidelines
                    .borrow()
                    .iter()
                    .filter(|_| show_project_guidelines)
                    .map(|g| (false, g)),
            )
            .chain(
                obj.project()
                    .metric_guidelines
                    .borrow()
                    .iter()
                    .filter(|_| show_metrics_guidelines)
                    .map(|g| (false, g)),
            )
        {
            let highlight = g.on_line_query(unit_mouse, None);
            {
                let cr2 = cr1.push();
                cr2.transform(matrix);
                g.draw(cr2, (width, height), highlight, show_origin);
            }
            if highlight {
                cr1.move_to(mouse.0.x, mouse.0.y);
                let line_height = cr1.text_extents("Guideline").unwrap().height * 1.5;
                cr1.show_text("Guideline").unwrap();
                for (i, line) in [
                    format!("Name: {}", g.name().as_deref().unwrap_or("-")),
                    format!("Identifier: {}", g.identifier().as_deref().unwrap_or("-")),
                    format!("Point: ({:.2}, {:.2})", g.x(), g.y()),
                    format!("Angle: {:01}deg", g.angle()),
                ]
                .into_iter()
                .enumerate()
                {
                    cr1.move_to(mouse.0.x, mouse.0.y + (i + 1) as f64 * line_height);
                    cr1.show_text(&line).unwrap();
                }
            } else if g.angle() == 0.0 {
                let cr2 = cr1.push();
                cr2.set_source_color_alpha(Color::from_hex("#bbbaae"));
                let ViewPoint(Point { y, .. }) =
                    viewport.unit_to_view_point(UnitPoint((0.0, g.y()).into()));
                let label = if let Some(name) = g.name().as_deref() {
                    format!("{name} ({:00})", g.y().ceil())
                } else {
                    format!("{}", g.y().ceil())
                };
                let extents = cr2.text_extents(&label).unwrap();
                cr2.move_to(width - 2.5 - extents.width, y - 0.5);
                cr2.show_text(&label).unwrap();
            }
            cr1.stroke().unwrap();
        }
    }
    Inhibit(false)
}

impl EditorInner {
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
            .pre_layers
            .borrow()
            .iter()
            .chain(self.viewport.layers.borrow().iter())
            .chain(self.viewport.post_layers.borrow().iter())
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
