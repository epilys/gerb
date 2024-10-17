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
    pub guidelines: RefCell<Vec<Guideline>>,
    pub metric_guidelines: RefCell<Vec<Guideline>>,
    pub fontinfo: RefCell<FontInfo>,
    pub metainfo: RefCell<MetaInfo>,
    pub layercontents: RefCell<LayerContents>,
    pub default_layer: ufo::objects::Layer,
    pub background_layer: RefCell<Option<ufo::objects::Layer>>,
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
            guidelines: RefCell::new(vec![]),
            metric_guidelines: RefCell::new(vec![]),
            fontinfo: RefCell::new(FontInfo::new()),
            metainfo: RefCell::new(MetaInfo::default()),
            layercontents: RefCell::new(LayerContents::default()),
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
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
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
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Project::NAME => self.name.borrow().to_value(),
            Project::MODIFIED => self.modified.get().to_value(),
            Project::FILENAME_STEM => {
                let fontinfo = self.fontinfo.borrow();
                let family_name = fontinfo.family_name.borrow();
                let style_name = fontinfo.style_name.borrow();
                match (family_name.len(), style_name.len()) {
                    (0, 0) => None::<String>.to_value(),
                    (0, 1..) => Some(style_name.to_string()).to_value(),
                    (_, 0) => Some(family_name.to_string()).to_value(),
                    _ => Some(format!("{family_name}-{style_name}")).to_value(),
                }
            }
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
            Project::MODIFIED => {
                self.modified.set(value.get().unwrap());
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
    pub const MODIFIED: &'static str = "modified";
    pub const NAME: &'static str = "name";
    pub const FILENAME_STEM: &'static str = "filename-stem";

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

        let fontinfo = FontInfo::from_path(path.clone()).map_err(|err| {
            format!(
                "couldn't read fontinfo.plist {}:\n\n{}",
                path.display(),
                err
            )
        })?;
        path.pop();
        path.push("metainfo.plist");
        let metainfo = ufo::MetaInfo::from_path(&path).map_err(|err| {
            format!(
                "couldn't read metainfo.plist {}:\n\n{}",
                path.display(),
                err
            )
        })?;

        path.pop();
        path.push("layercontents.plist");
        let layercontents = ufo::LayerContents::from_path(&path, ret.default_layer.clone(), false)
            .map_err(|err| format!("couldn't read layercontents.plist:\n\n{}", err))?;
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
        ret.link(&fontinfo);
        {
            let mut metric_guidelines = ret.metric_guidelines.borrow_mut();
            for (name, field) in [
                (FontInfo::X_HEIGHT, fontinfo.x_height.get()),
                (FontInfo::ASCENDER, fontinfo.ascender.get()),
                (FontInfo::DESCENDER, fontinfo.descender.get()),
                (FontInfo::CAP_HEIGHT, fontinfo.cap_height.get()),
            ] {
                let g = Guideline::builder()
                    .name(Some(name.to_string()))
                    .identifier(Some(name.to_string()))
                    .y(Some(field))
                    .color(Some(Color::from_hex("#bbbaae"))) // [ref:hardcoded_color_value]
                    .build();
                fontinfo.link(&g);
                fontinfo
                    .bind_property(name, &g, Guideline::Y)
                    .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                    .build();
                metric_guidelines.push(g);
            }
        }
        *ret.fontinfo.borrow_mut() = fontinfo;
        *ret.metainfo.borrow_mut() = metainfo;
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
            .map_err(|err| format!("Path looks invalid:\n\n{err}"))?;
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|err| format!("Could not create project:\n\n{err}"))?;
        }
        path.push("fontinfo.plist");
        let fontinfo_plist = ufo::FontInfo::default();
        fontinfo_plist
            .save(&path)
            .map_err(|err| format!("Could not create fontinfo.plist:\n\n{err}"))?;
        path.pop();
        path.push("layercontents.plist");
        let layercontents_plist = ufo::LayerContents::default();
        layercontents_plist
            .save(&path)
            .map_err(|err| format!("Could not create layercontents.plist:\n\n{err}"))?;
        path.pop();
        path.push("metainfo.plist");
        let metainfo = ufo::MetaInfo::default();
        metainfo
            .save(&path)
            .map_err(|err| format!("Could not create metainfo.plist:\n\n{err}"))?;
        path.pop();

        path.push("glyphs");
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|err| format!("Could not create glyphs/ folder:\n\n{err}"))?;
        }
        path.push("contents.plist");
        ufo::Contents::default()
            .save(Some(&path), true)
            .map_err(|err| format!("Could not create glyphs/contents.plist:\n\n{err}"))?;
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

    pub fn fontinfo(&self) -> FieldRef<'_, FontInfo> {
        self.fontinfo.borrow().into()
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl_modified!(Project);

impl_property_window!(delegate Project => { borrow() }, fontinfo);
