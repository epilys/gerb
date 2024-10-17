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

use super::constants;
use crate::prelude::*;

#[derive(Debug)]
pub struct FontInfoInner {
    modified: Cell<bool>,
    pub last_saved: RefCell<Option<u64>>,
    pub path: OnceCell<PathBuf>,
    pub family_name: RefCell<String>,
    pub style_name: RefCell<String>,
    pub style_map_family_name: RefCell<String>,
    pub style_map_style_name: RefCell<String>,
    pub year: Cell<u64>,
    // Generic Legal Information
    /// Copyright statement.
    pub copyright: RefCell<String>,
    /// Trademark statement.
    pub trademark: RefCell<String>,
    // Generic Dimension Information
    /// Units per em.
    pub units_per_em: Cell<f64>,
    /// Descender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
    pub descender: Cell<f64>,
    /// x-height value.
    pub x_height: Cell<f64>,
    /// Cap height value.
    pub cap_height: Cell<f64>,
    /// Ascender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
    pub ascender: Cell<f64>,
    /// Italic angle. This must be an angle in counter-clockwise degrees from the vertical.
    pub italic_angle: Cell<f64>,
    // Generic Miscellaneous Information
    /// Arbitrary note about the font.
    pub note: RefCell<String>,
    pub version_major: Cell<i64>,
    pub version_minor: Cell<u64>,
    /// A list of guideline definitions that apply to all glyphs in all layers in the font. This attribute is optional.
    pub guidelines: RefCell<Vec<super::GuidelineInfo>>,
    pub source: RefCell<ufo::FontInfo>,
}

