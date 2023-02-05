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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml_edit::{value as toml_value, Document, Item as TomlItem};

glib::wrapper! {
    pub struct Settings(ObjectSubclass<SettingsInner>);
}

#[derive(Debug, Default)]
pub struct SettingsInner {
    pub handle_size: Cell<f64>,
    pub line_width: Cell<f64>,
    pub guideline_width: Cell<f64>,
    pub warp_cursor: Cell<bool>,
    #[allow(clippy::type_complexity)]
    pub file: Rc<RefCell<Option<(PathBuf, BufWriter<File>)>>>,
    pub document: Rc<RefCell<Document>>,
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
            document[Settings::HANDLE_SIZE] = toml_value(self.handle_size.get());
            document[Settings::LINE_WIDTH] = toml_value(self.line_width.get());
            document[Settings::GUIDELINE_WIDTH] = toml_value(self.guideline_width.get());
            document[Settings::WARP_CURSOR] = toml_value(self.warp_cursor.get());
            file.rewind()?;
            file.write_all(document.to_string().as_bytes())?;
            file.flush()?;
            file.rewind()?;
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
        if let Some(v) = document
            .get(Settings::WARP_CURSOR)
            .and_then(TomlItem::as_bool)
        {
            self.warp_cursor.set(v);
        } else {
            document[Settings::WARP_CURSOR] = toml_value(self.warp_cursor.get());
            save = true;
        }
        drop(document);
        if save {
            self.save_settings()?;
        }
        Ok(())
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

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecDouble::new(
                        Settings::HANDLE_SIZE,
                        Settings::HANDLE_SIZE,
                        Settings::HANDLE_SIZE,
                        0.0001,
                        10.0,
                        SettingsInner::HANDLE_SIZE_INIT_VAL,
                        ParamFlags::READWRITE | crate::UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Settings::LINE_WIDTH,
                        Settings::LINE_WIDTH,
                        Settings::LINE_WIDTH,
                        0.0001,
                        10.0,
                        SettingsInner::LINE_WIDTH_INIT_VAL,
                        ParamFlags::READWRITE | crate::UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        Settings::GUIDELINE_WIDTH,
                        Settings::GUIDELINE_WIDTH,
                        Settings::GUIDELINE_WIDTH,
                        0.0001,
                        10.0,
                        SettingsInner::GUIDELINE_WIDTH_INIT_VAL,
                        ParamFlags::READWRITE | crate::UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Settings::WARP_CURSOR,
                        Settings::WARP_CURSOR,
                        Settings::WARP_CURSOR,
                        SettingsInner::WARP_CURSOR_INIT_VAL,
                        ParamFlags::READWRITE | crate::UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Settings::HANDLE_SIZE => self.handle_size.get().to_value(),
            Settings::LINE_WIDTH => self.line_width.get().to_value(),
            Settings::GUIDELINE_WIDTH => self.guideline_width.get().to_value(),
            Settings::WARP_CURSOR => self.warp_cursor.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
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

    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
