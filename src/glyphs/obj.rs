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

use super::*;

#[derive(Debug, Default)]
pub struct GlyphMetadataInner {
    pub modified: Cell<bool>,
    pub mark_color: Cell<Color>,
    pub relative_path: RefCell<PathBuf>,
    pub image: RefCell<Option<ImageRef>>,
    pub advance: Cell<Option<Advance>>,
    pub unicode: RefCell<Vec<Unicode>>,
    pub anchors: RefCell<Vec<Anchor>>,
    pub width: Cell<Option<f64>>,
    pub name: RefCell<String>,
    pub kinds: RefCell<(GlyphKind, Vec<GlyphKind>)>,
    pub filename: RefCell<String>,
    pub glif_source: RefCell<String>,
    pub glyph_ref: OnceCell<Rc<RefCell<Glyph>>>,
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
                    glib::ParamSpecString::new(
                        GlyphMetadata::NAME,
                        GlyphMetadata::NAME,
                        "Glyph name.",
                        None,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
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
                    glib::ParamSpecString::new(
                        GlyphMetadata::FILENAME,
                        GlyphMetadata::FILENAME,
                        "Filename.",
                        None,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            GlyphMetadata::NAME => Some(self.name.borrow().to_string()).to_value(),
            GlyphMetadata::MARK_COLOR => self.mark_color.get().to_value(),
            GlyphMetadata::MODIFIED => self.modified.get().to_value(),
            GlyphMetadata::RELATIVE_PATH => {
                self.relative_path.borrow().display().to_string().to_value()
            }
            GlyphMetadata::FILENAME => Some(self.filename.borrow().to_string()).to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            GlyphMetadata::NAME => {
                if let Ok(Some(name)) = value.get::<Option<String>>() {
                    *self.name.borrow_mut() = name;
                } else {
                    *self.name.borrow_mut() = String::new();
                }
            }
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
            GlyphMetadata::FILENAME => {
                if let Ok(Some(filename)) = value.get::<Option<String>>() {
                    *self.filename.borrow_mut() = filename;
                } else {
                    *self.filename.borrow_mut() = String::new();
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
    pub const FILENAME: &str = "filename";
    pub const NAME: &str = "name";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn name(&self) -> FieldRef<'_, String> {
        self.name.borrow().into()
    }

    pub fn filename(&self) -> FieldRef<'_, String> {
        self.filename.borrow().into()
    }

    pub fn kinds(&self) -> FieldRef<'_, (GlyphKind, Vec<GlyphKind>)> {
        self.kinds.borrow().into()
    }

    pub fn width(&self) -> Option<f64> {
        self.width.get()
    }
}

impl Default for GlyphMetadata {
    fn default() -> GlyphMetadata {
        Self::new()
    }
}

impl From<GlyphMetadata> for Glyph {
    fn from(metadata: GlyphMetadata) -> Glyph {
        Glyph {
            metadata,
            ..Glyph::default()
        }
    }
}