impl Default for FontInfoInner {
    fn default() -> Self {
        Self {
            modified: Cell::new(false),
            last_saved: RefCell::new(None),
            path: OnceCell::new(),
            family_name: RefCell::new("New project".to_string()),
            style_name: RefCell::new("New project".to_string()),
            style_map_family_name: RefCell::new(String::new()),
            style_map_style_name: RefCell::new(String::new()),
            year: Cell::new(1970),
            version_major: Cell::new(constants::VERSION_MAJOR),
            version_minor: Cell::new(constants::VERSION_MINOR),
            copyright: RefCell::new(String::new()),
            trademark: RefCell::new(String::new()),
            units_per_em: Cell::new(constants::UNITS_PER_EM),
            descender: Cell::new(constants::DESCENDER),
            x_height: Cell::new(constants::X_HEIGHT),
            cap_height: Cell::new(constants::CAP_HEIGHT),
            ascender: Cell::new(constants::ASCENDER),
            italic_angle: Cell::new(constants::ITALIC_ANGLE),
            note: RefCell::new(String::new()),
            guidelines: RefCell::new(vec![]),
            source: RefCell::new(ufo::FontInfo::default()),
        }
    }
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for FontInfoInner {
    const NAME: &'static str = "FontInfo";
    type Type = FontInfo;
    type ParentType = glib::Object;
    type Interfaces = ();
}

// Trait shared by all GObjects
impl ObjectImpl for FontInfoInner {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        FontInfo::MODIFIED,
                        FontInfo::MODIFIED,
                        FontInfo::MODIFIED,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    def_param!(str FontInfo::FAMILY_NAME),
                    def_param!(str FontInfo::STYLE_NAME),
                    def_param!(str FontInfo::STYLE_MAP_FAMILY_NAME),
                    def_param!(str FontInfo::STYLE_MAP_STYLE_NAME),
                    def_param!(str FontInfo::COPYRIGHT),
                    def_param!(str FontInfo::TRADEMARK),
                    def_param!(str FontInfo::NOTE),
                    def_param!(u64 FontInfo::YEAR),
                    def_param!(i64 FontInfo::VERSION_MAJOR, constants::VERSION_MAJOR),
                    def_param!(u64 FontInfo::VERSION_MINOR, constants::VERSION_MINOR),
                    def_param!(f64 FontInfo::UNITS_PER_EM, 1.0, constants::UNITS_PER_EM),
                    def_param!(f64 FontInfo::X_HEIGHT, 1.0, constants::X_HEIGHT),
                    def_param!(f64 FontInfo::ASCENDER, f64::MIN, constants::ASCENDER),
                    def_param!(f64 FontInfo::DESCENDER, f64::MIN, constants::DESCENDER),
                    def_param!(f64 FontInfo::CAP_HEIGHT, f64::MIN, constants::CAP_HEIGHT),
                    def_param!(f64 FontInfo::ITALIC_ANGLE, f64::MIN, constants::ITALIC_ANGLE),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            FontInfo::MODIFIED => self.modified.get().to_value(),
            FontInfo::FAMILY_NAME => self.family_name.borrow().to_value(),
            FontInfo::STYLE_NAME => self.style_name.borrow().to_value(),
            FontInfo::STYLE_MAP_FAMILY_NAME => self.style_map_family_name.borrow().to_value(),
            FontInfo::STYLE_MAP_STYLE_NAME => self.style_map_style_name.borrow().to_value(),
            FontInfo::YEAR => self.year.get().to_value(),
            FontInfo::COPYRIGHT => self.copyright.borrow().to_value(),
            FontInfo::TRADEMARK => self.trademark.borrow().to_value(),
            FontInfo::UNITS_PER_EM => self.units_per_em.get().to_value(),
            FontInfo::DESCENDER => self.descender.get().to_value(),
            FontInfo::X_HEIGHT => self.x_height.get().to_value(),
            FontInfo::CAP_HEIGHT => self.cap_height.get().to_value(),
            FontInfo::ASCENDER => self.ascender.get().to_value(),
            FontInfo::ITALIC_ANGLE => self.italic_angle.get().to_value(),
            FontInfo::NOTE => self.note.borrow().to_value(),
            FontInfo::VERSION_MAJOR => self.version_major.get().to_value(),
            FontInfo::VERSION_MINOR => self.version_minor.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &glib::ParamSpec) {
        macro_rules! set_cell {
            ($field:tt) => {{
                let val = value.get().unwrap();
                let mut src = self.source.borrow_mut();
                if Some(val) != src.$field {
                    self.instance().set_property(FontInfo::MODIFIED, true);
                    src.$field = Some(val);
                }
                self.$field.set(val);
            }};
        }
        macro_rules! set_refcell {
            ($field:ident) => {{
                let val: String = value.get().unwrap();
                let mut src = self.source.borrow_mut();
                if val != src.$field {
                    self.instance().set_property(FontInfo::MODIFIED, true);
                    src.$field = val.clone();
                }
                *self.$field.borrow_mut() = val;
            }};
            (opt $field:tt) => {{
                let val: String = value.get().unwrap();
                let mut src = self.source.borrow_mut();
                if Some(&val) != src.$field.as_ref() {
                    self.instance().set_property(FontInfo::MODIFIED, true);
                    src.$field = Some(val.clone());
                }
                *self.$field.borrow_mut() = val;
            }};
        }
        match pspec.name() {
            FontInfo::MODIFIED => {
                self.modified.set(value.get().unwrap());
            }
            FontInfo::FAMILY_NAME => {
                set_refcell!(family_name);
            }
            FontInfo::STYLE_NAME => {
                set_refcell!(style_name);
            }
            FontInfo::STYLE_MAP_FAMILY_NAME => {
                set_refcell!(style_map_family_name);
            }
            FontInfo::STYLE_MAP_STYLE_NAME => {
                set_refcell!(style_map_style_name);
            }
            FontInfo::COPYRIGHT => {
                set_refcell!(copyright);
            }
            FontInfo::TRADEMARK => {
                set_refcell!(trademark);
            }
            FontInfo::NOTE => {
                set_refcell!(opt note);
            }
            FontInfo::YEAR => {
                set_cell!(year);
            }
            FontInfo::VERSION_MAJOR => {
                set_cell!(version_major);
            }
            FontInfo::VERSION_MINOR => {
                set_cell!(version_minor);
            }
            FontInfo::UNITS_PER_EM => {
                set_cell!(units_per_em);
            }
            FontInfo::X_HEIGHT => {
                set_cell!(x_height);
            }
            FontInfo::ASCENDER => {
                set_cell!(ascender);
            }
            FontInfo::DESCENDER => {
                set_cell!(descender);
            }
            FontInfo::CAP_HEIGHT => {
                set_cell!(cap_height);
            }
            FontInfo::ITALIC_ANGLE => {
                set_cell!(italic_angle);
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct FontInfo(ObjectSubclass<FontInfoInner>);
}

impl_deref!(FontInfo, FontInfoInner);
impl_property_window!(FontInfo, { "Font Metadata" });

impl FontInfo {
    pub const MODIFIED: &'static str = "modified";
    pub const FAMILY_NAME: &'static str = "family-name";
    pub const STYLE_NAME: &'static str = "style-name";
    pub const STYLE_MAP_FAMILY_NAME: &'static str = "style-map-family-name";
    pub const STYLE_MAP_STYLE_NAME: &'static str = "style-map-style-name";
    pub const YEAR: &'static str = "year";
    pub const COPYRIGHT: &'static str = "copyright";
    pub const TRADEMARK: &'static str = "trademark";
    pub const UNITS_PER_EM: &'static str = "units-per-em";
    pub const DESCENDER: &'static str = "descender";
    pub const X_HEIGHT: &'static str = "x-height";
    pub const CAP_HEIGHT: &'static str = "cap-height";
    pub const ASCENDER: &'static str = "ascender";
    pub const ITALIC_ANGLE: &'static str = "italic-angle";
    pub const NOTE: &'static str = "note";
    pub const VERSION_MAJOR: &'static str = "version-major";
    pub const VERSION_MINOR: &'static str = "version-minor";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Err(format!("Directory <i>{}</i> does not exist.", path.display()).into());
        }
        if path.is_dir() {
            return Err(format!("Path {} is a directory.", path.display()).into());
        }
        let fontinfo = ufo::FontInfo::from_path(&path)
            .map_err(|err| format!("couldn't read fontinfo.plist {}: {}", path.display(), err))?;
        let ret: Self = Self::new();
        *ret.source.borrow_mut() = fontinfo.clone();
        let ufo::FontInfo {
            family_name,
            style_name,
            version_major,
            version_minor,
            copyright,
            trademark,
            units_per_em,
            ascender,
            descender,
            x_height,
            cap_height,
            italic_angle,
            guidelines,
            style_map_family_name,
            style_map_style_name,
            year,
            note,
            ..
        } = fontinfo;
        ret.modified.set(false);
        *ret.last_saved.borrow_mut() = None;
        ret.path.set(path).unwrap();
        *ret.family_name.borrow_mut() = family_name;
        *ret.style_name.borrow_mut() = style_name;
        *ret.style_map_family_name.borrow_mut() = style_map_family_name;
        *ret.style_map_style_name.borrow_mut() = style_map_style_name;
        *ret.copyright.borrow_mut() = copyright;
        *ret.trademark.borrow_mut() = trademark;
        if let Some(note) = note {
            *ret.note.borrow_mut() = note;
        }
        *ret.guidelines.borrow_mut() = guidelines;

        macro_rules! set_cell {
            ($field:expr, $val:expr) => {{
                if let Some(val) = $val {
                    $field.set(val);
                }
            }};
        }
        set_cell!(ret.version_major, version_major);
        set_cell!(ret.version_minor, version_minor);
        set_cell!(ret.year, year);
        set_cell!(ret.units_per_em, units_per_em);
        set_cell!(ret.ascender, ascender);
        set_cell!(ret.descender, descender);
        set_cell!(ret.x_height, x_height);
        set_cell!(ret.cap_height, cap_height);
        set_cell!(ret.italic_angle, italic_angle);
        Ok(ret)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.modified.get() {
            return Ok(());
        }
        self.source.borrow().save(self.path.get().unwrap())?;
        self.set_property(Self::MODIFIED, false);
        Ok(())
    }

    pub fn path(&self) -> &Path {
        self.path.get().unwrap()
    }

    pub fn modified(&self) -> bool {
        self.modified.get()
    }
}

impl Default for FontInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl_modified!(FontInfo);

mod layer {
    use super::*;

