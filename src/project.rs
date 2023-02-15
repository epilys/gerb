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

#[cfg(feature = "git")]
use crate::git;

use crate::ufo;
use glib::{
    ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecInt64, ParamSpecString,
    ParamSpecUInt64, Value,
};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::glyphs::{Glyph, Guideline};
use crate::prelude::*;

#[derive(Debug)]
pub struct ProjectInner {
    name: RefCell<String>,
    modified: Cell<bool>,
    pub last_saved: RefCell<Option<u64>>,
    pub glyphs: RefCell<HashMap<String, Rc<RefCell<Glyph>>>>,
    pub path: RefCell<PathBuf>,
    pub family_name: RefCell<String>,
    pub style_name: RefCell<String>,
    version_major: Cell<i64>,
    version_minor: Cell<u64>,
    /// Copyright statement.
    pub copyright: RefCell<String>,
    /// Trademark statement.
    pub trademark: RefCell<String>,
    /// Units per em.
    units_per_em: Cell<f64>,
    /// Descender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
    descender: Cell<f64>,
    /// x-height value.
    x_height: Cell<f64>,
    /// Cap height value.
    cap_height: Cell<f64>,
    /// Ascender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
    ascender: Cell<f64>,
    /// Italic angle. This must be an angle in counter-clockwise degrees from the vertical.
    italic_angle: Cell<f64>,
    /// Arbitrary note about the font.
    pub note: RefCell<String>,
    /// A list of guideline definitions that apply to all glyphs in all layers in the font. This attribute is optional.
    pub guidelines: RefCell<Vec<Guideline>>,
    pub metric_guidelines: RefCell<Vec<Guideline>>,
    pub fontinfo: RefCell<ufo::FontInfo>,
    pub contents: RefCell<ufo::Contents>,
    pub metainfo: RefCell<ufo::MetaInfo>,
    pub layercontents: RefCell<ufo::LayerContents>,
    pub repository: git::Repository,
}

