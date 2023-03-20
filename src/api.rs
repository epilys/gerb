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

#![allow(clippy::use_self)] // pyo3 derive macros cause this lint

//! # Python API
//!
//! To access application data from the python side, we create a [`Gerb`] object that contains
//! message passing channels. This object is then exposed to the python instance. See
//! [`crate::api::shell`] and [`Gerb`] for more details.
//!
//! The exposed types are in [`crate::api::types`].

use crate::prelude::{Application, Either, Error, Runtime};
use glib::{Continue, MainContext, PRIORITY_DEFAULT};
use gtk::gdk;
use gtk::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyString};
use pyo3::PyCell;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;

pub mod json_objects;
pub mod registry;
pub mod shell;
pub mod types;

pub use json_objects::*;
pub use registry::*;

// Define some type aliases to prevent ambiguating them with their API wrapper types:

use crate::app::Settings as SettingsParent;
use crate::prelude::Project as ProjectParent;
use crate::ufo::objects::FontInfo as FontInfoParent;
use crate::ufo::objects::Layer as LayerParent;

// [ref:needs_user_doc]
// [ref:TODO] Add cargo feature to statically embed python3
//  <https://pyo3.rs/v0.15.0/building_and_distribution.html#statically-embedding-the-python-interpreter>

/// Wrapper for glib main loop channel sender. [tag:python_api_main_loop_channel]
#[pyclass]
struct Sender(Option<glib::Sender<String>>);
/// Wrapper for API response channel receiver. [tag:python_api_response_channel]
#[pyclass]
struct Receiver(Option<mpsc::Receiver<String>>);

/// Runtime's global instance object.
///
/// It is exposed to python in order to allow it to communicate with the main thread and its data.
#[pyclass]
pub struct Gerb {
    __id: Uuid,
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

    pub fn get_field_id(
        self_: &PyRef<Self>,
        id: Uuid,
        type_name: &str,
        field_name: &str,
        py: Python<'_>,
    ) -> PyResult<Uuid> {
        let resp = self_.__send_rcv(
            Request::new_property(type_name.into(), id, field_name.into(), None),
            py,
        )?;
        let __id: Uuid = resp
            .extract::<Vec<u8>>(py)
            .map_err(|err| err.to_string())
            .and_then(|vec| Uuid::from_slice(&vec).map_err(|err| err.to_string()))
            .map_err(|err| PyException::new_err(Error::suggest_bug_report(&format!("expected a Uuid byte slice response ([u8; 16]) from the API but got {resp:?}: {err}"))))?;
        Ok(__id)
    }

    pub fn get_field_value(
        self_: &PyRef<Self>,
        id: Uuid,
        type_name: &str,
        field_name: &str,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        self_.__send_rcv(
            Request::new_property(type_name.into(), id, field_name.into(), None),
            py,
        ).map_err(|err| PyException::new_err(Error::suggest_bug_report(&format!("expected a value for field {field_name} of {type_name} with id {id} but got {err}"))))
    }
}

#[pymethods]
impl Gerb {
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "<Gerb global instance object {}>",
            crate::VERSION_INFO
        ))
    }

    pub fn __annotations__<'p>(&'p self, py: Python<'p>) -> &'p PyDict {
        self.__types_dict.as_ref(py)
    }

    /// Return the currently loaded project.
    #[getter(project)]
    pub fn project(self_: PyRef<Self>, py: Python<'_>) -> PyResult<types::Project> {
        let __id: Uuid = Self::get_field_id(
            &self_,
            self_.__id,
            Runtime::static_type().name(),
            "project",
            py,
        )?;
        Ok(types::Project {
            __id,
            __gerb: self_.into(),
        })
    }

    /// Return the global settings
    #[getter(settings)]
    pub fn settings(self_: PyRef<Self>, py: Python<'_>) -> PyResult<types::Settings> {
        let __id: Uuid = Self::get_field_id(
            &self_,
            self_.__id,
            Runtime::static_type().name(),
            "settings",
            py,
        )?;
        Ok(types::Settings {
            __id,
            __gerb: self_.into(),
        })
    }

    /// Process API request.
    pub fn __send_rcv(&self, request: String, py: Python<'_>) -> PyResult<Py<PyAny>> {
        // [ref:python_api_main_loop_channel]
        self.__send
            .as_ref(py)
            .borrow()
            .0
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("Could not communicate with main process"))?
            .send(request)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        // [ref:python_api_response_channel]
        let response_json = self
            .__rcv
            .as_ref(py)
            .borrow()
            .0
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("Could not communicate with main process"))?
            .recv()
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
        match serde_json::from_str(&response_json)
            .map_err(|err| PyRuntimeError::new_err(format!("Could not deserialize json response from main process: {err}\n\nResponse text was: {response_json}")))?
        {
            Some(Response::Error { message }) => Err(PyException::new_err(message)),
            Some(Response::List { value: _ }) => Ok(py.None()),
            Some(Response::Dict { value: _ }) => Ok(py.None()),
            Some(Response::Object(ObjectValue { py_type, value })) => Ok(py_type.into_any(value, py)),
            None => Ok(py.None()),
        }
    }
}

