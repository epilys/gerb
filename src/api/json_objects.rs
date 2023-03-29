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

use super::*;

/// Operations on properties of an object. See [`Request`].
#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Get,
    Set { value: String },
}

/// Request object from main thread to python thread that is serialized to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ObjectProperty {
        type_name: String,
        id: PyUuid,
        property: String,
        action: Action,
    },
}

impl Request {
    pub fn new_property(
        type_name: String,
        id: PyUuid,
        property: String,
        value: Option<String>,
    ) -> String {
        serde_json::json! {
            Request::ObjectProperty {
                type_name,
                id,
                property,
                action: if let Some(value) = value {
                    Action::Set {
                        value,
                    }
                } else {
                    Action::Get
                }
            }
        }
        .to_string()
    }
}

/// Response object from main thread to python thread that is serialized to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Error {
        message: String,
    },
    List {
        value: Vec<ObjectValue>,
    },
    Dict {
        value: IndexMap<String, ObjectValue>,
    },
    Object(ObjectValue),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectValue {
    pub py_type: PyType,
    pub value: serde_json::Value,
}

impl From<glib::Value> for Response {
    fn from(value: glib::Value) -> Self {
        use serde_json::json;
        match value.type_() {
            glib::types::Type::UNIT => Self::Object(ObjectValue {
                py_type: PyType::None,
                value: json!(null),
            }),
            glib::types::Type::BOOL => Self::Object(ObjectValue {
                py_type: PyType::Bool,
                value: json! {value.get::<bool>().unwrap()},
            }),
            glib::types::Type::STRING => Self::Object(ObjectValue {
                py_type: PyType::String,
                value: json! {value.get::<String>().unwrap()},
            }),
            glib::types::Type::F64 => Self::Object(ObjectValue {
                py_type: PyType::Float,
                value: json! {value.get::<f64>().unwrap()},
            }),
            glib::types::Type::U64 => Self::Object(ObjectValue {
                py_type: PyType::Int,
                value: json! {value.get::<u64>().unwrap()},
            }),
            glib::types::Type::I64 => Self::Object(ObjectValue {
                py_type: PyType::Int,
                value: json! {value.get::<i64>().unwrap()},
            }),
            other if other == crate::prelude::Continuity::static_type() => {
                Self::Object(ObjectValue {
                    py_type: PyType::Class,
                    value: json! {value.get::<Option<crate::prelude::Continuity>>().unwrap()},
                })
            }
            other => unimplemented!("{other:?}"),
        }
    }
}

impl From<ObjectValue> for Response {
    fn from(value: ObjectValue) -> Self {
        Self::Object(value)
    }
}

impl From<PyUuid> for Response {
    fn from(value: PyUuid) -> Self {
        use serde_json::json;
        Self::Object(ObjectValue {
            py_type: PyType::Bytes,
            value: json!(value.0.as_bytes()),
        })
    }
}

impl From<Either<PyUuid, ObjectValue>> for Response {
    fn from(value: Either<PyUuid, ObjectValue>) -> Self {
        match value {
            Either::A(uuid) => uuid.into(),
            Either::B(val) => val.into(),
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
    Class,
}

impl PyType {
    /// Convert a `serde_json::Value` into a `Py<PyAny>>` according to the type hint in `self`.
    pub fn into_any(self, value: serde_json::Value, py: Python<'_>) -> Py<PyAny> {
        use PyType::*;
        match self {
            Bool => PyBool::new(py, value.as_bool().unwrap()).into(),
            Bytes => PyBytes::new(
                py,
                &serde_json::value::from_value::<Vec<u8>>(value).unwrap(),
            )
            .into(),
            Dict => {
                let ret = PyDict::new(py);
                let map: IndexMap<std::string::String, PyUuid> =
                    serde_json::value::from_value(value).unwrap();
                for (k, v) in map {
                    ret.set_item(k, PyBytes::new(py, v.0.as_bytes())).unwrap();
                }

                ret.into()
            }
            List => {
                let vec: Vec<PyUuid> = serde_json::value::from_value(value).unwrap();
                PyList::new(
                    py,
                    vec.into_iter().map(|v| PyBytes::new(py, v.0.as_bytes())),
                )
                .into()
            }
            Float => PyFloat::new(py, value.as_f64().unwrap()).into(),
            UInt => value.as_u64().unwrap().into_py(py),
            Int => value.as_i64().unwrap().into_py(py),
            String => PyString::new(py, value.as_str().unwrap()).into(),
            Class => PyContinuity::into_py(serde_json::from_value(value).unwrap(), py),
            None => py.None(),
        }
    }
}
