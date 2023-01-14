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

use gtk::cairo::{Context, FontSlant, FontWeight};
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Debug, Default)]
pub struct MinimapInner {}

#[glib::object_subclass]
impl ObjectSubclass for MinimapInner {
    const NAME: &'static str = "Minimap";
    type Type = Minimap;
    type ParentType = gtk::DrawingArea;
}

impl ObjectImpl for MinimapInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_tooltip_text(Some("pangram minimap"));
        obj.set_visible(true);
        obj.set_expand(true);

        obj.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |_drar: &Minimap, cr: &Context| {
            const PANGRAM: &str = "A wizard's job is to vex chumps quickly in fog.";
            let (red, green, blue) = crate::utils::hex_color_to_rgb("#959595").unwrap();
            cr.set_source_rgb(red, green, blue);
            cr.paint().expect("Invalid cairo surface state");
            cr.select_font_face("Inter", FontSlant::Normal, FontWeight::Normal);
            cr.set_source_rgb(1., 1., 1.);
            cr.set_font_size(8.);
            let (x, mut y) = (2., 15.);
            cr.move_to(x, y);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 10.;
            cr.move_to(x, y);
            cr.set_font_size(14.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 10.;
            cr.move_to(x, y);
            cr.set_font_size(20.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 10.;
            cr.move_to(x, y);
            cr.set_font_size(32.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 25.;
            cr.move_to(x, y);
            cr.set_font_size(64.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height;
            cr.move_to(x, y);
            Inhibit(false)
        }));
        obj.style_context().add_class("project-minimap");
    }
}

impl MinimapInner {}

impl DrawingAreaImpl for MinimapInner {}
impl WidgetImpl for MinimapInner {}

glib::wrapper! {
    pub struct Minimap(ObjectSubclass<MinimapInner>)
        @extends gtk::DrawingArea, gtk::Widget;
}

impl Minimap {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Minimap");
        ret
    }
}