    #[derive(Debug)]
    pub struct LayerInner {
        modified: Cell<bool>,
        pub last_saved: RefCell<Option<u64>>,
        pub path: RefCell<PathBuf>,
        pub name: RefCell<String>,
        pub dir_name: RefCell<String>,
        pub contents_plist: RefCell<ufo::Contents>,
        pub glyphs: RefCell<IndexMap<String, Rc<RefCell<Glyph>>>>,
    }

    impl Default for LayerInner {
        fn default() -> Self {
            Self {
                modified: Cell::new(false),
                last_saved: RefCell::new(None),
                path: RefCell::new(PathBuf::default()),
                name: RefCell::new(String::new()),
                dir_name: RefCell::new(String::new()),
                contents_plist: RefCell::new(ufo::Contents::default()),
                glyphs: RefCell::new(IndexMap::default()),
            }
        }
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for LayerInner {
        const NAME: &'static str = "Layer";
        type Type = Layer;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    // Trait shared by all GObjects
    impl ObjectImpl for LayerInner {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
                once_cell::sync::Lazy::new(|| {
                    vec![
                        ParamSpecBoolean::new(
                            Layer::MODIFIED,
                            Layer::MODIFIED,
                            Layer::MODIFIED,
                            false,
                            glib::ParamFlags::READWRITE,
                        ),
                        def_param!(str Layer::NAME),
                        def_param!(str Layer::DIR_NAME),
                    ]
                });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                Layer::MODIFIED => self.modified.get().to_value(),
                Layer::NAME => self.name.borrow().to_value(),
                Layer::DIR_NAME => self.dir_name.borrow().to_value(),
                _ => unimplemented!("{}", pspec.name()),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &Value,
            pspec: &glib::ParamSpec,
        ) {
            macro_rules! set_refcell {
                ($field:ident) => {{
                    let val: String = value.get().unwrap();
                    let mut field = self.$field.borrow_mut();
                    if &val != &*field {
                        self.instance().set_property(Layer::MODIFIED, true);
                    }
                    *field = val;
                }};
            }
            match pspec.name() {
                Layer::MODIFIED => {
                    self.modified.set(value.get().unwrap());
                }
                Layer::NAME => {
                    set_refcell!(name);
                }
                Layer::DIR_NAME => {
                    set_refcell!(dir_name);
                }
                _ => unimplemented!("{}", pspec.name()),
            }
        }
    }

