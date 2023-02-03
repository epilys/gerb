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
#![allow(non_snake_case)]

use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::path::PathBuf;

use super::UFOInstance;

pub mod glyphsapp {
    use super::*;
    #[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
    pub enum GlyphsLibUfoModule {
        #[default]
        UFOLib2,
        Defcon,
    }

    impl GlyphsLibUfoModule {
        pub fn as_str(self) -> &'static str {
            match self {
                Self::UFOLib2 => "ufoLib2",
                Self::Defcon => "defcon",
            }
        }
    }

    #[pyclass]
    #[derive(Debug, Clone)]
    pub struct Glyphs2UFOOptions {
        #[pyo3(get, set)]
        glyphs_file: PathBuf,
        #[pyo3(get, set)]
        output_dir: Option<PathBuf>,
        #[pyo3(get, set)]
        designspace_path: Option<PathBuf>,
        #[pyo3(get, set)]
        instance_dir: Option<PathBuf>,
        #[pyo3(get, set)]
        ufo_module: String,
        #[pyo3(get, set)]
        minimal: bool,
        #[pyo3(get, set)]
        no_preserve_glyphsapp_metadata: bool,
        #[pyo3(get, set)]
        propagate_anchors: bool,
        #[pyo3(get, set)]
        generate_GDEF: bool,
        #[pyo3(get, set)]
        normalize_ufos: bool,
        #[pyo3(get, set)]
        create_background_layers: bool,
        #[pyo3(get, set)]
        no_store_editor_state: bool,
        #[pyo3(get, set)]
        write_public_skip_export_glyphs: bool,
        #[pyo3(get, set)]
        expand_includes: bool,
        #[pyo3(get, set)]
        glyph_data: Vec<PathBuf>,
        #[pyo3(get, set)]
        enable_last_change: bool,
        #[pyo3(get, set)]
        enable_automatic_alignment: bool,
    }

    macro_rules! gen_setter {
        ($($field_name:ident: $t:ty),* $(,)?) => {
            $(pub fn $field_name(mut self, value: $t) -> Self {
                self.$field_name = value;
                self
            })*
        };
    }

    impl Glyphs2UFOOptions {
        pub fn new(glyphs_file: PathBuf) -> Self {
            Self {
                glyphs_file,
                output_dir: None,
                designspace_path: None,
                instance_dir: None,
                ufo_module: GlyphsLibUfoModule::default().as_str().to_string(),
                minimal: false,
                no_preserve_glyphsapp_metadata: false,
                propagate_anchors: false,
                generate_GDEF: false,
                normalize_ufos: false,
                create_background_layers: true,
                no_store_editor_state: true,
                write_public_skip_export_glyphs: true,
                expand_includes: false,
                glyph_data: vec![],
                enable_last_change: true,
                enable_automatic_alignment: true,
            }
        }

        gen_setter! {
            output_dir: Option<PathBuf>,
            designspace_path: Option<PathBuf>,
            instance_dir: Option<PathBuf>,
            minimal: bool,
            no_preserve_glyphsapp_metadata: bool,
            propagate_anchors: bool,
            generate_GDEF: bool,
            normalize_ufos: bool,
            create_background_layers: bool,
            no_store_editor_state: bool,
            write_public_skip_export_glyphs: bool,
            expand_includes: bool,
            glyph_data: Vec<PathBuf>,
            enable_last_change: bool,
            enable_automatic_alignment: bool,
        }
        pub fn ufo_module(mut self, value: GlyphsLibUfoModule) -> Self {
            self.ufo_module = value.as_str().to_string();
            self
        }
    }

    const FUNC: &str = include_str!("glyphs_to_ufo3.py");

    pub fn import(
        options: Glyphs2UFOOptions,
    ) -> Result<Vec<UFOInstance>, Box<dyn std::error::Error>> {
        let res: PyResult<Vec<UFOInstance>> = Python::with_gil(|py| {
            let glyphs = PyModule::from_code(py, FUNC, "glyphs.py", "glyphs")?;
            let filenames: &PyList = glyphs
                .call_method1("glyphs2ufo", (Py::new(py, options)?.into_ref(py),))?
                .extract()?;
            let mut ret = Vec::with_capacity(filenames.len());
            for i in filenames.iter() {
                let (directory_name, full_path, family_name, style_name) = i.extract()?;
                ret.push(UFOInstance {
                    directory_name,
                    full_path,
                    family_name,
                    style_name,
                });
            }
            Ok(ret)
        });
        Ok(res?)
    }
}

pub use ufo2::*;
pub mod ufo2 {
    use super::*;

    #[pyclass]
    #[derive(Debug, Clone)]
    pub struct UFO2ToUFO3Options {
        #[pyo3(get, set)]
        input_dir: PathBuf,
        #[pyo3(get, set)]
        output_dir: PathBuf,
    }

    macro_rules! gen_setter {
        ($($field_name:ident: $t:ty),* $(,)?) => {
            $(pub fn $field_name(mut self, value: $t) -> Self {
                self.$field_name = value;
                self
            })*
        };
    }

    impl UFO2ToUFO3Options {
        pub fn new(input_dir: PathBuf, output_dir: PathBuf) -> Self {
            Self {
                input_dir,
                output_dir,
            }
        }

        gen_setter! {
            input_dir: PathBuf,
            output_dir: PathBuf,
        }
    }

    const FUNC: &str = include_str!("ufo2to3.py");

    pub fn import(options: UFO2ToUFO3Options) -> Result<UFOInstance, Box<dyn std::error::Error>> {
        let res: PyResult<UFOInstance> = Python::with_gil(|py| {
            let glyphs = PyModule::from_code(py, FUNC, "ufo2to3.py", "ufo2to3")?;
            let filename: &PyTuple = glyphs
                .call_method1("ufo2to3", (Py::new(py, options)?.into_ref(py),))?
                .extract()?;
            let (directory_name, full_path, family_name, style_name) = filename.extract()?;
            Ok(UFOInstance {
                directory_name,
                full_path,
                family_name,
                style_name,
            })
        });
        Ok(res?)
    }
}
