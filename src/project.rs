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

use std::path::{Path, PathBuf};

use crate::glyphs::{Glyph, Guideline};
use crate::prelude::*;

// [ref:FIXME]: how do we detect if a Project is no longer modified when a user undos the modifications?
//
// An idea is to keep a counter of single modifications, and decrease it when the user performs an
// undo action.

#[derive(Debug)]
pub struct ProjectInner {
    name: RefCell<String>,
    modified: Cell<bool>,
    pub last_saved: RefCell<Option<u64>>,
    pub path: RefCell<PathBuf>,
    pub family_name: RefCell<String>,
    pub style_name: RefCell<String>,
    style_map_family_name: RefCell<String>,
    style_map_style_name: RefCell<String>,
    version_major: Cell<i64>,
    version_minor: Cell<u64>,
    year: Cell<u64>,
    /// Copyright statement.
    copyright: RefCell<String>,
    /// Trademark statement.
    trademark: RefCell<String>,
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
    pub fontinfo: RefCell<ufo::objects::FontInfo>,
    pub metainfo: RefCell<ufo::MetaInfo>,
    pub layercontents: RefCell<ufo::LayerContents>,
    pub default_layer: ufo::objects::Layer,
    pub background_layer: RefCell<Option<ufo::objects::Layer>>,
    // [ref:FIXME]: check for reserved names when adding to all_layers
    pub all_layers: RefCell<Vec<ufo::objects::Layer>>,
    #[cfg(feature = "git")]
    pub repository: RefCell<Result<Option<git::Repository>, Box<dyn std::error::Error>>>,
}

