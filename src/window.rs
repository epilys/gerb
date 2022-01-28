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

mod tabinfo;
use tabinfo::*;

use glib::clone;
use gtk::cairo::{Context, FontSlant, FontWeight};
use gtk::glib;
use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::{sync::Lazy, unsync::OnceCell};
use std::sync::{Arc, Mutex};

use crate::app::GerbApp;
use crate::project::Project;

#[derive(Debug)]
struct WindowSidebar {
    main: gtk::Paned,
    project_info_sidebar: gtk::Paned,
    project_label: gtk::Label,
    minimap: gtk::DrawingArea,
}

impl WindowSidebar {
    #[inline(always)]
    fn new(
        main: gtk::Paned,
        notebook: &gtk::Notebook,
        obj: &<Window as ObjectSubclass>::Type,
    ) -> Self {
        let ret = Self {
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
            minimap: gtk::DrawingArea::builder()
                .expand(true)
                .visible(true)
                .name("main-window-minimap")
                .tooltip_text("pangram minimap")
                .build(),
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

        ret.minimap.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |_drar: &gtk::DrawingArea, cr: &Context| {
            const PANGRAM: &str = "A wizard's job is to vex chumps quickly in fog.";
            let (red, green, blue) = crate::utils::hex_color_to_rgb("#959595").unwrap();
            cr.set_source_rgb(red, green, blue);
            cr.paint().expect("Invalid cairo surface state");
            cr.select_font_face("Inter", FontSlant::Normal, FontWeight::Normal);
            cr.set_source_rgb(1., 1., 1.);
            cr.set_font_size(8.);
            let (x, mut y) = (2., 15.);
            cr.move_to(x, y);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 10.;
            cr.move_to(x, y);
            cr.set_font_size(14.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 10.;
            cr.move_to(x, y);
            cr.set_font_size(20.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 10.;
            cr.move_to(x, y);
            cr.set_font_size(32.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height + 25.;
            cr.move_to(x, y);
            cr.set_font_size(64.);
            let extends = cr.text_extents(PANGRAM).unwrap();
            cr.show_text(PANGRAM).expect("Invalid cairo surface state");
            y += extends.height;
            cr.move_to(x, y);
            Inhibit(false)
        }));
        ret.minimap.style_context().add_class("project-minimap");
        sidebar.pack2(&ret.minimap, true, false);
        sidebar.style_context().add_class("sidebar");

        let tabinfo = TabInfo::new(notebook);
        ret.project_info_sidebar.pack1(&tabinfo, true, true);
        ret.project_info_sidebar
            .pack2(&ret.project_label, true, true);

        sidebar.pack1(&ret.project_info_sidebar, true, false);
        ret.main.pack1(&sidebar, true, true);
        ret
    }

    fn load_project(&self, project: &Project) {
        self.project_label.set_markup(&format!("<big>{name}</big>\n\nMajor version: {version_major}\nMinor version: {version_minor}\n\nUnits per <i>em</i>: {units_per_em}\ndescender: {descender}\nascender: {ascender}\n<i>x</i>-height: {x_height}\ncap height: {cap_height}\nitalic angle: {italic_angle}", name=&project.name, version_major=project.version_major,version_minor=project.version_minor, units_per_em=project.units_per_em, descender=project.descender, x_height=project.x_height, cap_height=project.cap_height, ascender=project.ascender, italic_angle=project.italic_angle));
        self.project_label.set_single_line_mode(false);
        self.project_label.set_use_markup(true);
        self.project_label.queue_draw();
        self.minimap.queue_draw();
    }
}

#[derive(Debug)]
struct WindowWidgets {
    headerbar: gtk::HeaderBar,
    sidebar: WindowSidebar,
    //tool_palette: gtk::ToolPalette,
    //create_item_group: gtk::ToolItemGroup,
    //project_item_group: gtk::ToolItemGroup,
    notebook: gtk::Notebook,
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

        obj.set_child(Some(&paned));
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
                sidebar: WindowSidebar::new(paned, &notebook, obj),
                notebook,
                //tool_palette,
                //create_item_group,
                //project_item_group,
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

        let mutex = self.project.get().unwrap();
        let mut lck = mutex.lock().unwrap();
        *lck = Some(project);
        drop(lck);
        let glyphs_view =
            crate::views::GlyphsOverview::new(self.app.get().unwrap().clone(), mutex.clone());
        widgets.notebook.add(&glyphs_view);
        let tab_label = gtk::Label::builder()
            .label(&glyphs_view.property::<String>("tab-title"))
            .use_markup(true)
            .build();
        widgets
            .notebook
            .set_tab_label(&glyphs_view, Some(&tab_label));
        widgets.notebook.set_tab_reorderable(&glyphs_view, false);
        //widgets.notebook.set_visible_child(&glyphs_view);
        //widgets.tool_palette.queue_draw();
        widgets.notebook.queue_draw();
    }

    pub fn edit_glyph(&self, glyph: &crate::glyphs::Glyph) {
        let widgets = self.widgets.get().unwrap();
        let mutex = self.project.get().unwrap();
        let edit_view = crate::views::GlyphEditView::new(
            self.app.get().unwrap().clone(),
            mutex.clone(),
            glyph.clone(),
        );
        widgets.notebook.add(&edit_view);
        let tab_label = gtk::Label::builder()
            .label(&edit_view.property::<String>("tab-title"))
            .use_markup(true)
            .build();
        widgets.notebook.set_tab_label(&edit_view, Some(&tab_label));
        widgets.notebook.set_tab_reorderable(&edit_view, true);
        let mut children_no = 0;
        widgets.notebook.foreach(|_| {
            children_no += 1;
        });
        widgets.notebook.set_page(children_no - 1);
        //widgets.notebook.set_visible_child(&edit_view);
        widgets.notebook.queue_draw();
        edit_view.queue_draw();
        let close_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Close glyph"));
        close_button.set_visible(true);
        let toolbar = edit_view.imp().toolbar_box.get().unwrap();
        toolbar.add(&close_button);
        toolbar.queue_draw();
        let obj = self.super_.get().unwrap().clone();
        close_button.connect_clicked(clone!(@strong obj, @strong toolbar => move |_self| {
            let widgets = obj.imp().widgets.get().unwrap();
            widgets.notebook.remove(&edit_view);
            widgets.notebook.queue_draw();
            toolbar.remove(_self);
        }));
        /*
        widgets.project_item_group.add(&close_button);
        let obj = self.super_.get().unwrap().clone();
        close_button.connect_clicked(clone!(@strong obj => move |_self| {
            let widgets = obj.imp().widgets.get().unwrap();
            widgets.notebook.remove(&edit_view);
            widgets.notebook.queue_draw();
            widgets.project_item_group.remove(_self);
        }));
        widgets.tool_palette.queue_draw();
        */
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
