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

use crate::prelude::*;

#[derive(Debug, Default)]
pub struct GlyphMetadataInner {
    pub modified: Cell<bool>,
    pub mark_color: Cell<Color>,
    pub relative_path: RefCell<PathBuf>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphMetadataInner {
    const NAME: &'static str = "Glyph";
    type Type = GlyphMetadata;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for GlyphMetadataInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.mark_color.set(Color::TRANSPARENT);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        GlyphMetadata::MODIFIED,
                        GlyphMetadata::MODIFIED,
                        GlyphMetadata::MODIFIED,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoxed::new(
                        GlyphMetadata::MARK_COLOR,
                        GlyphMetadata::MARK_COLOR,
                        GlyphMetadata::MARK_COLOR,
                        Color::static_type(),
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecString::new(
                        GlyphMetadata::RELATIVE_PATH,
                        GlyphMetadata::RELATIVE_PATH,
                        "Filesystem path.",
                        None,
                        glib::ParamFlags::READWRITE | UI_READABLE | UI_PATH,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            GlyphMetadata::MARK_COLOR => self.mark_color.get().to_value(),
            GlyphMetadata::MODIFIED => self.modified.get().to_value(),
            GlyphMetadata::RELATIVE_PATH => {
                self.relative_path.borrow().display().to_string().to_value()
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            GlyphMetadata::MARK_COLOR => {
                self.mark_color.set(value.get().unwrap());
            }
            GlyphMetadata::MODIFIED => {
                self.modified.set(value.get().unwrap());
            }
            GlyphMetadata::RELATIVE_PATH => {
                if let Ok(Some(relative_path)) = value.get::<Option<String>>() {
                    *self.relative_path.borrow_mut() = relative_path.into();
                } else {
                    *self.relative_path.borrow_mut() = PathBuf::new();
                }
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct GlyphMetadata(ObjectSubclass<GlyphMetadataInner>);
}

impl std::ops::Deref for GlyphMetadata {
    type Target = GlyphMetadataInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl GlyphMetadata {
    pub const MODIFIED: &str = "modified";
    pub const MARK_COLOR: &str = "mark-color";
    pub const RELATIVE_PATH: &str = "relative-path";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }
}

impl Default for GlyphMetadata {
    fn default() -> GlyphMetadata {
        Self::new()
    }
}
