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
use gtk::glib;
use gtk::prelude::*;
use std::f64::consts::PI;

pub mod colors;
pub mod curves;
pub mod menu;
pub mod points;
pub mod range_query;
pub use colors::*;
pub use points::{IPoint, Point};

pub const CODEPOINTS: &str = r##"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;

pub const UI_EDITABLE: glib::ParamFlags = glib::ParamFlags::USER_1;

pub fn draw_round_rectangle(
    cr: &Context,
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

pub fn object_to_property_grid(obj: glib::Object) -> gtk::Grid {
    let grid = gtk::Grid::builder()
        .expand(true)
        .visible(true)
        .can_focus(true)
        .column_spacing(5)
        .margin(10)
        .row_spacing(5)
        .build();
    grid.attach(
        &gtk::Label::builder()
            .label(obj.type_().name())
            .visible(true)
            .build(),
        0,
        0,
        1,
        1,
    );
    grid.attach(
        &gtk::Separator::builder()
            .expand(true)
            .visible(true)
            .vexpand(false)
            .valign(gtk::Align::Start)
            .build(),
        0,
        1,
        2,
        1,
    );
    for (row, prop) in obj
        .list_properties()
        .as_slice()
        .iter()
        .filter(|p| {
            p.flags()
                .contains(glib::ParamFlags::READWRITE | glib::ParamFlags::USER_1)
                && p.owner_type() == obj.type_()
        })
        .enumerate()
    {
        grid.attach(
            &gtk::Label::builder()
                .label(prop.name())
                .visible(true)
                .build(),
            0,
            row as i32 + 2,
            1,
            1,
        );
        grid.attach(&get_widget_for_value(&obj, prop), 1, row as i32 + 2, 1, 1);
    }
    grid
}

pub fn get_widget_for_value(obj: &glib::Object, property: &glib::ParamSpec) -> gtk::Widget {
    let val: glib::Value = obj.property(property.name());
    let readwrite = property.flags().contains(glib::ParamFlags::READWRITE);
    let flags = if readwrite {
        glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE
    } else {
        glib::BindingFlags::SYNC_CREATE
    };
    match val.type_().name() {
        "gboolean" => {
            let val = val.get::<bool>().unwrap();
            let entry = gtk::Switch::builder()
                .visible(true)
                .active(val)
                .sensitive(readwrite)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            obj.bind_property(property.name(), &entry, "active")
                .flags(flags)
                .build();

            entry.upcast()
        }
        "gchararray" => {
            let val = val.get::<Option<String>>().unwrap().unwrap_or_default();
            let entry = gtk::Entry::builder()
                .visible(true)
                .sensitive(readwrite)
                .build();
            entry.buffer().set_text(&val);
            obj.bind_property(property.name(), &entry.buffer(), "text")
                .flags(flags)
                .build();

            entry.upcast()
        }
        "gint64" => {
            let val = val.get::<i64>().unwrap();
            let entry = gtk::Entry::builder()
                .sensitive(readwrite)
                .input_purpose(gtk::InputPurpose::Number)
                .visible(true)
                .build();
            entry.buffer().set_text(&val.to_string());
            obj.bind_property(property.name(), &entry.buffer(), "text")
                .transform_from(|_, value| {
                    let number = value.get::<String>().ok()?;
                    Some(number.parse::<i64>().ok()?.to_value())
                })
                .transform_to(|_, value| {
                    let number = value.get::<i64>().ok()?;
                    Some(number.to_string().to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "guint64" => {
            let val = val.get::<u64>().unwrap();
            let entry = gtk::Entry::builder()
                .sensitive(readwrite)
                .input_purpose(gtk::InputPurpose::Number)
                .visible(true)
                .build();
            entry.buffer().set_text(&val.to_string());
            obj.bind_property(property.name(), &entry.buffer(), "text")
                .transform_from(|_, value| {
                    let number = value.get::<String>().ok()?;
                    Some(number.parse::<u64>().ok()?.to_value())
                })
                .transform_to(|_, value| {
                    let number = value.get::<u64>().ok()?;
                    Some(number.to_string().to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "gdouble" => {
            let val = val.get::<f64>().unwrap();
            let entry = gtk::Entry::builder()
                .input_purpose(gtk::InputPurpose::Number)
                .sensitive(readwrite)
                .visible(true)
                .build();
            entry.buffer().set_text(&val.to_string());
            obj.bind_property(property.name(), &entry.buffer(), "text")
                .transform_from(|_, value| {
                    let number = value.get::<String>().ok()?;
                    Some(number.parse::<f64>().ok()?.to_value())
                })
                .transform_to(|_, value| {
                    let number = value.get::<f64>().ok()?;
                    Some(number.to_string().to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "Color" => {
            let val = val.get::<Color>().unwrap();
            let entry = gtk::ColorButton::builder()
                .rgba(&val.into())
                .sensitive(readwrite)
                .visible(true)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .show_editor(true)
                .build();
            entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let new_val = self_.rgba();
                _ = obj.try_set_property::<Color>(property.name(), new_val.into());
            }));
            entry.upcast()
        }
        _other => gtk::Label::builder()
            .label(&format!("{:?}", val))
            .visible(true)
            .width_chars(5)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Start)
            .wrap(true)
            .build()
            .upcast(),
    }
}

pub fn new_property_window(obj: glib::Object, title: &str) -> gtk::Window {
    let w = gtk::Window::builder()
        .deletable(true)
        .destroy_with_parent(true)
        .focus_on_map(true)
        .resizable(true)
        .title(title)
        .visible(true)
        .expand(true)
        .default_width(640)
        .default_height(480)
        .build();
    let scrolled_window = gtk::ScrolledWindow::builder()
        .expand(true)
        .visible(true)
        .can_focus(true)
        .build();
    let grid = crate::utils::object_to_property_grid(obj);
    scrolled_window.set_child(Some(&grid));
    w.set_child(Some(&scrolled_window));
    w
}
