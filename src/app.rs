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

use crate::utils::property_window::CreatePropertyWindow;
use crate::window::Window;

use gio::ApplicationFlags;
use gtk::{gio, glib};
use std::cell::RefCell;

mod undo;
pub use undo::*;
pub mod settings;
pub use settings::*;

glib::wrapper! {
    pub struct Application(ObjectSubclass<ApplicationInner>)
        @extends gio::Application, gtk::Application;
}

impl std::ops::Deref for Application {
    type Target = ApplicationInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Application {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &crate::APPLICATION_ID),
            ("flags", &ApplicationFlags::empty()), //&(ApplicationFlags::HANDLES_OPEN | ApplicationFlags::HANDLES_COMMAND_LINE)),
        ])
        .expect("Failed to create App")
    }

    pub fn warp_cursor(&self, device: Option<gtk::gdk::Device>, delta: (i32, i32)) -> Option<()> {
        let device = device?;
        let (screen, rootx, rooty) = device.position();
        let (x, y) = (rootx + delta.0, rooty + delta.1);
        device.warp(&screen, x, y);
        Some(())
    }
}

#[derive(Debug, Default)]
pub struct ApplicationInner {
    pub window: Window,
    pub settings: RefCell<Settings>,
    pub undo_db: RefCell<undo::UndoDatabase>,
    pub env_args: OnceCell<Vec<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ApplicationInner {
    const NAME: &'static str = "Application";
    type Type = Application;
    type ParentType = gtk::Application;
}

impl ObjectImpl for ApplicationInner {}

/// When our application starts, the `startup` signal will be fired.
/// This gives us a chance to perform initialisation tasks that are not directly
/// related to showing a new window. After this, depending on how
/// the application is started, either `activate` or `open` will be called next.
impl ApplicationImpl for ApplicationInner {
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
        self.add_actions(app);
        self.window.setup_actions();
        self.build_system_menu(app);

        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(include_bytes!("./custom.css"))
            .unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        self.window.show_all();
        if let Some(path) = self.env_args.get().unwrap().clone().pop() {
            self.window.emit_by_name::<()>("open-project", &[&path]);
            self.window.welcome_banner.set_visible(false);
            self.window.notebook.set_visible(true);
        } else {
            self.window.welcome_banner.set_visible(true);
            self.window.notebook.set_visible(false);
        }
        self.window.present();
    }
}

impl GtkApplicationImpl for ApplicationInner {}

