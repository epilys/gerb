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
    let UnitPoint(camera) = viewport.imp().transformation.camera();
    cr.save().unwrap();
    //cr.scale(scale, scale);
    cr.transform(matrix);
    cr.save().unwrap();
    cr.set_line_width(2.5);

    obj.imp().new_statusbar_message(&format!("Mouse: ({:.2}, {:.2}), Unit mouse: ({:.2}, {:.2}), Camera: ({:.2}, {:.2}), Size: ({width:.2}, {height:.2}), Scale: {scale:.2}", mouse.0.x, mouse.0.y, unit_mouse.0.x, unit_mouse.0.y, camera.x, camera.y));

    cr.restore().unwrap();
    //cr.transform(matrix);

    if viewport.property::<bool>(Canvas::SHOW_TOTAL_AREA) {
        /* Draw em square of units_per_em units: */
        cr.set_source_rgba(210. / 255., 227. / 255., 252. / 255., 0.6);
        cr.rectangle(
            0.,
            0.,
            glyph_state.glyph.borrow().width.unwrap_or(units_per_em),
            1000.0,
        );
        cr.fill().unwrap();
    }
    /* Draw the glyph */
    cr.move_to(0.0, 0.0);

    {
        let options = GlyphDrawingOptions {
            outline: (0.2, 0.2, 0.2, if inner_fill { 0. } else { 0.6 }),
            inner_fill: if inner_fill {
                Some((0., 0., 0., 1.))
            } else {
                None
            },
            highlight: if glyph_state.tool.can_highlight() {
                obj.imp().hovering.get()
            } else {
                None
            },
            matrix: Matrix::identity(),
            units_per_em,
            line_width: obj
                .imp()
                .settings
                .get()
                .unwrap()
                .property(Settings::LINE_WIDTH),
        };
        glyph_state.glyph.borrow().draw(cr, options);
    }

    if viewport.property::<bool>(Canvas::SHOW_HANDLES) {
        let handle_size: f64 = obj
            .imp()
            .settings
            .get()
            .unwrap()
            .property(Settings::HANDLE_SIZE);
        for cp in glyph_state.points.borrow().values() {
            let p = cp.position;
            if crate::utils::distance_between_two_points(p, unit_mouse.0) <= 10.0 {
                cr.set_source_rgba(1., 0., 0., 0.8);
            } else {
                if inner_fill {
                    cr.set_source_rgba(0.9, 0.9, 0.9, 1.0);
                } else {
                    cr.set_source_rgba(0.0, 0.0, 1.0, 0.5);
                }
            }
            match &cp.kind {
                Endpoint { .. } => {
                    cr.rectangle(
                        p.x - handle_size / (4.0 * ppu),
                        p.y - handle_size / (4.0 * ppu),
                        handle_size / (2.0 * ppu),
                        handle_size / (2.0 * ppu),
                    );
                    cr.stroke().unwrap();
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                    cr.rectangle(
                        p.x - handle_size / (4.0 * ppu),
                        p.y - handle_size / (4.0 * ppu),
                        handle_size / (2.0 * ppu),
                        handle_size / (2.0 * ppu) + 1.0,
                    );
                    cr.stroke().unwrap();
                }
                Handle { ref end_points } => {
                    cr.arc(
                        p.x,
                        p.y,
                        handle_size / (2.0 * ppu),
                        0.,
                        2.0 * std::f64::consts::PI,
                    );
                    cr.fill().unwrap();
                    for ep in end_points {
                        let ep = glyph_state.points.borrow()[ep].position;
                        cr.move_to(p.x, p.y);
                        cr.line_to(ep.x, ep.y);
                        cr.stroke().unwrap();
                    }
                    cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                    cr.arc(
                        p.x,
                        p.y,
                        handle_size / (2.0 * ppu) + 1.0,
                        0.,
                        2.0 * std::f64::consts::PI,
                    );
                    cr.stroke().unwrap();
                }
            }
        }
    }
    cr.restore().unwrap();

    Inhibit(false)
}

pub fn draw_guidelines(viewport: &Canvas, cr: &gtk::cairo::Context, obj: GlyphEditView) -> Inhibit {
    if viewport.property::<bool>(Canvas::SHOW_GUIDELINES) {
        let glyph_state = obj.imp().glyph_state.get().unwrap();
        let matrix = viewport.imp().transformation.matrix();
        let scale: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
        let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
        let ppu = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let mouse = viewport.get_mouse();
        let UnitPoint(unit_mouse) = viewport.view_to_unit_point(mouse);
        cr.set_line_width(2.5);
        let (width, height) = ((width * scale) * ppu, (height * scale) * ppu);
        let glyph_state_ref = glyph_state.borrow();
        for g in glyph_state_ref.glyph.borrow().guidelines.iter() {
            let highlight = if glyph_state_ref.tool.can_highlight() {
                g.imp().on_line_query(unit_mouse, None)
            } else {
                false
            };
            g.imp().draw(cr, matrix, (width, height), highlight);
            if highlight {
                cr.move_to(mouse.0.x, mouse.0.y);
                let line_height = cr.text_extents("Guideline").unwrap().height * 1.5;
                cr.show_text("Guideline").unwrap();
                for (i, line) in [
                    format!("Name: {}", g.name().as_deref().unwrap_or("-")),
                    format!("Identifier: {}", g.identifier().as_deref().unwrap_or("-")),
                    format!("Point: ({:.2}, {:.2})", g.x(), g.y()),
                    format!("Angle: {:02}deg", g.angle()),
                ]
                .into_iter()
                .enumerate()
                {
                    cr.move_to(mouse.0.x, mouse.0.y + (i + 1) as f64 * line_height);
                    cr.show_text(&line).unwrap();
                }
            }
        }
    }
    Inhibit(false)
}
