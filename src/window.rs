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

mod workspace;
pub use workspace::*;
mod tabinfo;
pub use tabinfo::*;
mod minimap;
pub use minimap::*;

use glib::clone;
use gtk::glib;
use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::{sync::Lazy, unsync::OnceCell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::GerbApp;
use crate::project::Project;

#[derive(Debug)]
pub struct WindowSidebar {
    pub tabinfo: TabInfo,
    pub main: gtk::Paned,
    pub project_info_sidebar: gtk::Paned,
    pub project_label: gtk::Label,
    pub minimap: Minimap,
}

impl WindowSidebar {
    #[inline(always)]
    fn new(main: gtk::Paned, _obj: &<Window as ObjectSubclass>::Type) -> Self {
        let ret = Self {
            tabinfo: TabInfo::new(),
            main,
            project_info_sidebar: gtk::Paned::builder()
                .orientation(gtk::Orientation::Vertical)
                .expand(true)
                .position(50)
                .visible(true)
                .can_focus(true)
                .name("main-window-subsidebar")
                .build(),
            project_label: gtk::Label::builder()
                .label("No project loaded.")
                .expand(true)
                .visible(true)
                .name("main-window-project-label")
                .build(),
            minimap: Minimap::new(),
        };
        let sidebar = gtk::Paned::builder()
            .orientation(gtk::Orientation::Vertical)
            .expand(true)
            .position(300)
            .visible(true)
            .can_focus(true)
            .name("main-window-sidebar")
            .build();
        ret.project_label.set_valign(gtk::Align::Start);
        ret.project_label.style_context().add_class("project-label");
        sidebar.pack2(&ret.minimap, true, false);
        sidebar.style_context().add_class("sidebar");

        ret.project_info_sidebar.pack1(&ret.tabinfo, true, true);
        ret.project_info_sidebar
            .pack2(&ret.project_label, true, true);

        sidebar.pack1(&ret.project_info_sidebar, true, false);
        ret.main.pack1(&sidebar, true, true);
        ret
    }

    fn load_project(&self, project: &Project) {
        self.project_label.set_markup(&format!("<big>{name}</big>\n\nMajor version: {version_major}\nMinor version: {version_minor}\n\nUnits per <i>em</i>: {units_per_em}\ndescender: {descender}\nascender: {ascender}\n<i>x</i>-height: {x_height}\ncap height: {cap_height}\nitalic angle: {italic_angle}", name=&project.property::<String>("name").as_str(), version_major=project.property::<i64>("version-major"), version_minor=project.property::<u64>("version-minor"), units_per_em=project.property::<f64>("units-per-em"), descender=project.property::<f64>("descender"), x_height=project.property::<f64>("x-height"), cap_height=project.property::<f64>("cap-height"), ascender=project.property::<f64>("ascender"), italic_angle=project.property::<f64>("italic-angle")));
        self.project_label.set_single_line_mode(false);
        self.project_label.set_use_markup(true);
        self.project_label.queue_draw();
        self.minimap.queue_draw();
    }

    #[allow(dead_code)]
    fn unload_project(&self) {
        self.project_label.set_markup("No project loaded.");
        self.project_label.queue_draw();
        self.minimap.queue_draw();
    }
}

#[derive(Debug)]
pub struct WindowWidgets {
    headerbar: gtk::HeaderBar,
    pub sidebar: WindowSidebar,
    pub statusbar: gtk::Statusbar,
    //tool_palette: gtk::ToolPalette,
    //create_item_group: gtk::ToolItemGroup,
    //project_item_group: gtk::ToolItemGroup,
    notebook: gtk::Notebook,
}

#[derive(Debug, Default)]
pub struct Window {
    app: OnceCell<gtk::Application>,
    super_: OnceCell<MainWindow>,
    pub widgets: OnceCell<WindowWidgets>,
    project: RefCell<Project>,
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

        let notebook = gtk::Notebook::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .name("main-window-notebook")
            .show_tabs(true)
            .scrollable(true)
            .enable_popup(true)
            .show_border(true)
            .build();
        let paned = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(true)
            .visible(true)
            .wide_handle(true)
            .position(130)
            .name("main-window-paned")
            .build();
        paned.pack2(&notebook, true, false);

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .expand(true)
            .spacing(5)
            .visible(true)
            .can_focus(true)
            .build();
        vbox.pack_start(&paned, true, true, 0);

        let statusbar = gtk::Statusbar::builder()
            .vexpand(false)
            .hexpand(true)
            .visible(true)
            .can_focus(true)
            .margin(0)
            .build();
        vbox.pack_start(&statusbar, false, false, 0);

        obj.set_child(Some(&vbox));
        obj.set_titlebar(Some(&headerbar));
        obj.set_default_size(640, 480);
        obj.set_events(
            gtk::gdk::EventMask::POINTER_MOTION_MASK
                | gtk::gdk::EventMask::ENTER_NOTIFY_MASK
                | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK,
        );

        obj.connect_local("open-glyph-edit", false, clone!(@weak obj => @default-return Some(false.to_value()), move |v: &[gtk::glib::Value]| {
            println!("open-glyph-edit received!");
            let glyph_box = v[1].get::<crate::views::GlyphBox>().unwrap();
            obj.imp().edit_glyph(glyph_box.imp().glyph.get().unwrap());

            None
        }));

        obj.connect_local("open-project", false, clone!(@weak obj => @default-return Some(false.to_value()), move |v: &[gtk::glib::Value]| {
            //println!("open-project received!");
            match v[1].get::<String>().map_err(|err| err.into()).and_then(|path| Project::from_path(&path)) {
                Ok(project) => {
                    obj.imp().load_project(project);
                    obj.queue_draw();
                }
                Err(err) => {
                    let dialog = gtk::MessageDialog::new(
                        Some(&obj),
                        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                        gtk::MessageType::Error,
                        gtk::ButtonsType::Close,
                        &err.to_string());
                    dialog.set_title("Error: Could not open project");
                    dialog.set_use_markup(true);
                    dialog.run();
                    dialog.hide();
                },
            }

            None
        }));

        /*
        let tool_palette = gtk::ToolPalette::builder()
            .border_width(2)
            .hexpand(false)
            .vexpand(true)
            .build();
        let create_item_group = gtk::ToolItemGroup::new("Create/load project");
        let new_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Create"));
        let load_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Load..."));
        create_item_group.add(&new_button);
        create_item_group.add(&load_button);
        tool_palette.add(&create_item_group);
        */
        //let project_item_group = gtk::ToolItemGroup::new("Edit");
        //let new_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("New glyph"));
        //project_item_group.add(&new_button);

        self.widgets
            .set(WindowWidgets {
                headerbar,
                sidebar: WindowSidebar::new(paned, obj),
                statusbar,
                notebook,
                //tool_palette,
                //create_item_group,
                //project_item_group,
            })
            .expect("Failed to initialize window state");
        *self.project.borrow_mut() = Project::new();
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![
                Signal::builder(
                    // Signal name
                    "open-glyph-edit",
                    // Types of the values which will be sent to the signal handler
                    &[crate::views::GlyphBox::static_type().into()],
                    // Type of the value the signal handler sends back
                    <()>::static_type().into(),
                )
                .build(),
                Signal::builder(
                    // Signal name
                    "open-project",
                    // Types of the values which will be sent to the signal handler
                    &[String::static_type().into()],
                    // Type of the value the signal handler sends back
                    <()>::static_type().into(),
                )
                .build(),
            ]
        });
        SIGNALS.as_ref()
    }
}

