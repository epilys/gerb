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

//! Static icons, images, etc.

use gtk::prelude::WidgetExt;

#[derive(Copy, Clone)]
pub struct UIIcon {
    svg: &'static [u8],
    png: &'static [u8],
}

macro_rules! decl_icon {
    ($ident:ident, $path:literal) => {
        pub const $ident: UIIcon = UIIcon {
            svg: include_bytes!(concat!($path, ".svg")),
            png: include_bytes!(concat!($path, ".png")),
        };
    };
}

decl_icon! { G_GLYPH, "./resources/g" }

impl UIIcon {
    pub fn to_image_widget(&self) -> gtk::Image {
        if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(self.svg) {
            let image = gtk::Image::from_pixbuf(Some(&pixbuf));
            image.set_visible(true);
            return image;
        }
        if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(self.png) {
            let image = gtk::Image::from_pixbuf(Some(&pixbuf));
            image.set_visible(true);
            return image;
        }

        println!("Failed to load UIIcon as pixbuf.");
        gtk::Image::default()
    }

    pub fn to_pixbuf(&self) -> Option<gtk::gdk_pixbuf::Pixbuf> {
        if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(self.svg) {
            return Some(pixbuf);
        }
        if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(self.png) {
            return Some(pixbuf);
        }
        println!("Failed to load SVG as pixbuf.");
        None
    }

    pub fn image_into_surface(
        image: &gtk::Image,
        scale_factor: i32,
        win: Option<gtk::gdk::Window>,
    ) {
        use gtk::gdk::prelude::GdkPixbufExt;
        use gtk::prelude::ImageExt;
        if let Some(mut pixbuf) = image.pixbuf() {
            if scale_factor == 1 {
                pixbuf = pixbuf
                    .scale_simple(32, 32, gtk::gdk_pixbuf::InterpType::Bilinear)
                    .unwrap();
            }
            let surf = pixbuf.create_surface(scale_factor, win.as_ref());
            if surf.is_some() {
                image.set_surface(surf.as_ref());
            }
        }
    }
}

pub mod icons {
    use super::UIIcon;

    decl_icon! {GRAB_ICON, "./resources/grab-icon"}
    decl_icon! {PEN_ICON, "./resources/pen-icon"}
    decl_icon! {ZOOM_IN_ICON, "./resources/zoom-in-icon"}
    decl_icon! {ZOOM_OUT_ICON, "./resources/zoom-out-icon"}
    decl_icon! {BEZIER_ICON, "./resources/bezier-icon"}
    decl_icon! {BSPLINE_ICON, "./resources/b-spline-icon"}
    decl_icon! {RECTANGLE_ICON, "./resources/rectangle-icon"}
    decl_icon! {ELLIPSE_ICON, "./resources/ellipse-icon"}
    decl_icon! {CHECKBOX_ICON, "./resources/icons/checkbox"}
    decl_icon! {CHECKBOX_CHECKED_ICON, "./resources/icons/checkbox-checked"}
    decl_icon! {RIGHT_MOUSE_BUTTON, "./resources/icons/right_mouse_button"}
    decl_icon! {LEFT_MOUSE_BUTTON, "./resources/icons/left_mouse_button"}
    decl_icon! {ESC_BUTTON, "./resources/icons/esc_button"}
    decl_icon! {MARK, "./resources/icons/mark"}
}

pub mod cursors {
    use super::UIIcon;

    decl_icon! {PEN_CURSOR, "./resources/pen-cursor"}
    decl_icon! {RECTANGLE_CURSOR, "./resources/rectangle-cursor"}
    decl_icon! {CIRCLE_CURSOR, "./resources/circle-cursor"}
    decl_icon! {ARROW_CURSOR, "./resources/arrow-cursor"}
    decl_icon! {ARROW_PLUS_CURSOR, "./resources/arrow-plus-cursor"}
    decl_icon! {ARROW_MINUS_CURSOR, "./resources/arrow-minus-cursor"}
}
