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
use std::path::PathBuf;

pub use ufo_compile::*;
pub mod ufo_compile {
    use super::*;

    #[pyclass(module = "export")]
    #[derive(Debug, Clone, Copy)]
    pub enum OutputFormat {
        Otf,
        Ttf,
    }

    #[pyclass]
    #[derive(Debug, Clone)]
    pub struct UFOCompileOptions {
        #[pyo3(get, set)]
        input_dir: PathBuf,
        #[pyo3(get, set)]
        output_dir: PathBuf,
        #[pyo3(get, set)]
        format: OutputFormat,
        #[pyo3(get, set)]
        filename_stem: Option<String>,
        #[pyo3(get, set)]
        output_path: Option<PathBuf>,
    }

    impl Default for UFOCompileOptions {
        fn default() -> Self {
            Self::new()
        }
    }

    #[pymethods]
    impl UFOCompileOptions {
        #[new]
        fn new_python() -> Self {
            Self {
                input_dir: PathBuf::new(),
                output_dir: PathBuf::new(),
                format: OutputFormat::Otf,
                filename_stem: None,
                output_path: None,
            }
        }
    }

    macro_rules! gen_setter {
        ($($field_name:ident: $t:ty),* $(,)?) => {
            $(pub fn $field_name(mut self, value: $t) -> Self {
                self.$field_name = value;
                self
            })*
        };
    }

    impl UFOCompileOptions {
        pub fn new() -> Self {
            Self {
                input_dir: PathBuf::new(),
                output_dir: PathBuf::new(),
                format: OutputFormat::Otf,
                filename_stem: None,
                output_path: None,
            }
        }

        gen_setter! {
            input_dir: PathBuf,
            output_dir: PathBuf,
            format: OutputFormat,
            filename_stem: Option<String>,
            output_path: Option<PathBuf>,
        }
    }

    const FUNC: &str = include_str!("export.py");

    pub fn export(options: UFOCompileOptions) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let res: PyResult<PathBuf> = Python::with_gil(|py| {
            let module = PyModule::from_code(py, FUNC, "export.py", "export")?;
            module.add_class::<OutputFormat>()?;
            let filename: &PyAny = module
                .call_method1("export", (Py::new(py, options)?.into_ref(py),))?
                .extract()?;
            filename.extract()
        });
        Ok(res?)
    }

    pub fn export_action_cb(window: gtk::Window, project: crate::prelude::Project) {
        use crate::prelude::*;
        const OPEN_FOLDER: gtk::ResponseType = gtk::ResponseType::Other(0);
        const OPEN_ARTIFACT: gtk::ResponseType = gtk::ResponseType::Other(1);

        let input_dir = project.path.borrow().clone();
        let filename_stem = project.property::<Option<String>>(Project::FILENAME_STEM);
        let otf_filter = gtk::FileFilter::new();
        // mime types don't seem to work, dunno why.
        //otf_filter.add_mime_type("font/otf");
        otf_filter.add_pattern("*.otf");
        otf_filter.set_name(Some("OpenType (.otf)"));
        let ttf_filter = gtk::FileFilter::new();
        //ttf_filter.add_mime_type("font/ttf");
        ttf_filter.add_pattern("*.ttf");
        ttf_filter.set_name(Some("TrueType (.ttf)"));
        let filechooser = gtk::FileChooserNative::builder()
            .accept_label("Export")
            .create_folders(true)
            .do_overwrite_confirmation(true)
            .title("Select output path")
            .action(gtk::FileChooserAction::Save)
            .transient_for(&window)
            .build();
        filechooser.add_filter(&otf_filter);
        filechooser.add_filter(&ttf_filter);
        filechooser.set_filter(&otf_filter);
        _ = filechooser.add_shortcut_folder(&input_dir);
        filechooser.set_current_folder(&input_dir);
        if let Some(f) = filename_stem.as_ref() {
            filechooser.set_filename(&format!("{f}.otf"));
        }

        return_if_not_ok_or_accept!(filechooser.run());

        let Some(f) = filechooser.filename() else { return; };
        let Some(output_dir) = f.to_str() else { return; };
        let (output_path, format) = if let Some(path) = filechooser.filename() {
            let filter = |p: &PathBuf| !p.is_dir();
            match filechooser.filter().as_ref() {
                Some(f) if f == &ttf_filter => (Some(path).filter(filter), OutputFormat::Ttf),
                Some(f) if f == &otf_filter => (Some(path).filter(filter), OutputFormat::Otf),
                other => unreachable!("{other:?}"),
            }
        } else {
            match filechooser.filter().as_ref() {
                Some(f) if f == &ttf_filter => (None, OutputFormat::Ttf),
                Some(f) if f == &otf_filter => (None, OutputFormat::Otf),
                other => unreachable!("{other:?}"),
            }
        };
        filechooser.hide();
        match export(
            UFOCompileOptions::new()
                .input_dir(input_dir)
                .output_dir(output_dir.into())
                .format(format)
                .filename_stem(filename_stem)
                .output_path(output_path),
        ) {
            Ok(result_path) => {
                let dialog = crate::utils::widgets::new_simple_info_dialog(
                    match format {
                        OutputFormat::Otf => Some("Exported OTF artifact."),
                        OutputFormat::Ttf => Some("Exported TTF artifact."),
                    },
                    &format!(
                        "Project was exported successfully to\n<tt>{}</tt>",
                        result_path.display()
                    ),
                    None,
                    &window,
                );
                dialog.add_button("Open folder", OPEN_FOLDER);
                dialog.add_button(
                    match format {
                        OutputFormat::Otf => "Open OTF file",
                        OutputFormat::Ttf => "Open TTF file",
                    },
                    OPEN_ARTIFACT,
                );
                loop {
                    match dialog.run() {
                        response if matches!(response, OPEN_FOLDER) => {
                            let uri =
                                glib::filename_to_uri(result_path.parent().unwrap(), None).unwrap();
                            gtk::gio::AppInfo::launch_default_for_uri(
                                &uri,
                                gtk::gio::AppLaunchContext::NONE,
                            )
                            .unwrap();
                        }
                        response if matches!(response, OPEN_ARTIFACT) => {
                            let uri = glib::filename_to_uri(&result_path, None).unwrap();
                            gtk::gio::AppInfo::launch_default_for_uri(
                                &uri,
                                gtk::gio::AppLaunchContext::NONE,
                            )
                            .unwrap();
                        }
                        _ => break,
                    }
                }
                dialog.emit_close();
            }
            Err(err) => {
                let dialog = crate::utils::widgets::new_simple_error_dialog(
                    Some("Error: could not perform export"),
                    &err.to_string(),
                    None,
                    &window,
                );
                dialog.run();
                dialog.emit_close();
            }
        }
    }
}
