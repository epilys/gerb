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
pub mod shortcuts;
pub mod widgets;
pub use colors::*;
pub use points::{CurvePoint, IPoint, Point};

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
            .label(&format!(
                "<big>Options for <i>{}</i></big>",
                obj.type_().name()
            ))
            .use_markup(true)
            .margin_top(5)
            .halign(gtk::Align::Start)
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
            .margin_bottom(10)
            .valign(gtk::Align::Start)
            .build(),
        0,
        1,
        2,
        1,
    );
    let mut row: i32 = 2;
    for prop in obj.list_properties().as_slice().iter().filter(|p| {
        p.flags()
            .contains(glib::ParamFlags::READWRITE | glib::ParamFlags::USER_1)
            && p.owner_type() == obj.type_()
    }) {
        grid.attach(
            &gtk::Label::builder()
                .label(&{
                    let blurb = prop.blurb();
                    let name = prop.name();
                    let type_name: &str = match prop.value_type().name() {
                        "gboolean" => "bool",
                        "gchararray" => "string",
                        "guint64"|"gint64" => "int",
                        "gdouble"=> "float",
                        "Color" => "color",
                        "DrawOptions" => "theme options",
                        _other => _other,
                    };
                    if blurb == name {
                        format!("Key: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
                    } else {
                        format!("<span insert_hyphens=\"true\" allow_breaks=\"true\" foreground=\"#222222\">{blurb}</span>\n\nKey: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
                    }
                })
                .visible(true)
                .selectable(true)
                .wrap_mode(gtk::pango::WrapMode::Char)
                .use_markup(true)
                .max_width_chars(30)
                .halign(gtk::Align::Start)
                .wrap(true)
                .build(),
            0,
            row,
            1,
            1,
        );
        grid.attach(&get_widget_for_value(&obj, prop), 1, row, 1, 1);
        grid.attach(
            &gtk::Separator::builder()
                .expand(true)
                .visible(true)
                .vexpand(false)
                .valign(gtk::Align::Start)
                .build(),
            0,
            row + 1,
            2,
            1,
        );
        row += 2;
    }
    if row != 2 {
        grid.remove_row(row - 1);
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
            let entry = widgets::ToggleButton::new();
            entry.set_visible(true);
            entry.set_active(val);
            entry.set_sensitive(readwrite);
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Start);
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
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Center)
                .build();
            entry.buffer().set_text(&val);
            obj.bind_property(property.name(), &entry.buffer(), "text")
                .flags(flags)
                .build();

            entry.upcast()
        }
        "gint64" => {
            let val = val.get::<i64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecInt64>() {
                (spec.minimum(), spec.maximum())
            } else {
                (i64::MIN, i64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    val as f64, min as f64, max as f64, 1.00, 1.00, 1.00,
                )),
                1.0,
                0,
            );
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Center);
            entry.set_input_purpose(gtk::InputPurpose::Digits);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
                .transform_to(|_, value| {
                    let val = value.get::<i64>().ok()?;
                    Some((val as f64).to_value())
                })
                .transform_from(|_, value| {
                    let val = value.get::<f64>().ok()?;
                    Some((val as i64).to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "guint64" => {
            let val = val.get::<u64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecUInt64>() {
                (0, spec.maximum())
            } else {
                (0, u64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    val as f64, min as f64, max as f64, 1.00, 1.00, 1.00,
                )),
                1.0,
                0,
            );
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Center);
            entry.set_input_purpose(gtk::InputPurpose::Digits);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
                .transform_to(|_, value| {
                    let val = value.get::<u64>().ok()?;
                    Some((val as f64).to_value())
                })
                .transform_from(|_, value| {
                    let val = value.get::<f64>().ok()?;
                    Some((val as u64).to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "gdouble" => {
            let val = val.get::<f64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecDouble>() {
                (spec.minimum(), spec.maximum())
            } else {
                (f64::MIN, f64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(val, min, max, 0.05, 0.01, 0.01)),
                1.0,
                2,
            );
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Center);
            entry.set_input_purpose(gtk::InputPurpose::Number);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
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
                .use_alpha(true)
                .show_editor(true)
                .build();
            entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let new_val = self_.rgba();
                _ = obj.try_set_property::<Color>(property.name(), new_val.into());
            }));
            entry.upcast()
        }
        "DrawOptions" => {
            let opts = val.get::<DrawOptions>().unwrap();
            let grid = gtk::Grid::builder()
                .expand(true)
                .visible(true)
                .sensitive(readwrite)
                .column_spacing(5)
                .margin(10)
                .row_spacing(5)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            let has_bg = opts.bg.is_some();

            let fg_entry = gtk::ColorButton::builder()
                .rgba(&opts.color.into())
                .sensitive(readwrite)
                .visible(true)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .use_alpha(true)
                .show_editor(true)
                .build();
            fg_entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let opts = obj.property::<DrawOptions>(property.name());
                let new_val = self_.rgba();
                _ = obj.try_set_property::<DrawOptions>(property.name(), DrawOptions { color: new_val.into(), ..opts });
            }));
            grid.attach(
                &gtk::Label::builder()
                    .label(if has_bg { "fg color" } else { "color" })
                    .halign(gtk::Align::End)
                    .visible(true)
                    .build(),
                0,
                0,
                1,
                1,
            );
            grid.attach(&fg_entry, 1, 0, 1, 1);
            if let Some(bg) = opts.bg {
                let bg_entry = gtk::ColorButton::builder()
                    .rgba(&bg.into())
                    .sensitive(readwrite)
                    .visible(true)
                    .halign(gtk::Align::Fill)
                    .valign(gtk::Align::Start)
                    .use_alpha(true)
                    .show_editor(true)
                    .build();
                bg_entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                    let opts = obj.property::<DrawOptions>(property.name());
                    let new_val = self_.rgba();
                    _ = obj.try_set_property::<DrawOptions>(property.name(), DrawOptions { bg: Some(new_val.into()), ..opts });
                }));
                grid.attach(
                    &gtk::Label::builder()
                        .label("bg color")
                        .visible(true)
                        .halign(gtk::Align::End)
                        .build(),
                    0,
                    1,
                    1,
                    1,
                );
                grid.attach(&bg_entry, 1, 1, 1, 1);
            }
            let listbox = gtk::ListBox::builder()
                .visible(true)
                .expand(true)
                .sensitive(readwrite)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            let size_entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    opts.size,
                    0.0,
                    f64::MAX,
                    0.05,
                    0.01,
                    0.01,
                )),
                1.0,
                2,
            );
            size_entry.set_halign(gtk::Align::Start);
            size_entry.set_valign(gtk::Align::Center);
            size_entry.set_input_purpose(gtk::InputPurpose::Number);
            size_entry.set_sensitive(readwrite);
            size_entry.set_visible(true);
            size_entry.connect_value_notify(clone!(@weak obj, @strong property => move |self_| {
                let opts = obj.property::<DrawOptions>(property.name());
                let size = self_.value();
                obj.set_property(property.name(), DrawOptions { size, ..opts });
            }));
            obj.bind_property(property.name(), &size_entry, "value")
                .transform_to(|_, value| {
                    let opts = value.get::<DrawOptions>().ok()?;
                    Some(opts.size.to_value())
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            listbox.add(&size_entry);
            if let Some((from, val)) = opts.inherit_size {
                if val {
                    size_entry.set_sensitive(false);
                }
                let inherit_entry = widgets::ToggleButton::new();
                inherit_entry.set_label("Inherit global value");
                inherit_entry.set_visible(true);
                inherit_entry.set_active(val);
                inherit_entry.set_relief(gtk::ReliefStyle::None);
                inherit_entry.set_sensitive(readwrite);
                inherit_entry.set_halign(gtk::Align::Start);
                inherit_entry.set_valign(gtk::Align::Center);
                obj.bind_property(property.name(), &inherit_entry, "active")
                    .transform_to(|_, value| {
                        let opts = value.get::<DrawOptions>().ok()?;
                        opts.inherit_size.map(|(_, b)| b.to_value())
                    })
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                let inherit_value = gtk::Label::builder()
                    .label(&format!("{:.2}", obj.property::<f64>(from)))
                    .visible(val)
                    .width_chars(5)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Center)
                    .sensitive(false)
                    .wrap(true)
                    .build();
                inherit_entry.connect_clicked(clone!(@weak obj, @strong property, @weak inherit_value, @weak size_entry => move |_| {
                    let opts = obj.property::<DrawOptions>(property.name());
                    if let Some((from, b)) = opts.inherit_size {
                        inherit_value.set_visible(!b);
                        size_entry.set_sensitive(b);
                        obj.set_property(property.name(), DrawOptions { inherit_size: Some((from, !b)), ..opts });
                    }
                }));
                let inherit_box = gtk::Box::builder()
                    .visible(true)
                    .expand(true)
                    .sensitive(readwrite)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Start)
                    .orientation(gtk::Orientation::Horizontal)
                    .build();
                inherit_box.add(
                    &gtk::ListBoxRow::builder()
                        .child(&inherit_entry)
                        .activatable(false)
                        .selectable(false)
                        .visible(true)
                        .build(),
                );
                inherit_box.add(
                    &gtk::ListBoxRow::builder()
                        .child(&inherit_value)
                        .activatable(false)
                        .selectable(false)
                        .visible(true)
                        .build(),
                );
                listbox.add(&inherit_box);
            }
            listbox.set_selection_mode(gtk::SelectionMode::None);
            grid.attach(
                &gtk::Label::builder()
                    .label("width/length")
                    .visible(true)
                    .build(),
                0,
                if has_bg { 2 } else { 1 },
                1,
                1,
            );
            grid.attach(&listbox, 1, if has_bg { 2 } else { 1 }, 1, 1);
            grid.upcast()
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
