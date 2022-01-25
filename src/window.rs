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
use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::{sync::Lazy, unsync::OnceCell};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::app::GerbApp;
use crate::project::Project;

#[derive(Debug)]
struct WindowWidgets {
    headerbar: gtk::HeaderBar,
    grid: gtk::Grid,
    tool_palette: gtk::ToolPalette,
    create_item_group: gtk::ToolItemGroup,
    project_item_group: gtk::ToolItemGroup,
    stack: gtk::Stack,
}

#[derive(Debug, Default)]
pub struct Window {
    app: OnceCell<gtk::Application>,
    super_: OnceCell<MainWindow>,
    widgets: OnceCell<WindowWidgets>,
    project: OnceCell<Arc<Mutex<Option<Project>>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "Window";
    type Type = MainWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for Window {
    // Here we are overriding the glib::Object::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.super_.set(obj.clone()).unwrap();

        let headerbar = gtk::HeaderBar::new();

        headerbar.set_title(Some("gerb"));
        headerbar.set_show_close_button(true);

        let stack = gtk::Stack::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        let grid = gtk::Grid::builder().expand(true).visible(true).build();
        grid.attach(&stack, 1, 0, 1, 1);

        obj.set_child(Some(&grid));
        obj.set_titlebar(Some(&headerbar));
        obj.set_default_size(640, 480);
        obj.set_events(
            gtk::gdk::EventMask::POINTER_MOTION_MASK
                | gtk::gdk::EventMask::ENTER_NOTIFY_MASK
                | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK,
        );

        obj.connect_local("open-glyph-edit", false, clone!(@weak obj => @default-return Some(false.to_value()), move |v: &[gtk::glib::Value]| {
            println!("open-glyph-edit received!");
            let glyph_box = v[1].get::<crate::views::GlyphBoxItem>().unwrap();
            obj.imp().edit_glyph(glyph_box.imp().glyph.get().unwrap());

            None
        }));

        let tool_palette = gtk::ToolPalette::new();
        tool_palette.set_border_width(2);
        let create_item_group = gtk::ToolItemGroup::new("Create/load project");
        let new_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Create"));
        let load_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Load..."));
        create_item_group.add(&new_button);
        create_item_group.add(&load_button);
        tool_palette.add(&create_item_group);
        grid.attach(&tool_palette, 0, 0, 1, 1);

        let project_item_group = gtk::ToolItemGroup::new("Edit");
        let new_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("New glyph"));
        project_item_group.add(&new_button);

        self.widgets
            .set(WindowWidgets {
                headerbar,
                grid,
                stack,
                tool_palette,
                create_item_group,
                project_item_group,
            })
            .expect("Failed to initialize window state");
        self.project.set(Arc::new(Mutex::new(None))).unwrap();
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                // Signal name
                "open-glyph-edit",
                // Types of the values which will be sent to the signal handler
                &[crate::views::GlyphBoxItem::static_type().into()],
                // Type of the value the signal handler sends back
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }
}

impl Window {
    pub fn load_project(&self, project: Project) {
        let widgets = self.widgets.get().unwrap();
        widgets
            .headerbar
            .set_subtitle(Some(&format!("Loaded project: {}", project.name.as_str())));
        let item_groups = widgets.tool_palette.children();
        if item_groups
            .iter()
            .any(|g| g == widgets.create_item_group.upcast_ref::<gtk::Widget>())
        {
            widgets.tool_palette.remove(&widgets.create_item_group);
        }
        if !item_groups
            .iter()
            .any(|g| g == widgets.project_item_group.upcast_ref::<gtk::Widget>())
        {
            widgets.tool_palette.add(&widgets.project_item_group);
            widgets.project_item_group.set_visible(true);
        }
        let mutex = self.project.get().unwrap();
        let mut lck = mutex.lock().unwrap();
        *lck = Some(project);
        drop(lck);
        let glyphs_view =
            crate::views::GlyphsOverview::new(self.app.get().unwrap().clone(), mutex.clone());
        widgets.stack.add(&glyphs_view);
        widgets.stack.set_visible_child(&glyphs_view);
        widgets.tool_palette.queue_draw();
        widgets.stack.queue_draw();
    }

    pub fn edit_glyph(&self, glyph: &crate::project::Glyph) {
        let widgets = self.widgets.get().unwrap();
        let edit_view =
            crate::views::GlyphEditView::new(self.app.get().unwrap().clone(), glyph.clone());
        widgets.stack.add(&edit_view);
        widgets.stack.set_visible_child(&edit_view);
        widgets.stack.queue_draw();
        edit_view.queue_draw();
        let close_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Close glyph"));
        close_button.set_visible(true);
        widgets.project_item_group.add(&close_button);
        let obj = self.super_.get().unwrap().clone();
        close_button.connect_clicked(clone!(@strong obj => move |_self| {
            let widgets = obj.imp().widgets.get().unwrap();
            widgets.stack.remove(&edit_view);
            widgets.stack.queue_draw();
            widgets.project_item_group.remove(_self);
        }));
        widgets.tool_palette.queue_draw();
    }

    pub fn unload_project(&self) {
        let widgets = self.widgets.get().unwrap();
        widgets.headerbar.set_subtitle(None);
        let item_groups = widgets.tool_palette.children();
        if item_groups
            .iter()
            .any(|g| g == widgets.project_item_group.upcast_ref::<gtk::Widget>())
        {
            widgets.tool_palette.remove(&widgets.project_item_group);
        }
        if !item_groups
            .iter()
            .any(|g| g == widgets.create_item_group.upcast_ref::<gtk::Widget>())
        {
            widgets.tool_palette.add(&widgets.create_item_group);
        }
        widgets.tool_palette.queue_draw();
        widgets.stack.queue_draw();
        let mutex = self.project.get().unwrap();
        let mut lck = mutex.lock().unwrap();
        *lck = None;
    }
}

impl WidgetImpl for Window {}
impl ContainerImpl for Window {}
impl BinImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<Window>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MainWindow {
    pub fn new(app: &GerbApp) -> Self {
        let ret: Self =
            glib::Object::new(&[("application", app)]).expect("Failed to create Main Window");
        ret.imp()
            .app
            .set(app.upcast_ref::<gtk::Application>().clone())
            .unwrap();

        ret.imp().load_project(Project::default());
        ret
    }
}
