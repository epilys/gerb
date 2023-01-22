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

use crate::window::MainWindow;

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
            ("application-id", &"com.epilys.gerb"),
            ("flags", &ApplicationFlags::empty()), //&(ApplicationFlags::HANDLES_OPEN | ApplicationFlags::HANDLES_COMMAND_LINE)),
        ])
        .expect("Failed to create App")
    }
}

#[derive(Debug, Default)]
pub struct Application {
    pub window: OnceCell<MainWindow>,
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
        let app = app.downcast_ref::<super::GerbApp>().unwrap();
        let imp = app.imp();
        let window = imp
            .window
            .get()
            .expect("Should always be initialized in gio_application_startup");
        gtk::Window::set_interactive_debugging(true);

        //window.set_app_paintable(true); // crucial for transparency
        window.set_resizable(true);
        app.add_actions();
        app.build_system_menu();
        if let Some(path) = self.env_args.get().unwrap().clone().pop() {
            window.emit_by_name::<()>("open-project", &[&path]);
        } else {
            //window.imp().load_project(crate::project::Project::default());
        }
        window.show_all();
        window.present();
    }

    /// `gio::Application` is bit special. It does not get initialized
    /// when `new` is called and the object created, but rather
    /// once the `startup` signal is emitted and the `gio::Application::startup`
    /// is called.
    ///
    /// Due to this, we create and initialize the `MainWindow` widget
    /// here. Widgets can't be created before `startup` has been called.
    fn startup(&self, app: &Self::Type) {
        self.parent_startup(app);

        let app = app.downcast_ref::<super::GerbApp>().unwrap();
        let imp = app.imp();
        let window = MainWindow::new(app);
        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(include_bytes!("./custom.css"))
            .unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        imp.window
            .set(window)
            .expect("Failed to initialize application window");
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
        application.set_accels_for_action("edit.show-grid", &["<Primary>G"]);
        application.set_accels_for_action("edit.show-guidelines", &["<Primary><Shift>G"]);
        application.set_accels_for_action("edit.show-handles", &["<Primary><Shift>H"]);
        application.set_accels_for_action("edit.inner-fill", &["<Primary><Shift>I"]);
        application.set_accels_for_action("edit.show-total-area", &["<Primary><Shift>T"]);
        let window = self.imp().window.get().unwrap().upcast_ref::<gtk::Window>();
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

        let open = gtk::gio::SimpleAction::new("open", None);
        open.connect_activate(glib::clone!(@weak window => move |_, _| {
            let dialog = gtk::FileChooserNative::new(
                Some("Open font.ufo directory..."),
                Some(&window),
                gtk::FileChooserAction::SelectFolder,
                None,
                None
            );
            let response = dialog.run();
            std::dbg!(&response);
            std::dbg!(&dialog.filename());
            if let Some(f) = dialog.filename() {
                if let Some(path) = f.to_str() {
                    window.emit_by_name::<()>("open-project", &[&path]);
                    window.show_all();
                }
            }
            dialog.hide();
        }));
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

        application.add_action(&settings);
        application.add_action(&about);
        application.add_action(&open);
        application.add_action(&undo);
        application.add_action(&redo);
        application.add_action(&quit);
    }

    fn build_system_menu(&self) {
        let application = self.upcast_ref::<gtk::Application>();
        let menu_bar = gio::Menu::new();
        let more_menu = gio::Menu::new();
        let file_menu = gio::Menu::new();
        let settings_menu = gio::Menu::new();
        let submenu = gio::Menu::new();

        // The first argument is the label of the menu item whereas the second is the action name. It'll
        // makes more sense when you'll be reading the "add_actions" function.

        file_menu.append(Some("File"), Some("app.file"));
        file_menu.append(Some("Open"), Some("app.open"));
        file_menu.append(Some("Quit"), Some("app.quit"));
        menu_bar.append_submenu(Some("_File"), &file_menu);

        settings_menu.append(Some("Settings"), Some("app.settings"));
        settings_menu.append(None, None);
        settings_menu.append(Some("Undo"), Some("app.undo"));
        settings_menu.append(Some("Redo"), Some("app.redo"));
        settings_menu.append(Some("Sub another"), Some("app.sub_another"));
        submenu.append(Some("Sub sub another"), Some("app.sub_sub_another"));
        submenu.append(Some("Sub sub another2"), Some("app.sub_sub_another2"));
        settings_menu.append_submenu(Some("Sub menu"), &submenu);
        menu_bar.append_submenu(Some("_Edit"), &settings_menu);

        more_menu.append(Some("About"), Some("app.about"));
        menu_bar.append_submenu(Some("Gerb"), &more_menu);

        //application.set_app_menu(Some(&menu));
        application.set_menubar(Some(&menu_bar));
    }

    pub fn statusbar(&self) -> gtk::Statusbar {
        let window = self.imp().window.get().unwrap();
        window.imp().widgets.get().unwrap().statusbar.clone()
    }

    /*pub fn tabinfo(&self) -> crate::window::TabInfo {
        let window = self.imp().window.get().unwrap();
        window.imp().widgets.get().unwrap().sidebar.tabinfo.clone()
    }*/
}
