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

use crate::prelude::Application;
use glib::{Continue, MainContext, PRIORITY_DEFAULT};
use gtk::gdk;
use gtk::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyString};
use pyo3::PyCell;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;

pub mod shell;
pub mod types;

// [ref:needs_user_doc]
// [ref:TODO] Add cargo feature to statically embed python3
//  <https://pyo3.rs/v0.15.0/building_and_distribution.html#statically-embedding-the-python-interpreter>

/// Wrapper for glib main loop channel sender. [tag:python_api_main_loop_channel]
#[pyclass]
struct Sender(Option<glib::Sender<String>>);
/// Wrapper for API response channel receiver. [tag:python_api_response_channel]
#[pyclass]
struct Receiver(Option<mpsc::Receiver<String>>);

/// Application's global instance object.
///
/// It is exposed to python in order to allow it to communicate with the main thread and its data.
#[pyclass]
pub struct Gerb {
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
        Ok("Gerb global instance object.".to_string())
    }

    fn __annotations__<'p>(&'p self, py: Python<'p>) -> &'p PyDict {
        self.__types_dict.as_ref(py)
    }

    /// Return the currently loaded project.
    #[getter(project)]
    fn project(self_: PyRef<Self>) -> types::Project {
        types::Project {
            __gerb: self_.into(),
        }
    }

    /// Process API request.
    fn __send_rcv(&self, request: String, py: Python<'_>) -> PyResult<Py<PyAny>> {
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
            Some(Response::Object { py_type, value }) => Ok(py_type.into_any(value, py)),
            None => Ok(py.None()),
        }
    }
}

fn process_api_request(app: &Application, msg: String) -> Result<String, String> {
    let request: Request = serde_json::from_str(&msg).map_err(|err| err.to_string())?;
    // [ref:TODO] Make some kind of object registry instead of manually matching on `type_name`
    match request {
        Request::ObjectProperty {
            type_name,
            kind: Property::Get { property },
        } => Ok(match type_name.as_str() {
            "Project" => match app.window.project().try_property_value(&property) {
                Err(err) => serde_json::to_string(&Response::Error {
                    message: err.to_string(),
                })
                .unwrap(),
                Ok(val) => serde_json::to_string(&Response::from(val)).unwrap(),
            },
            _ => serde_json::to_string(&Response::Error {
                message: "Invalid object.".to_string(),
            })
            .unwrap(),
        }),
        Request::ObjectProperty {
            type_name,
            kind: Property::Set { property, value },
        } => Ok(match type_name.as_str() {
            "Project" => {
                let project = crate::prelude::Project::obj_ref(None, app);
                match serde_json::from_str(&value)
                    .map_err(|err| err.to_string())
                    .and_then(|val| project.set(&property, val).map_err(|err| err.to_string()))
                {
                    Err(err) => serde_json::to_string(&Response::Error { message: err }).unwrap(),
                    Ok(_val) => serde_json::to_string(&serde_json::json! { null }).unwrap(),
                }
            }
            _ => serde_json::to_string(&Response::Error {
                message: "Invalid object.".to_string(),
            })
            .unwrap(),
        }),
        Request::ObjectProperty {
            type_name: _,
            kind: Property::GetMany { properties: _ },
        } => todo!(),
        Request::ObjectProperty {
            type_name: _,
            kind: Property::SetMany { properties: _ },
        } => todo!(),
        Request::Action { name } => {
            app.upcast_ref::<gtk::gio::Application>()
                .activate_action(&name, None);
            Ok(serde_json::to_string(&serde_json::json! { null }).unwrap())
        }
    }
}

/// Operations on properties of an object. See [`Request`].
#[derive(Debug, Serialize, Deserialize)]
pub enum Property {
    Get { property: String },
    Set { property: String, value: String },
    GetMany { properties: Vec<String> },
    SetMany { properties: Vec<(String, String)> },
}

/// Request object from main thread to python thread that is serialized to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ObjectProperty { type_name: String, kind: Property },
    Action { name: String },
}

/// Response object from main thread to python thread that is serialized to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Error {
        message: String,
    },
    List {
        value: Vec<String>,
    },
    Dict {
        value: IndexMap<String, String>,
    },
    Object {
        py_type: PyType,
        value: serde_json::Value,
    },
}

impl From<glib::Value> for Response {
    fn from(value: glib::Value) -> Self {
        use serde_json::json;
        match value.type_() {
            glib::types::Type::UNIT => Self::Object {
                py_type: PyType::None,
                value: json!(null),
            },
            glib::types::Type::BOOL => Self::Object {
                py_type: PyType::Bool,
                value: json! {value.get::<bool>().unwrap()},
            },
            glib::types::Type::STRING => Self::Object {
                py_type: PyType::String,
                value: json! {value.get::<String>().unwrap()},
            },
            glib::types::Type::F64 => Self::Object {
                py_type: PyType::Float,
                value: json! {value.get::<f64>().unwrap()},
            },
            glib::types::Type::U64 => Self::Object {
                py_type: PyType::Int,
                value: json! {value.get::<u64>().unwrap()},
            },
            glib::types::Type::I64 => Self::Object {
                py_type: PyType::Int,
                value: json! {value.get::<i64>().unwrap()},
            },
            other => unimplemented!("{other:?}"),
        }
    }
}

/// Convenience enum to encode what kind of python object a request/response contains so that we
/// can deserialize it correctly.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PyType {
    Bool,
    Bytes,
    Dict,
    List,
    Float,
    UInt,
    Int,
    String,
    None,
}

