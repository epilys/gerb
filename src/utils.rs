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

use gtk::glib;
use std::f64::consts::PI;

pub mod colors;
pub mod curves;
pub mod menu;
pub mod points;
pub mod property_window;
pub mod range_query;
pub mod shortcuts;
pub mod widgets;
pub use colors::*;
pub use points::{CurvePoint, IPoint, Point};
pub use property_window::{get_widget_for_value, new_property_window, object_to_property_grid};

pub const CODEPOINTS: &str = r##"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;

pub const UI_EDITABLE: glib::ParamFlags = glib::ParamFlags::USER_1;
pub const UI_READABLE: glib::ParamFlags = glib::ParamFlags::USER_2;
pub const UI_PATH: glib::ParamFlags = glib::ParamFlags::USER_3;

pub fn draw_round_rectangle(
    cr: ContextRef,
    p: Point,
    (width, height): (f64, f64),
    aspect_ratio: f64,
    line_width: f64,
) -> (Point, (f64, f64)) {
    let (x, y) = (p.x, p.y);
    /*
       double x         = 25.6,        /* parameters like cairo_rectangle */
    y         = 25.6,
    width         = 204.8,
    height        = 204.8,
    aspect        = 1.0,     /* aspect ratio */
    */
    let corner_radius: f64 = height / 10.0; /* and corner curvature radius */

    let radius: f64 = corner_radius / aspect_ratio;
    let degrees: f64 = PI / 180.0;

    cr.move_to(x, y);
    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -90. * degrees,
        0. * degrees,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0. * degrees,
        90. * degrees,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        90. * degrees,
        180. * degrees,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        180. * degrees,
        270. * degrees,
    );
    cr.close_path();

    (
        (x + line_width, y + line_width).into(),
        (width - 2. * line_width, height - 2. * line_width),
    )
}

pub fn distance_between_two_points<K: Into<Point>, L: Into<Point>>(p_k: K, p_l: L) -> f64 {
    let p_k: Point = p_k.into();
    let p_l: Point = p_l.into();
    let xlk = p_l.x - p_k.x;
    let ylk = p_l.y - p_k.y;
    f64::sqrt((xlk * xlk + ylk * ylk) as f64) // FIXME overflow check
}

#[repr(transparent)]
pub struct ContextRef<'a, 'b: 'a>(&'a mut &'b gtk::cairo::Context);

impl std::ops::Deref for ContextRef<'_, '_> {
    type Target = gtk::cairo::Context;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Drop for ContextRef<'_, '_> {
    fn drop(&mut self) {
        self.0.restore().unwrap();
    }
}

impl<'c, 'a: 'c, 'b: 'a> ContextRef<'a, 'b> {
    pub fn save(&self) {}

    pub fn restore(&self) {}

    pub fn push(self: &'c mut ContextRef<'a, 'b>) -> ContextRef<'c, 'b> {
        self.0.save().unwrap();
        ContextRef(self.0)
    }
}

impl<'a, 'b> From<&'a mut &'b gtk::cairo::Context> for ContextRef<'a, 'b> {
    fn from(cr: &'a mut &'b gtk::cairo::Context) -> Self {
        cr.save().unwrap();
        ContextRef(cr)
    }
}

pub trait ContextExt: colors::ColorExt {
    fn push<'a, 'b: 'a>(self: &'a mut &'b Self) -> ContextRef<'a, 'b>;
}

impl<'a, 'b> ColorExt for ContextRef<'a, 'b> {
    fn set_source_color(&self, color: Color) {
        self.0.set_source_color(color)
    }

    fn set_source_color_alpha(&self, color: Color) {
        self.0.set_source_color_alpha(color)
    }

    fn set_draw_opts(&self, opts: DrawOptions) {
        self.0.set_draw_opts(opts)
    }

    fn show_text_with_bg(&self, text: &str, margin: f64, fg: Color, bg: Color) {
        self.0.show_text_with_bg(text, margin, fg, bg)
    }
}

impl ContextExt for gtk::cairo::Context {
    fn push<'a, 'b: 'a>(self: &'a mut &'b Self) -> ContextRef<'a, 'b> {
        self.save().unwrap();
        ContextRef(self)
    }
}

pub struct FieldRef<'container, T> {
    inner: std::cell::Ref<'container, T>,
}

impl<T> std::ops::Deref for FieldRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: std::fmt::Display> std::fmt::Display for FieldRef<'_, T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(fmt)
    }
}

impl<T: AsRef<A>, A> AsRef<A> for FieldRef<'_, T> {
    fn as_ref(&self) -> &A {
        self.inner.as_ref()
    }
}

impl<'a, T> From<std::cell::Ref<'a, T>> for FieldRef<'a, T> {
    fn from(inner: std::cell::Ref<'a, T>) -> Self {
        Self { inner }
    }
}
