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
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::{Cell, RefCell};

#[derive(Debug)]
struct TabInfoWidgets {
    grid: gtk::Grid,
}

#[derive(Debug, Default)]
pub struct WorkspaceInner {
    scrolled_window: OnceCell<gtk::ScrolledWindow>,
    grid: OnceCell<gtk::Grid>,
    main_widget: OnceCell<gtk::Widget>,
    other_widgets: RefCell<Vec<gtk::Widget>>,
    reorderable: Cell<bool>,
    closeable: Cell<bool>,
    title: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for WorkspaceInner {
    const NAME: &'static str = "WorkspaceInner";
    type Type = Workspace;
    type ParentType = gtk::Box;
}

impl ObjectImpl for WorkspaceInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .margin_top(0)
            .margin_start(0)
            .build();
        let grid = gtk::Grid::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .column_spacing(5)
            .row_spacing(5)
            .build();

        scrolled_window.set_child(Some(&grid));
        obj.pack_start(&scrolled_window, true, true, 0);
        obj.set_visible(true);
        obj.set_expand(true);
        self.scrolled_window.set(scrolled_window).unwrap();
        self.grid.set(grid).unwrap();
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        "reorderable",
                        "reorderable",
                        "reorderable",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "closeable",
                        "closeable",
                        "closeable",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new("title", "title", "title", None, ParamFlags::READWRITE),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            "reorderable" => self.reorderable.get().to_value(),
            "closeable" => self.closeable.get().to_value(),
            "title" => self.title.borrow().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "reorderable" => self.reorderable.set(value.get().unwrap()),
            "closeable" => {
                self.closeable.set(value.get().unwrap());
            }
            "title" => {
                *self.title.borrow_mut() = value.get().unwrap();
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl WorkspaceInner {}

impl WidgetImpl for WorkspaceInner {}
impl ContainerImpl for WorkspaceInner {}
impl BoxImpl for WorkspaceInner {}

glib::wrapper! {
    pub struct Workspace(ObjectSubclass<WorkspaceInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl Workspace {
    pub fn new(main_widget: &gtk::Widget) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Workspace");
        for property in ["title", "closeable"] {
            main_widget
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        ret.imp().main_widget.set(main_widget.clone()).unwrap();
        ret.imp()
            .grid
            .get()
            .unwrap()
            .attach(main_widget, 0, 0, 1, 1);
        ret
    }
}
