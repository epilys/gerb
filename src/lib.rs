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

#[cfg(feature = "python")]
pub mod api;
pub mod app;
pub mod editor;
#[cfg(feature = "git")]
pub mod git;
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
    pub use app::*;
    pub use app::{types::*, Settings};
    pub use editor::*;
    pub use glyphs::obj::GlyphMetadata;
    pub use glyphs::{Continuity, Glyph, GlyphPointIndex, Guideline};
    pub use gtk::prelude::*;
    pub use gtk::subclass::prelude::ObjectSubclassIsExt;
    pub use indexmap::{IndexMap, IndexSet};
    pub use project::Project;
    pub use ufo;
    pub use utils::colors::*;
    pub use utils::points::*;
    pub use utils::property_window::*;
    pub use utils::range_query::Coordinate;
    pub use utils::shortcuts::{Shortcut, ShortcutAction};
    pub use utils::{ContextExt, ContextRef};
    pub use utils::{Either, FieldRef, Modified, UI_EDITABLE, UI_PATH, UI_READABLE};
    pub use views::{canvas, Canvas, Collection, Overlay, Transformation, UnitPoint, ViewPoint};
    pub use window::Workspace;

    pub use glib::prelude::*;
    pub use glib::subclass::Signal;
    pub use glib::{
        ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecString, Value,
    };
    pub use gtk::prelude::ToValue;
    pub use gtk::subclass::prelude::*;
    pub use gtk::{cairo, gdk, gdk_pixbuf, gio, glib};
    pub use std::borrow::Cow;
    pub use std::cell::RefCell;
    pub use std::cell::{Cell, Ref, RefMut};
    pub use std::collections::HashSet;
    pub use std::path::{Path, PathBuf};
    pub use std::rc::Rc;
    pub use std::str::FromStr;

    pub use once_cell::sync::{Lazy, OnceCell};
    pub use uuid::Uuid;

    mod macros {
        #[macro_export]
        macro_rules! def_param {
            (str $name:expr) => {
                glib::ParamSpecString::new(
                    $name,
                    $name,
                    $name,
                    Some(""),
                    ParamFlags::READWRITE | UI_EDITABLE,
                )
            };
            (i64 $name:expr, $default:expr) => {
                glib::ParamSpecInt64::new(
                    $name,
                    $name,
                    $name,
                    0,
                    std::i64::MAX,
                    $default,
                    ParamFlags::READWRITE | UI_EDITABLE,
                )
            };
            (i64 $name:expr) => {
                $crate::def_param!(i64 $name, 1)
            };
            (u64 $name:expr, $default:expr) => {
                glib::ParamSpecUInt64::new(
                    $name,
                    $name,
                    $name,
                    0,
                    std::u64::MAX,
                    $default,
                    ParamFlags::READWRITE | UI_EDITABLE,
                )
            };
            (u64 $name:expr) => {
                $crate::def_param!(u64 $name, 1)
            };
            (f64 $name:expr, $min:expr, $default:expr) => {
                glib::ParamSpecDouble::new(
                    $name,
                    $name,
                    $name,
                    $min,
                    std::f64::MAX,
                    $default,
                    ParamFlags::READWRITE | UI_EDITABLE,
                )
            };
        }

        #[macro_export]
        macro_rules! inherit_property {
            ($t:ty, $($prop:ident),*) => {
                $(
                pub const $prop: &str = <$t>::$prop;
                )*
            }
        }
    }
    pub use macros::*;
}
