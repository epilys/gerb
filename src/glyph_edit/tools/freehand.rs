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

use nalgebra::base::{Matrix4, RowVector4, *};

use super::{new_contour_action, tool_impl::*};
use crate::glyphs::Contour;
use crate::utils::{curves::Bezier, distance_between_two_points, Point};
use crate::views::canvas::{Canvas, Layer, LayerBuilder, UnitPoint, ViewPoint};
use crate::GlyphEditView;
use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::cairo::Context;
use gtk::Inhibit;
use gtk::{glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;
use std::cell::{Cell, RefCell};

#[derive(Default)]
pub struct FreehandToolInner {
    layer: OnceCell<Layer>,
    active: Cell<bool>,
    down: Cell<bool>,
    cursor: OnceCell<Option<gtk::gdk_pixbuf::Pixbuf>>,
    points: RefCell<Vec<UnitPoint>>,
}

#[glib::object_subclass]
impl ObjectSubclass for FreehandToolInner {
    const NAME: &'static str = "FreehandTool";
    type ParentType = ToolImpl;
    type Type = FreehandTool;
}

impl ObjectImpl for FreehandToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_property::<bool>(FreehandTool::ACTIVE, false);
        obj.set_property::<String>(ToolImpl::NAME, "Create freehand curve".to_string());
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::icons::FREEHAND_ICON.to_image_widget(),
        );
        self.down.set(false);
        self.cursor
            .set(crate::resources::cursors::PEN_CURSOR.to_pixbuf())
            .unwrap();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    FreehandTool::ACTIVE,
                    FreehandTool::ACTIVE,
                    FreehandTool::ACTIVE,
                    true,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            FreehandTool::ACTIVE => self.active.get().to_value(),
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
            FreehandTool::ACTIVE => self.active.set(value.get().unwrap()),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl ToolImplImpl for FreehandToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        if event.button() == gtk::gdk::BUTTON_PRIMARY {
            if !self.down.get() {
                self.down.set(true);
                let mut points = self.points.borrow_mut();
                points.clear();
                let point = viewport.view_to_unit_point(ViewPoint(event.position().into()));
                points.push(point);
            }
            return Inhibit(true);
        }
        Inhibit(false)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        _viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        match (event.button(), self.down.get()) {
            (gtk::gdk::BUTTON_PRIMARY, true) => {
                self.down.set(false);

                Inhibit(true)
            }
            _ => Inhibit(false),
        }
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        _view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        if !self.down.get() {
            return Inhibit(false);
        }
        let mut points = self.points.borrow_mut();
        let point = viewport.view_to_unit_point(ViewPoint(event.position().into()));
        points.push(point);
        //dbg!(event.axis(gtk::gdk::AxisUse::Pressure));

        Inhibit(true)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        let layer =
            LayerBuilder::new()
                .set_name(Some("pen"))
                .set_active(false)
                .set_hidden(true)
                .set_callback(Some(Box::new(clone!(@weak view => @default-return Inhibit(false), move |viewport: &Canvas, cr: &Context| {
                    FreehandTool::draw_layer(viewport, cr, view)
                }))))
                .build();
        self.instance()
            .bind_property(FreehandTool::ACTIVE, &layer, Layer::ACTIVE)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.layer.set(layer.clone()).unwrap();
        view.imp().viewport.add_post_layer(layer);

        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(FreehandTool::ACTIVE, true);
        if let Some(pixbuf) = self.cursor.get().unwrap().clone() {
            view.imp().viewport.set_cursor_from_pixbuf(pixbuf);
        } else {
            view.imp().viewport.set_cursor("grab");
        }
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.instance()
            .set_property::<bool>(FreehandTool::ACTIVE, false);
        view.imp().viewport.set_cursor("default");
        self.parent_on_deactivate(obj, view)
    }
}

