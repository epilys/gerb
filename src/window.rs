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
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::app::GerbApp;
use crate::project::Project;

#[derive(Debug)]
struct WindowWidgets {
    headerbar: gtk::HeaderBar,
    label: gtk::Label,
    drawing_area: gtk::DrawingArea,
    grid: gtk::Grid,
    tool_palette: gtk::ToolPalette,
    create_item_group: gtk::ToolItemGroup,
    project_item_group: gtk::ToolItemGroup,
    stack: gtk::Stack,
}

#[derive(Debug, Default)]
pub struct Window {
    widgets: OnceCell<WindowWidgets>,
    counter: Cell<u64>,
    project: OnceCell<Arc<Mutex<Option<Project>>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "Window";
    type Type = MainWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for Window {
    // Here we are overriding the glib::Objcet::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let headerbar = gtk::HeaderBar::new();
        let increment = gtk::Button::with_label("Increment!");
        let label = gtk::Label::new(Some("toolbar here"));

        headerbar.set_title(Some("gerb"));
        headerbar.set_show_close_button(true);
        //headerbar.pack_start(&increment);

        // Connect our method `on_increment_clicked` to be called
        // when the increment button is clicked.
        increment.connect_clicked(clone!(@weak obj => move |_| {
            let imp = obj.imp();
            imp.on_increment_clicked();
        }));

        let stack = gtk::Stack::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        let drawing_area = gtk::DrawingArea::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |_drar: &gtk::DrawingArea, cr: &gtk::cairo::Context| {
            cr.scale(500f64, 500f64);
            cr.set_source_rgb(1., 1., 1.);
            cr.paint().expect("Invalid cairo surface state");
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.set_line_width(0.0005);
            for x in (0..100).into_iter().step_by(5) {
                let x = x as f64 / 100.;
                cr.move_to(x, 0.);
                cr.line_to(x, 1.);
                cr.stroke().expect("Invalid cairo surface state");
            }
            for y in (0..100).into_iter().step_by(5) {
                let y = y as f64 / 100.;
                cr.move_to(0., y);
                cr.line_to(1., y);
                cr.stroke().expect("Invalid cairo surface state");
            }
            /*
            cr.set_source_rgb(0., 0., 0.);

            cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
            cr.set_font_size(0.35);

            cr.move_to(0.04, 0.53);
            cr.show_text("Hello").expect("Invalid cairo surface state");

            cr.move_to(0.27, 0.65);
            cr.text_path("void");
            cr.set_source_rgb(0.5, 0.5, 1.0);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(0.01);
            cr.stroke().expect("Invalid cairo surface state");

            cr.set_source_rgba(1.0, 0.2, 0.2, 0.6);
            cr.arc(0.04, 0.53, 0.02, 0.0, PI * 2.);
            cr.arc(0.27, 0.65, 0.02, 0.0, PI * 2.);
            cr.fill().expect("Invalid cairo surface state");
            */

            //if let Ok(lck) = obj.imp().project.get().unwrap().lock() {
            //    if let Some(project) = &*lck {
            //        let glyph = &project.glyphs[&('R' as u32)];
            //        glyph.draw(drar, cr);
            //    }
            //}

            Inhibit(false)
        }
        ));
        //stack.add_named(&drawing_area, "main");
        let grid = gtk::Grid::builder().expand(true).visible(true).build();
        //grid.attach(&drawing_area, 1, 0, 1, 1);
        grid.attach(&stack, 1, 0, 1, 1);

        obj.set_child(Some(&grid));
        obj.set_titlebar(Some(&headerbar));
        obj.set_default_size(640, 480);
        obj.set_events(
            gtk::gdk::EventMask::POINTER_MOTION_MASK
                | gtk::gdk::EventMask::ENTER_NOTIFY_MASK
                | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK,
        );

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
                label,
                drawing_area,
                grid,
                stack,
                tool_palette,
                create_item_group,
                project_item_group,
            })
            .expect("Failed to initialize window state");
        self.project.set(Arc::new(Mutex::new(None))).unwrap();

        self.load_project(Project::default());
    }
}

impl Window {
    fn on_increment_clicked(&self) {
        self.counter.set(self.counter.get() + 1);
        let w = self.widgets.get().unwrap();
        w.label
            .set_text(&format!("Counter is {}", self.counter.get()));
    }

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
        let edit_view = crate::views::GlyphEditView::new(
            mutex.lock().unwrap().as_ref().unwrap().glyphs[&('R' as u32)].clone(),
        );
        let glyphs_view = crate::views::GlyphsOverview::new(mutex.clone());
        widgets.stack.add(&glyphs_view);
        widgets.stack.add(&edit_view);
        widgets.stack.set_visible_child(&glyphs_view);
        widgets.tool_palette.queue_draw();
        widgets.stack.queue_draw();
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
        glib::Object::new(&[("application", app)]).expect("Failed to create Main Window")
    }
}
