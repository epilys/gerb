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

use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use toml_edit::{value as toml_value, Document, Item as TomlItem};

use crate::prelude::*;

pub mod types;

glib::wrapper! {
    pub struct Settings(ObjectSubclass<SettingsInner>);
}

impl std::ops::Deref for Settings {
    type Target = SettingsInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

#[derive(Debug, Default)]
pub struct SettingsInner {
    pub handle_size: Cell<f64>,
    pub line_width: Cell<f64>,
    pub guideline_width: Cell<f64>,
    pub warp_cursor: Cell<bool>,
    pub mark_color: Cell<types::MarkColor>,
    /// Weak references to objects grouped by type name. Whenever a property changes for one
    /// object, all of its kin is updated as well.
    pub obj_entries: RefCell<IndexMap<String, Vec<glib::object::WeakRef<glib::Object>>>>,
    /// Weak references to setting objects
    pub settings_entries: RefCell<IndexMap<String, glib::Object>>,
    #[allow(clippy::type_complexity)]
    pub file: Rc<RefCell<Option<(PathBuf, BufWriter<File>)>>>,
    pub document: Rc<RefCell<Document>>,
    pub show_prerelease_warning: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for SettingsInner {
    const NAME: &'static str = "Settings";
    type Type = Settings;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for SettingsInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.handle_size.set(Self::HANDLE_SIZE_INIT_VAL);
        self.line_width.set(Self::LINE_WIDTH_INIT_VAL);
        self.guideline_width.set(Self::GUIDELINE_WIDTH_INIT_VAL);
        self.warp_cursor.set(Self::WARP_CURSOR_INIT_VAL);
        self.show_prerelease_warning.set(true);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::new(
                        Settings::HANDLE_SIZE,
                        Settings::HANDLE_SIZE,
                        Settings::HANDLE_SIZE,
                        0.0001,
                        10.0,
                        SettingsInner::HANDLE_SIZE_INIT_VAL,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecDouble::new(
                        Settings::LINE_WIDTH,
                        Settings::LINE_WIDTH,
                        Settings::LINE_WIDTH,
                        0.0001,
                        10.0,
                        SettingsInner::LINE_WIDTH_INIT_VAL,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecDouble::new(
                        Settings::GUIDELINE_WIDTH,
                        Settings::GUIDELINE_WIDTH,
                        Settings::GUIDELINE_WIDTH,
                        0.0001,
                        10.0,
                        SettingsInner::GUIDELINE_WIDTH_INIT_VAL,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Settings::WARP_CURSOR,
                        Settings::WARP_CURSOR,
                        Settings::WARP_CURSOR,
                        SettingsInner::WARP_CURSOR_INIT_VAL,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        Settings::SHOW_PRERELEASE_WARNING,
                        Settings::SHOW_PRERELEASE_WARNING,
                        Settings::SHOW_PRERELEASE_WARNING,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecEnum::new(
                        Settings::MARK_COLOR,
                        Settings::MARK_COLOR,
                        "Show glyph mark colors in UI.",
                        types::MarkColor::static_type(),
                        types::MarkColor::None as i32,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Settings::HANDLE_SIZE => self.handle_size.get().to_value(),
            Settings::LINE_WIDTH => self.line_width.get().to_value(),
            Settings::GUIDELINE_WIDTH => self.guideline_width.get().to_value(),
            Settings::WARP_CURSOR => self.warp_cursor.get().to_value(),
            Settings::SHOW_PRERELEASE_WARNING => self.show_prerelease_warning.get().to_value(),
            Settings::MARK_COLOR => self.mark_color.get().to_value(),
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
            Settings::HANDLE_SIZE => {
                self.handle_size.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            Settings::LINE_WIDTH => {
                self.line_width.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            Settings::GUIDELINE_WIDTH => {
                self.guideline_width.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            Settings::WARP_CURSOR => {
                self.warp_cursor.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            Settings::SHOW_PRERELEASE_WARNING => {
                self.show_prerelease_warning.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            Settings::MARK_COLOR => {
                self.mark_color.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl SettingsInner {
    pub const HANDLE_SIZE_INIT_VAL: f64 = 5.0;
    pub const LINE_WIDTH_INIT_VAL: f64 = 0.85;
    pub const GUIDELINE_WIDTH_INIT_VAL: f64 = 1.0;
    pub const WARP_CURSOR_INIT_VAL: bool = false;

    pub fn get_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        fn validate_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
            if !path.exists() && path.parent().is_some() {
                let parent = path.parent().unwrap();
                std::fs::create_dir_all(parent)?;
            } else if OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(path)
                .is_ok()
            {
                return Ok(());
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)?;
            Ok(())
        }

        let xdg_dirs = xdg::BaseDirectories::with_prefix("gerb")
            .map_err(|err| format!("Could not detect XDG directories for user: {}", err))?;

        if let Ok(path) = std::env::var("GERB_CONFIG") {
            let retval = PathBuf::from(path);
            if let Err(err) = validate_path(&retval) {
                eprintln!("Could not access configuration file `{}` from environment variable `GERB_CONFIG`: {}\nFalling back to default configuration location...", retval.display(), err);
            } else {
                return Ok(retval);
            }
        }

        let path = xdg_dirs.place_config_file("config.toml").map_err(|err| {
            format!(
                "Cannot create configuration directory in {}: {}",
                xdg_dirs.get_config_home().display(),
                err
            )
        })?;
        validate_path(&path).map_err(|err| {
            format!(
                "Cannot access configuration file in {}: {}",
                path.display(),
                err
            )
        })?;
        Ok(path)
    }

    pub fn init_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_file()?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let mut toml = String::new();

        file.read_to_string(&mut toml)?;
        file.rewind()?;

        let doc = toml.parse::<Document>()?;
        *self.file.borrow_mut() = Some((path, BufWriter::new(file)));
        *self.document.borrow_mut() = doc;
        Ok(())
    }

    pub fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some((_, file)) = self.file.borrow_mut().as_mut() {
            let mut document = self.document.borrow_mut();
            document[Settings::HANDLE_SIZE] = toml_value(self.handle_size.get());
            document[Settings::LINE_WIDTH] = toml_value(self.line_width.get());
            document[Settings::GUIDELINE_WIDTH] = toml_value(self.guideline_width.get());
            document[Settings::WARP_CURSOR] = toml_value(self.warp_cursor.get());
            document[Settings::MARK_COLOR] = toml_value(self.mark_color.get().name());
            document[Settings::SHOW_PRERELEASE_WARNING] =
                toml_value(self.show_prerelease_warning.get());
            file.rewind()?;
            file.get_mut().set_len(0)?;
            file.write_all(document.to_string().as_bytes())?;
            file.flush()?;
        }
        Ok(())
    }

    fn read_new_setting(&self, obj: &glib::Object, prop: &glib::ParamSpec) {
        if !(prop
            .flags()
            .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
            && prop.owner_type() == obj.type_())
        {
            return;
        }
        let friendly_name = self.friendly_name(obj);
        let kebab_name = friendly_name.replace(' ', "-").to_ascii_lowercase();
        let mut document = self.document.borrow_mut();

        document.insert_formatted(
            &kebab_name.as_str().into(),
            toml_edit::Item::Table(toml_edit::Table::new()),
        );
        macro_rules! get_if_neq {
            ($ty:ty, str) => {{
                let new_val = obj.property::<$ty>(prop.name()).to_string();
                if document[&kebab_name]
                    .get(prop.name())
                    .and_then(TomlItem::as_str)
                    == Some(&new_val)
                {
                    return;
                }
                toml_value(new_val)
            }};
            ($ty:ty, enum) => {{
                let new_val = obj.property::<$ty>(prop.name()).name();
                if document[&kebab_name]
                    .get(prop.name())
                    .and_then(TomlItem::as_str)
                    == Some(&new_val)
                {
                    return;
                }
                toml_value(new_val)
            }};
            ($ty:ty, $fn:expr) => {{
                let new_val = obj.property::<$ty>(prop.name());
                if document[&kebab_name].get(prop.name()).and_then($fn) == Some(new_val) {
                    return;
                }
                toml_value(new_val)
            }};
        }
        /* Avoid loading properties unless the value has changed */
        let new_val = if prop.value_type() == bool::static_type() {
            get_if_neq!(bool, TomlItem::as_bool)
        } else if prop.value_type() == f64::static_type() {
            get_if_neq!(f64, TomlItem::as_float)
        } else if prop.value_type() == Color::static_type() {
            get_if_neq!(Color, str)
        } else if prop.value_type() == i64::static_type() {
            get_if_neq!(i64, TomlItem::as_integer)
        } else if prop.value_type() == types::ShowMinimap::static_type() {
            get_if_neq!(types::ShowMinimap, enum)
        } else if prop.value_type() == types::MarkColor::static_type() {
            get_if_neq!(types::MarkColor, enum)
        } else if prop.value_type() == types::Theme::static_type() {
            get_if_neq!(types::Theme, enum)
        } else {
            return;
        };
        document[&kebab_name][prop.name()] = new_val;

        /* Update other objects of the same type
         * Do not call set_property without dropping `document` first since it's borrowed mutably. */
        drop(document);

        for e in self
            .obj_entries
            .borrow()
            .get(&friendly_name)
            .map(|e| e.iter())
            .into_iter()
            .flatten()
            .filter(|wref| wref.upgrade().as_ref() != Some(obj))
        {
            let Some(obj) = e.upgrade() else { continue; };
            let document = self.document.borrow();
            macro_rules! set_if_neq {
                ($ty:ty, $val:expr) => {{
                    let val = { $val };
                    // set_property might notify recursively read_settings which will ask to borrow
                    // document mutably, so drop this here.
                    drop(document);
                    if obj.property::<$ty>(prop.name()) != val {
                        obj.set_property(prop.name(), val);
                    }
                }};
                ($ty:ty, opt $val:expr) => {{
                    let val = { $val };
                    if let Some(val) = val {
                        set_if_neq!($ty, val);
                    }
                }};
            }
            /* Avoid setting properties unless the value has changed */
            if prop.value_type() == bool::static_type() {
                set_if_neq!(bool, document[&kebab_name][prop.name()].as_bool().unwrap());
            } else if prop.value_type() == f64::static_type() {
                set_if_neq!(f64, document[&kebab_name][prop.name()].as_float().unwrap());
            } else if prop.value_type() == Color::static_type() {
                set_if_neq!(
                    Color,
                    Color::from_hex(document[&kebab_name][prop.name()].as_str().unwrap())
                );
            } else if prop.value_type() == i64::static_type() {
                set_if_neq!(
                    i64,
                    document[&kebab_name][prop.name()].as_integer().unwrap()
                );
            } else if prop.value_type() == types::MarkColor::static_type() {
                set_if_neq!(types::MarkColor, opt types::MarkColor::toml_deserialize(document[&kebab_name].get(prop.name())));
            } else if prop.value_type() == types::ShowMinimap::static_type() {
                set_if_neq!(types::ShowMinimap, opt types::ShowMinimap::toml_deserialize(document[&kebab_name].get(prop.name())));
            } else if prop.value_type() == types::Theme::static_type() {
                set_if_neq!(
                    types::Theme,
                    opt types::Theme::toml_deserialize(document[&kebab_name].get(prop.name()))
                );
            }
        }
    }

    pub fn load_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut document = self.document.borrow_mut();
        let mut save = false;
        /* floats */
        for (prop, field) in [
            (Settings::HANDLE_SIZE, &self.handle_size),
            (Settings::LINE_WIDTH, &self.line_width),
            (Settings::GUIDELINE_WIDTH, &self.guideline_width),
        ] {
            if let Some(v) = document.get(prop).and_then(TomlItem::as_float) {
                field.set(v);
            } else {
                document[prop] = toml_value(field.get());
                save = true;
            }
        }
        /* bools */
        for (prop, field) in [
            (Settings::WARP_CURSOR, &self.warp_cursor),
            (
                Settings::SHOW_PRERELEASE_WARNING,
                &self.show_prerelease_warning,
            ),
        ] {
            if let Some(v) = document.get(prop).and_then(TomlItem::as_bool) {
                field.set(v);
            } else {
                document[prop] = toml_value(field.get());
                save = true;
            }
        }
        /* enums */
        for (prop, field) in [(Settings::MARK_COLOR, &self.mark_color)] {
            if let Some(v) = types::MarkColor::toml_deserialize(document.get(prop)) {
                field.set(v);
            } else {
                document[prop] = toml_value(field.get().name());
                save = true;
            }
        }
        drop(document);
        if save {
            self.save_settings()?;
        }
        Ok(())
    }

    pub fn register_settings_type(&self, settings_type: impl FriendlyNameInSettings) {
        let name = settings_type.friendly_name().to_string();
        self.inner_register_settings_type(settings_type.upcast(), name)
    }

    fn inner_register_settings_type(&self, settings_type: glib::Object, friendly_name: String) {
        let mut settings_entries = self.settings_entries.borrow_mut();
        if !settings_entries.contains_key(&friendly_name) {
            let kebab_name = friendly_name.replace(' ', "-").to_ascii_lowercase();
            let document = self.document.borrow();
            if document.contains_key(&kebab_name) {
                for prop in glib::Object::list_properties(&settings_type)
                    .as_slice()
                    .iter()
                    .filter(|p| {
                        p.flags()
                            .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                            && p.owner_type() == settings_type.type_()
                    })
                {
                    if document[&kebab_name].get(prop.name()).is_some() {
                        if prop.value_type() == bool::static_type() {
                            settings_type.set_property(
                                prop.name(),
                                document[&kebab_name][prop.name()].as_bool().unwrap(),
                            );
                        } else if prop.value_type() == f64::static_type() {
                            settings_type.set_property(
                                prop.name(),
                                document[&kebab_name][prop.name()].as_float().unwrap(),
                            );
                        } else if prop.value_type() == Color::static_type() {
                            settings_type.set_property(
                                prop.name(),
                                Color::from_hex(
                                    document[&kebab_name][prop.name()].as_str().unwrap(),
                                ),
                            );
                        } else if prop.value_type() == i64::static_type() {
                            settings_type.set_property(
                                prop.name(),
                                document[&kebab_name][prop.name()].as_integer().unwrap(),
                            );
                        } else if prop.value_type() == types::MarkColor::static_type() {
                            if let Some(v) = types::MarkColor::toml_deserialize(
                                document[&kebab_name].get(prop.name()),
                            ) {
                                settings_type.set_property(prop.name(), v);
                            }
                        } else if prop.value_type() == types::Theme::static_type() {
                            if let Some(v) = types::Theme::toml_deserialize(
                                document[&kebab_name].get(prop.name()),
                            ) {
                                settings_type.set_property(prop.name(), v);
                            }
                        } else if prop.value_type() == types::ShowMinimap::static_type() {
                            if let Some(v) = types::ShowMinimap::toml_deserialize(
                                document[&kebab_name].get(prop.name()),
                            ) {
                                settings_type.set_property(prop.name(), v);
                            }
                        }
                    }
                }
            }
            let instance = self.instance();
            settings_type.connect_notify_local(
                None,
                clone!(@strong instance as obj => move |self_, param| {
                    if param.flags()
                        .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                        && param.owner_type() == self_.type_() {
                            obj.read_new_setting(self_, param);
                            _ = obj.save_settings();

                    }
                }),
            );
            settings_entries.insert(friendly_name, settings_type);
        }
    }

    pub fn register_obj(&self, obj: &glib::Object, settings_type: impl FriendlyNameInSettings) {
        let name = settings_type.friendly_name().to_string();
        self.inner_register_settings_type(settings_type.upcast(), name.clone());
        self.inner_register_obj(obj, name)
    }

    fn inner_register_obj(&self, obj: &glib::Object, name: String) {
        let settings_entries = self.settings_entries.borrow();
        let settings_type = &settings_entries[&name];
        for prop in glib::Object::list_properties(settings_type)
            .as_slice()
            .iter()
            .filter(|p| {
                p.flags()
                    .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                    && p.owner_type() == settings_type.type_()
            })
        {
            settings_type
                .bind_property(prop.name(), obj, prop.name())
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
    }

    // [tag:settings_path()_sync_return_value]
    pub fn path(&self) -> Option<PathBuf> {
        self.file.borrow().as_ref().map(|(p, _)| p.to_path_buf())
    }

    fn friendly_name(&self, obj: &glib::Object) -> String {
        if let Some(name) = self
            .obj_entries
            .borrow()
            .iter()
            .find_map(|(name, weakrefs)| {
                if let Some(strongref) = weakrefs.iter().find_map(|w| w.upgrade()) {
                    if strongref.type_() == obj.type_() {
                        return Some(name.to_string());
                    }
                }
                None
            })
        {
            return name;
        }

        if let Some(name) = self
            .settings_entries
            .borrow()
            .iter()
            .find_map(|(name, strongref)| {
                if strongref.type_() == obj.type_() {
                    return Some(name.to_string());
                }
                None
            })
        {
            return name;
        }
        obj.type_().name().to_ascii_lowercase()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub const HANDLE_SIZE: &str = "handle-size";
    pub const LINE_WIDTH: &str = "line-width";
    pub const GUIDELINE_WIDTH: &str = "guideline-width";
    pub const WARP_CURSOR: &str = "warp-cursor";
    pub const MARK_COLOR: &str = "mark-color";
    pub const SHOW_PRERELEASE_WARNING: &str = "show-prerelease-warning";

    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}

impl_friendly_name!(Settings);
impl crate::utils::property_window::CreatePropertyWindow for Settings {
    fn new_property_window(
        &self,
        app: &crate::prelude::Application,
        _create: bool,
    ) -> crate::prelude::PropertyWindow
    where
        Self: glib::IsA<glib::Object>,
    {
        let ret = PropertyWindow::builder(
            self.downgrade().upgrade().unwrap().upcast::<glib::Object>(),
            app,
        )
        .title(self.friendly_name())
        .friendly_name(self.friendly_name())
        .type_(PropertyWindowType::Modify)
        .build();
        for (name, obj) in self.settings_entries.borrow().iter() {
            ret.add_extra_obj(obj.clone(), Some(name.to_string().into()));
        }
        ret
    }
}