fn add_tab(notebook: &gtk::Notebook, widget: &gtk::Widget, reorderable: bool, closeable: bool) {
    notebook.add(widget);
    let tab_label = gtk::Label::builder().visible(true).use_markup(true).build();
    widget
        .bind_property("title", &tab_label, "label")
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
        .build();
    if closeable {
        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .spacing(5)
            .visible(true)
            .can_focus(true)
            .build();
        hbox.pack_start(&tab_label, false, false, 0);
        let image = gtk::Image::builder()
            .icon_name("window-close")
            .visible(true)
            .build();
        let close_button = gtk::Button::builder()
            .image(&image)
            .always_show_image(true)
            .relief(gtk::ReliefStyle::None)
            .visible(true)
            .build();
        close_button.connect_clicked(clone!(@strong notebook, @strong widget => move |_self| {
            if widget.property::<bool>("closeable") {
                notebook.remove(&widget);
                notebook.queue_draw();
            }
        }));
        close_button.style_context().add_class("tab-button");
        hbox.pack_start(&close_button, false, false, 0);
        notebook.set_tab_label(widget, Some(&hbox));
    } else {
        notebook.set_tab_label(widget, Some(&tab_label));
    }
    notebook.set_tab_reorderable(widget, reorderable);
    let mut children_no = 0;
    notebook.foreach(|_| {
        children_no += 1;
    });
    notebook.set_page(children_no - 1);
    widget.grab_focus();
    notebook.queue_draw();
    widget.queue_draw();
}

impl Window {
    pub fn load_project(&self, project: Project) {
        let widgets = self.widgets.get().unwrap();
        widgets.headerbar.set_subtitle(Some(&format!(
            "Loaded project: {}",
            project.property::<String>("name").as_str()
        )));
        widgets.statusbar.push(
            widgets.statusbar.context_id("main"),
            &format!(
                "Loaded project: {}",
                project.property::<String>("name").as_str()
            ),
        );
        /*
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
        */
        widgets.sidebar.load_project(&project);
        {
            *self.project.borrow_mut() = project.clone();
        }

        let collection = crate::views::Collection::new(self.app.get().unwrap().clone(), project);
        add_tab(
            &widgets.notebook,
            Workspace::new(collection.upcast_ref::<gtk::Widget>()).upcast_ref::<gtk::Widget>(),
            false,
            false,
        );
    }

    pub fn edit_glyph(&self, glyph: &Rc<RefCell<crate::glyphs::Glyph>>) {
        let widgets = self.widgets.get().unwrap();
        let edit_view = crate::views::GlyphEditView::new(
            self.app.get().unwrap().clone(),
            self.project.borrow().clone(),
            glyph.clone(),
        );
        add_tab(
            &widgets.notebook,
            Workspace::new(edit_view.upcast_ref::<gtk::Widget>()).upcast_ref::<gtk::Widget>(),
            true,
            true,
        );
    }

    pub fn unload_project(&self) {
        let widgets = self.widgets.get().unwrap();
        widgets.headerbar.set_subtitle(None);
        /*
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
        */
        widgets.notebook.queue_draw();
        *self.project.borrow_mut() = Project::new();
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
        ret
    }
}
