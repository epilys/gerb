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

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use crate::window::Window;

use gio::ApplicationFlags;
use gtk::{gio, glib};
use std::cell::RefCell;

mod undo;
pub use undo::*;
mod settings;
pub use settings::*;

glib::wrapper! {
    pub struct GerbApp(ObjectSubclass<Application>)
        @extends gio::Application, gtk::Application;
}

impl GerbApp {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &crate::APPLICATION_ID),
            ("flags", &ApplicationFlags::empty()), //&(ApplicationFlags::HANDLES_OPEN | ApplicationFlags::HANDLES_COMMAND_LINE)),
        ])
        .expect("Failed to create App")
    }
}

#[derive(Debug, Default)]
pub struct Application {
    pub window: Window,
    pub settings: RefCell<Settings>,
    pub undo_db: RefCell<undo::UndoDatabase>,
    pub env_args: OnceCell<Vec<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Application {
    const NAME: &'static str = "Application";
    type Type = super::GerbApp;
    type ParentType = gtk::Application;
}

impl ObjectImpl for Application {}

/// When our application starts, the `startup` signal will be fired.
/// This gives us a chance to perform initialisation tasks that are not directly
/// related to showing a new window. After this, depending on how
/// the application is started, either `activate` or `open` will be called next.
impl ApplicationImpl for Application {
    /// `gio::Application::activate` is what gets called when the
    /// application is launched by the desktop environment and
    /// asked to present itself.
    fn activate(&self, app: &Self::Type) {
        self.parent_activate(app);
        #[cfg(debug_assertions)]
        gtk::Window::set_interactive_debugging(true);
        //self.window.set_app_paintable(true); // crucial for transparency
        self.window.set_resizable(true);
    }