    glib::wrapper! {
        pub struct Layer(ObjectSubclass<LayerInner>);
    }

    impl std::ops::Deref for Layer {
        type Target = LayerInner;

        fn deref(&self) -> &Self::Target {
            self.imp()
        }
    }

    impl Layer {
        pub const MODIFIED: &'static str = "modified";
        pub const NAME: &'static str = "name";
        pub const DIR_NAME: &'static str = "dir-name";

        pub fn new() -> Self {
            let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
            ret
        }

        pub fn init_from_path(
            &self,
            name: String,
            dir_name: String,
            mut root_path: PathBuf,
            create: bool,
        ) -> Result<(), Box<dyn std::error::Error>> {
            root_path.push(&dir_name);
            let mut path = root_path;
            if !create {
                if !path.exists() {
                    return Err(format!("Path <i>{}</i> does not exist.", path.display()).into());
                }
                if !path.is_dir() {
                    return Err(format!("Path {} is not a directory.", path.display()).into());
                }
            }
            self.modified.set(false);
            *self.last_saved.borrow_mut() = None;
            *self.dir_name.borrow_mut() = dir_name;
            *self.name.borrow_mut() = name;
            path.push("contents.plist");
            let contents = ufo::Contents::from_path(&path, create).map_err(|err| {
                format!("couldn't read contents.plist {}: {}", path.display(), err)
            })?;
            path.pop();
            let glyphs = Glyph::from_ufo(path.clone(), &contents)?;
            for g in glyphs.values() {
                self.link(&g.borrow().metadata);
            }
            *self.glyphs.borrow_mut() = glyphs;
            *self.contents_plist.borrow_mut() = contents;
            *self.path.borrow_mut() = path;

            Ok(())
        }

        pub fn save(
            &self,
            _layercontents: &mut crate::ufo::LayerContents,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if !self.modified.get() {
                return Ok(());
            }
            self.contents_plist
                .borrow_mut()
                .save(None, false)
                .map_err(|err| {
                    format!(
                        "Saving contents.plist of layer {} failed: {err}",
                        self.name.borrow()
                    )
                })?;
            let prefix = self.path.borrow();
            for g in self
                .glyphs
                .borrow()
                .values()
                .filter(|g| g.borrow().modified())
            {
                let g = g.borrow();
                g.save(&prefix)?;
                g.metadata.set_property(GlyphMetadata::MODIFIED, false);
            }
            self.modified.set(false);
            Ok(())
        }

        pub fn new_glyph(
            &self,
            name: String,
            glyph: Rc<RefCell<Glyph>>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let mut contents = self.contents_plist.borrow_mut();
            if let Some(f) = contents.glyphs().get(&name) {
                return Err(format!(
                    "Glyph `{name}` already exists in layer {layer} and its filename is `{f}`.",
                    layer = self.name.borrow()
                )
                .into());
            }
            glyph
                .borrow()
                .metadata
                .set_property(GlyphMetadata::LAYER, Some(self.clone()));
            self.set_property(Self::MODIFIED, true);
            contents.insert(name.clone(), glyph.borrow().metadata.filename().to_string());
            self.glyphs.borrow_mut().insert(name, glyph);
            Ok(())
        }

        pub fn path(&'_ self) -> FieldRef<'_, PathBuf> {
            self.path.borrow().into()
        }

        pub fn glyphs(&'_ self) -> FieldRef<'_, IndexMap<String, Rc<RefCell<Glyph>>>> {
            self.glyphs.borrow().into()
        }

        pub fn modified(&self) -> bool {
            self.modified.get()
        }
    }

    impl Default for Layer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl_modified!(Layer);
}

pub use layer::*;
