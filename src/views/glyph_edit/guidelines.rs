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

pub fn draw_guidelines(
    viewport: &Canvas,
    cr: &gtk::cairo::Context,
    glyph_state: &Rc<RefCell<GlyphState>>,
) -> Inhibit {
    if viewport.property::<bool>(Canvas::SHOW_GUIDELINES) {
        let matrix = viewport.imp().transformation.matrix();
        let scale: f64 = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::SCALE);
        let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
        let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
        let matrix = viewport.imp().transformation.matrix();
        let ppu = viewport
            .imp()
            .transformation
            .property::<f64>(Transformation::PIXELS_PER_UNIT);
        let mouse = viewport.get_mouse();
        let UnitPoint(mouse) = viewport.view_to_unit_point(mouse);
        cr.set_line_width(2.5);
        let (width, height) = ((width * scale) * ppu, (height * scale) * ppu);
        for g in glyph_state.borrow().glyph.borrow().guidelines.iter() {
            let highlight = g.imp().on_line_query(mouse, None);
            g.imp().draw(cr, matrix, (width, height), highlight);
            if highlight {
                cr.move_to(mouse.x, mouse.y);
                let line_height = cr.text_extents("Guideline").unwrap().height * 1.5;
                cr.show_text("Guideline").unwrap();
                for (i, line) in [
                    format!(
                        "Name: {}",
                        g.name().as_ref().map(String::as_str).unwrap_or("-")
                    ),
                    format!(
                        "Identifier: {}",
                        g.identifier().as_ref().map(String::as_str).unwrap_or("-")
                    ),
                    format!("Point: ({}, {})", g.x(), g.y()),
                    format!("Angle: {:02}deg", g.angle()),
                ]
                .into_iter()
                .enumerate()
                {
                    cr.move_to(mouse.x, mouse.y + (i + 1) as f64 * line_height);
                    cr.show_text(&line).unwrap();
                }
            }
        }
    }
    Inhibit(false)
}
