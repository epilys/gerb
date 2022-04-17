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

use gtk::cairo::Context;
use std::f64::consts::PI;

pub mod curves;
pub mod range_query;

pub const CODEPOINTS: &str = r##"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;

pub type Point = (f64, f64);

pub fn draw_round_rectangle(
    cr: &Context,
    (x, y): (f64, f64),
    (width, height): (f64, f64),
    aspect_ratio: f64,
    line_width: f64,
) -> (Point, Point) {
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
        (x + line_width, y + line_width),
        (width - 2. * line_width, height - 2. * line_width),
    )
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

pub fn distance_between_two_points<K: Into<(i64, i64)>, L: Into<(i64, i64)>>(
    p_k: K,
    p_l: L,
) -> f64 {
    let p_k = p_k.into();
    let p_l = p_l.into();
    let (x_k, y_k) = p_k;
    let (x_l, y_l) = p_l;
    let xlk = x_l - x_k;
    let ylk = y_l - y_k;
    f64::sqrt((xlk * xlk + ylk * ylk) as f64) // FIXME overflow check
}
