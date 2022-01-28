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

use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

#[derive(Debug)]
struct TabInfoWidgets {
    flow_box: gtk::FlowBox,
}

#[derive(Debug, Default)]
pub struct TabInfoInner {
    widgets: OnceCell<TabInfoWidgets>,
}

#[glib::object_subclass]
impl ObjectSubclass for TabInfoInner {
    const NAME: &'static str = "TabInfoInner";
    type Type = TabInfo;
    type ParentType = gtk::Box;
}

impl ObjectImpl for TabInfoInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let flow_box = gtk::FlowBox::builder().expand(true).visible(true).build();
        //flow_box.add(&gtk::Separator::builder().expand(false).visible(true).build());
        /*let edit = gtk::Button::builder()
            .label("edit")
            .expand(true)
            .visible(true)
            .build();
        flow_box.add(&edit);
        */
        obj.pack_start(&flow_box, true, true, 0);
        obj.set_visible(true);
        obj.set_expand(true);
        self.widgets
            .set(TabInfoWidgets { flow_box })
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

impl TabInfo {
    pub fn new(notebook: &gtk::Notebook) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create TabInfo");
        notebook.connect_switch_page(clone!(@strong ret => move |_self_, _page_widget, _page| {
            //println!("switched {:?} {}", page_widget, page);
            //ret.imp().widgets.get().unwrap().flow_box.add(&widg);

        }));
        /*ret.imp()
            .app
            .set(app.upcast_ref::<gtk::Application>().clone())
            .unwrap();
        */

        ret
    }
}
