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

#![deny(
    /* groups */
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::cargo,
    clippy::nursery,
    /* restriction */
    clippy::dbg_macro,
    clippy::rc_buffer,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    /* pedantic */
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::ptr_as_ptr,
    clippy::bool_to_int_with_if,
    clippy::borrow_as_ptr,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_ptr_alignment,
    clippy::naive_bytecount,
)]
// known problems/false negative
#![allow(
    clippy::use_self,
    clippy::multiple_crate_versions, // [ref:TODO] bundle generational-arena? it seems unmaintained.
    clippy::missing_const_for_fn,
    clippy::fallible_impl_from, // [ref:TODO]
    clippy::option_if_let_else, // [ref:TODO]
    clippy::cognitive_complexity, // [ref:TODO]
)]

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
pub const ISSUE_TRACKER: &str = "https://github.com/epilys/gerb/issues";
pub const VERSION_INFO: &str = build_info::format!("{} v{} commit {}", $.crate_info.name, $.crate_info.version, $.version_control?.git()?.commit_short_id);
pub const INFO: &str = build_info::format!("\n                 ,adPPYb,d8  \n                a8\"    `Y88  \n                8b       88  \n                \"8a,   ,d88  \n                 `\"YbbdP\"Y8  \n                 aa,    ,88  \n                  \"Y8bbdP\"   \n\n{} Copyright (C) 2022 Emmanouil Pitsidianakis\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to\nredistribute it under certain conditions; See\nLICENSE.md for more details.\n\nVersion: {}\nCommit SHA: {}\nAuthors: {}\nLicense: GPL version 3 or later\nCompiler: {}\nBuild-Date: {}\nEnabled-features: {}", $.crate_info.name, $.crate_info.version, $.version_control?.git()?.commit_short_id, $.crate_info.authors, $.compiler, $.timestamp, $.crate_info.enabled_features);

/* Annotations:
 *
 * Global tags (in tagref format <https://github.com/stepchowfun/tagref>) for source code
 * annotation:
 *
 * - [tag:hardcoded_color_value] Replace hardcoded color values with user configurable ones.
 * - [tag:needs_unit_test]
 * - [tag:needs_user_doc]
 * - [tag:needs_dev_doc]
 * - [tag:FIXME]
 * - [tag:TODO]
 */

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
    // cairo context API wrapper:
    pub use utils::{ContextExt, ContextRef};
    // utility types:
    pub use utils::{Either, FieldRef, UI_EDITABLE, UI_PATH, UI_READABLE};
    // utility traits:
    pub use utils::{Modified, StyleReadOnly};
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

        #[macro_export]
        macro_rules! return_if_not_ok_or_accept {
            ($response:expr) => {{
                let response = $response; // evaluate
                if matches!(
                    response,
                    gtk::ResponseType::Cancel
                        | gtk::ResponseType::DeleteEvent
                        | gtk::ResponseType::Close
                ) {
                    return;
                }
                response
            }};
        }
    }
    pub use macros::*;
}
