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
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

#[derive(Debug)]
struct TabInfoWidgets {
    grid: gtk::Grid,
}

#[derive(Debug, Default)]
pub struct TabInfoInner {
    widgets: OnceCell<TabInfoWidgets>,
}

#[glib::object_subclass]
impl ObjectSubclass for TabInfoInner {
    const NAME: &'static str = "TabInfo";
    type Type = TabInfo;
    type ParentType = gtk::Box;
}

impl ObjectImpl for TabInfoInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .margin_top(5)
            .margin_start(5)
            .build();
        let grid = gtk::Grid::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .column_spacing(5)
            .row_spacing(5)
            .build();

        scrolled_window.set_child(Some(&grid));
        //flow_box.add(&gtk::Separator::builder().expand(false).visible(true).build());
        /*let edit = gtk::Button::builder()
            .label("edit")
            .expand(true)
            .visible(true)
            .build();
        flow_box.add(&edit);
        */
        obj.pack_start(&scrolled_window, true, true, 0);
        obj.set_visible(true);
        obj.set_expand(true);
        self.widgets
            .set(TabInfoWidgets { grid })
            .expect("Failed to initialize TabInfoInner state");
    }
}

impl TabInfoInner {}

impl WidgetImpl for TabInfoInner {}
impl ContainerImpl for TabInfoInner {}
impl BoxImpl for TabInfoInner {}

glib::wrapper! {
    pub struct TabInfo(ObjectSubclass<TabInfoInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl Default for TabInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl TabInfo {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create TabInfo");
        /*
        notebook.connect_switch_page(clone!(@strong ret => move |_self_, _page_widget, _page| {
            //println!("switched {:?} {}", page_widget, page);
            //ret.imp().widgets.get().unwrap().flow_box.add(&widg);

        }));
        ret.imp()
            .app
            .set(app)
            .unwrap();
        */
        ret
    }

    pub fn get_widget_for_value(obj: &glib::Object, property: &str) -> gtk::Widget {
        let val: glib::Value = obj.property(property);
        match val.type_().name() {
            "gboolean" => {
                let val = val.get::<bool>().unwrap();
                let entry = gtk::CheckButton::builder()
                    .visible(true)
                    .active(val)
                    .build();
                entry
                    .bind_property("active", obj, property)
                    .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                    .build();

                entry.upcast()
            }
            "gchararray" => {
                let val = val.get::<Option<String>>().unwrap().unwrap_or_default();
                let entry = gtk::Entry::builder().visible(true).build();
                entry.buffer().set_text(&val);
                entry
                    .buffer()
                    .bind_property("text", obj, property)
                    .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                    .build();

                entry.upcast()
            }
            "gint64" => {
                let val = val.get::<i64>().unwrap();
                let entry = gtk::Entry::builder()
                    .input_purpose(gtk::InputPurpose::Number)
                    .visible(true)
                    .build();
                entry.buffer().set_text(&val.to_string());
                entry
                    .buffer()
                    .bind_property("text", obj, property)
                    .transform_to(|_, value| {
                        let number = value.get::<String>().ok()?;
                        Some(number.parse::<i64>().ok()?.to_value())
                    })
                    .transform_from(|_, value| {
                        let number = value.get::<i64>().ok()?;
                        Some(number.to_string().to_value())
                    })
                    .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                    .build();
                entry.upcast()
            }
            "gdouble" => {
                let val = val.get::<f64>().unwrap();
                let entry = gtk::Entry::builder()
                    .input_purpose(gtk::InputPurpose::Number)
                    .visible(true)
                    .build();
                entry.buffer().set_text(&val.to_string());
                entry
                    .buffer()
                    .bind_property("text", obj, property)
                    .transform_to(|_, value| {
                        let number = value.get::<String>().ok()?;
                        Some(number.parse::<f64>().ok()?.to_value())
                    })
                    .transform_from(|_, value| {
                        let number = value.get::<f64>().ok()?;
                        Some(number.to_string().to_value())
                    })
                    .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                    .build();
                entry.upcast()
            }
            _other => gtk::Label::builder()
                .label(&format!("{:?}", val))
                .visible(true)
                .build()
                .upcast(),
        }
    }

    pub fn set_object(&self, new_obj: Option<glib::Object>) {
        if let Some(obj) = new_obj {
            self.set_visible(true);
            let grid = self.imp().widgets.get().unwrap().grid.clone();
            let children = grid.children();
            for c in children {
                grid.remove(&c);
            }
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
            for (row, prop) in obj.list_properties().as_slice().iter().enumerate() {
                grid.attach(
                    &gtk::Label::builder()
                        .label(prop.name())
                        .visible(true)
                        .build(),
                    0,
                    row as i32 + 1,
                    1,
                    1,
                );
                //let val: glib::Value = std::dbg!(obj.property(prop.name()));
                grid.attach(
                    &Self::get_widget_for_value(&obj, prop.name()),
                    1,
                    row as i32 + 1,
                    1,
                    1,
                );
            }
            grid.queue_draw();
        } else {
            self.set_visible(false);
        }
        self.queue_draw();
    }
}
