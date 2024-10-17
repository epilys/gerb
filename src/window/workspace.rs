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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::unsync::OnceCell;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
pub struct WorkspaceInner {
    scrolled_window: gtk::ScrolledWindow,
    grid: gtk::Grid,
    menubar: RefCell<gtk::MenuBar>,
    child: OnceCell<gtk::Widget>,
    reorderable: Cell<bool>,
    closeable: Cell<bool>,
    is_menu_visible: Cell<bool>,
    title: RefCell<String>,
    menumodel: RefCell<Option<gio::Menu>>,
}

#[glib::object_subclass]
impl ObjectSubclass for WorkspaceInner {
    const NAME: &'static str = "Workspace";
    type Type = Workspace;
    type ParentType = gtk::Box;
}

impl ObjectImpl for WorkspaceInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.style_context().add_class("workspace");
        self.reorderable.set(Self::REORDERABLE_INIT_VAL);
        self.closeable.set(Self::CLOSEABLE_INIT_VAL);
        self.is_menu_visible.set(Self::IS_MENU_VISIBLE_INIT_VAL);
        obj.upcast_ref::<gtk::Box>()
            .set_orientation(gtk::Orientation::Vertical);

        self.scrolled_window.set_expand(true);
        self.scrolled_window.set_visible(true);
        self.scrolled_window.set_can_focus(true);
        self.scrolled_window.set_margin_top(0);
        self.scrolled_window.set_margin_start(0);
        self.grid.set_expand(true);
        self.grid.set_visible(true);
        self.grid.set_can_focus(true);
        self.grid.set_column_spacing(5);
        self.grid.set_row_spacing(5);

        self.scrolled_window.set_child(Some(&self.grid));

        let menubar = self.menubar.borrow();
        menubar.set_visible(false);

        obj.pack_start(&*menubar, false, false, 0);
        obj.pack_end(&self.scrolled_window, true, true, 0);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);
        obj.bind_property(Workspace::IS_MENU_VISIBLE, &*menubar, "visible")
            .flags(glib::BindingFlags::DEFAULT)
            .build();
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        Workspace::REORDERABLE,
                        Workspace::REORDERABLE,
                        Workspace::REORDERABLE,
                        WorkspaceInner::REORDERABLE_INIT_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Workspace::CLOSEABLE,
                        Workspace::CLOSEABLE,
                        Workspace::CLOSEABLE,
                        WorkspaceInner::CLOSEABLE_INIT_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Workspace::IS_MENU_VISIBLE,
                        Workspace::IS_MENU_VISIBLE,
                        Workspace::IS_MENU_VISIBLE,
                        WorkspaceInner::IS_MENU_VISIBLE_INIT_VAL,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        Workspace::TITLE,
                        Workspace::TITLE,
                        Workspace::TITLE,
                        None,
                        ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Workspace::MENUBAR,
                        Workspace::MENUBAR,
                        Workspace::MENUBAR,
                        gtk::MenuBar::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Workspace::CHILD,
                        Workspace::CHILD,
                        Workspace::CHILD,
                        gtk::Widget::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Workspace::MENUMODEL,
                        Workspace::MENUMODEL,
                        Workspace::MENUMODEL,
                        gio::Menu::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Workspace::REORDERABLE => self.reorderable.get().to_value(),
            Workspace::CLOSEABLE => self.closeable.get().to_value(),
            Workspace::TITLE => self.title.borrow().to_value(),
            Workspace::IS_MENU_VISIBLE => self.is_menu_visible.get().to_value(),
            Workspace::MENUBAR => self.menubar.borrow().to_value(),
            Workspace::CHILD => self.child.get().unwrap().to_value(),
            Workspace::MENUMODEL => self.menumodel.borrow().as_ref().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Workspace::REORDERABLE => self.reorderable.set(value.get().unwrap()),
            Workspace::CLOSEABLE => {
                self.closeable.set(value.get().unwrap());
            }
            Workspace::TITLE => {
                *self.title.borrow_mut() = value.get().unwrap();
            }
            Workspace::IS_MENU_VISIBLE => self.is_menu_visible.set(value.get().unwrap()),
            Workspace::MENUBAR => {
                let new_menubar: gtk::MenuBar = value.get().unwrap();
                obj.bind_property(Workspace::IS_MENU_VISIBLE, &new_menubar, "visible")
                    .flags(glib::BindingFlags::DEFAULT)
                    .build();
                obj.remove(&*self.menubar.borrow());
                obj.pack_start(&new_menubar, false, false, 0);
                obj.show_all();
                *self.menubar.borrow_mut() = new_menubar;
                obj.queue_draw();
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl WorkspaceInner {
    pub const REORDERABLE_INIT_VAL: bool = false;
    pub const CLOSEABLE_INIT_VAL: bool = false;
    pub const IS_MENU_VISIBLE_INIT_VAL: bool = false;
}

impl WidgetImpl for WorkspaceInner {}
impl ContainerImpl for WorkspaceInner {}
impl BoxImpl for WorkspaceInner {}

glib::wrapper! {
    pub struct Workspace(ObjectSubclass<WorkspaceInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl Workspace {
    pub const REORDERABLE: &'static str = "reorderable";
    pub const CLOSEABLE: &'static str = "closeable";
    pub const TITLE: &'static str = "title";
    pub const IS_MENU_VISIBLE: &'static str = "is-menu-visible";
    pub const MENUBAR: &'static str = "menubar";
    pub const CHILD: &'static str = "child";
    pub const MENUMODEL: &'static str = "menu-model";

    pub fn new(child: &gtk::Widget) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Workspace");
        let child_properties = child.list_properties();
        for property in [
            Self::TITLE,
            Self::MENUBAR,
            Self::IS_MENU_VISIBLE,
            Self::CLOSEABLE,
        ] {
            if !child_properties
                .as_slice()
                .iter()
                .any(|p| p.name() == property)
            {
                continue;
            }
            ret.set_property_from_value(property, &child.property_value(property));
            child
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        ret.bind_property("is-focus", child, "is-focus").build();
        ret.imp().child.set(child.clone()).unwrap();
        ret.imp().grid.attach(child, 0, 0, 1, 1);
        ret.connect_button_press_event(|_self, event| {
            Inhibit(event.button() == gtk::gdk::BUTTON_SECONDARY)
        });

        ret
    }
}
