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

#[macro_use]
extern crate glib;

pub mod app;
pub mod glyph_edit;
pub mod glyphs;
pub mod project;
pub mod resources;
pub mod ufo;
pub mod unicode;
pub mod utils;
pub mod views;
pub mod window;

pub const APPLICATION_NAME: &str = "gerb";
pub const APPLICATION_ID: &str = "com.epilys.gerb";

pub mod prelude {
    pub use super::*;
    pub use app::Application;
    pub use app::Settings;
    pub use app::*;
    pub use glyph_edit::*;
    pub use glyphs::Glyph;
    pub use gtk::prelude::*;
    pub use gtk::subclass::prelude::ObjectSubclassIsExt;
    pub use indexmap::{IndexMap, IndexSet};
    pub use project::Project;
    pub use utils::colors::*;
    pub use utils::points::*;
    pub use utils::shortcuts::{Shortcut, ShortcutAction};
    pub use utils::UI_EDITABLE;
    pub use utils::{ContextExt, ContextRef};
    pub use views::{
        canvas::{Layer, LayerBuilder},
        Canvas, Collection, Overlay, Transformation, UnitPoint, ViewPoint,
    };
    pub use window::Workspace;

    pub use glib::prelude::*;
    pub use gtk::prelude::ToValue;
    pub use gtk::subclass::prelude::*;
    pub use gtk::{cairo, gdk, gdk_pixbuf, gio, glib};
    pub use std::borrow::Cow;
    pub use std::cell::RefCell;
    pub use std::cell::{Cell, Ref, RefMut};
    pub use std::rc::Rc;

    pub use once_cell::sync::{Lazy, OnceCell};
}
