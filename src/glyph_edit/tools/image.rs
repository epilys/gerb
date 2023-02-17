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

use super::tool_impl::*;
use cairo::ImageSurface;

use crate::prelude::*;

#[derive(Default)]
pub struct ImageToolInner {
    image_data: RefCell<Option<ImageSurface>>,
    matrix: Cell<cairo::Matrix>,
    color: Cell<Option<Color>>,
    layer: OnceCell<Layer>,
    glyph: OnceCell<Rc<RefCell<Glyph>>>,
    active: Cell<bool>,
    descender: Cell<f64>,
    ascender: Cell<f64>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageToolInner {
    const NAME: &'static str = "ImageTool";
    type ParentType = ToolImpl;
    type Type = ImageTool;
}

impl ObjectImpl for ImageToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_property::<bool>(ImageTool::ACTIVE, false);
        obj.set_property::<String>(ToolImpl::NAME, "image".to_string());
        obj.set_property::<String>(ToolImpl::DESCRIPTION, "Glyph image".to_string());
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::icons::RECTANGLE_ICON.to_image_widget(),
        );
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        ImageTool::ACTIVE,
                        ImageTool::ACTIVE,
                        ImageTool::ACTIVE,
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecDouble::new(
                        ImageTool::ASCENDER,
                        ImageTool::ASCENDER,
                        ImageTool::ASCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        700.0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecDouble::new(
                        ImageTool::DESCENDER,
                        ImageTool::DESCENDER,
                        ImageTool::DESCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        -200.0,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            ImageTool::ACTIVE => self.active.get().to_value(),
            ImageTool::ASCENDER => self.ascender.get().to_value(),
            ImageTool::DESCENDER => self.descender.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            ImageTool::ACTIVE => self.active.set(value.get().unwrap()),
            ImageTool::ASCENDER => {
                self.ascender.set(value.get().unwrap());
            }
            ImageTool::DESCENDER => {
                self.descender.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl ToolImplImpl for ImageToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        if !self.active.get() {
            return Inhibit(false);
        }
        Inhibit(false)
    }

    fn setup_toolbox(&self, _: &ToolImpl, _: &gtk::Toolbar, view: &GlyphEditView) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("image"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: ContextRef| {
                    ImageTool::draw_layer(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(ImageTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.viewport.add_pre_layer(layer);
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(ImageTool::ACTIVE, true);
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(ImageTool::ACTIVE, false);
        self.parent_on_deactivate(obj, view)
    }
}

impl ImageToolInner {}

glib::wrapper! {
    pub struct ImageTool(ObjectSubclass<ImageToolInner>)
        @extends ToolImpl;
}

impl ImageTool {
    pub const ACTIVE: &str = "active";
    pub const ASCENDER: &str = Project::ASCENDER;
    pub const DESCENDER: &str = Project::DESCENDER;

    pub fn new(glyph: Rc<RefCell<Glyph>>, project: Project) -> Self {
        let ret: Self = glib::Object::new(&[]).unwrap();
        for property in [ImageTool::ASCENDER, ImageTool::DESCENDER] {
            project
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }

        if let Some(image_ref) = glyph.borrow().image.as_ref() {
            if let Some(file_name) = image_ref.file_name.as_ref() {
                // FIXME error handling
                *ret.imp().image_data.borrow_mut() = Some(project.load_image(file_name).unwrap());
                ret.imp().active.set(true);
                ret.imp().color.set(image_ref.color);
                let xx = image_ref.x_scale;
                let yy = image_ref.y_scale;
                let xy = image_ref.xy_scale;
                let yx = image_ref.yx_scale;
                let x0 = image_ref.x_offset;
                let y0 = image_ref.y_offset;
                ret.imp()
                    .matrix
                    .set(cairo::Matrix::new(xx, yx, xy, yy, x0, y0));
            }
        }
        ret.imp().glyph.set(glyph).unwrap();

        ret
    }

    pub fn draw_layer(viewport: &Canvas, cr: ContextRef, obj: GlyphEditView) -> Inhibit {
        let glyph_state = obj.glyph_state.get().unwrap().borrow();
        let t = glyph_state.tools[&ImageTool::static_type()]
            .clone()
            .downcast::<ImageTool>()
            .unwrap();
        if !t.imp().active.get() {
            return Inhibit(false);
        }
        if let Some(surface) = t.imp().image_data.borrow().as_ref() {
            let matrix = viewport.transformation.matrix();
            let ppu = viewport
                .transformation
                .property::<f64>(Transformation::PIXELS_PER_UNIT);
            let units_per_em = viewport
                .transformation
                .property::<f64>(Transformation::UNITS_PER_EM);

            let m = t.imp().matrix.get();
            let (h, w) = (surface.height() as f64, surface.width() as f64);
            let mut gm = cairo::Matrix::identity();
            let width: f64 = ppu
                * viewport
                    .transformation
                    .property::<f64>(Transformation::CONTENT_WIDTH);
            let height: f64 = ppu * (t.imp().ascender.get() - t.imp().descender.get());
            gm.translate(0.0, units_per_em);
            gm.scale(width / w, -height / h);
            gm.scale(1.0 / ppu, 1.0 / ppu);
            cr.transform(matrix);
            cr.transform(gm);
            cr.transform(m);
            cr.set_source_surface(surface, 0.0, 0.0).unwrap();
            cr.paint().unwrap();
        }
        Inhibit(true)
    }
}
