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
    /// aksed to present itself.
    fn activate(&self, app: &Self::Type) {
        let app = app.downcast_ref::<super::GerbApp>().unwrap();
        let imp = app.imp();
        let window = imp
            .window
            .get()
            .expect("Should always be initiliazed in gio_application_startup");
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
            dialog.hide();
        }));

        application.add_action(&about);
        application.add_action(&open);
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
}