fn generate_basis_matrix() -> Matrix4<f64> {
    let n: usize = 4;
    fn binom(n: usize, k: usize) -> f64 {
        let mut res = 1.0;
        for i in 0..k {
            res = (res * ((n - i) as f64)) / ((i + 1) as f64);
        }
        res
    }

    let mut ret = Matrix4::zeros();

    // populate the main diagonal
    for i in 0..4 {
        ret[(i, i)] = binom(3, i);
    }

    // compute the remaining values
    for c in 0..n {
        for r in (c + 1)..n {
            let sign = if (r + c) % 2 == 0 { 1.0 } else { -1.0 };
            let value = binom(r, c) * ret[(r, r)];
            ret[(r, c)] = sign * value;
        }
    }

    ret
}

fn form_tmatrix([t1, t2, t3, t4]: [f64; 4], _n: usize) -> (Matrix4<f64>, Matrix4<f64>) {
    let tt = Matrix4::<f64>::from_rows(&[
        RowVector4::new(t1.powi(0), t2.powi(0), t3.powi(0), t4.powi(0)),
        RowVector4::new(t1.powi(1), t2.powi(1), t3.powi(1), t4.powi(1)),
        RowVector4::new(t1.powi(2), t2.powi(2), t3.powi(2), t4.powi(2)),
        RowVector4::new(t1.powi(3), t2.powi(3), t3.powi(3), t4.powi(3)),
    ]);
    let t = tt.transpose();

    (t, tt)
}

impl FreehandToolInner {
    fn fit_cubic_bezier_to_points(pts: &[UnitPoint]) -> Bezier {
        const T_VALUES: [f64; 4] = [0.0, 0.33, 0.67, 1.0];
        assert!(!pts.is_empty());
        let (t, tt) = form_tmatrix(T_VALUES, 4);
        let m = generate_basis_matrix();
        let m1 = {
            let mut t = m;
            t.try_inverse_mut();
            t
        };

        let ttt1 = {
            let mut t = tt * t;
            t.try_inverse_mut();
            t
        };
        let step1 = ttt1 * tt;
        let step2 = m1 * step1;

        let x = Matrix4x1::<f64>::from_iterator(pts[0..4].iter().map(|p| p.0.x));
        let cx = step2 * x;

        let y = Matrix4x1::<f64>::from_iterator(pts[0..4].iter().map(|p| p.0.y));
        let cy = step2 * y;
        let bpoints = cx.data.0[0]
            .iter()
            .enumerate()
            .map(|(i, r)| Point::new(*r, cy.data.0[0][i]))
            .collect::<Vec<Point>>();

        /*
          const tm = this.formTMatrix(tvalues, n),
                T = tm.T,
                Tt = tm.Tt,
                M = this.generateBasisMatrix(n),
                M1 = M.invert(),
                TtT1 = Tt.multiply(T).invert(),
                step1 = TtT1.multiply(Tt),
                step2 = M1.multiply(step1),
                // almost there...
                X = new Matrix(points.map((v) => [v.x])),
                Cx = step2.multiply(X),
                x = Cx.data,
                // almost...
                Y = new Matrix(points.map((v) => [v.y])),
                Cy = step2.multiply(Y),
                y = Cy.data,
                // last step!
                bpoints = x.map((r,i) => ({x: r[0], y: y[i][0]}));

        formTMatrix(row, n) {
                nalgebra::base::Matrix3
                let m1 = Matrix3x3::from_rows(&[
            RowVector3::new(1.1, 1.2, 1.3),
            RowVector3::new(1.1, 1.2, 1.3),
            RowVector3::new(2.1, 2.2, 2.3)
        ]);
          // it's actually easier to create the transposed
          // version, and then (un)transpose that to get T!
          let data = [];
          for (var i = 0; i < n; i++) {
            data.push(row.map((v) => v ** i));
          }
          const Tt = new Matrix(n, n, data);
          const T = Tt.transpose();
          return { T, Tt };
        }

        generateBasisMatrix(n) {
          const M = new Matrix(n, n);

          // populate the main diagonal
          var k = n - 1;
          for (let i = 0; i < n; i++) {
            M.set(i, i, binomial(k, i));
          }

          // compute the remaining values
          for (var c = 0, r; c < n; c++) {
            for (r = c + 1; r < n; r++) {
              var sign = (r + c) % 2 === 0 ? 1 : -1;
              var value = binomial(r, c) * M.get(r, r);
              M.set(r, c, sign * value);
            }
          }

          return M;
        }
                fn form_tmatrix(

                 */

        Bezier::new(bpoints)
    }
}