    /// `gio::Application` is bit special. It does not get initialized
    /// when `new` is called and the object created, but rather
    /// once the `startup` signal is emitted and the `gio::Application::startup`
    /// is called.
    ///
    /// Due to this, we create and initialize the `Window` widget
    /// here. Widgets can't be created before `startup` has been called.
    fn startup(&self, app: &Self::Type) {
        self.parent_startup(app);
        self.window.set_application(Some(app));
        self.instance().add_actions();
        self.instance().build_system_menu();

        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(include_bytes!("./custom.css"))
            .unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        if let Some(path) = self.env_args.get().unwrap().clone().pop() {
            self.window.emit_by_name::<()>("open-project", &[&path]);
        }
        self.window.show_all();
        self.window.present();
    }
}

impl GtkApplicationImpl for Application {}

impl GerbApp {
    fn add_actions(&self) {
        let application = self.upcast_ref::<gtk::Application>();
        application.set_accels_for_action("app.quit", &["<Primary>Q", "Q"]);
        application.set_accels_for_action("app.about", &["question", "F1"]);
        application.set_accels_for_action("app.undo", &["<Primary>Z"]);
        application.set_accels_for_action("app.redo", &["<Primary>R"]);
        application.set_accels_for_action("view.show-grid", &["<Primary>G"]);
        application.set_accels_for_action("view.show-guidelines", &["<Primary><Shift>G"]);
        application.set_accels_for_action("view.show-handles", &["<Primary><Shift>H"]);
        application.set_accels_for_action("view.inner-fill", &["<Primary><Shift>I"]);
        application.set_accels_for_action("view.show-total-area", &["<Primary><Shift>T"]);
        application.set_accels_for_action("view.zoom.in", &["<Primary>plus", "plus"]);
        application.set_accels_for_action("view.zoom.out", &["<Primary>minus", "minus"]);
        let window = self.imp().window.upcast_ref::<gtk::Window>();
        let quit = gtk::gio::SimpleAction::new("quit", None);
        quit.connect_activate(glib::clone!(@weak window => move |_, _| {
            window.close();
        }));

        let about = gtk::gio::SimpleAction::new("about", None);
        about.connect_activate(glib::clone!(@weak window => move |_, _| {
            let p = gtk::AboutDialog::new();
            p.set_program_name("gerb");
            //p.set_logo(Some(&gtk::gdk_pixbuf::Pixbuf::from_xpm_data(
            //            ICON,
            //)));
            p.set_website_label(Some("https://github.com/epilys/gerb"));
            p.set_website(Some("https://github.com/epilys/gerb"));
            p.set_authors(&["Manos Pitsidianakis"]);
            p.set_copyright(Some("2022 - Manos Pitsidianakis"));
            p.set_title("About Gerb");
            p.set_license_type(gtk::License::Gpl30);
            p.set_transient_for(Some(&window));
            p.set_comments(Some(""));
            p.show_all();
        }));

        let settings = gtk::gio::SimpleAction::new("settings", None);
        settings.connect_activate(glib::clone!(@weak application as app => move |_, _| {
            let gapp = app.downcast_ref::<super::GerbApp>().unwrap();
            let obj: glib::Object = gapp.imp().settings.borrow().clone().upcast();
            let w = crate::utils::new_property_window(obj, "Settings");
            w.present();
        }));
        let import_glyphs = gtk::gio::SimpleAction::new("project.import.glyphs", None);

        import_glyphs.connect_activate(glib::clone!(@weak window => move |_, _| {
            #[cfg(feature = "python")]
            {
                let dialog = gtk::FileChooserNative::new(
                    Some("Select glyphs file"),
                    Some(&window),
                    gtk::FileChooserAction::Open,
                    None,
                    None,
                );
                let _response = dialog.run();
                dialog.hide();
                if let Some(f) = dialog.filename() {
                    if let Some(path) = f.to_str() {
                        match crate::ufo::import::import(
                            crate::ufo::import::Glyphs2UFOOptions::new(path.into()).output_dir(None),
                        ) {
                            Ok(instances) => {
                                if instances.len() == 1 {
                                    window.emit_by_name::<()>(
                                        "open-project",
                                        &[&instances[0].full_path.display().to_string()],
                                    );
                                } else {
                                    let dialog = gtk::MessageDialog::new(
                                        Some(&window),
                                        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                                        gtk::MessageType::Info,
                                        gtk::ButtonsType::Close,
                                        &format!(
                                            "Generated {} instances:\n{:?}",
                                            instances.len(),
                                            instances
                                            .iter()
                                            .map(|i| (
                                                    i.family_name.as_str(),
                                                    i.style_name.as_str(),
                                                    i.full_path.as_path()
                                            ))
                                            .collect::<Vec<_>>()
                                        ),
                                    );
                                    dialog.set_title(
                                        "Info: generated more than once instance, open one manually.",
                                    );
                                    dialog.set_use_markup(true);
                                    dialog.run();
                                    dialog.emit_close();
                                }
                            }
                            Err(err) => {
                                let dialog = gtk::MessageDialog::new(
                                    Some(&window),
                                    gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Close,
                                    &err.to_string(),
                                );
                                dialog.set_title(
                                    "Error: could not perform conversion to UFOv3 with glyphsLib",
                                );
                                dialog.set_use_markup(true);
                                dialog.run();
                                dialog.emit_close();
                            }
                        }
                    }
                }
            }
            #[cfg(not(feature = "python"))]
            {
                let dialog = gtk::MessageDialog::new(
                    Some(&window),
                    gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                    gtk::MessageType::Error,
                    gtk::ButtonsType::Close,
                    "This application build doesn't include python support. Glyphs import is performed with the glyphsLib python3 library. Compile the app with `python` Cargo feature.",
                );
                dialog.set_title("Error");
                dialog.set_use_markup(true);
                dialog.run();
                dialog.emit_close();
            }
        }));
        let open = gtk::gio::SimpleAction::new("project.open", None);
        open.connect_activate(glib::clone!(@weak window => move |_, _| {
            let dialog = gtk::FileChooserNative::new(
                Some("Open font.ufo directory..."),
                Some(&window),
                gtk::FileChooserAction::SelectFolder,
                None,
                None
            );
            let _response = dialog.run();
            if let Some(f) = dialog.filename() {
                if let Some(path) = f.to_str() {
                    window.emit_by_name::<()>("open-project", &[&path]);
                    window.show_all();
                }
            }
            dialog.hide();
        }));
        let new_project = gtk::gio::SimpleAction::new("project.new", None);
        {
            let window = &self.imp().window;
            new_project.connect_activate(glib::clone!(@weak window => move |_, _| {
                window.imp().load_project(crate::project::Project::default());
            }));
        }
        let undo = gtk::gio::SimpleAction::new("undo", None);
        undo.set_enabled(false);
        undo.connect_activate(glib::clone!(@weak self as _self => move |_, _| {
            _self.imp().undo_db.borrow_mut().undo();
        }));
        let redo = gtk::gio::SimpleAction::new("redo", None);
        redo.set_enabled(false);
        redo.connect_activate(glib::clone!(@weak self as _self => move |_, _| {
            _self.imp().undo_db.borrow_mut().redo();
        }));
        undo.bind_property("enabled", &*self.imp().undo_db.borrow(), "can-undo")
            .flags(glib::BindingFlags::BIDIRECTIONAL)
            .build();
        redo.bind_property("enabled", &*self.imp().undo_db.borrow(), "can-redo")
            .flags(glib::BindingFlags::BIDIRECTIONAL)
            .build();

        let project_properties = gtk::gio::SimpleAction::new("project.properties", None);
        project_properties.connect_activate(glib::clone!(@weak application as app => move |_, _| {
            let gapp = app.downcast_ref::<super::GerbApp>().unwrap();
            let obj: glib::Object = gapp.imp().window.imp().project.borrow().clone().upcast();
            let w = crate::utils::new_property_window(obj, "Project");
            w.present();
        }));
        application.add_action(&project_properties);
        application.add_action(&import_glyphs);
        application.add_action(&settings);
        application.add_action(&about);
        application.add_action(&open);
        application.add_action(&new_project);
        application.add_action(&undo);
        application.add_action(&redo);
        application.add_action(&quit);
    }