impl Default for ProjectInner {
    fn default() -> Self {
        Self {
            name: RefCell::new("New project".to_string()),
            modified: Cell::new(false),
            last_saved: RefCell::new(None),
            path: RefCell::new(std::env::current_dir().unwrap_or_default()),
            family_name: RefCell::new("New project".to_string()),
            style_name: RefCell::new("New project".to_string()),
            style_map_family_name: RefCell::new(String::new()),
            style_map_style_name: RefCell::new(String::new()),
            year: Cell::new(1970),
            version_major: Cell::new(ufo::constants::VERSION_MAJOR),
            version_minor: Cell::new(ufo::constants::VERSION_MINOR),
            copyright: RefCell::new(String::new()),
            trademark: RefCell::new(String::new()),
            units_per_em: Cell::new(ufo::constants::UNITS_PER_EM),
            descender: Cell::new(ufo::constants::DESCENDER),
            x_height: Cell::new(ufo::constants::X_HEIGHT),
            cap_height: Cell::new(ufo::constants::CAP_HEIGHT),
            ascender: Cell::new(ufo::constants::ASCENDER),
            italic_angle: Cell::new(ufo::constants::ITALIC_ANGLE),
            note: RefCell::new(String::new()),
            guidelines: RefCell::new(vec![]),
            metric_guidelines: RefCell::new(vec![]),
            fontinfo: RefCell::new(ufo::objects::FontInfo::new()),
            metainfo: RefCell::new(ufo::MetaInfo::default()),
            layercontents: RefCell::new(ufo::LayerContents::default()),
            default_layer: ufo::objects::Layer::new(),
            background_layer: RefCell::new(None),
            all_layers: RefCell::new(vec![]),
            #[cfg(feature = "git")]
            repository: RefCell::new(Ok(None)),
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
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> = once_cell::sync::Lazy::new(
            || {
                vec![
                    ParamSpecString::new(
                        Project::NAME,
                        Project::NAME,
                        Project::NAME,
                        Some("New project"),
                        glib::ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        Project::FILENAME_STEM,
                        Project::FILENAME_STEM,
                        Project::FILENAME_STEM,
                        None,
                        glib::ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        Project::MODIFIED,
                        Project::MODIFIED,
                        Project::MODIFIED,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    def_param!(str Project::FAMILY_NAME),
                    def_param!(str Project::STYLE_NAME),
                    def_param!(str Project::STYLE_MAP_FAMILY_NAME),
                    def_param!(str Project::STYLE_MAP_STYLE_NAME),
                    def_param!(str Project::COPYRIGHT),
                    def_param!(str Project::TRADEMARK),
                    def_param!(str Project::NOTE),
                    def_param!(u64 Project::YEAR),
                    def_param!(i64 Project::VERSION_MAJOR, ufo::constants::VERSION_MAJOR),
                    def_param!(u64 Project::VERSION_MINOR, ufo::constants::VERSION_MINOR),
                    def_param!(f64 Project::UNITS_PER_EM, 1.0, ufo::constants::UNITS_PER_EM),
                    def_param!(f64 Project::X_HEIGHT, 1.0, ufo::constants::X_HEIGHT),
                    def_param!(f64 Project::ASCENDER, std::f64::MIN, ufo::constants::ASCENDER),
                    def_param!(f64 Project::DESCENDER, std::f64::MIN, ufo::constants::DESCENDER),
                    def_param!(f64 Project::CAP_HEIGHT, std::f64::MIN, ufo::constants::CAP_HEIGHT),
                    def_param!(f64 Project::ITALIC_ANGLE, std::f64::MIN, ufo::constants::ITALIC_ANGLE),
                ]
            },
        );
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Project::NAME => self.name.borrow().to_value(),
            Project::FILENAME_STEM => {
                let family_name = self.family_name.borrow();
                let style_name = self.style_name.borrow();
                match (family_name.len(), style_name.len()) {
                    (0, 0) => None::<String>.to_value(),
                    (0, 1..) => Some(style_name.to_string()).to_value(),
                    (_, 0) => Some(family_name.to_string()).to_value(),
                    _ => Some(format!("{family_name}-{style_name}")).to_value(),
                }
            }
            Project::FAMILY_NAME => self.family_name.borrow().to_value(),
            Project::STYLE_NAME => self.style_name.borrow().to_value(),
            Project::STYLE_MAP_FAMILY_NAME => self.style_map_family_name.borrow().to_value(),
            Project::STYLE_MAP_STYLE_NAME => self.style_map_style_name.borrow().to_value(),
            Project::COPYRIGHT => self.copyright.borrow().to_value(),
            Project::TRADEMARK => self.trademark.borrow().to_value(),
            Project::NOTE => self.note.borrow().to_value(),
            Project::YEAR => self.year.get().to_value(),
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

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
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
            Project::STYLE_MAP_FAMILY_NAME => {
                *self.style_map_family_name.borrow_mut() = value.get().unwrap();
            }
            Project::STYLE_MAP_STYLE_NAME => {
                *self.style_map_style_name.borrow_mut() = value.get().unwrap();
            }
            Project::COPYRIGHT => {
                *self.copyright.borrow_mut() = value.get().unwrap();
            }
            Project::TRADEMARK => {
                *self.trademark.borrow_mut() = value.get().unwrap();
            }
            Project::NOTE => {
                *self.note.borrow_mut() = value.get().unwrap();
            }
            Project::YEAR => {
                self.year.set(value.get().unwrap());
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
    pub const MODIFIED: &str = "modified";
    pub const NAME: &str = "name";
    pub const FILENAME_STEM: &str = "filename-stem";
    inherit_property!(
        ufo::objects::FontInfo,
        ASCENDER,
        CAP_HEIGHT,
        DESCENDER,
        ITALIC_ANGLE,
        YEAR,
        COPYRIGHT,
        TRADEMARK,
        FAMILY_NAME,
        STYLE_MAP_FAMILY_NAME,
        STYLE_MAP_STYLE_NAME,
        STYLE_NAME,
        UNITS_PER_EM,
        VERSION_MAJOR,
        VERSION_MINOR,
        X_HEIGHT,
        NOTE
    );

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        Self::from_path_inner(path)
    }

    fn from_path_inner(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path: PathBuf = std::fs::canonicalize(Path::new(path))?;
        if !path.exists() {
            return Err(format!("Directory <i>{}</i> does not exist.", path.display()).into());
        }
        if !path.is_dir() {
            return Err(format!("Path {} is not a directory.", path.display()).into());
        }
        path.push("fontinfo.plist");
        let ret: Self = Self::new();

        let fontinfo = ufo::objects::FontInfo::from_path(path.clone())
            .map_err(|err| format!("couldn't read fontinfo.plist {}: {}", path.display(), err))?;
        path.pop();
        path.push("metainfo.plist");
        let metainfo_exists = path.exists();
        let metainfo = ufo::MetaInfo::from_path(&path)
            .map_err(|err| format!("couldn't read metainfo.plist {}: {}", path.display(), err))?;
        if !metainfo_exists {
            metainfo.save(&path)?;
        }

        path.pop();
        path.push("layercontents.plist");
        let layercontents = ufo::LayerContents::from_path(&path, ret.default_layer.clone(), false)
            .map_err(|err| {
                format!(
                    "couldn't read layercontents.plist {}: {}",
                    path.display(),
                    err
                )
            })?;
        if let Some(background_layer) = layercontents.objects.get("public.background") {
            *ret.background_layer.borrow_mut() = Some(background_layer.clone());
        }
        let all_layers: Vec<ufo::objects::Layer> =
            layercontents.objects.values().cloned().collect();
        for obj in all_layers.iter() {
            ret.link(obj);
        }
        *ret.all_layers.borrow_mut() = all_layers;
        *ret.layercontents.borrow_mut() = layercontents;
        path.pop();
        let name = fontinfo.family_name.borrow().clone();
        if !name.is_empty() {
            ret.set_property(Self::NAME, name);
        } else if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            ret.set_property(
                Self::NAME,
                name.strip_suffix(".ufo").unwrap_or_else(|| name.as_ref()),
            );
        }
        ret.set_property(Self::MODIFIED, false);
        *ret.last_saved.borrow_mut() = None;

        #[cfg(feature = "git")]
        {
            if let Ok(path) = std::fs::canonicalize(&path) {
                *ret.repository.borrow_mut() = git::Repository::new(&path);
            } else if !path.is_relative() {
                *ret.repository.borrow_mut() = git::Repository::new(&path);
            };
            //dbg!(&ret.repository);
        }
        std::env::set_current_dir(&path).unwrap();
        *ret.path.borrow_mut() = path;
        *ret.guidelines.borrow_mut() = fontinfo
            .source
            .borrow()
            .guidelines
            .clone()
            .into_iter()
            .map(Guideline::from)
            .map(|g| {
                ret.link(&g);
                g
            })
            .collect::<Vec<Guideline>>();
        for property in [
            Self::FAMILY_NAME,
            Self::STYLE_NAME,
            Self::STYLE_MAP_FAMILY_NAME,
            Self::STYLE_MAP_STYLE_NAME,
            Self::YEAR,
            Self::COPYRIGHT,
            Self::TRADEMARK,
            Self::UNITS_PER_EM, // [tag:project_bind_metrics]
            Self::DESCENDER,
            Self::X_HEIGHT,
            Self::CAP_HEIGHT,
            Self::ASCENDER,
            Self::ITALIC_ANGLE,
            Self::NOTE,
            Self::VERSION_MAJOR,
            Self::VERSION_MINOR,
        ] {
            fontinfo
                .bind_property(property, &ret, property)
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();
        }
        ret.link(&fontinfo);
        *ret.fontinfo.borrow_mut() = fontinfo;
        *ret.metainfo.borrow_mut() = metainfo;
        {
            let mut metric_guidelines = ret.metric_guidelines.borrow_mut();
            for (name, field) in [
                (Self::X_HEIGHT, ret.x_height.get()),
                (Self::ASCENDER, ret.ascender.get()),
                (Self::DESCENDER, ret.descender.get()),
                (Self::CAP_HEIGHT, ret.cap_height.get()),
            ] {
                let g = Guideline::builder()
                    .name(Some(name.to_string()))
                    .identifier(Some(name.to_string()))
                    .y(Some(field))
                    .color(Some(Color::from_hex("#bbbaae"))) // [ref:hardcoded_color_value]
                    .build();
                ret.link(&g);
                ret.bind_property(name, &g, Guideline::Y)
                    .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                    .build();
                metric_guidelines.push(g);
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

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let fontinfo = self.fontinfo.borrow();
        {
            let mut f_guidelines = fontinfo.guidelines.borrow_mut();
            for (i, g) in self
                .guidelines
                .borrow()
                .iter()
                .enumerate()
                .filter(|(_, obj)| obj.modified())
            {
                g.set_property(Guideline::MODIFIED, false);
                if i >= f_guidelines.len() {
                    f_guidelines.push(g.into());
                    debug_assert_eq!(f_guidelines.len(), i + 1);
                } else {
                    f_guidelines[i] = g.into();
                }
            }
        }
        fontinfo.save()?;
        for obj in self.all_layers.borrow().iter().filter(|obj| obj.modified()) {
            obj.save(&mut self.layercontents.borrow_mut())?;
        }
        /* Metric guidelines have their value properties bound with FontInfo via Project
         * [ref:project_bind_metrics] */
        for g in self
            .metric_guidelines
            .borrow()
            .iter()
            .filter(|obj| obj.modified())
        {
            g.set_property(Guideline::MODIFIED, false);
        }
        self.set_property(Self::MODIFIED, false);
        Ok(())
    }

    pub fn create(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path: PathBuf = std::fs::canonicalize(Path::new(path))
            .map_err(|err| format!("Path looks invalid: {err}"))?;
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|err| format!("Could not create project: {err}"))?;
        }
        path.push("fontinfo.plist");
        let fontinfo_plist = ufo::FontInfo::default();
        fontinfo_plist
            .save(&path)
            .map_err(|err| format!("Could not create fontinfo.plist: {err}"))?;
        path.pop();
        path.push("layercontents.plist");
        let layercontents_plist = ufo::LayerContents::default();
        layercontents_plist
            .save(&path)
            .map_err(|err| format!("Could not create layercontents.plist: {err}"))?;
        path.pop();
        path.push("metainfo.plist");
        let metainfo = ufo::MetaInfo::default();
        metainfo
            .save(&path)
            .map_err(|err| format!("Could not create metainfo.plist: {err}"))?;
        path.pop();

        path.push("glyphs");
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|err| format!("Could not create glyphs/ folder: {err}"))?;
        }
        path.push("contents.plist");
        ufo::Contents::default()
            .save(Some(&path), true)
            .map_err(|err| format!("Could not create glyphs/contents.plist: {err}"))?;
        path.pop();
        path.pop();
        Self::from_path(path)
    }

    pub fn new_glyph(
        &self,
        name: String,
        glyph: Rc<RefCell<Glyph>>,
        layer: Option<&ufo::objects::Layer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let layer = layer.unwrap_or(&self.default_layer);
        layer.new_glyph(name, glyph)
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl_modified!(Project);

impl_property_window!(Project);
