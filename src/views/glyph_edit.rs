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

use glib::{
    clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecString, Value,
};
use gtk::cairo::Matrix;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

use crate::glyphs::{Contour, Glyph, GlyphDrawingOptions, Guideline};
use crate::project::Project;
use crate::utils::{curves::Bezier, Point};
use crate::views::canvas::LayerBuilder;

mod bezier_pen;
mod viewhide;

use super::{Canvas, Transformation, UnitPoint, ViewPoint};

const EM_SQUARE_PIXELS: f64 = 200.0;

#[derive(Debug, Clone)]
enum Tool {
    Panning,
    Manipulate,
}

impl Default for Tool {
    fn default() -> Tool {
        Tool::Manipulate
    }
}

impl Tool {
    fn is_manipulate(&self) -> bool {
        matches!(self, Tool::Manipulate)
    }

    fn is_panning(&self) -> bool {
        matches!(self, Tool::Panning)
    }
}

#[derive(Debug, Clone)]
struct GlyphState {
    app: gtk::Application,
    glyph: Rc<RefCell<Glyph>>,
    reference: Rc<RefCell<Glyph>>,
    drar: Canvas,
    tool: Tool,
}

impl GlyphState {
    fn new(glyph: &Rc<RefCell<Glyph>>, app: gtk::Application, drar: Canvas) -> Self {
        Self {
            app,
            glyph: Rc::new(RefCell::new(glyph.borrow().clone())),
            reference: Rc::clone(glyph),
            drar,
            tool: Tool::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GlyphEditArea {
    app: OnceCell<gtk::Application>,
    glyph: OnceCell<Rc<RefCell<Glyph>>>,
    glyph_state: OnceCell<RefCell<GlyphState>>,
    viewport: Canvas,
    statusbar_context_id: Cell<Option<u32>>,
    overlay: super::Overlay,
    pub toolbar_box: OnceCell<gtk::Box>,
    pub viewhidebox: OnceCell<viewhide::ViewHideBox>,
    zoom_percent_label: OnceCell<gtk::Label>,
    units_per_em: Cell<f64>,
    descender: Cell<f64>,
    x_height: Cell<f64>,
    cap_height: Cell<f64>,
    ascender: Cell<f64>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphEditArea {
    const NAME: &'static str = "GlyphEditArea";
    type Type = GlyphEditView;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GlyphEditArea {
    // Here we are overriding the glib::Object::contructed
    // method. Its what gets called when we create our Object
    // and where we can initialize things.
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.statusbar_context_id.set(None);
        self.viewport.set_mouse(ViewPoint((0.0, 0.0).into()));

        self.viewport.connect_scroll_event(
            clone!(@weak obj => @default-return Inhibit(false), move |drar, event| {
                Inhibit(false)
            }),
        );

        self.viewport.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                match glyph_state.tool {
                    Tool::Manipulate => {
                        match event.button() {
                            gtk::gdk::BUTTON_MIDDLE => {
                                glyph_state.tool = Tool::Panning;
                            },
                            _ => {},
                        }
                    },
                    Tool::Panning => {
                        match event.button() {
                            gtk::gdk::BUTTON_MIDDLE => {
                                glyph_state.tool = Tool::Panning;
                            },
                            _ => {},
                        }
                    },
                }

                viewport.set_mouse(ViewPoint(event.position().into()));
                Inhibit(false)
            }),
        );

        self.viewport.connect_button_release_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                match glyph_state.tool {
                    Tool::Manipulate => {
                    },
                    Tool::Panning => {
                        match event.button() {
                            gtk::gdk::BUTTON_MIDDLE => {
                                glyph_state.tool = Tool::Manipulate;
                            },
                            _ => {},
                        }
                    },
                }

                viewport.set_mouse(ViewPoint(event.position().into()));
                Inhibit(false)
            }),
        );

        self.viewport.connect_motion_notify_event(
            clone!(@weak obj => @default-return Inhibit(false), move |viewport, event| {
                let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
                if glyph_state.tool.is_panning() {
                    let mouse: ViewPoint = viewport.get_mouse();
                    let delta = <_ as Into<Point>>::into(event.position()) - mouse.0;
                    viewport.imp().transformation.move_camera_by_delta(ViewPoint(delta));
                }
                viewport.set_mouse(ViewPoint(event.position().into()));
                viewport.queue_draw();
                Inhibit(false)
            }),
        );

        self.viewport.add_layer(
            LayerBuilder::new()
                .set_name(Some("glyph"))
                .set_active(true)
                .set_hidden(false)
                .set_callback(Some(Box::new(clone!(@weak obj => @default-return Inhibit(false), move |viewport: &Canvas, cr: &gtk::cairo::Context| {
            let inner_fill = viewport.property::<bool>(Canvas::INNER_FILL);
            let scale: f64 = viewport.imp().transformation.property::<f64>(Transformation::SCALE);
            let width: f64 = viewport.property::<f64>(Canvas::VIEW_WIDTH);
            let height: f64 = viewport.property::<f64>(Canvas::VIEW_HEIGHT);
            let units_per_em = obj.property::<f64>(GlyphEditView::UNITS_PER_EM);
            let matrix = viewport.imp().transformation.matrix();
            let ppu = viewport
                .imp()
                .transformation
                .property::<f64>(Transformation::PIXELS_PER_UNIT);

            let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
            let mouse = viewport.get_mouse();
            let unit_mouse = viewport.view_to_unit_point(mouse);
            let UnitPoint(camera) = viewport.imp().transformation.camera();
            let ViewPoint(view_camera) = viewport.unit_to_view_point(UnitPoint(camera));
            cr.save().unwrap();
            //cr.scale(scale, scale);
            cr.transform(matrix);
            cr.save().unwrap();
            cr.set_line_width(2.5);

            obj.imp().new_statusbar_message(&format!("Mouse: ({:.2}, {:.2}), Unit mouse: ({:.2}, {:.2}), Camera: ({:.2}, {:.2}), Size: ({width:.2}, {height:.2}), Scale: {scale:.2}", mouse.0.x, mouse.0.y, unit_mouse.0.x, unit_mouse.0.y, camera.x, camera.y));

            let (unit_width, unit_height) = ((width * scale) * ppu, (height * scale) * ppu);
            cr.restore().unwrap();
            //cr.transform(matrix);

            if viewport.property::<bool>(Canvas::SHOW_TOTAL_AREA) {
                /* Draw em square of units_per_em units: */
                cr.set_source_rgba(210./255., 227./255., 252./255., 0.6);
                cr.rectangle(0., 0., glyph_state.glyph.borrow().width.unwrap_or(units_per_em), 1000.0);
                cr.fill().unwrap();
            }
            /* Draw the glyph */
            cr.move_to(0.0, 0.0);

            if true {
                let options = GlyphDrawingOptions {
                    outline: (0.2, 0.2, 0.2, if inner_fill { 0. } else { 0.6 }),
                    inner_fill: if inner_fill {
                        Some((0., 0., 0., 1.))
                    } else {
                        None
                    },
                    highlight: None,
                    matrix: Matrix::identity(),
                    units_per_em,
                    line_width: 2.0,
                };
                glyph_state.glyph.borrow().draw(cr, options);
            }
            cr.restore().unwrap();

           Inhibit(false)
        }))))
                .build(),
        );
        self.viewport.add_layer(
            LayerBuilder::new()
                .set_name(Some("rules"))
                .set_active(true)
                .set_hidden(true)
                .set_callback(Some(Box::new(Canvas::draw_rulers)))
                .build(),
        );
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

        let panning_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::GRAB_ICON_SVG,
            )),
            Some("panning"),
        );
        panning_button.set_visible(true);
        // FIXME: doesn't seem to work?
        panning_button.set_tooltip_text(Some("Pan"));
        panning_button.connect_clicked(clone!(@weak obj => move |_self| {
        }));

        let manipulate_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::SELECT_ICON_SVG,
            )),
            Some("Manipulate"),
        );
        manipulate_button.set_visible(true);
        // FIXME: doesn't seem to work?
        manipulate_button.set_tooltip_text(Some("Pan"));
        manipulate_button.connect_clicked(clone!(@weak obj => move |_self| {
        }));

        let bezier_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::BEZIER_ICON_SVG,
            )),
            Some("Create Bézier curve"),
        );
        bezier_button.set_visible(true);
        // FIXME: doesn't seem to work?
        bezier_button.set_tooltip_text(Some("Create Bézier curve"));
        bezier_button.connect_clicked(clone!(@weak obj => move |_self| {
        }));

        let bspline_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::BSPLINE_ICON_SVG,
            )),
            Some("Create b-spline curve"),
        );
        bspline_button.set_visible(true);
        // FIXME: doesn't seem to work?
        bspline_button.set_tooltip_text(Some("Create b-spline curve"));

        /*let pen_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::PEN_ICON_SVG,
            )),
            Some("Pen"),
        );
        pen_button.set_visible(true);
        pen_button.set_tooltip_text(Some("Pen"));*/

        let zoom_in_button = gtk::ToolButton::new(
            Some(&crate::resources::svg_to_image_widget(
                crate::resources::ZOOM_IN_ICON_SVG,
            )),
            Some("Zoom in"),
        );
        zoom_in_button.set_visible(true);
        zoom_in_button.set_tooltip_text(Some("Zoom in"));
        zoom_in_button.connect_clicked(clone!(@weak obj => move |_| {
            let t = &obj.imp().viewport.imp().transformation;
            let v = t.property::<f64>(Transformation::SCALE);
            if v < 5.0 {
                t.set_property::<f64>(Transformation::SCALE, v + 0.1);
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
            let t = &obj.imp().viewport.imp().transformation;
            let v = t.property::<f64>(Transformation::SCALE);
            if v > 0.2 {
                t.set_property::<f64>(Transformation::SCALE, v - 0.1);
            }
        }));

        let zoom_percent_label = gtk::Label::new(Some("100%"));
        zoom_percent_label.set_visible(true);
        zoom_percent_label.set_selectable(true); // So that the widget can receive the button-press event
        zoom_percent_label.set_width_chars(5); // So that if 2 digit zoom (<100%) has the same length as a widget with a three digit zoom value. For example 75% and 125% should result in the same width
        zoom_percent_label.set_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK);
        zoom_percent_label.set_tooltip_text(Some("Interface zoom percentage"));

        zoom_percent_label.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                Inhibit(false)
            }),
        );
        self.viewport
            .imp()
            .transformation
            .bind_property(Transformation::SCALE, &zoom_percent_label, "label")
            .transform_to(|_, scale: &Value| {
                let scale: f64 = scale.get().ok()?;
                Some(format!("{:.0}%", scale * 100.).to_value())
            })
            .build();
        let debug_button = gtk::ToolButton::new(gtk::ToolButton::NONE, Some("Debug info"));
        debug_button.set_visible(true);
        debug_button.set_tooltip_text(Some("Debug info"));
        debug_button.connect_clicked(clone!(@weak obj => move |_| {
            let glyph_state = obj.imp().glyph_state.get().unwrap().borrow();
            let glyph = glyph_state.glyph.borrow();
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
        toolbar.add(&panning_button);
        toolbar.set_item_homogeneous(&panning_button, false);
        toolbar.add(&manipulate_button);
        toolbar.set_item_homogeneous(&manipulate_button, false);
        toolbar.add(&bezier_button);
        toolbar.set_item_homogeneous(&bezier_button, false);
        toolbar.add(&bspline_button);
        toolbar.set_item_homogeneous(&bspline_button, false);
        toolbar.add(&zoom_in_button);
        toolbar.set_item_homogeneous(&zoom_in_button, false);
        toolbar.add(&zoom_out_button);
        toolbar.set_item_homogeneous(&zoom_out_button, false);
        toolbar_box.pack_start(&toolbar, false, false, 0);
        toolbar_box.pack_start(&zoom_percent_label, false, false, 0);
        toolbar_box.pack_start(&debug_button, false, false, 0);
        toolbar_box.style_context().add_class("glyph-edit-toolbox");
        let viewhidebox = viewhide::ViewHideBox::new(&self.viewport);
        self.overlay.set_child(&self.viewport);
        self.overlay.add_overlay(
            &gtk::Expander::builder()
                .child(&viewhidebox)
                .expanded(false)
                .visible(true)
                .can_focus(true)
                .tooltip_text("Toggle overlay visibilities")
                .halign(gtk::Align::End)
                .valign(gtk::Align::End)
                .build(),
            true,
        );
        self.overlay.add_overlay(&toolbar_box, true);
        obj.add(&self.overlay);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);

        self.zoom_percent_label
            .set(zoom_percent_label)
            .expect("Failed to initialize window state");
        self.toolbar_box
            .set(toolbar_box)
            .expect("Failed to initialize window state");
        self.viewhidebox
            .set(viewhidebox)
            .expect("Failed to initialize window state");
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        GlyphEditView::TITLE,
                        GlyphEditView::TITLE,
                        GlyphEditView::TITLE,
                        Some("edit glyph"),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        GlyphEditView::CLOSEABLE,
                        GlyphEditView::CLOSEABLE,
                        GlyphEditView::CLOSEABLE,
                        true,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::UNITS_PER_EM,
                        GlyphEditView::UNITS_PER_EM,
                        GlyphEditView::UNITS_PER_EM,
                        1.0,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::X_HEIGHT,
                        GlyphEditView::X_HEIGHT,
                        GlyphEditView::X_HEIGHT,
                        1.0,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::ASCENDER,
                        GlyphEditView::ASCENDER,
                        GlyphEditView::ASCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        700.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::DESCENDER,
                        GlyphEditView::DESCENDER,
                        GlyphEditView::DESCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        -200.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphEditView::CAP_HEIGHT,
                        GlyphEditView::CAP_HEIGHT,
                        GlyphEditView::CAP_HEIGHT,
                        std::f64::MIN,
                        std::f64::MAX,
                        650.0,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            GlyphEditView::TITLE => {
                if let Some(name) = obj
                    .imp()
                    .glyph_state
                    .get()
                    .map(|s| s.borrow().glyph.borrow().name_markup())
                {
                    format!("edit <i>{}</i>", name).to_value()
                } else {
                    "edit glyph".to_value()
                }
            }
            GlyphEditView::CLOSEABLE => true.to_value(),
            GlyphEditView::UNITS_PER_EM => self.units_per_em.get().to_value(),
            GlyphEditView::X_HEIGHT => self.x_height.get().to_value(),
            GlyphEditView::ASCENDER => self.ascender.get().to_value(),
            GlyphEditView::DESCENDER => self.descender.get().to_value(),
            GlyphEditView::CAP_HEIGHT => self.cap_height.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            GlyphEditView::UNITS_PER_EM => {
                self.units_per_em.set(value.get().unwrap());
            }
            GlyphEditView::X_HEIGHT => {
                self.x_height.set(value.get().unwrap());
            }
            GlyphEditView::ASCENDER => {
                self.ascender.set(value.get().unwrap());
            }
            GlyphEditView::DESCENDER => {
                self.descender.set(value.get().unwrap());
            }
            GlyphEditView::CAP_HEIGHT => {
                self.cap_height.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
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

    fn select_object(&self, new_obj: Option<glib::Object>) {
        if let Some(app) = self
            .app
            .get()
            .and_then(|app| app.downcast_ref::<crate::GerbApp>())
        {
            let tabinfo = app.tabinfo();
            tabinfo.set_object(new_obj);
        }
    }
}

glib::wrapper! {
    pub struct GlyphEditView(ObjectSubclass<GlyphEditArea>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl GlyphEditView {
    pub const ASCENDER: &str = Project::ASCENDER;
    pub const CAP_HEIGHT: &str = Project::CAP_HEIGHT;
    pub const CLOSEABLE: &str = "closeable";
    pub const DESCENDER: &str = Project::DESCENDER;
    pub const TITLE: &str = "title";
    pub const UNITS_PER_EM: &str = Project::UNITS_PER_EM;
    pub const X_HEIGHT: &str = Project::X_HEIGHT;

    pub fn new(app: gtk::Application, project: Project, glyph: Rc<RefCell<Glyph>>) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp().glyph.set(glyph.clone()).unwrap();
        ret.imp().app.set(app.clone()).unwrap();
        for property in [
            GlyphEditView::ASCENDER,
            GlyphEditView::CAP_HEIGHT,
            GlyphEditView::DESCENDER,
            GlyphEditView::UNITS_PER_EM,
            GlyphEditView::X_HEIGHT,
        ] {
            project
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        ret.imp().viewport.imp().transformation.set_property::<f64>(
            Transformation::PIXELS_PER_UNIT,
            {
                let val = EM_SQUARE_PIXELS / project.property::<f64>(Project::UNITS_PER_EM);
                val
            },
        );
        ret.imp()
            .viewport
            .imp()
            .transformation
            .bind_property(
                Transformation::PIXELS_PER_UNIT,
                &project,
                Project::UNITS_PER_EM,
            )
            .transform_from(|_, units_per_em: &Value| {
                let units_per_em: f64 = units_per_em.get().ok()?;
                Some((EM_SQUARE_PIXELS / units_per_em).to_value())
            })
            .build();
        app.downcast_ref::<crate::GerbApp>()
            .unwrap()
            .imp()
            .settings
            .borrow()
            .bind_property(
                Canvas::WARP_CURSOR,
                &ret.imp().viewport,
                Canvas::WARP_CURSOR,
            )
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        ret.imp()
            .glyph_state
            .set(RefCell::new(GlyphState::new(
                &glyph,
                app,
                ret.imp().viewport.clone(),
            )))
            .expect("Failed to create glyph state");
        ret
    }
}
