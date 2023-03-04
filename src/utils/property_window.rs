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

use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum PropertyWindowButtons {
    Modify {
        close: gtk::Button,
    },
    Create {
        cancel: gtk::Button,
        save: gtk::Button,
    },
}

impl Default for PropertyWindowButtons {
    fn default() -> Self {
        Self::Modify {
            close: gtk::Button::builder()
                .label("Close")
                .relief(gtk::ReliefStyle::None)
                .visible(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build(),
        }
    }
}

#[derive(Default, Debug)]
pub struct PropertyWindowInner {
    pub obj: OnceCell<glib::Object>,
    pub app: OnceCell<crate::prelude::Application>,
    pub grid: OnceCell<gtk::Grid>,
    pub buttons: OnceCell<PropertyWindowButtons>,
}

#[glib::object_subclass]
impl ObjectSubclass for PropertyWindowInner {
    const NAME: &'static str = "PropertyWindow";
    type Type = PropertyWindow;
    type ParentType = gtk::Window;
}

impl ObjectImpl for PropertyWindowInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_deletable(true);
        obj.set_destroy_with_parent(true);
        obj.set_focus_on_map(true);
        obj.set_resizable(true);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_type_hint(gtk::gdk::WindowTypeHint::Utility);
        obj.set_window_position(gtk::WindowPosition::CenterOnParent);
    }
}

impl WidgetImpl for PropertyWindowInner {}
impl ContainerImpl for PropertyWindowInner {}
impl BinImpl for PropertyWindowInner {}
impl WindowImpl for PropertyWindowInner {}

glib::wrapper! {
    pub struct PropertyWindow(ObjectSubclass<PropertyWindowInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window;
}

#[derive(Default, Copy, Clone, Debug)]
pub enum PropertyWindowType {
    #[default]
    Modify,
    Create,
}

pub struct PropertyWindowBuilder {
    obj: glib::Object,
    app: crate::prelude::Application,
    title: Cow<'static, str>,
    type_: PropertyWindowType,
}

impl PropertyWindow {
    pub fn builder(obj: glib::Object, app: &crate::prelude::Application) -> PropertyWindowBuilder {
        PropertyWindowBuilder::new(obj, app)
    }
}

impl PropertyWindowBuilder {
    pub fn new(obj: glib::Object, app: &crate::prelude::Application) -> Self {
        Self {
            obj,
            app: app.clone(),
            title: "".into(),
            type_: PropertyWindowType::default(),
        }
    }

    pub fn title(mut self, title: Cow<'static, str>) -> Self {
        self.title = title;
        self
    }

    pub fn type_(mut self, type_: PropertyWindowType) -> Self {
        self.type_ = type_;
        self
    }

    pub fn build(self) -> PropertyWindow {
        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .min_content_height(400)
            .min_content_width(400)
            .build();
        let grid = crate::utils::object_to_property_grid(
            self.obj.clone(),
            matches!(self.type_, PropertyWindowType::Create),
        );
        let b = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin(5)
            .margin_bottom(10)
            .visible(true)
            .build();
        b.pack_start(&grid, true, true, 0);

        let ret: PropertyWindow = glib::Object::new(&[]).unwrap();
        ret.set_transient_for(Some(&self.app.window));
        ret.set_attached_to(Some(&self.app.window));
        ret.set_application(Some(&self.app));
        ret.set_title(&self.title);
        ret.imp().grid.set(grid).unwrap();
        ret.imp()
            .buttons
            .set(match self.type_ {
                PropertyWindowType::Modify => {
                    let close = gtk::Button::builder()
                        .label("Close")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    b.pack_end(&close, false, false, 0);
                    close.connect_clicked(clone!(@weak ret => move |_| {
                        ret.close();
                    }));
                    PropertyWindowButtons::Modify { close }
                }
                PropertyWindowType::Create => {
                    let btns = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .margin(5)
                        .margin_bottom(10)
                        .visible(true)
                        .build();
                    let cancel = gtk::Button::builder()
                        .label("Cancel")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    cancel.connect_clicked(clone!(@weak ret => move |_| {
                        ret.close();
                    }));
                    let save = gtk::Button::builder()
                        .label("Save")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    btns.pack_end(&save, false, false, 5);
                    btns.pack_end(&cancel, false, false, 5);
                    b.pack_end(&btns, true, false, 5);

                    PropertyWindowButtons::Create { cancel, save }
                }
            })
            .unwrap();
        ret.imp().obj.set(self.obj).unwrap();

        scrolled_window.set_child(Some(&b));
        ret.set_child(Some(&scrolled_window));

        ret
    }
}
