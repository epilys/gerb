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
    // clippy::cast_possible_wrap, [ref:TODO] re-enable
    clippy::ptr_as_ptr,
    clippy::bool_to_int_with_if,
    clippy::borrow_as_ptr,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_ptr_alignment,
    clippy::naive_bytecount,
    rustdoc::broken_intra_doc_links,

)]
// known problems/false negative
#![allow(
    clippy::use_self,
    clippy::multiple_crate_versions, // [ref:TODO] bundle generational-arena? it seems unmaintained.
    clippy::missing_const_for_fn,
    clippy::fallible_impl_from, // [ref:TODO]
    clippy::option_if_let_else, // [ref:TODO]
    clippy::cognitive_complexity, // [ref:TODO]
    clippy::while_float, // [ref:TODO]
    clippy::redundant_guards, // [ref:TODO]
    clippy::bad_bit_mask, // [ref:TODO]
)]

#[macro_use]
extern crate glib;

pub use serde_json;

#[cfg(feature = "python")]
pub mod api;
pub mod app;
pub mod editor;
pub mod error;
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

const fn _get_package_git_sha() -> Option<&'static str> {
    option_env!("PACKAGE_GIT_SHA")
}

const _PACKAGE_COMMIT_SHA: Option<&str> = _get_package_git_sha();

pub fn get_git_sha() -> std::borrow::Cow<'static, str> {
    if let Some(r) = _PACKAGE_COMMIT_SHA {
        return r.into();
    }
    /*
    build_info::build_info!(fn build_info);
    let info = build_info();
    if let Some(v) = info
        .version_control
        .as_ref()
        .and_then(|v| v.git())
        .map(|g| g.commit_short_id.clone())
    {
        v.into()
    } else {
        "<unknown>".into()
    }
    */
    "<unknown>".into()
}

pub const APPLICATION_NAME: &str = "gerb";
pub const APPLICATION_ID: &str = "io.github.epilys.gerb";
pub const ISSUE_TRACKER: &str = "https://github.com/epilys/gerb/issues";
/*
pub const VERSION_INFO: &str = build_info::format!("{}", $.crate_info.version);
pub const BUILD_INFO: &str = build_info::format!("{}\t{}\t{}\t{}", $.crate_info.version, $.compiler, $.timestamp, $.crate_info.enabled_features);
pub const CLI_INFO: &str = build_info::format!("\n                 ,adPPYb,d8  \n                a8\"    `Y88  \n                8b       88  \n                \"8a,   ,d88  \n                 `\"YbbdP\"Y8  \n                 aa,    ,88  \n                  \"Y8bbdP\"   \n\n{} Copyright (C) 2022 Emmanouil Pitsidianakis\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to\nredistribute it under certain conditions; See\nLICENSE.md for more details.\n\nVersion: {}\nAuthors: {}\nLicense: GPL version 3 or later\nCompiler: {}\nBuild-Date: {}\nEnabled-features: {}", $.crate_info.name, $.crate_info.version, $.crate_info.authors, $.compiler, $.timestamp, $.crate_info.enabled_features);
*/

pub const VERSION_INFO: &str = "<unknown>";
pub const BUILD_INFO: &str = "<unknown>";
pub const CLI_INFO: &str = "<unknown>";
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
 * - [tag:VERIFY] Verify whether this is the correct way to do something
 */

pub mod prelude {
    pub use super::*;
    pub use app::Application;
    pub use app::*;
    pub use app::{types::*, Settings};
    pub use editor::*;
    pub use error::Error;
    pub use glyphs::metadata::GlyphMetadata;
    pub use glyphs::{Continuity, Glyph, GlyphPointIndex, Guideline};
    pub use gtk::prelude::*;
    pub use gtk::subclass::prelude::ObjectSubclassIsExt;
    pub use indexmap::{IndexMap, IndexSet};
    pub use project::Project;
    pub use ufo;
    pub use ufo::objects::FontInfo;
    pub use ufo::{LayerContents, MetaInfo};
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
    pub use views::{
        canvas, canvas::CanvasSettings, Canvas, Collection, Overlay, Transformation, UnitPoint,
        ViewPoint,
    };
    pub use window::Workspace;

    pub use glib::subclass::Signal;
    pub use glib::{
        ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecBoxed, ParamSpecDouble, ParamSpecObject,
        ParamSpecString, Value,
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
        macro_rules! impl_modified {
            ($ty:ty) => {
                $crate::impl_modified!($ty, MODIFIED);
            };
            ($ty:ty, $property_name:ident) => {
                impl $crate::utils::Modified for $ty {
                    const PROPERTY_NAME: &'static str = Self::$property_name;
                }
            };
        }

        #[macro_export]
        macro_rules! impl_deref {
            ($ty:ty, $inner:ty) => {
                impl std::ops::Deref for $ty {
                    type Target = $inner;

                    fn deref(&self) -> &Self::Target {
                        self.imp()
                    }
                }
            };
        }

        #[macro_export]
        macro_rules! impl_friendly_name {
            ($ty:ty) => {
                impl $crate::utils::property_window::FriendlyNameInSettings for $ty {}
            };
            ($ty:ty, $friendly_name:expr) => {
                impl $crate::utils::property_window::FriendlyNameInSettings for $ty {
                    fn friendly_name(&self) -> Cow<'static, str> {
                        $friendly_name.into()
                    }

                    fn static_friendly_name() -> Cow<'static, str> {
                        $friendly_name.into()
                    }
                }
            };
        }

        #[macro_export]
        macro_rules! impl_property_window {
            ($ty:ty) => {
                impl $crate::utils::property_window::FriendlyNameInSettings for $ty {}
                impl $crate::utils::property_window::CreatePropertyWindow for $ty {}
            };
            ($ty:ty$(,)? $({ $friendly_name:expr })?) => {
                $(
                    $crate::impl_friendly_name!($ty, $friendly_name);
                )*
                impl $crate::utils::property_window::CreatePropertyWindow for $ty {}
            };
            (delegate $ty:ty => { $($access:tt)+ }, $($field:tt)+) => {
                impl $crate::utils::property_window::FriendlyNameInSettings for $ty {}
                impl $crate::utils::property_window::CreatePropertyWindow for $ty {
                    fn new_property_window(
                        &self,
                        app: &$crate::prelude::Application,
                        create: bool,
                    ) -> $crate::prelude::PropertyWindow
                    where
                        Self: glib::IsA<glib::Object>,
                    {
                        let inner=self.$($field)*.$($access)*;
                        inner.new_property_window(app, create)
                    }
                }
            };
        }

        /// Helper macro to define user editable GObject properties.
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
                    i64::MAX,
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
                    u64::MAX,
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
                    f64::MAX,
                    $default,
                    ParamFlags::READWRITE | UI_EDITABLE,
                )
            };
        }

        /// Helper macro to easily "inherit" property names from another type.
        /// The convention is that property names are stored as type constant string slices.
        #[macro_export]
        macro_rules! inherit_property {
            ($t:ty, $($prop:ident),+$(,)?) => {
                $(
                pub const $prop: &'static str = <$t>::$prop;
                )*
            }
        }

        /// Helper macro to call return; if a gtk dialog response calls for it
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
}
