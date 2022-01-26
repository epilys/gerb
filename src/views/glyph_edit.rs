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

use crate::project::Glyph;

#[derive(Debug, Default)]
pub struct GlyphEditArea {
    app: OnceCell<gtk::Application>,
    glyph: OnceCell<Glyph>,
    drawing_area: OnceCell<gtk::DrawingArea>,
    camera: Cell<(f64, f64)>,
    mouse: Cell<(f64, f64)>,
    button: Cell<Option<u32>>,
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
        self.camera.set((0., 0.));
        self.mouse.set((0., 0.));

        let drawing_area = gtk::DrawingArea::builder()
            .expand(true)
            .visible(true)
            .build();
        drawing_area.set_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK
                | gtk::gdk::EventMask::BUTTON_RELEASE_MASK
                | gtk::gdk::EventMask::POINTER_MOTION_MASK,
        );
        drawing_area.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {

                obj.imp().mouse.set(event.position());
                obj.imp().button.set(Some(event.button()));

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
                if let Some(gtk::gdk::BUTTON_SECONDARY) = obj.imp().button.get(){
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
                }
                obj.imp().mouse.set(event.position());
                _self.queue_draw();

                Inhibit(false)
            }),
        );

        drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |drar: &gtk::DrawingArea, cr: &gtk::cairo::Context| {
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
                while y < height {
                    cr.move_to(0., y);
                    cr.line_to(width, y);
                    y += step;
                }
                cr.stroke().unwrap();
                let mut x = (camera.0 % step).floor() + 0.5;
                while x < width {
                    cr.move_to(x, 0.);
                    cr.line_to(x, height);
                    x += step;
                }
                cr.stroke().unwrap();
            }

            /* Draw em square of 1000 units: */

            cr.save().unwrap();
            cr.translate(camera.0, camera.1);
            cr.set_source_rgba(210./255., 227./255., 252./255., 0.6);
            cr.rectangle(0., 0., 200., 200.);
            cr.fill().unwrap();

            /* Draw the glyph */

            if let Some(glyph) = obj.imp().glyph.get() {
                //println!("cairo drawing glyph {}", glyph.name);
                glyph.draw(drar, cr, (0.0, 0.0), (200., 200.));
                cr.set_source_rgb(1.0, 0.0, 0.0);
                cr.set_line_width(1.5);
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

            /* Draw rulers */
            cr.rectangle(0., 0., width, 11.);
            cr.set_source_rgb(1., 1., 1.);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0., 0., 0.);
            cr.stroke_preserve().unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.move_to(mouse.0, 0.);
            cr.line_to(mouse.0, 11.);
            cr.stroke().unwrap();


            cr.rectangle(0., 0., 11., height);
            cr.set_source_rgb(1., 1., 1.);
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgb(0., 0., 0.);
            cr.stroke_preserve().unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.move_to(0., mouse.1);
            cr.line_to(11., mouse.1);
            cr.stroke().unwrap();


           Inhibit(false)
        }));
        let toolbar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .spacing(5)
            .visible(true)
            .can_focus(true)
            .build();
        let edit_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Edit"));
        edit_button.set_visible(true);
        let pen_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Pen"));
        pen_button.set_visible(true);
        let zoom_in_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Zoom in"));
        zoom_in_button.set_visible(true);
        let zoom_out_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Zoom out"));
        zoom_out_button.set_visible(true);
        let zoom_percent_label = gtk::Label::new(Some("100%"));
        zoom_percent_label.set_visible(true);
        toolbar.pack_start(&edit_button, false, false, 0);
        toolbar.pack_start(&pen_button, false, false, 0);
        toolbar.pack_start(&zoom_in_button, false, false, 0);
        toolbar.pack_start(&zoom_out_button, false, false, 0);
        toolbar.pack_start(&zoom_percent_label, false, false, 0);
        toolbar.style_context().add_class("glyph-edit-toolbox");
        let overlay = gtk::Overlay::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        overlay.add_overlay(&drawing_area);
        overlay.add_overlay(&toolbar);
        obj.add(&overlay);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);

        self.drawing_area
            .set(drawing_area)
            .expect("Failed to initialize window state");
    }
}

impl WidgetImpl for GlyphEditArea {}
impl ContainerImpl for GlyphEditArea {}
impl BinImpl for GlyphEditArea {}

glib::wrapper! {
    pub struct GlyphEditView(ObjectSubclass<GlyphEditArea>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl GlyphEditView {
    pub fn new(app: gtk::Application, glyph: Glyph) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp().glyph.set(glyph).unwrap();
        ret.imp().app.set(app).unwrap();
        ret
    }
}
