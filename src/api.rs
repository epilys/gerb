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

use crate::prelude::*;
use glib::{Continue, MainContext, PRIORITY_DEFAULT};

use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyCell;

use std::sync::mpsc;

mod shell;
pub use shell::*;

#[pyfunction]
pub fn add_one(x: i64) -> i64 {
    x + 1
}

#[pyclass]
struct Sender(Option<glib::Sender<String>>);
#[pyclass]
struct Receiver(Option<mpsc::Receiver<String>>);

#[pyclass]
struct Gerb {
    #[pyo3(get)]
    __stdout: pyo3::PyObject,
    #[pyo3(get)]
    __shell: pyo3::PyObject,
    #[pyo3(get)]
    __send: Py<Sender>,
    #[pyo3(get)]
    __rcv: Py<Receiver>,
    __types_dict: Py<PyDict>,
}

impl Gerb {
    fn types(py: Python<'_>) -> Py<PyDict> {
        PyDict::new(py).into()
    }
}

#[pymethods]
impl Gerb {
    fn __repr__(&self) -> PyResult<String> {
        Ok("GerbInstance".to_string())
    }

    fn __annotations__<'p>(&'p self, py: Python<'p>) -> &'p PyDict {
        self.__types_dict.as_ref(py)
    }

    /// Return the currently loaded project name.
    #[pyo3(text_signature = "() -> typing.Optional[str]")]
    fn project_name(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.__send
            .as_ref(py)
            .borrow()
            .0
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err(""))?
            .send("project-name".to_string())
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        let name = self
            .__rcv
            .as_ref(py)
            .borrow()
            .0
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err(""))?
            .recv()
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
        if name.is_empty() {
            Ok(None)
        } else {
            Ok(Some(name))
        }
    }
}