glib::wrapper! {
    pub struct FreehandTool(ObjectSubclass<FreehandToolInner>)
        @extends ToolImpl;
}

impl Default for FreehandTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FreehandTool {
    pub const ACTIVE: &str = "active";

    pub const PRESSURE_DEFAULT: f64 = 0.5;

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn draw_layer(viewport: &Canvas, cr: &Context, obj: GlyphEditView) -> Inhibit {
        use crate::utils::colors::*;

        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
        if FreehandTool::static_type() != glyph_state.active_tool {
            return Inhibit(false);
        }
        let t = glyph_state.tools[&glyph_state.active_tool]
            .clone()
            .downcast::<FreehandTool>()
            .unwrap();
        if !t.imp().active.get() {
            return Inhibit(false);
        }
        let inner_fill = viewport.property::<bool>(Canvas::INNER_FILL);
        let line_width = obj
            .imp()
            .settings
            .get()
            .unwrap()
            .property::<f64>(crate::Settings::LINE_WIDTH);
        let outline = Color::new_alpha(0.2, 0.2, 0.2, if inner_fill { 0.0 } else { 0.6 });
        let matrix = viewport.imp().transformation.matrix();
        let handle_size: f64 = 3.0;

        cr.save().expect("Invalid cairo surface state");
        cr.transform(matrix);
        cr.set_line_width(line_width);
        cr.set_source_color_alpha(outline);

        let mut i = 0;
        let mut points = t.imp().points.borrow_mut();
        let contour_index = glyph_state.glyph.borrow().contours.len();
        let contour = if contour_index == 0
            || !glyph_state.glyph.borrow().contours[contour_index - 1]
                .property::<bool>(Contour::OPEN)
        {
            let contour = Contour::new();
            let subaction = glyph_state.add_contour(&contour, contour_index);
            let mut action =
                new_contour_action(glyph_state.glyph.clone(), contour.clone(), subaction);
            (action.redo)();
            glyph_state.add_undo_action(action);
            contour
        } else {
            glyph_state.glyph.borrow().contours.last().unwrap().clone()
        };

        while i + 4 <= points.len() {
            let current_curve = FreehandToolInner::fit_cubic_bezier_to_points(&points[i..(i + 4)]);
            current_curve.set_property(Bezier::SMOOTH, true);
            contour.push_curve(current_curve);
            i += 3;
        }
        if points.len() > 4 {
            let last_point = &points[points.len() - 1];
            if dbg!(distance_between_two_points(last_point.0, points[0].0)) < 4.0 {
                contour.close();
            }
        }

        let draw_point = |p: Point| {
            if inner_fill {
                cr.set_source_rgba(0.9, 0.9, 0.9, 1.0);
            } else {
                cr.set_source_rgba(0.0, 0.0, 1.0, 0.5);
            }
            cr.arc(p.x, p.y, handle_size / 2.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap();
            cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
            cr.arc(
                p.x,
                p.y,
                handle_size / 2.0 + 1.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cr.stroke().unwrap();
        };

        for p in points.iter() {
            draw_point(p.0);
        }
        points.drain(0..i.saturating_sub(3));
        cr.restore().expect("Invalid cairo surface state");

        Inhibit(true)
    }
}
