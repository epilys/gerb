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

use glib::clone;
use gtk::cairo::{Context, FontSlant, FontWeight};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::Cell;

use crate::project::Glyph;

#[derive(Debug, Default)]
pub struct GlyphEditArea {
    glyph: OnceCell<Glyph>,
    drawing_area: OnceCell<gtk::DrawingArea>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphEditArea {
    const NAME: &'static str = "GlyphEditArea";
    type Type = GlyphEditView;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GlyphEditArea {
    // Here we are overriding the glib::Objcet::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let drawing_area = gtk::DrawingArea::builder()
            .expand(true)
            .visible(true)
            .build();
        drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |drar: &gtk::DrawingArea, cr: &gtk::cairo::Context| {
            cr.scale(500f64, 500f64);
            cr.set_source_rgb(1., 1., 1.);
            cr.paint().expect("Invalid cairo surface state");
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.set_line_width(0.0005);
            for x in (0..100).into_iter().step_by(5) {
                let x = x as f64 / 100.;
                cr.move_to(x, 0.);
                cr.line_to(x, 1.);
                cr.stroke().expect("Invalid cairo surface state");
            }
            for y in (0..100).into_iter().step_by(5) {
                let y = y as f64 / 100.;
                cr.move_to(0., y);
                cr.line_to(1., y);
                cr.stroke().expect("Invalid cairo surface state");
            }
            /*
            cr.set_source_rgb(0., 0., 0.);

            cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
            cr.set_font_size(0.35);

            cr.move_to(0.04, 0.53);
            cr.show_text("Hello").expect("Invalid cairo surface state");

            cr.move_to(0.27, 0.65);
            cr.text_path("void");
            cr.set_source_rgb(0.5, 0.5, 1.0);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(0.01);
            cr.stroke().expect("Invalid cairo surface state");

            cr.set_source_rgba(1.0, 0.2, 0.2, 0.6);
            cr.arc(0.04, 0.53, 0.02, 0.0, PI * 2.);
            cr.arc(0.27, 0.65, 0.02, 0.0, PI * 2.);
            cr.fill().expect("Invalid cairo surface state");
            */

            if let Some(glyph) = obj.imp().glyph.get() {
                println!("cairo drawing glyph {}", glyph.name);
                glyph.draw(drar, cr);
            } else {
                println!("cairo drawing without glyph");
            }

            Inhibit(false)
        }
        ));
        obj.add(&drawing_area);

        self.drawing_area
            .set(drawing_area)
            .expect("Failed to initialize window state");
    }
}

impl WidgetImpl for GlyphEditArea {}
impl ContainerImpl for GlyphEditArea {}
impl BinImpl for GlyphEditArea {}

glib::wrapper! {
    pub struct GlyphEditView(ObjectSubclass<GlyphEditArea>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl GlyphEditView {
    pub fn new(glyph: Glyph) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp().glyph.set(glyph).unwrap();
        ret
    }
}
