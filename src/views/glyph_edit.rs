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
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                obj.imp().mouse.set((0., 0.));
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
                    obj.imp().mouse.set(event.position());
                    if let Some(screen) = _self.window() {
                        let display = screen.display();
                        screen.set_cursor(Some(
                                &gtk::gdk::Cursor::from_name(&display, "grab").unwrap(),
                        ));
                    }
                    _self.queue_draw()
                }

                Inhibit(false)
            }),
        );

        drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |drar: &gtk::DrawingArea, cr: &gtk::cairo::Context| {
            let width = drar.allocated_width() as f64;
            let height = drar.allocated_height() as f64;
            cr.set_source_rgb(1., 1., 1.);
            cr.paint().expect("Invalid cairo surface state");

            let camera = obj.imp().camera.get();
            cr.set_line_width(1.0);
            for &(color, step) in &[(0.8, 5.0), (0.2, 100.0)] {
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

            if let Some(glyph) = obj.imp().glyph.get() {
                //println!("cairo drawing glyph {}", glyph.name);
                cr.translate(camera.0, camera.1);
                glyph.draw(drar, cr, (10.0, 10.0), (500., 800.));
            } else {
                //println!("cairo drawing without glyph");
            }

           Inhibit(false)
        }));
        obj.add(&drawing_area);
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
