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
    pub entries: RefCell<IndexMap<String, glib::Object>>,
    #[allow(clippy::type_complexity)]
    pub file: Rc<RefCell<Option<(PathBuf, BufWriter<File>)>>>,
    pub document: Rc<RefCell<Document>>,
    pub ui_font: Rc<RefCell<gtk::pango::FontDescription>>,
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
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path)?;
            } else {
                if OpenOptions::new()
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
            }
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
            for (type_name, obj) in self.entries.borrow().iter() {
                document.insert_formatted(
                    &type_name.as_str().into(),
                    toml_edit::Item::Table(toml_edit::Table::new()),
                );
                for prop in glib::Object::list_properties(obj)
                    .as_slice()
                    .iter()
                    .filter(|p| {
                        p.flags()
                            .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                            && p.owner_type() == obj.type_()
                    })
                {
                    if prop.value_type() == bool::static_type() {
                        document[type_name][prop.name()] =
                            toml_value(obj.property::<bool>(prop.name()));
                    } else if prop.value_type() == f64::static_type() {
                        document[type_name][prop.name()] =
                            toml_value(obj.property::<f64>(prop.name()));
                    } else if prop.value_type() == Color::static_type() {
                        document[type_name][prop.name()] =
                            toml_value(obj.property::<Color>(prop.name()).to_string());
                    } else if prop.value_type() == i64::static_type() {
                        document[type_name][prop.name()] =
                            toml_value(obj.property::<i64>(prop.name()));
                    } else if prop.value_type() == types::MarkColor::static_type() {
                        document[type_name][prop.name()] =
                            toml_value(&obj.property::<MarkColor>(prop.name()).name());
                    }
                }
            }
            document[Settings::HANDLE_SIZE] = toml_value(self.handle_size.get());
            document[Settings::LINE_WIDTH] = toml_value(self.line_width.get());
            document[Settings::GUIDELINE_WIDTH] = toml_value(self.guideline_width.get());
            document[Settings::WARP_CURSOR] = toml_value(self.warp_cursor.get());
            document[Settings::MARK_COLOR] = toml_value(self.mark_color.get().name());
            file.rewind()?;
            file.get_mut().set_len(0)?;
            file.write_all(document.to_string().as_bytes())?;
            file.flush()?;
        }
        Ok(())
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
        for (prop, field) in [(Settings::WARP_CURSOR, &self.warp_cursor)] {
            if let Some(v) = document.get(prop).and_then(TomlItem::as_bool) {
                field.set(v);
            } else {
                document[prop] = toml_value(field.get());
                save = true;
            }
        }
        /* enums */
        for (prop, field) in [(Settings::MARK_COLOR, &self.mark_color)] {
            if let Some(v) = types::MarkColor::deserialize(document.get(prop)) {
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

    pub fn register_obj(&self, obj: glib::Object) {
        let document = self.document.borrow();
        let type_name = obj.type_().name().to_ascii_lowercase();
        if document.contains_key(&type_name) {
            for prop in glib::Object::list_properties(&obj)
                .as_slice()
                .iter()
                .filter(|p| {
                    p.flags()
                        .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                        && p.owner_type() == obj.type_()
                })
            {
                if document[&type_name].get(prop.name()).is_some() {
                    if prop.value_type() == bool::static_type() {
                        obj.set_property(
                            prop.name(),
                            document[&type_name][prop.name()].as_bool().unwrap(),
                        );
                    } else if prop.value_type() == f64::static_type() {
                        obj.set_property(
                            prop.name(),
                            document[&type_name][prop.name()].as_float().unwrap(),
                        );
                    } else if prop.value_type() == Color::static_type() {
                        obj.set_property(
                            prop.name(),
                            Color::from_hex(document[&type_name][prop.name()].as_str().unwrap()),
                        );
                    } else if prop.value_type() == i64::static_type() {
                        obj.set_property(
                            prop.name(),
                            document[&type_name][prop.name()].as_integer().unwrap(),
                        );
                    }
                }
            }
        }
        let instance = self.instance();
        obj.connect_notify_local(
            None,
            clone!(@strong instance as obj => move |self_, param| {
                if param.flags()
                    .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                    && param.owner_type() == self_.type_() {
                        _ = obj.save_settings();

                }
            }),
        );
        self.entries.borrow_mut().insert(type_name, obj);
    }
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
        self.handle_size.set(SettingsInner::HANDLE_SIZE_INIT_VAL);
        self.line_width.set(SettingsInner::LINE_WIDTH_INIT_VAL);
        self.guideline_width
            .set(SettingsInner::GUIDELINE_WIDTH_INIT_VAL);
        self.warp_cursor.set(SettingsInner::WARP_CURSOR_INIT_VAL);

        self.init_file().unwrap();
        self.load_settings().unwrap();
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
                    glib::ParamSpecEnum::new(
                        Settings::MARK_COLOR,
                        Settings::MARK_COLOR,
                        "Show glyph mark colors in UI.",
                        types::MarkColor::static_type(),
                        types::MarkColor::None as i32,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        Settings::UI_FONT,
                        Settings::UI_FONT,
                        Settings::UI_FONT,
                        gtk::pango::FontDescription::static_type(),
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
            Settings::MARK_COLOR => self.mark_color.get().to_value(),
            Settings::UI_FONT => self.ui_font.borrow().to_value(),
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
            Settings::MARK_COLOR => {
                self.mark_color.set(value.get().unwrap());
                self.save_settings().unwrap();
            }
            Settings::UI_FONT => {
                *self.ui_font.borrow_mut() = value.get().unwrap();
                self.save_settings().unwrap();
            }
            _ => unimplemented!("{}", pspec.name()),
        }
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
    pub const UI_FONT: &str = "ui-font";

    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