#[derive(Copy, Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct PyUuid(pub Uuid);

impl pyo3::IntoPy<pyo3::PyObject> for PyUuid {
    fn into_py(self, py: Python<'_>) -> pyo3::PyObject {
        PyBytes::new(py, self.0.as_bytes()).into()
    }
}

impl<'source> pyo3::FromPyObject<'source> for PyUuid {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        Ok(PyUuid(
            obj.extract::<Vec<u8>>()
                .map_err(|err| err.to_string())
                .and_then(|vec| Uuid::from_slice(&vec).map_err(|err| err.to_string()))
                .map_err(|err| {
                    PyException::new_err(format!(
                        "expected a Uuid byte slice but got {obj:?}: {err}"
                    ))
                })?,
        ))
    }
}

pub fn process_api_request(
    runtime: &Runtime,
    msg: String,
) -> Result<serde_json::Value, serde_json::Value> {
    let valid_types = [
        ProjectParent::static_type().name(),
        SettingsParent::static_type().name(),
        FontInfoParent::static_type().name(),
        LayerParent::static_type().name(),
        Runtime::static_type().name(),
        crate::prelude::GlyphMetadata::static_type().name(),
    ];
    let request: Request = serde_json::from_str(&msg).map_err(|err| {
        serde_json::json!(Response::Error {
            message: err.to_string(),
        })
    })?;
    match request {
        Request::ObjectProperty {
            type_name,
            id,
            property,
            action: Action::Get,
        } => {
            if !valid_types.contains(&type_name.as_str()) {
                return Err(serde_json::json!(Response::Error {
                    message: format!("Invalid object type: {type_name}."),
                }));
            }
            let obj = runtime.get_obj(id).unwrap();
            if let Some(field) =
                ProjectParent::expose_field(type_name.as_str(), &obj, Some(id), &property, runtime)
                    .or_else(|| {
                        SettingsParent::expose_field(
                            type_name.as_str(),
                            &obj,
                            Some(id),
                            &property,
                            runtime,
                        )
                    })
                    .or_else(|| {
                        Runtime::expose_field(
                            type_name.as_str(),
                            &obj,
                            Some(id),
                            &property,
                            runtime,
                        )
                    })
                    .or_else(|| {
                        crate::ufo::objects::FontInfo::expose_field(
                            type_name.as_str(),
                            &obj,
                            Some(id),
                            &property,
                            runtime,
                        )
                    })
                    .or_else(|| {
                        crate::ufo::objects::Layer::expose_field(
                            type_name.as_str(),
                            &obj,
                            Some(id),
                            &property,
                            runtime,
                        )
                    })
                    .or_else(|| {
                        crate::prelude::GlyphMetadata::expose_field(
                            type_name.as_str(),
                            &obj,
                            Some(id),
                            &property,
                            runtime,
                        )
                    })
            {
                return Ok(serde_json::json!(Response::from(field)));
            }

            Ok(match obj.try_property_value(&property) {
                Err(err) => Err(serde_json::json!(Response::Error {
                    message: err.to_string(),
                }))?,
                Ok(val) => serde_json::json!(Response::from(val)),
            })
        }
        Request::ObjectProperty {
            type_name,
            id,
            property,
            action: Action::Set { value },
        } => {
            if !valid_types.contains(&type_name.as_str()) {
                return Err(serde_json::json!(Response::Error {
                    message: format!("Invalid object type: {type_name}."),
                }));
            }
            let obj = runtime.get_obj(id).unwrap();
            Ok(
                match serde_json::from_str(&value)
                    .map_err(|err| err.to_string())
                    .and_then(|val| obj.set(&property, val).map_err(|err| err.to_string()))
                {
                    Err(err) => Err(serde_json::json!(Response::Error { message: err }))?,
                    Ok(_val) => serde_json::json! { null },
                },
            )
        }
    }
}
