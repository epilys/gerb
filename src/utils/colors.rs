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

use gtk::{gdk, glib};
use std::hash::Hash;

#[derive(Clone, Debug, Copy, Hash, glib::Boxed)]
#[boxed_type(name = "Color", nullable)]
#[repr(transparent)]
pub struct Color(gdk::RGBA);

impl Color {
    // Constants re-exports
    pub const BLACK: Self = Self(gdk::RGBA::BLACK);
    pub const BLUE: Self = Self(gdk::RGBA::BLUE);
    pub const GREEN: Self = Self(gdk::RGBA::GREEN);
    pub const RED: Self = Self(gdk::RGBA::RED);
    pub const WHITE: Self = Self(gdk::RGBA::WHITE);
}

impl Default for Color {
    fn default() -> Self {
        Self(gdk::RGBA::BLACK)
    }
}

pub fn hex_color_to_rgb(s: &str) -> Option<(f64, f64, f64)> {
    if s.starts_with('#')
        && s.len() == 7
        && s[1..].as_bytes().iter().all(|&b| {
            (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
        })
    {
        Some((
            u8::from_str_radix(&s[1..3], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&s[3..5], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&s[5..7], 16).ok()? as f64 / 255.0,
        ))
    } else if s.starts_with('#')
        && s.len() == 4
        && s[1..].as_bytes().iter().all(|&b| {
            (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
        })
    {
        Some((
            (17 * u8::from_str_radix(&s[1..2], 16).ok()?) as f64 / 255.0,
            (17 * u8::from_str_radix(&s[2..3], 16).ok()?) as f64 / 255.0,
            (17 * u8::from_str_radix(&s[3..4], 16).ok()?) as f64 / 255.0,
        ))
    } else {
        None
    }
}

pub trait ColorExt {
    fn set_source_color(&self, color: Color);
    fn set_source_color_alpha(&self, color: Color);
    fn show_text_with_bg(&self, text: &str, margin: f64, fg: Color, bg: Color);
}

impl ColorExt for gtk::cairo::Context {
    fn set_source_color(&self, color: Color) {
        self.set_source_rgb(color.0.red(), color.0.green(), color.0.blue());
    }

    fn set_source_color_alpha(&self, color: Color) {
        self.set_source_rgba(
            color.0.red(),
            color.0.green(),
            color.0.blue(),
            color.0.alpha(),
        );
    }

    fn show_text_with_bg(&self, text: &str, margin: f64, fg: Color, bg: Color) {
        let (x, y) = self.current_point().unwrap();
        let extents = self.text_extents(text).unwrap();
        self.save().unwrap();
        self.set_source_color(bg);
        self.rectangle(
            x - margin,
            y - extents.height - margin,
            extents.width + 2.0 * margin,
            extents.height + 2.0 * margin,
        );
        self.fill().unwrap();
        self.restore().unwrap();

        self.move_to(x, y);
        self.set_source_color(fg);
        self.show_text(text).unwrap();
    }
}
