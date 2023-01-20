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

use gtk::prelude::WidgetExt;

pub const SELECT_ICON_SVG: &str = include_str!("./resources/select-icon-small.svg");

pub const GRAB_ICON_SVG: &str = include_str!("./resources/grab-icon-small.svg");

pub const PEN_ICON_SVG: &str = include_str!("./resources/pen-icon.svg");

pub const ZOOM_IN_ICON_SVG: &str = include_str!("./resources/zoom-in-icon.svg");

pub const ZOOM_OUT_ICON_SVG: &str = include_str!("./resources/zoom-out-icon.svg");

pub const BEZIER_ICON_SVG: &str = include_str!("./resources/bezier-icon.svg");

pub const BSPLINE_ICON_SVG: &str = include_str!("./resources/b-spline-icon.svg");

pub const RECTANGLE_ICON_SVG: &str = include_str!("./resources/rectangle-icon.svg");

pub const ELLIPSE_ICON_SVG: &str = include_str!("./resources/ellipse-icon.svg");

pub const PEN_CURSOR_SVG: &str = include_str!("./resources/pen-cursor.svg");
pub const RECTANGLE_CURSOR_SVG: &str = include_str!("./resources/rectangle-cursor.svg");
pub const CIRCLE_CURSOR_SVG: &str = include_str!("./resources/circle-cursor.svg");

pub fn svg_to_image_widget(svg: &'static str) -> gtk::Image {
    if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(svg.as_bytes()) {
        let pixbuf = pixbuf
            .scale_simple(24, 24, gtk::gdk_pixbuf::InterpType::Tiles)
            .unwrap();
        let image = gtk::Image::from_pixbuf(Some(&pixbuf));
        image.set_visible(true);
        image
    } else {
        println!("Failed to load SVG as pixbuf.");
        gtk::Image::default()
    }
}

pub fn svg_to_pixbuf(svg: &'static str) -> Option<gtk::gdk_pixbuf::Pixbuf> {
    if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(svg.as_bytes()) {
        Some(pixbuf)
    } else {
        println!("Failed to load SVG as pixbuf.");
        None
    }
}
