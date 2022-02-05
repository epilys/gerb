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

use glib::{clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::glyphs::Glyph;
use crate::project::Project;

const EM_SQUARE_PIXELS: f64 = 200.0;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum MotionMode {
    _Zoom = 0,
    Pan,
}

#[derive(Debug, Default)]
pub struct GlyphEditArea {
    app: OnceCell<gtk::Application>,
    glyph: OnceCell<Glyph>,
    drawing_area: OnceCell<gtk::DrawingArea>,
    hovering: Cell<Option<(usize, usize)>>,
    statusbar_context_id: Cell<Option<u32>>,
    overlay: OnceCell<gtk::Overlay>,
    pub toolbar_box: OnceCell<gtk::Box>,
    points: OnceCell<Arc<Mutex<Vec<(i64, i64)>>>>,
    kd_tree: OnceCell<Arc<Mutex<crate::utils::range_query::KdTree>>>,
    zoom_percent_label: OnceCell<gtk::Label>,
    camera: Cell<(f64, f64)>,
    mouse: Cell<(f64, f64)>,
    zoom: Cell<f64>,
    button: Cell<Option<MotionMode>>,
    project: OnceCell<Arc<Mutex<Option<Project>>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphEditArea {
    const NAME: &'static str = "GlyphEditArea";
    type Type = GlyphEditView;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GlyphEditArea {
    // Here we are overriding the glib::Objcet::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.hovering.set(None);
        self.statusbar_context_id.set(None);
        self.camera.set((EM_SQUARE_PIXELS, 100.));
        self.mouse.set((0., 0.));
        self.zoom.set(1.);
        self.points.set(Arc::new(Mutex::new(vec![]))).unwrap();
        self.kd_tree
            .set(Arc::new(Mutex::new(
                crate::utils::range_query::KdTree::new(&[]),
            )))
            .unwrap();

        let drawing_area = gtk::DrawingArea::builder()
            .expand(true)
            .visible(true)
            .build();
        drawing_area.set_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK
                | gtk::gdk::EventMask::BUTTON_RELEASE_MASK
                | gtk::gdk::EventMask::BUTTON_MOTION_MASK
                | gtk::gdk::EventMask::SCROLL_MASK
                | gtk::gdk::EventMask::SMOOTH_SCROLL_MASK
                | gtk::gdk::EventMask::POINTER_MOTION_MASK,
        );
        drawing_area.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                obj.imp().mouse.set(event.position());
                if event.button() == gtk::gdk::BUTTON_MIDDLE {
                    obj.imp().button.set(Some(MotionMode::Pan));
                }
                if event.button() == 3 {
                    return Inhibit(true);
                }
                Inhibit(false)
            }),
        );
        drawing_area.connect_button_release_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, _event| {
                //obj.imp().mouse.set((0., 0.));
                obj.imp().button.set(None);
                if let Some(screen) = _self.window() {
                    let display = screen.display();
                    screen.set_cursor(Some(
                            &gtk::gdk::Cursor::from_name(&display, "default").unwrap(),
                    ));
                }

                Inhibit(false)
            }),
        );
        drawing_area.connect_motion_notify_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                if let Some(MotionMode::Pan) = obj.imp().button.get(){
                    let mut camera = obj.imp().camera.get();
                    let mouse = obj.imp().mouse.get();
                    camera.0 += event.position().0 - mouse.0;
                    camera.1 += event.position().1 - mouse.1;
                    obj.imp().camera.set(camera);
                    if let Some(screen) = _self.window() {
                        let display = screen.display();
                        screen.set_cursor(Some(
                                &gtk::gdk::Cursor::from_name(&display, "grab").unwrap(),
                        ));
                    }
                } else {
                    let zoom_factor = obj.imp().zoom.get();
                    let camera = obj.imp().camera.get();
                    let position = event.position();
                    let f =  1000. / EM_SQUARE_PIXELS ;
                    let position = (((position.0*f - camera.0*f * zoom_factor)/zoom_factor) as i64, ((position.1*f-camera.1*f * zoom_factor)/zoom_factor) as i64);
                    let pts = obj.imp().kd_tree.get().unwrap().lock().unwrap().query(position, 10);
                    if pts.is_empty() {
                        obj.imp().hovering.set(None);
                        if let Some(screen) = _self.window() {
                            let display = screen.display();
                            screen.set_cursor(Some(
                                    &gtk::gdk::Cursor::from_name(&display, "default").unwrap(),
                            ));
                        }
                    } else if let Some(screen) = _self.window() {
                        let glyph = obj.imp().glyph.get().unwrap();
                        for (ic, contour) in glyph.contours.iter().enumerate() {
                            for (jc, curve) in contour.curves.iter().enumerate() {
                                for p in &pts {
                                    if curve.points.contains(&p.1) {
                                        obj.imp().new_statusbar_message(&format!("{:?}", curve));
                                        obj.imp().hovering.set(Some((ic, jc)));
                                        break;
                                    }
                                }
                            }
                        }
                        let display = screen.display();
                        screen.set_cursor(Some(
                                &gtk::gdk::Cursor::from_name(&display, "grab").unwrap(),
                        ));
                    }
                }
                obj.imp().mouse.set(event.position());
                _self.queue_draw();

                Inhibit(false)
            }),
        );

        drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |drar: &gtk::DrawingArea, cr: &gtk::cairo::Context| {
            let zoom_factor = obj.imp().zoom.get();
            cr.save().unwrap();
            cr.scale(zoom_factor, zoom_factor);
            let (units_per_em, x_height, cap_height, _ascender, _descender) = {
                let mutex = obj.imp().project.get().unwrap();
                let lck = mutex.lock().unwrap();
                if lck.is_none() {
                    return Inhibit(false);
                }
                let p = lck.as_ref().unwrap();
                (p.units_per_em, p.x_height, p.cap_height, p.ascender, p.descender)
            };
            let f = EM_SQUARE_PIXELS/units_per_em ;
            let width = drar.allocated_width() as f64;
            let height = drar.allocated_height() as f64;
            cr.set_source_rgb(1., 1., 1.);
            cr.paint().expect("Invalid cairo surface state");

            cr.set_line_width(1.0);

            let camera = obj.imp().camera.get();
            let mouse = obj.imp().mouse.get();

            for &(color, step) in &[(0.9, 5.0), (0.8, 100.0)] {
                cr.set_source_rgb(color, color, color);
                let mut y = (camera.1 % step).floor() + 0.5;
                while y < (height/zoom_factor) {
                    cr.move_to(0., y);
                    cr.line_to(width/zoom_factor, y);
                    y += step;
                }
                cr.stroke().unwrap();
                let mut x = (camera.0 % step).floor() + 0.5;
                while x < (width/zoom_factor) {
                    cr.move_to(x, 0.);
                    cr.line_to(x, height/zoom_factor);
                    x += step;
                }
                cr.stroke().unwrap();
            }
            /* Draw em square of 1000 units: */

            cr.save().unwrap();
            cr.translate(camera.0, camera.1);

            cr.set_source_rgba(210./255., 227./255., 252./255., 0.6);
            let glyph_width = f*obj.imp().glyph.get().unwrap().width.unwrap_or(1000) as f64;
            cr.rectangle(0., 0., glyph_width, EM_SQUARE_PIXELS);
            cr.fill().unwrap();

            /* Draw x-height */
            cr.set_source_rgba(0., 0., 1., 0.6);
            cr.set_line_width(2.0);
            cr.move_to(0., x_height*0.2);
            cr.line_to(glyph_width*1.2, x_height*0.2);
            cr.stroke().unwrap();
            cr.move_to(glyph_width*1.2, x_height*0.2);
            cr.show_text("x-height").unwrap();

            /* Draw baseline */
            cr.move_to(0., units_per_em*0.2);
            cr.line_to(glyph_width*1.2, units_per_em*0.2);
            cr.stroke().unwrap();
            cr.move_to(glyph_width*1.2, units_per_em*0.2);
            cr.show_text("baseline").unwrap();

            /* Draw cap height */
            cr.move_to(0., EM_SQUARE_PIXELS-cap_height*0.2);
            cr.line_to(glyph_width*1.2, EM_SQUARE_PIXELS-cap_height*0.2);
            cr.stroke().unwrap();
            cr.move_to(glyph_width*1.2, EM_SQUARE_PIXELS-cap_height*0.2);
            cr.show_text("cap height").unwrap();

            /* Draw the glyph */

            if let Some(glyph) = obj.imp().glyph.get() {
                //println!("cairo drawing glyph {}", glyph.name);
                glyph.draw(drar, cr, (0.0, 0.0), (EM_SQUARE_PIXELS, EM_SQUARE_PIXELS), obj.imp().hovering.get());
                cr.save().unwrap();
                cr.set_source_rgba(0.0, 0.0, 1.0, 0.5);
                for p in obj.imp().points.get().unwrap().lock().unwrap().iter() {
                    cr.set_line_width(0.5);
                    //println!("darwing point ({}, {}) at ({}, {})", p.0, p.1, p.0 as f64*f*zoom_factor-50., p.1 as f64 * f*zoom_factor-50.);
                    cr.rectangle(p.0 as f64* f - 2.5, p.1 as f64* f - 2.5, 5., 5.);
                    cr.stroke().unwrap();
                }
                cr.restore().unwrap();

                /*for c in &glyph.curves {
                    for &(x, y) in &c.points {
                        cr.rectangle(x as f64, y as f64, 5., 5.);
                        cr.stroke_preserve().expect("Invalid cairo surface state");
                    }
                }
                */
            } else {
                //println!("cairo drawing without glyph");
            }
            cr.restore().unwrap();
            cr.restore().unwrap();

            /* Draw rulers */
            const RULER_BREADTH: f64 = 13.;
            cr.rectangle(0., 0., width, RULER_BREADTH);
            cr.set_source_rgb(1., 1., 1.);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0., 0., 0.);
            cr.stroke_preserve().unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.move_to(mouse.0, 0.);
            cr.line_to(mouse.0, RULER_BREADTH);
            cr.stroke().unwrap();
            cr.move_to(mouse.0+1., 2.*RULER_BREADTH/3.);
            cr.set_font_size(6.);
            cr.show_text(&format!("{:.0}", (mouse.0 - camera.0 * zoom_factor) / (f * zoom_factor))).unwrap();


            cr.rectangle(0., RULER_BREADTH, RULER_BREADTH, height);
            cr.set_source_rgb(1., 1., 1.);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0., 0., 0.);
            cr.stroke_preserve().unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.move_to(0., mouse.1);
            cr.line_to(RULER_BREADTH, mouse.1);
            cr.stroke().unwrap();
            cr.move_to(2.*RULER_BREADTH/3., mouse.1-1.);
            cr.set_font_size(6.);
            cr.save().expect("Invalid cairo surface state");
            cr.rotate(-std::f64::consts::FRAC_PI_2);
            cr.show_text(&format!("{:.0}", (mouse.1 - camera.1 * zoom_factor) / (f * zoom_factor))).unwrap();
            cr.restore().expect("Invalid cairo surface state");


           Inhibit(false)
        }));
        drawing_area.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
            let zoom_factor = obj.imp().zoom.get();
            let camera = obj.imp().camera.get();
            let position = event.position();
            let f =  1000. / EM_SQUARE_PIXELS ;
            let position = (((position.0*f - camera.0*f * zoom_factor)/zoom_factor) as i64, ((position.1*f-camera.1*f * zoom_factor)/zoom_factor) as i64);
            //std::dbg!(obj.imp().kd_tree.get().unwrap().lock().unwrap());
            std::dbg!(obj.imp().kd_tree.get().unwrap().lock().unwrap().query(position, 10));
            Inhibit(true)
        }));
        let toolbar_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .spacing(5)
            .visible(true)
            .can_focus(true)
            .build();
        let toolbar = gtk::Toolbar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            //.toolbar_style(gtk::ToolbarStyle::Both)
            .visible(true)
            .can_focus(true)
            .build();

        let edit_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::GRAB_ICON_SVG,
            )),
            Some("Edit"),
        );
        edit_button.set_visible(true);
        // FIXME: doesn't seem to work?
        edit_button.set_tooltip_text(Some("Pan"));

        let pen_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::PEN_ICON_SVG,
            )),
            Some("Pen"),
        );
        pen_button.set_visible(true);
        pen_button.set_tooltip_text(Some("Pen"));

        let zoom_in_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::ZOOM_IN_ICON_SVG,
            )),
            Some("Zoom in"),
        );
        zoom_in_button.set_visible(true);
        zoom_in_button.set_tooltip_text(Some("Zoom in"));
        zoom_in_button.connect_clicked(clone!(@weak obj => move |_| {
            let imp = obj.imp();
            let zoom_factor = imp.zoom.get() + 0.25;
            if zoom_factor < 7.25 {
                imp.zoom.set(zoom_factor);
                imp.zoom_percent_label.get().unwrap().set_text(&format!("{:.0}%", zoom_factor * 100.));
                imp.overlay.get().unwrap().queue_draw();
            }
        }));
        let zoom_out_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::ZOOM_OUT_ICON_SVG,
            )),
            Some("Zoom out"),
        );
        zoom_out_button.set_visible(true);
        zoom_out_button.set_tooltip_text(Some("Zoom out"));
        zoom_out_button.connect_clicked(clone!(@weak obj => move |_| {
            let imp = obj.imp();
            let zoom_factor = imp.zoom.get() - 0.25;
            if zoom_factor > 0.05 {
                imp.zoom.set(zoom_factor);
                imp.zoom_percent_label.get().unwrap().set_text(&format!("{:.0}%", zoom_factor * 100.));
                imp.overlay.get().unwrap().queue_draw();
            }
        }));

        drawing_area.connect_scroll_event(clone!(@weak obj => @default-return Inhibit(false), move |_drar, event| {
            if event.state().contains(gtk::gdk::ModifierType::SHIFT_MASK) {
                obj.imp().mouse.set(event.position());
                let (mut dx, mut dy) = event.delta();
                if event.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    if dy.abs() > dx.abs() {
                        dx = dy;
                    }
                    dy = 0.0;
                }

                let dx = dx * EM_SQUARE_PIXELS/2. * obj.imp().zoom.get();
                let dy = dy * EM_SQUARE_PIXELS/2. * obj.imp().zoom.get();
                let mut camera = obj.imp().camera.get();
                let mouse = obj.imp().mouse.get();
                camera.0 += event.position().0 - mouse.0 - dx;
                camera.1 += event.position().1 - mouse.1 - dy;
                obj.imp().camera.set(camera);
                _drar.queue_draw();
                return Inhibit(false);
            }
            match event.direction() {
                gtk::gdk::ScrollDirection::Up | gtk::gdk::ScrollDirection::Down | gtk::gdk::ScrollDirection::Smooth => {
                    /* zoom */
                    let (_, dy) = event.delta();
                    let imp = obj.imp();
                    let mut camera =imp.camera.get();
                    let mouse = imp.mouse.get();
                    camera.0 += event.position().0 - mouse.0;
                    camera.1 += event.position().1 - mouse.1;
                    imp.mouse.set(event.position());
                    imp.camera.set(camera);
                    let zoom_factor = imp.zoom.get() - 0.25* dy;
                    if zoom_factor > 0.05 && zoom_factor < 7.25 {
                        imp.zoom.set(zoom_factor);
                        imp.zoom_percent_label.get().unwrap().set_text(&format!("{:.0}%", zoom_factor * 100.));
                        imp.overlay.get().unwrap().queue_draw();
                    }
                        _drar.queue_draw();
                },
                _ => {
                    /* ignore */
                }
            }
            Inhibit(false)
        }));

        let zoom_percent_label = gtk::Label::new(Some("100%"));
        zoom_percent_label.set_visible(true);
        zoom_percent_label.set_selectable(true); // So that the widget can receive the button-press event
        zoom_percent_label.set_width_chars(5); // So that if 2 digit zoom (<100%) has the same length as a widget with a three digit zoom value. For example 75% and 125% should result in the same width
        zoom_percent_label.set_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK);
        zoom_percent_label.set_tooltip_text(Some("Interface zoom percentage"));

        zoom_percent_label.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                if event.button() == gtk::gdk::BUTTON_PRIMARY &&
                 event.event_type() == gtk::gdk::EventType::DoubleButtonPress {
                    let imp = obj.imp();
                    let zoom_factor = imp.zoom.get();
                    if (zoom_factor - 1.0).abs() > f64::EPSILON {
                        imp.zoom.set(1.0);
                        imp.zoom_percent_label.get().unwrap().set_text("100%");
                        imp.overlay.get().unwrap().queue_draw();
                    }
                }
                Inhibit(false)
            }),
        );
        let debug_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Debug info"));
        debug_button.set_visible(true);
        debug_button.set_tooltip_text(Some("Debug info"));
        debug_button.connect_clicked(clone!(@weak obj => move |_| {
            let glyph = obj.imp().glyph.get().unwrap();
            let window = gtk::Window::new(gtk::WindowType::Toplevel);
            window.set_default_size(640, 480);
            let hbox = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .valign(gtk::Align::Fill)
                .expand(false)
                .spacing(5)
                .visible(true)
                .can_focus(true)
                .build();
            let glyph_info = gtk::Label::new(Some(&format!("{:#?}", glyph.contours)));
            glyph_info.set_halign(gtk::Align::Start);
            let scrolled_window = gtk::ScrolledWindow::builder()
                .expand(true)
                .visible(true)
                .can_focus(true)
                .margin_start(5)
                .build();
            scrolled_window.set_child(Some(&glyph_info));
            hbox.pack_start(&scrolled_window, true, true, 0);
            hbox.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, true, 0);
            let scrolled_window = gtk::ScrolledWindow::builder()
                .expand(true)
                .visible(true)
                .can_focus(true)
                .margin_start(5)
                .build();
            let glif_info = gtk::Label::new(Some(&glyph.glif_source));
            glif_info.set_halign(gtk::Align::Start);
            scrolled_window.set_child(Some(&glif_info));
            hbox.pack_start(&scrolled_window, true, true, 0);
            window.add(&hbox);
            window.show_all();
        }));
        toolbar.add(&edit_button);
        toolbar.set_item_homogeneous(&edit_button, false);
        toolbar.add(&pen_button);
        toolbar.set_item_homogeneous(&pen_button, false);
        toolbar.add(&zoom_in_button);
        toolbar.set_item_homogeneous(&zoom_in_button, false);
        toolbar.add(&zoom_out_button);
        toolbar.set_item_homogeneous(&zoom_out_button, false);
        toolbar_box.pack_start(&toolbar, false, false, 0);
        toolbar_box.pack_start(&zoom_percent_label, false, false, 0);
        toolbar_box.pack_start(&debug_button, false, false, 0);
        toolbar_box.style_context().add_class("glyph-edit-toolbox");
        let overlay = gtk::Overlay::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        overlay.add_overlay(&drawing_area);
        overlay.add_overlay(&toolbar_box);
        obj.add(&overlay);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);

        self.zoom_percent_label
            .set(zoom_percent_label)
            .expect("Failed to initialize window state");
        self.overlay
            .set(overlay)
            .expect("Failed to initialize window state");
        self.toolbar_box
            .set(toolbar_box)
            .expect("Failed to initialize window state");
        self.drawing_area
            .set(drawing_area)
            .expect("Failed to initialize window state");
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        // Name
                        "tab-title",
                        // Nickname
                        "tab-title",
                        // Short description
                        "tab-title",
                        // Default value
                        Some("edit glyph"),
                        // The property can be read and written to
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "tab-can-close",
                        "tab-can-close",
                        "tab-can-close",
                        true,
                        ParamFlags::READABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "tab-title" => {
                if let Some(name) = obj.imp().glyph.get().map(|g| g.name_markup()) {
                    format!("edit <i>{}</i>", name).to_value()
                } else {
                    "edit glyph".to_value()
                }
            }
            "tab-can-close" => true.to_value(),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for GlyphEditArea {}