impl Default for ProjectInner {
    fn default() -> Self {
        ProjectInner {
            name: RefCell::new("New project".to_string()),
            modified: Cell::new(false),
            last_saved: RefCell::new(None),
            glyphs: RefCell::new(HashMap::default()),
            path: RefCell::new(std::env::current_dir().unwrap_or_default()),
            family_name: RefCell::new("New project".to_string()),
            style_name: RefCell::new(String::new()),
            version_major: Cell::new(0),
            version_minor: Cell::new(0),
            copyright: RefCell::new(String::new()),
            trademark: RefCell::new(String::new()),
            units_per_em: Cell::new(1000.0),
            descender: Cell::new(-200.),
            x_height: Cell::new(450.),
            cap_height: Cell::new(650.),
            ascender: Cell::new(700.),
            italic_angle: Cell::new(0.),
            note: RefCell::new(String::new()),
            guidelines: RefCell::new(vec![]),
            metric_guidelines: RefCell::new(vec![]),
            fontinfo: RefCell::new(ufo::FontInfo::default()),
            contents: RefCell::new(ufo::Contents::default()),
            metainfo: RefCell::new(ufo::MetaInfo::default()),
            layercontents: RefCell::new(ufo::LayerContents::default()),
            repository: git::Repository::default(),
        }
    }
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ProjectInner {
    const NAME: &'static str = "Project";
    type Type = Project;
    type ParentType = glib::Object;
    type Interfaces = ();
}

// Trait shared by all GObjects
impl ObjectImpl for ProjectInner {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        Project::NAME,
                        Project::NAME,
                        Project::NAME,
                        Some("New project"),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecString::new(
                        Project::FAMILY_NAME,
                        Project::FAMILY_NAME,
                        Project::FAMILY_NAME,
                        Some(""),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecString::new(
                        Project::STYLE_NAME,
                        Project::STYLE_NAME,
                        Project::STYLE_NAME,
                        Some(""),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Project::MODIFIED,
                        Project::MODIFIED,
                        Project::MODIFIED,
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecInt64::new(
                        Project::VERSION_MAJOR,
                        Project::VERSION_MAJOR,
                        Project::VERSION_MAJOR,
                        0,
                        std::i64::MAX,
                        1,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecUInt64::new(
                        Project::VERSION_MINOR,
                        Project::VERSION_MINOR,
                        Project::VERSION_MINOR,
                        0,
                        std::u64::MAX,
                        1,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Project::UNITS_PER_EM,
                        Project::UNITS_PER_EM,
                        Project::UNITS_PER_EM,
                        1.0,
                        std::f64::MAX,
                        1000.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Project::X_HEIGHT,
                        Project::X_HEIGHT,
                        Project::X_HEIGHT,
                        1.0,
                        std::f64::MAX,
                        450.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Project::ASCENDER,
                        Project::ASCENDER,
                        Project::ASCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        700.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Project::DESCENDER,
                        Project::DESCENDER,
                        Project::DESCENDER,
                        std::f64::MIN,
                        std::f64::MAX,
                        -200.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Project::CAP_HEIGHT,
                        Project::CAP_HEIGHT,
                        Project::CAP_HEIGHT,
                        std::f64::MIN,
                        std::f64::MAX,
                        650.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Project::ITALIC_ANGLE,
                        Project::ITALIC_ANGLE,
                        Project::ITALIC_ANGLE,
                        std::f64::MIN,
                        std::f64::MAX,
                        0.0,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Project::NAME => self.name.borrow().to_value(),
            Project::FAMILY_NAME => self.family_name.borrow().to_value(),
            Project::STYLE_NAME => self.style_name.borrow().to_value(),
            Project::MODIFIED => self.modified.get().to_value(),
            Project::VERSION_MAJOR => self.version_major.get().to_value(),
            Project::VERSION_MINOR => self.version_minor.get().to_value(),
            Project::UNITS_PER_EM => self.units_per_em.get().to_value(),
            Project::X_HEIGHT => self.x_height.get().to_value(),
            Project::ASCENDER => self.ascender.get().to_value(),
            Project::DESCENDER => self.descender.get().to_value(),
            Project::CAP_HEIGHT => self.cap_height.get().to_value(),
            Project::ITALIC_ANGLE => self.italic_angle.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Project::NAME => {
                *self.name.borrow_mut() = value.get().unwrap();
            }
            Project::FAMILY_NAME => {
                *self.family_name.borrow_mut() = value.get().unwrap();
            }
            Project::STYLE_NAME => {
                *self.style_name.borrow_mut() = value.get().unwrap();
            }
            Project::MODIFIED => {
                self.modified.set(value.get().unwrap());
            }
            Project::VERSION_MAJOR => {
                self.version_major.set(value.get().unwrap());
            }
            Project::VERSION_MINOR => {
                self.version_minor.set(value.get().unwrap());
            }
            Project::UNITS_PER_EM => {
                self.units_per_em.set(value.get().unwrap());
            }
            Project::X_HEIGHT => {
                self.x_height.set(value.get().unwrap());
            }
            Project::ASCENDER => {
                self.ascender.set(value.get().unwrap());
            }
            Project::DESCENDER => {
                self.descender.set(value.get().unwrap());
            }
            Project::CAP_HEIGHT => {
                self.cap_height.set(value.get().unwrap());
            }
            Project::ITALIC_ANGLE => {
                self.italic_angle.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct Project(ObjectSubclass<ProjectInner>);
}

impl std::ops::Deref for Project {
    type Target = ProjectInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Project {
    pub const ASCENDER: &str = "ascender";
    pub const CAP_HEIGHT: &str = "cap-height";
    pub const DESCENDER: &str = "descender";
    pub const ITALIC_ANGLE: &str = "italic-angle";
    pub const MODIFIED: &str = "modified";
    pub const NAME: &str = "name";
    pub const FAMILY_NAME: &str = "family-name";
    pub const STYLE_NAME: &str = "style-name";
    pub const UNITS_PER_EM: &str = "units-per-em";
    pub const VERSION_MAJOR: &str = "version-major";
    pub const VERSION_MINOR: &str = "version-minor";
    pub const X_HEIGHT: &str = "x-height";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn from_path(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path: PathBuf = Path::new(path).into();
        if !path.exists() {
            return Err(format!("Directory <i>{}</i> does not exist.", path.display()).into());
        }
        if !path.is_dir() {
            return Err(format!("Path {} is not a directory.", path.display()).into());
        }
        path.push("fontinfo.plist");
        let fontinfo @ ufo::FontInfo {
            family_name: _,
            style_name: _,
            version_major,
            version_minor,
            copyright: _,
            trademark: _,
            units_per_em,
            ascender,
            descender,
            x_height,
            cap_height,
            italic_angle,
            guidelines: _,
            style_map_family_name: _,
            style_map_style_name: _,
            year: _,
            note: _,
        } = ufo::FontInfo::from_path(&path)
            .map_err(|err| format!("couldn't read fontinfo.plist {}: {}", path.display(), err))?;
        path.pop();
        path.push("metainfo.plist");
        let metainfo = ufo::MetaInfo::from_path(&path)
            .map_err(|err| format!("couldn't read metainfo.plist {}: {}", path.display(), err))?;
        path.pop();
        path.push("layercontents.plist");
        let layercontents = ufo::LayerContents::from_path(&path).map_err(|err| {
            format!(
                "couldn't read layercontents.plist {}: {}",
                path.display(),
                err
            )
        })?;
        path.pop();
        path.push("glyphs");
        path.push("contents.plist");
        let contents = ufo::Contents::from_path(&path)
            .map_err(|err| format!("couldn't read contents.plist {}: {}", path.display(), err))?;
        path.pop();
        path.pop();
        let glyphs = Glyph::from_ufo(&path, &contents);
        let ret: Self = Self::new();
        ret.set_property(Project::NAME, fontinfo.family_name.clone());
        ret.set_property(Project::MODIFIED, false);
        *ret.last_saved.borrow_mut() = None;
        *ret.glyphs.borrow_mut() = glyphs?;
        std::env::set_current_dir(&path).unwrap();
        *ret.path.borrow_mut() = path;
        *ret.family_name.borrow_mut() = fontinfo.family_name.clone();
        *ret.style_name.borrow_mut() = fontinfo.style_name.clone();
        if let Some(v) = version_major {
            ret.set_property(Project::VERSION_MAJOR, v);
        }
        if let Some(v) = version_minor {
            ret.set_property(Project::VERSION_MINOR, v);
        }
        *ret.copyright.borrow_mut() = fontinfo.copyright.clone();
        *ret.trademark.borrow_mut() = fontinfo.trademark.clone();
        if let Some(v) = units_per_em {
            ret.set_property(Project::UNITS_PER_EM, v);
        }
        if let Some(v) = ascender {
            ret.set_property(Project::ASCENDER, v);
        }
        if let Some(v) = descender {
            ret.set_property(Project::DESCENDER, v);
        }
        if let Some(v) = x_height {
            ret.set_property(Project::X_HEIGHT, v);
        }
        if let Some(v) = cap_height {
            ret.set_property(Project::CAP_HEIGHT, v);
        }
        if let Some(v) = italic_angle {
            ret.set_property(Project::ITALIC_ANGLE, v);
        }
        *ret.note.borrow_mut() = String::new();
        *ret.guidelines.borrow_mut() = fontinfo
            .guidelines
            .clone()
            .into_iter()
            .map(Guideline::try_from)
            .collect::<Result<Vec<Guideline>, String>>()?;
        *ret.fontinfo.borrow_mut() = fontinfo;
        *ret.metainfo.borrow_mut() = metainfo;
        *ret.contents.borrow_mut() = contents;
        *ret.layercontents.borrow_mut() = layercontents;
        {
            let mut metric_guidelines = ret.metric_guidelines.borrow_mut();
            for (name, field) in [
                (Project::X_HEIGHT, ret.x_height.get()),
                (Project::ASCENDER, ret.ascender.get()),
                (Project::DESCENDER, ret.descender.get()),
                (Project::CAP_HEIGHT, ret.cap_height.get()),
            ] {
                metric_guidelines.push(
                    Guideline::builder()
                        .name(Some(name.to_string()))
                        .identifier(Some(name.to_string()))
                        .y(field)
                        .color(Some(Color::from_hex("#bbbaae")))
                        .build(),
                );
            }
        }
        Ok(ret)
    }

    pub fn load_image(
        &self,
        file_name: &str,
    ) -> Result<cairo::ImageSurface, Box<dyn std::error::Error>> {
        let prefix = &self.path.borrow();
        let bytes = gio::File::for_path(prefix.join("images").join(file_name))
            .load_bytes(gio::Cancellable::NONE)?
            .0;
        Ok(cairo::ImageSurface::create_from_png(&mut bytes.as_ref())?)
    }
}

impl Default for Project {
    fn default() -> Self {
        let ret: Self = Self::new();
        *ret.last_saved.borrow_mut() = None;
        *ret.glyphs.borrow_mut() = HashMap::default();
        *ret.family_name.borrow_mut() = "New project".to_string();
        *ret.style_name.borrow_mut() = String::new();
        *ret.copyright.borrow_mut() = String::new();
        *ret.trademark.borrow_mut() = String::new();
        *ret.note.borrow_mut() = String::new();
        *ret.guidelines.borrow_mut() = vec![];
        ret
    }
}