    fn build_system_menu(&self) {
        let application = self.upcast_ref::<gtk::Application>();
        let menu_bar = gio::Menu::new();
        let meta_menu = gio::Menu::new();
        let file_menu = gio::Menu::new();
        let import_menu = gio::Menu::new();
        let edit_menu = gio::Menu::new();

        file_menu.append(Some("New"), Some("app.project.new"));
        file_menu.append(Some("Open"), Some("app.project.open"));
        import_menu.append(
            Some("Import Glyphs file"),
            Some("app.project.import.glyphs"),
        );
        file_menu.append_submenu(Some("Import"), &import_menu);
        let project_section = gio::Menu::new();
        project_section.append(Some("Properties"), Some("app.project.properties"));
        file_menu.append_section(Some("Project"), &project_section);
        file_menu.append(Some("Quit"), Some("app.quit"));
        menu_bar.append_submenu(Some("_File"), &file_menu);

        edit_menu.append(Some("Settings"), Some("app.settings"));
        let undo_section = gio::Menu::new();
        undo_section.append(Some("Undo"), Some("app.undo"));
        undo_section.append(Some("Redo"), Some("app.redo"));
        edit_menu.append_section(Some("Action history"), &undo_section);
        menu_bar.append_submenu(Some("_Edit"), &edit_menu);

        meta_menu.append(Some("Report issue"), Some("app.bug_report"));
        meta_menu.append(Some("About"), Some("app.about"));
        menu_bar.append_submenu(Some("Gerb"), &meta_menu);

        //application.set_app_menu(Some(&menu));
        application.set_menubar(Some(&menu_bar));
    }

    pub fn statusbar(&self) -> gtk::Statusbar {
        self.imp().window.imp().statusbar.clone()
    }
}