impl ContainerImpl for GlyphEditArea {}
impl BinImpl for GlyphEditArea {}

impl GlyphEditArea {
    fn new_statusbar_message(&self, msg: &str) {
        if let Some(app) = self
            .app
            .get()
            .and_then(|app| app.downcast_ref::<crate::GerbApp>())
        {
            let statusbar = app.statusbar();
            if self.statusbar_context_id.get().is_none() {
                self.statusbar_context_id.set(Some(
                    statusbar
                        .context_id(&format!("GlyphEditArea-{:?}", &self.glyph.get().unwrap())),
                ));
            }
            if let Some(cid) = self.statusbar_context_id.get().as_ref() {
                statusbar.push(*cid, msg);
            }
        }
    }
}

glib::wrapper! {
    pub struct GlyphEditView(ObjectSubclass<GlyphEditArea>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl GlyphEditView {
    pub fn new(app: gtk::Application, project: Arc<Mutex<Option<Project>>>, glyph: Glyph) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        *ret.imp().kd_tree.get().unwrap().lock().unwrap() =
            crate::utils::range_query::KdTree::new(&glyph.points());
        *ret.imp().points.get().unwrap().lock().unwrap() = glyph.points();
        ret.imp().glyph.set(glyph).unwrap();
        ret.imp().app.set(app).unwrap();
        ret.imp().project.set(project).unwrap();
        ret
    }
}
