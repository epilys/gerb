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

use gtk::glib::subclass::Signal;

use crate::prelude::*;

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
    #[allow(dead_code)]
    fn new(main: gtk::Paned, _obj: &<WindowInner as ObjectSubclass>::Type) -> Self {
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

    #[allow(dead_code)]
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

#[derive(Debug, Default)]
pub struct WindowInner {
    pub project: RefCell<Project>,
    pub headerbar: gtk::HeaderBar,
    pub statusbar: gtk::Statusbar,
    pub notebook: gtk::Notebook,
    pub action_group: gtk::gio::SimpleActionGroup,
}

#[glib::object_subclass]
impl ObjectSubclass for WindowInner {
    const NAME: &'static str = "Window";
    type Type = Window;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for WindowInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.headerbar.set_title(Some("gerb"));
        self.headerbar.set_show_close_button(true);

        self.notebook.set_expand(true);
        self.notebook.set_visible(true);
        self.notebook.set_can_focus(true);
        self.notebook.set_widget_name("main-window-notebook");
        self.notebook.set_show_tabs(true);
        self.notebook.set_scrollable(true);
        self.notebook.set_enable_popup(true);
        self.notebook.set_show_border(true);

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .expand(true)
            .spacing(0)
            .visible(true)
            .can_focus(true)
            .build();
        vbox.pack_start(&self.notebook, true, true, 0);

        self.statusbar.set_vexpand(false);
        self.statusbar.set_hexpand(true);
        self.statusbar.set_visible(true);
        self.statusbar.set_can_focus(true);
        self.statusbar.set_margin(0);
        {
            if let Some(label) = self
                .statusbar
                .message_area()
                .and_then(|box_| box_.children().pop())
                .and_then(|widget| widget.downcast::<gtk::Label>().ok())
            {
                label
                    .bind_property("label", &label, "tooltip_text")
                    .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
                    .build();
            }
        }
        vbox.pack_start(&self.statusbar, false, false, 0);

        obj.set_child(Some(&vbox));
        obj.set_titlebar(Some(&self.headerbar));
        obj.set_default_size(640, 480);
        obj.set_events(
            gtk::gdk::EventMask::POINTER_MOTION_MASK
                | gtk::gdk::EventMask::ENTER_NOTIFY_MASK
                | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK,
        );

        obj.connect_local("open-glyph-edit", false, clone!(@weak obj => @default-return Some(false.to_value()), move |v: &[gtk::glib::Value]| {
            let glyph_box = v[1].get::<crate::views::GlyphBox>().unwrap();
            obj.edit_glyph(glyph_box.imp().glyph.get().unwrap());

            None
        }));

        obj.connect_local("open-project", false, clone!(@weak obj => @default-return Some(false.to_value()), move |v: &[gtk::glib::Value]| {
            match v[1].get::<String>().map_err(|err| err.into()).and_then(|path| Project::from_path(&path)) {
                Ok(project) => {
                    obj.load_project(project);
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
                    dialog.emit_close();
                },
            }

            None
        }));
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
    notebook.set_page(notebook.n_pages() as i32 - 1);
    widget.grab_focus();
    notebook.queue_draw();
    widget.queue_draw();
}

impl WindowInner {
    pub fn setup_actions(&self) {
        let action_group = &self.action_group;
        let next_tab = gtk::gio::SimpleAction::new("next_tab", None);
        next_tab.connect_activate(glib::clone!(@weak self.notebook as obj => move |_, _| {
            let Some(cur) = obj.current_page() else { return; };
            let n = obj.n_pages() as i32;
            obj.set_page((cur as i32 + 1) % n);
        }));
        action_group.add_action(&next_tab);
        let prev_tab = gtk::gio::SimpleAction::new("prev_tab", None);
        prev_tab.connect_activate(glib::clone!(@weak self.notebook as obj => move |_, _| {
            let Some(cur) = obj.current_page() else { return; };
            let n = obj.n_pages() as i32;
            obj.set_page((cur as i32 + n - 1) % n);
        }));
        self.notebook.connect_page_added(
            clone!(@weak next_tab, @weak prev_tab => move |self_, _, _| {
                let enabled = self_.n_pages() as i32 > 1;
                next_tab.set_enabled(enabled);
                prev_tab.set_enabled(enabled);
            }),
        );
        self.notebook.connect_page_removed(
            clone!(@weak next_tab, @weak prev_tab => move |self_, _, _| {
                let enabled = self_.n_pages() as i32 > 1;
                next_tab.set_enabled(enabled);
                prev_tab.set_enabled(enabled);
            }),
        );
        action_group.add_action(&prev_tab);
        self.instance()
            .insert_action_group("win", Some(action_group));
    }

    pub fn load_project(&self, project: Project) {
        project
            .bind_property("name", &self.headerbar, "subtitle")
            .transform_from(|_b, v| {
                Some(format!("Loaded project: {}", v.get::<String>().unwrap()).to_value())
            })
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.statusbar.push(
            self.statusbar.context_id("main"),
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
        //widgets.sidebar.load_project(&project);
        {
            *self.project.borrow_mut() = project.clone();
        }
        self.notebook.foreach(|tab| {
            self.notebook.remove(tab);
        });

        let collection = Collection::new(self.application(), project);
        add_tab(
            &self.notebook,
            Workspace::new(collection.upcast_ref::<gtk::Widget>()).upcast_ref::<gtk::Widget>(),
            false,
            false,
        );
        self.notebook.show_all();
        self.notebook.queue_draw();
    }

    pub fn edit_glyph(&self, glyph: &Rc<RefCell<crate::glyphs::Glyph>>) {
        let edit_view = Editor::new(
            self.application(),
            self.project.borrow().clone(),
            glyph.clone(),
        );
        add_tab(
            &self.notebook,
            Workspace::new(edit_view.upcast_ref::<gtk::Widget>()).upcast_ref::<gtk::Widget>(),
            true,
            true,
        );
    }

    pub fn unload_project(&self) {
        self.headerbar.set_subtitle(None);
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
        self.notebook.queue_draw();
        *self.project.borrow_mut() = Project::new();
    }

    pub fn application(&self) -> Application {
        self.instance()
            .application()
            .and_then(|app| app.downcast::<Application>().ok())
            .unwrap()
    }
}

impl WidgetImpl for WindowInner {}
impl ContainerImpl for WindowInner {}
impl BinImpl for WindowInner {}
impl WindowImpl for WindowInner {}
impl ApplicationWindowImpl for WindowInner {}

glib::wrapper! {
    pub struct Window(ObjectSubclass<WindowInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl std::ops::Deref for Window {
    type Target = WindowInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

impl Window {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Main Window")
    }
}
