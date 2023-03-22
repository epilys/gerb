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

use crate::prelude::*;
use gtk::glib::subclass::Signal;

#[derive(Debug, Default)]
pub struct WindowInner {
    pub root_box: gtk::Box,
    pub welcome_banner: gtk::Box,
    pub headerbar: gtk::HeaderBar,
    pub statusbar: gtk::Statusbar,
    pub notebook: gtk::Notebook,
    pub action_group: gtk::gio::SimpleActionGroup,
    pub subwindows: RefCell<Vec<glib::object::WeakRef<gtk::Window>>>,
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
        self.notebook.set_visible(false);
        self.notebook.set_can_focus(true);
        self.notebook.set_widget_name("main-window-notebook");
        self.notebook.set_show_tabs(true);
        self.notebook.set_scrollable(true);
        self.notebook.set_enable_popup(true);
        self.notebook.set_show_border(true);

        self.root_box.set_orientation(gtk::Orientation::Vertical);

        let welcome_label = gtk::Label::builder().label(
            "You can create a new UFO project, open an existing one or import from a compatible format."
        ).visible(true).wrap(true).halign(gtk::Align::Center).build();
        self.welcome_banner.set_visible(true);
        self.welcome_banner.set_expand(true);
        self.welcome_banner.set_halign(gtk::Align::Center);
        self.welcome_banner.set_valign(gtk::Align::Center);
        self.welcome_banner
            .set_orientation(gtk::Orientation::Vertical);
        let new_project_btn = gtk::Button::builder()
            .relief(gtk::ReliefStyle::None)
            .label("New")
            .halign(gtk::Align::Center)
            .visible(true)
            .build();
        new_project_btn.set_action_name(Some("app.project.new"));
        let recent_mgr = gtk::RecentManager::default().unwrap_or_default();
        let mut items = recent_mgr
            .items()
            .into_iter()
            .filter(|i| {
                i.last_application().map(|a| a == "gerb").unwrap_or(false)
                    && i.mime_type()
                        .map(|a| a == "inode/directory")
                        .unwrap_or(false)
                    && i.uri_display()
                        .map(|a| Path::new(&a).exists())
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if !items.is_empty() {
            items.sort_by_key(|i| -i.modified());
            let recent_box = gtk::ListBox::builder()
                .expand(false)
                .visible(true)
                .can_focus(true)
                .sensitive(true)
                .halign(gtk::Align::Center)
                .selection_mode(gtk::SelectionMode::Browse)
                .build();
            for (uri, name) in items
                .into_iter()
                .filter_map(|i| Some((i.uri_display()?, i.short_name()?)))
                .take(10)
            {
                let label = gtk::Label::new(Some(&name));
                label.set_visible(true);
                label.set_tooltip_text(Some(&uri));
                label.set_sensitive(true);
                label.set_halign(gtk::Align::Start);
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                label.set_single_line_mode(true);
                let wrapper = gtk::EventBox::new();
                wrapper.add(&label);
                wrapper.connect_button_press_event(
                    clone!(@weak obj => @default-return Inhibit(false), move |_, _| {
                        obj.emit_by_name::<()>("open-project", &[&uri]);
                        Inhibit(true)
                    }),
                );
                recent_box.add(&wrapper);
            }
            let label = gtk::Label::new(Some("Recently opened:"));
            label.set_visible(true);
            label.set_sensitive(false);
            label.set_halign(gtk::Align::Center);
            self.welcome_banner.pack_end(&recent_box, false, false, 5);
            self.welcome_banner.pack_end(&label, false, false, 5);
        }
        self.welcome_banner
            .pack_end(&new_project_btn, false, false, 5);
        self.welcome_banner.pack_end(&welcome_label, true, false, 5);
        self.root_box
            .pack_start(&self.welcome_banner, false, false, 0);

        self.root_box.set_expand(true);
        self.root_box.set_spacing(0);
        self.root_box.set_visible(true);
        self.root_box.set_can_focus(true);
        self.root_box.pack_start(&self.notebook, true, true, 0);

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
        self.root_box.pack_end(&self.statusbar, false, false, 0);

        obj.set_child(Some(&self.root_box));
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

        obj.connect_local("open-project", true, clone!(@weak obj => @default-return Some(false.to_value()), move |v: &[gtk::glib::Value]| {
            {
                let mut subwindows = obj.subwindows.borrow_mut();
                subwindows.retain(|win| win.upgrade().is_some());
                if !subwindows.is_empty() {
                    let dialog = gtk::MessageDialog::builder()
                        .attached_to(&obj)
                        .border_width(10)
                        .destroy_with_parent(true)
                        .modal(true)
                        .buttons(gtk::ButtonsType::None)
                        .secondary_text("If you open a new project you will lose any unsaved data and session history from this shell window.")
                        .title("Warning: any unsaved data and session history will be lost.")
                        .build();
                    dialog.add_button("Cancel", gtk::ResponseType::Close);
                    dialog.add_button("Continue", gtk::ResponseType::Accept);
                    let retval: bool = dialog.run() == gtk::ResponseType::Accept;
                    dialog.hide();
                    dialog.close();
                    if retval {
                        for s in subwindows.iter() {
                            if let Some(w) = s.upgrade() {
                                w.hide();
                                w.close();
                            }
                        }
                        subwindows.clear();
                    } else {
                        obj.stop_signal_emission_by_name("open-project");
                        return None;
                    }
                }
            }
            match v[1].get::<String>().map_err(|err| err.into()).and_then(Project::from_path) {
                Ok(project) => {
                    #[cfg(feature = "python")]
                    obj.imp().application().register_obj(project.upcast_ref());
                    obj.load_project(project);
                    let app = obj.imp().application();
                    let settings = &app.runtime.settings;
                    if settings.property::<bool>(Settings::SHOW_PRERELEASE_WARNING) {
                        let dialog = crate::utils::widgets::new_simple_info_dialog(
                            Some("Warning: protecting your UFO data"),
                            "This is a pre-release version. It should not be used for production and/or with important data that are not backed up. Some metadata fields are not saved, and saving a .glif file will regenerate the XML instead of only what changed.",
                            Some("You can use UFO projects inside git repositories so that you can revert changes or use folder copies you do not want to keep."),
                            obj.upcast_ref(),
                        );

                        let area = dialog.message_area();
                        if let Ok(box_) = area.downcast::<gtk::Box>() {
                            let btn = crate::utils::widgets::ToggleButton::new();
                            btn.set_visible(true);
                            btn.set_active(false);
                            btn.set_sensitive(true);
                            btn.style_read_only(true);
                            btn.set_halign(gtk::Align::Start);
                            btn.set_valign(gtk::Align::Start);
                            btn.set_label("Show this warning every time a project is loaded (lest you forget)");
                            settings.bind_property(
                                Settings::SHOW_PRERELEASE_WARNING,
                                &btn,
                                "active"
                            )
                            .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                            .build();
                            box_.add(&btn);
                        }
                        dialog.run();
                        dialog.emit_close();
                    }
                    obj.queue_draw();
                }
                Err(err) => {
                    let dialog = crate::utils::widgets::new_simple_error_dialog(
                        Some("Error: Could not open project"),
                        &err.to_string(),
                        None,
                        obj.upcast_ref(),
                    );
                    dialog.run();
                    dialog.emit_close();
                },
            }

            None
        }));
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
    #[allow(clippy::cast_possible_wrap)]
    notebook.set_page(notebook.n_pages() as i32 - 1);
    widget.grab_focus();
    notebook.queue_draw();
    widget.queue_draw();
}

impl WindowInner {
    #[allow(clippy::cast_possible_wrap)]
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
        if let Ok(uri) = glib::filename_to_uri(&*project.path.borrow(), None) {
            /* add directory to user's Recent Files database */

            // Get gtk's default manager or create new
            let recent_mgr = gtk::RecentManager::default().unwrap_or_default();
            let recent_data = gtk::RecentData {
                display_name: None,
                description: None,
                mime_type: "inode/directory".to_string(),
                app_name: crate::APPLICATION_NAME.to_string(),
                app_exec: std::fs::read_link("/proc/self/exe")
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                groups: vec![],
                is_private: false,
            };
            recent_mgr.add_full(&uri, &recent_data);
        }

        project
            .bind_property(Project::MODIFIED, &self.instance(), "title")
            .transform_to(|_b, v| {
                if v.get::<bool>().ok()? {
                    Some(format!("{APPLICATION_NAME}*").to_value())
                } else {
                    Some(APPLICATION_NAME.to_value())
                }
            })
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.welcome_banner.set_visible(false);
        self.notebook.set_visible(true);
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
        {
            *self.application().runtime.project.borrow_mut() = project.clone();
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
        let edit_view = Editor::new(self.application(), glyph.clone());
        add_tab(
            &self.notebook,
            Workspace::new(edit_view.upcast_ref::<gtk::Widget>()).upcast_ref::<gtk::Widget>(),
            true,
            true,
        );
    }

    pub fn unload_project(&self) {
        self.headerbar.set_subtitle(None);
        self.notebook.queue_draw();
        *self.application().runtime.project.borrow_mut() = Project::new();
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