impl PyType {
    /// Convert a `serde_json::Value` into a `Py<PyAny>>` according to the type hint in `self`.
    pub fn into_any(self, value: serde_json::Value, py: Python<'_>) -> Py<PyAny> {
        use PyType::*;
        match self {
            Bool => PyBool::new(py, value.as_bool().unwrap()).into(),
            Bytes => unimplemented!("Python bytes() objects have not been implemented."),
            Dict => unimplemented!("Python dict() objects have not been implemented."),
            List => unimplemented!("Python list() objects have not been implemented."),
            Float => PyFloat::new(py, value.as_f64().unwrap()).into(),
            UInt => value.as_u64().unwrap().into_py(py),
            Int => value.as_i64().unwrap().into_py(py),
            String => PyString::new(py, value.as_str().unwrap()).into(),
            None => py.None(),
        }
    }
}

pub trait AttributeGetSet<'app, 'ident>: glib::ObjectExt {
    fn obj_ref(identifier: Option<&'ident str>, app: &'app Application) -> Self;
    fn get(&self, name: &str) -> serde_json::Value {
        self.property::<String>(name).into()
    }
    fn set(
        &self,
        name: &str,
        value: serde_json::Value,
    ) -> Result<&Self, Box<dyn std::error::Error>> {
        match value {
            serde_json::Value::Null => {
                todo!();
            }
            serde_json::Value::Bool(val) => {
                self.try_set_property::<bool>(name, val)?;
            }
            serde_json::Value::Number(val) => {
                macro_rules! try_into {
                    ($prop_ty: ty, $best_ty:ty, $best_fn:ident, $sec_ty:ty, $sec_fn:ident,) => {
                        val.$best_fn()
                            .and_then(|val| {
                                self.try_set_property::<$prop_ty>(name, val.try_into().ok()?)
                                    .ok()
                            })
                            .or_else(|| {
                                val.$sec_fn().and_then(|val| {
                                    self.try_set_property::<$prop_ty>(name, val.try_into().ok()?)
                                        .ok()
                                })
                            })
                            .or_else(|| {
                                val.as_f64().and_then(|val| {
                                    self.try_set_property::<$prop_ty>(name, val as $prop_ty)
                                        .ok()
                                })
                            })
                            .ok_or_else(|| {
                                concat!("Cannot fit value to type ", stringify!($prop_ty))
                            })?
                    };
                    ($prop_ty: ty, $best_ty:ty, $best_fn:ident, $sec_ty:ty, $sec_fn:ident,) => {
                        if let Some(val) = val.$best_fn() {
                            self.try_set_property::<$prop_ty>(name, val.try_into()?)?;
                        } else if let Some(val) = val.$sec_fn() {
                            self.try_set_property::<$prop_ty>(name, val.try_into()?)?;
                        } else if let Some(val) = val.as_f64() {
                            self.try_set_property::<$prop_ty>(name, val as $prop_ty)?;
                        } else {
                            unreachable!("fixme?");
                        }
                    };
                    (float, $prop_ty: ty, $best_ty:ty, $best_fn:ident, $sec_ty:ty, $sec_fn:ident, $third_ty:ty, $third_fn:ident,) => {
                        if let Some(val) = val.$best_fn() {
                            self.try_set_property::<$prop_ty>(name, val as $prop_ty)?;
                        } else if let Some(val) = val.$sec_fn() {
                            self.try_set_property::<$prop_ty>(name, val as $prop_ty)?;
                        } else if let Some(val) = val.$third_fn() {
                            self.try_set_property::<$prop_ty>(name, val as $prop_ty)?;
                        } else {
                            unreachable!("fixme?");
                        }
                    };
                }
                match self
                    .find_property(name)
                    .expect("TODO return Err(_)")
                    .value_type()
                {
                    glib::types::Type::I8 => {
                        try_into! {
                            i8,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::U8 => {
                        try_into! {
                            u8,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::I32 => {
                        try_into! {
                            i32,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::U32 => {
                        try_into! {
                            u32,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::I_LONG => {
                        try_into! {
                            std::os::raw::c_long,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::U_LONG => {
                        try_into! {
                            std::os::raw::c_ulong,
                            u64, as_u64,
                            i64, as_i64,
                        }
                    }
                    glib::types::Type::I64 => {
                        try_into! {
                            i64,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::U64 => {
                        try_into! {
                            u64,
                            u64, as_u64,
                            i64, as_i64,
                        }
                    }
                    glib::types::Type::F32 => {
                        try_into! {
                            float,
                            f32,
                            f64, as_f64,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    glib::types::Type::F64 => {
                        try_into! {
                            float,
                            f64,
                            f64, as_f64,
                            i64, as_i64,
                            u64, as_u64,
                        }
                    }
                    other => return Err(format!("Attribute {name} is of type {other}").into()),
                }
            }
            serde_json::Value::String(val) => {
                self.try_set_property::<String>(name, val)?;
            }
            serde_json::Value::Array(_) => {
                todo!();
            }
            serde_json::Value::Object(_) => {
                todo!();
            }
        }
        Ok(self)
    }
}

impl<'app, 'ident> AttributeGetSet<'app, 'ident> for crate::prelude::Project {
    fn obj_ref(_: Option<&'ident str>, app: &'app Application) -> Self {
        app.window.project().clone()
    }
}

impl<'app, 'ident> AttributeGetSet<'app, 'ident> for crate::prelude::Settings {
    fn obj_ref(_: Option<&'ident str>, app: &'app Application) -> Self {
        app.settings.borrow().clone()
    }
}
impl<'app, 'ident> AttributeGetSet<'app, 'ident> for crate::ufo::objects::FontInfo {
    fn obj_ref(_: Option<&'ident str>, app: &'app Application) -> Self {
        app.window.project().fontinfo.borrow().clone()
    }
}
