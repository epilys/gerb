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
struct BreadcrumbWidgets {
    flow_box: gtk::FlowBox,
}

#[derive(Debug, Default)]
pub struct BreadcrumbsInner {
    widgets: OnceCell<BreadcrumbWidgets>,
}

#[glib::object_subclass]
impl ObjectSubclass for BreadcrumbsInner {
    const NAME: &'static str = "BreadcrumbsInner";
    type Type = Breadcrumbs;
    type ParentType = gtk::Box;
}

impl ObjectImpl for BreadcrumbsInner {
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
            .set(BreadcrumbWidgets { flow_box })
            .expect("Failed to initialize BreadcrumbsInner state");
    }
}

impl BreadcrumbsInner {}

impl WidgetImpl for BreadcrumbsInner {}
impl ContainerImpl for BreadcrumbsInner {}
impl BoxImpl for BreadcrumbsInner {}

glib::wrapper! {
    pub struct Breadcrumbs(ObjectSubclass<BreadcrumbsInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl Breadcrumbs {
    pub fn new(stack: &gtk::Stack) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Breadcrumbs");
        stack.connect_add(clone!(@strong ret => move |_self_, new_widget| {
            println!("added {:?}", new_widget);
            let widg = gtk::Button::builder()
                .label("overview")
                .expand(true)
                .visible(true)
                .build();
            ret.imp().widgets.get().unwrap().flow_box.add(&widg);

        }));
        stack.connect_remove(clone!(@strong ret => move |_self_, removed_widget| {
            println!("removed {:?}", removed_widget);

        }));
        /*ret.imp()
            .app
            .set(app.upcast_ref::<gtk::Application>().clone())
            .unwrap();
        */

        ret
    }
}
