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

#[derive(Copy, Clone)]
pub struct UIIcon {
    svg: &'static [u8],
    png: &'static [u8],
}

pub const G_GLYPH: UIIcon = UIIcon {
    svg: include_bytes!("./resources/g.svg"),
    png: include_bytes!("./resources/g.png"),
};

pub const GRAB_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/grab-icon-small.svg"),
    png: include_bytes!("./resources/grab-icon.png"),
};

pub const PEN_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/pen-icon.svg"),
    png: include_bytes!("./resources/pen-icon.png"),
};

pub const ZOOM_IN_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/zoom-in-icon.svg"),
    png: include_bytes!("./resources/zoom-in-icon.png"),
};

pub const ZOOM_OUT_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/zoom-out-icon.svg"),
    png: include_bytes!("./resources/zoom-out-icon.png"),
};

pub const BEZIER_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/bezier-icon.svg"),
    png: include_bytes!("./resources/bezier-icon.png"),
};

pub const BSPLINE_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/b-spline-icon.svg"),
    png: include_bytes!("./resources/b-spline-icon.png"),
};

pub const RECTANGLE_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/rectangle-icon.svg"),
    png: include_bytes!("./resources/rectangle-icon.png"),
};

pub const ELLIPSE_ICON: UIIcon = UIIcon {
    svg: include_bytes!("./resources/ellipse-icon.svg"),
    png: include_bytes!("./resources/ellipse-icon.png"),
};

pub const PEN_CURSOR: UIIcon = UIIcon {
    svg: include_bytes!("./resources/pen-cursor.svg"),
    png: include_bytes!("./resources/pen-cursor.png"),
};
pub const RECTANGLE_CURSOR: UIIcon = UIIcon {
    svg: include_bytes!("./resources/rectangle-cursor.svg"),
    png: include_bytes!("./resources/rectangle-cursor.png"),
};
pub const CIRCLE_CURSOR: UIIcon = UIIcon {
    svg: include_bytes!("./resources/circle-cursor.svg"),
    png: include_bytes!("./resources/circle-cursor.png"),
};

pub const ARROW_CURSOR: UIIcon = UIIcon {
    svg: include_bytes!("./resources/arrow-cursor.svg"),
    png: include_bytes!("./resources/arrow-cursor.png"),
};

pub const ARROW_PLUS_CURSOR: UIIcon = UIIcon {
    svg: include_bytes!("./resources/arrow-plus-cursor.svg"),
    png: include_bytes!("./resources/arrow-plus-cursor.png"),
};

pub const ARROW_MINUS_CURSOR: UIIcon = UIIcon {
    svg: include_bytes!("./resources/arrow-minus-cursor.svg"),
    png: include_bytes!("./resources/arrow-minus-cursor.png"),
};

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