impl ApplicationInner {
    fn add_actions(&self, obj: &Application) {
        let application = obj.upcast_ref::<gtk::Application>();
        application.set_accels_for_action("app.quit", &["<Primary>Q"]);
        application.set_accels_for_action("app.about", &["question", "F1"]);
        application.set_accels_for_action("app.undo", &["<Primary>Z"]);
        application.set_accels_for_action("app.redo", &["<Primary>R"]);
        #[cfg(debug_assertions)]
        application.set_accels_for_action("app.snapshot", &["F12"]);
        application.set_accels_for_action("app.project.open", &["<Primary>O"]);
        application.set_accels_for_action("app.project.new", &["<Primary>N"]);
        application.set_accels_for_action("app.project.properties", &["<Primary><Shift>D"]);
        application.set_accels_for_action("app.project.save", &["<Primary>S"]);
        application
            .set_accels_for_action("win.next_tab", &["<Primary>Page_Down", "<Primary>greater"]);
        application.set_accels_for_action("win.prev_tab", &["<Primary>Page_Up", "<Primary>less"]);
        application.set_accels_for_action("glyph.show.grid", &["<Primary>G"]);
        application.set_accels_for_action("glyph.show.guideline", &["<Primary><Shift>G"]);
        application.set_accels_for_action("glyph.show.handles", &["<Primary><Shift>H"]);
        application.set_accels_for_action("glyph.show.inner-fill", &["<Primary><Shift>I"]);
        application.set_accels_for_action("glyph.show.total-area", &["<Primary><Shift>T"]);
        application.set_accels_for_action("view.zoom.in", &["<Primary>plus", "plus"]);
        application.set_accels_for_action("view.zoom.out", &["<Primary>minus", "minus"]);
        application.set_accels_for_action("glyph.show.guideline.metrics", &["F2"]);
        application.set_accels_for_action("glyph.show.guideline.inner-fill", &["F3"]);
        application.set_accels_for_action("view.preview", &["grave"]);
        let window = self.window.upcast_ref::<gtk::Window>();
        #[cfg(debug_assertions)]
        {
            let snapshot = gtk::gio::SimpleAction::new("snapshot", None);
            snapshot.connect_activate(glib::clone!(@weak window, @weak application => move |_, _| {
                let path = "/tmp/t.svg";
                let (w, h) = (window.allocated_width(), window.allocated_height());
                let svg_surface = cairo::SvgSurface::new(w as f64, h as f64, Some(path)).unwrap();
                let ctx = gtk::cairo::Context::new(&svg_surface).unwrap();
                window.draw(&ctx);
                svg_surface.flush();
                svg_surface.finish();
                eprintln!("saved to path: {path}");
            }));
            application.add_action(&snapshot);
        }

        let quit = gtk::gio::SimpleAction::new("quit", None);
        quit.connect_activate(glib::clone!(@weak window => move |_, _| {
            window.close();
        }));

        let about = gtk::gio::SimpleAction::new("about", None);
        about.connect_activate(glib::clone!(@weak window => move |_, _| {
            let p = gtk::AboutDialog::new();
            p.set_program_name("gerb");
            p.set_logo(crate::resources::G_GLYPH.to_pixbuf().as_ref());
            p.set_website_label(Some("https://github.com/epilys/gerb"));
            p.set_website(Some("https://github.com/epilys/gerb"));
            p.set_authors(&["Manos Pitsidianakis"]);
            p.set_copyright(Some("2022 - Manos Pitsidianakis"));
            p.set_title("About gerb");
            p.set_license_type(gtk::License::Gpl30);
            p.set_transient_for(Some(&window));
            p.set_comments(Some(""));
            /* find image widget */
            if let Some(logo_image) =  p.children().iter().find_map(gtk::Widget::downcast_ref::<gtk::Box>).map(gtk::Box::children).and_then(|v| v.iter().find_map(gtk::Widget::downcast_ref::<gtk::Box>).cloned()).as_ref().map(gtk::Box::children).and_then(|v| v.iter().find_map(gtk::Widget::downcast_ref::<gtk::Image>).cloned()) {
                crate::resources::UIIcon::image_into_surface(&logo_image, window.scale_factor(), window.window());
            }
            p.show_all();
        }));
        #[cfg(feature = "python")]
        {
            let shell = gtk::gio::SimpleAction::new("shell", None);
            shell.connect_activate(glib::clone!(@weak obj => move |_, _| {
                // [ref:FIXME]: prevent more than one window from launching.
                crate::api::shell::new_shell_window(obj).present();
            }));
            application.add_action(&shell);
        }

        let settings = gtk::gio::SimpleAction::new("settings", None);
        settings.connect_activate(
            glib::clone!(@strong self.settings as settings, @weak obj as app => move |_, _| {
                let w = settings.borrow().new_property_window(&app, false);
                w.present();
            }),
        );
        let import_glyphs = gtk::gio::SimpleAction::new("project.import.glyphs", None);

        import_glyphs.connect_activate(glib::clone!(@weak window => move |_, _| {
            #[cfg(feature = "python")]
            {
                crate::ufo::import::glyphsapp::import_action_cb(window);
            }
            #[cfg(not(feature = "python"))]
            {
                // [ref:needs_user_doc] Add compilation instructions and/or url to docs.
                let dialog = crate::utils::widgets::new_simple_error_dialog(
                    None,
                    "This application build doesn't include python support. <i>Glyphs</i> import is performed with the <tt>glyphsLib</tt> python3 library.\n\nCompile or install the app with <tt>python</tt> Cargo feature enabled.",
                    None,
                    &window,
                );
                dialog.run();
                dialog.emit_close();
            }
        }));
        let import_ufo2 = gtk::gio::SimpleAction::new("project.import.ufo2", None);

        import_ufo2.connect_activate(glib::clone!(@weak window => move |_, _| {
            #[cfg(feature = "python")]
            {
                crate::ufo::import::ufo2::import_action_cb(window);
            }
            #[cfg(not(feature = "python"))]
            {
                // [ref:needs_user_doc] Add compilation instructions and/or url to docs.
                let dialog = crate::utils::widgets::new_simple_error_dialog(
                    None,
                    "This application build doesn't include python support. <i>UFOv2</i> import is performed with the <tt>fontTools</tt> python3 library.\n\nCompile or install the app with <tt>python</tt> Cargo feature enabled.",
                    None,
                    &window,
                );
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
            dialog.hide();
            let Some(f) = dialog.filename() else { return; };
            let Some(path) = f.to_str() else { return; };
            window.emit_by_name::<()>("open-project", &[&path]);
        }));
        let new_project = gtk::gio::SimpleAction::new("project.new", None);
        {
            new_project.connect_activate(glib::clone!(@weak self.window as window => move |_, _| {
                window.load_project(crate::prelude::Project::default());
            }));
        }
        let undo = gtk::gio::SimpleAction::new("undo", None);
        undo.set_enabled(false);
        undo.connect_activate(glib::clone!(@weak obj as _self => move |_, _| {
            _self.undo_db.borrow_mut().undo();
        }));
        let redo = gtk::gio::SimpleAction::new("redo", None);
        redo.set_enabled(false);
        redo.connect_activate(glib::clone!(@weak obj as _self => move |_, _| {
            _self.undo_db.borrow_mut().redo();
        }));
        {
            let db = self.undo_db.borrow();
            db.bind_property(UndoDatabase::CAN_UNDO, &undo, "enabled")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            db.bind_property(UndoDatabase::CAN_REDO, &redo, "enabled")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }

        let project_properties = gtk::gio::SimpleAction::new("project.properties", None);
        project_properties.connect_activate(
            glib::clone!(@weak self.window as window, @weak obj as app => move |_, _| {
                let w = window.project.borrow().new_property_window(&app, false);
                w.present();
            }),
        );
        let project_save = gtk::gio::SimpleAction::new("project.save", None);
        project_save.connect_activate(glib::clone!(@weak self.window as window => move |_, _| {
            if let Err(err) = window.project.borrow().save() {
                let dialog = crate::utils::widgets::new_simple_error_dialog(
                    Some("Error: could not perform conversion to UFOv3 with glyphsLib"),
                    &err.to_string(),
                    None,
                    window.upcast_ref(),
                );
                dialog.run();
                dialog.emit_close();
            };
        }));
        let project_export = gtk::gio::SimpleAction::new("project.export", None);
        project_export
            .connect_activate(glib::clone!(@weak self.window as window => move |_, _| {
            #[cfg(feature = "python")]
            {
                crate::ufo::export::ufo_compile::export_action_cb(
                    window.clone().upcast(),
                    window.project().clone(),
                );
            }
            #[cfg(not(feature = "python"))]
            {
                // [ref:needs_user_doc] Add compilation instructions and/or url to docs.
                let dialog = crate::utils::widgets::new_simple_error_dialog(
                    None,
                    "This application build doesn't include python support. <i>UFOv3</i> export is performed with the <tt>ufo2ft</tt> python3 library.\n\nCompile or install the app with <tt>python</tt> Cargo feature enabled.",
                    None,
                    window.upcast_ref(),
                );
                dialog.run();
                dialog.emit_close();
            }
        }));
        let bug_report = gtk::gio::SimpleAction::new("bug_report", None);
        bug_report.connect_activate(|_, _| {
            gtk::gio::AppInfo::launch_default_for_uri(
                crate::ISSUE_TRACKER,
                gtk::gio::AppLaunchContext::NONE,
            )
            .unwrap();
        });
        application.add_action(&project_properties);
        application.add_action(&project_save);
        application.add_action(&project_export);
        application.add_action(&import_glyphs);
        application.add_action(&import_ufo2);
        application.add_action(&settings);
        application.add_action(&about);
        application.add_action(&bug_report);
        application.add_action(&open);
        application.add_action(&new_project);
        application.add_action(&undo);
        application.add_action(&redo);
        application.add_action(&quit);
    }

    fn build_system_menu(&self, obj: &Application) {
        let application = obj.upcast_ref::<gtk::Application>();
        let menu_bar = gio::Menu::new();

        {
            let file_menu = gio::Menu::new();
            let import_menu = gio::Menu::new();
            file_menu.append(Some("New"), Some("app.project.new"));
            file_menu.append(Some("Open"), Some("app.project.open"));
            file_menu.append(Some("Save"), Some("app.project.save"));
            import_menu.append(
                Some("Import Glyphs file"),
                Some("app.project.import.glyphs"),
            );
            import_menu.append(
                Some("Import UFOv2 directory"),
                Some("app.project.import.ufo2"),
            );
            file_menu.append_submenu(Some("Import"), &import_menu);
            file_menu.append(Some("Export"), Some("app.project.export"));
            let project_section = gio::Menu::new();
            project_section.append(Some("Properties"), Some("app.project.properties"));
            #[cfg(feature = "python")]
            {
                project_section.append(Some("Open Python Shell"), Some("app.shell"));
            }
            file_menu.append_section(Some("Project"), &project_section);
            file_menu.append(Some("Quit"), Some("app.quit"));
            menu_bar.append_submenu(Some("_File"), &file_menu);
        }

        {
            let edit_menu = gio::Menu::new();
            edit_menu.append(Some("Settings"), Some("app.settings"));
            let undo_section = gio::Menu::new();
            undo_section.append(Some("Undo"), Some("app.undo"));
            undo_section.append(Some("Redo"), Some("app.redo"));
            edit_menu.append_section(Some("Action history"), &undo_section);
            menu_bar.append_submenu(Some("_Edit"), &edit_menu);
        }

        {
            let win_menu = gio::Menu::new();
            win_menu.append(Some("Next tab"), Some("win.next_tab"));
            win_menu.append(Some("Previous tab"), Some("win.prev_tab"));
            menu_bar.append_submenu(Some("_Window"), &win_menu);
        }

        {
            let meta_menu = gio::Menu::new();
            meta_menu.append(Some("Report issue"), Some("app.bug_report"));
            meta_menu.append(Some("About"), Some("app.about"));
            menu_bar.append_submenu(Some("Gerb"), &meta_menu);
        }

        //application.set_app_menu(Some(&menu));
        application.set_menubar(Some(&menu_bar));
    }

    pub fn statusbar(&self) -> gtk::Statusbar {
        self.window.statusbar.clone()
    }

    #[cfg(not(feature = "notifications"))]
    pub fn show_notification(_: &str, _: &str) {}

    #[cfg(feature = "notifications")]
    pub fn show_notification(summary: &str, body: &str) {
        use notify_rust::Notification;

        let mut n = Notification::new();
        n.appname(crate::APPLICATION_NAME)
            .summary(summary)
            .body(body)
            .urgency(notify_rust::Urgency::Normal);
        #[cfg(all(unix, not(target_os = "macos"), not(target_os = "windows")))]
        {
            n.image_data(crate::resources::G_GLYPH.to_rust_image().unwrap());
        }
        if let Err(err) = n.show() {
            eprintln!("Could not display desktop notification: {err}");
        }
    }
}
