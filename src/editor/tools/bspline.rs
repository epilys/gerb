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
use gtk::Inhibit;

use crate::prelude::*;

#[derive(Default)]
pub struct BSplineToolInner;

#[glib::object_subclass]
impl ObjectSubclass for BSplineToolInner {
    const NAME: &'static str = "BSplineTool";
    type ParentType = ToolImpl;
    type Type = BSplineTool;
}

impl ObjectImpl for BSplineToolInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_property::<String>(ToolImpl::NAME, "Create b-spline curve".to_string());
        obj.set_property::<gtk::Image>(
            ToolImpl::ICON,
            crate::resources::icons::BSPLINE_ICON.to_image_widget(),
        );
    }
}

impl ToolImplImpl for BSplineToolInner {
    fn on_button_press_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_button_release_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _obj: &ToolImpl,
        _view: Editor,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        Inhibit(false)
    }
}

impl BSplineToolInner {}

glib::wrapper! {
    pub struct BSplineTool(ObjectSubclass<BSplineToolInner>)
        @extends ToolImpl;
}

impl Default for BSplineTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BSplineTool {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}
