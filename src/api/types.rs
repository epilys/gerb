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

//! Wrapper types to expose to Python.
//!
//! Since pyo3 is not send safe, we cannot pass `GObjects` through channels. An
//! alternative is to create types with the same name and process all attribute get/set calls or
//! method calls via a message passing channel to the main thread.
//!
//! The singleton object in every python session is [`crate::api::Gerb`]. It contains the channel
//! receivers and senders.
//!
//! The API types will all need a `Py<Gerb>` reference in order to access the
//! API channel from the python thread to the main thread. `Py<_>` means a shared reference that is
//! owned by the Python side of things and can only be accessed with a `Python<'_>` token.
//!
//! That means every object is defined as:
//!
//! ```ignore
//! pub struct Object {
//!     pub(in crate::api) __gerb: Py<Gerb>,
//! }
//! ```
//!
//! and any kind of data access goes through the `__gerb` value as a JSON serialization of
//! [`crate::api::Request`]. [`Gerb`] will reply with a [`crate::api::Response`] JSON string.
//!
//! So any API type function has to perform this dance:
//!
//! ```ignore
//! // class method or static method or getter or setter.
//! fn method_name(&self, /* python arguments */, py: Python<'_>) -> PyResult</* result */> {
//!     self.__gerb
//!         .as_ref(py)
//!         .borrow()
//!         .__send_rcv(
//!             serde_json::to_string(&Request::/* Appropriate Request variant for what we want to do */)
//!             .unwrap(),
//!             py,
//!         )?
//!         .extract(py)
//! }
//! ```
//!
//! Unfortunately there's no succinct way to do this with the `pyo3` derive macros because objects
//! might have a lot of attributes and we have to write a getter method and a setter method for all
//! of them, and since the `#[pymethods]` is a proc_macro we cannot generate the methods inside of
//! it because of the way macros expand (in specific, this hypothetical macro won't have access to
//! the derive's macro attributes like `#[getter]` and `#[setter]` etc.).
//!
//! So this really ugly module is a manual implementation of a python class for `pyo3` that
//! generates all methods for any given number of fields.

use super::*;

macro_rules! generate_getter_method_def {
    ($struct:tt, $($field:ident, $docstr:literal, $ty:ty),*) => {

        $(
            _pyo3::class::PyMethodDefType::Getter({
                _pyo3::class::PyGetterDef::new(
                    concat!(stringify!($field), "\0"),
                    _pyo3::impl_::pymethods::PyGetter({
                        unsafe extern "C" fn trampoline(
                            slf: *mut _pyo3::ffi::PyObject,
                            closure: *mut ::std::os::raw::c_void,
                        ) -> *mut _pyo3::ffi::PyObject
                        {
                            _pyo3::impl_::trampoline::getter(
                                slf,
                                closure,
                                $field::get_tramp,
                            )
                        }
                        trampoline
                    }),
                    concat!($docstr, "\0"),
                )
            })
        )*

    }
 }

macro_rules! generate_setter_method_def {
    ($struct:tt, $($field:ident, $docstr:literal, $ty:ty),*) => {

        $(
            _pyo3::class::PyMethodDefType::Setter({
                _pyo3::class::PySetterDef::new(
                    concat!(stringify!($field), "\0"),
                    _pyo3::impl_::pymethods::PySetter({
                        unsafe extern "C" fn trampoline(
                            slf: *mut _pyo3::ffi::PyObject,
                            value: *mut _pyo3::ffi::PyObject,
                            closure: *mut ::std::os::raw::c_void,
                        ) -> ::core::ffi::c_int
                        {
                            _pyo3::impl_::trampoline::setter(
                                slf,
                                value,
                                closure,
                                $field::set_tramp,
                            )
                        }
                        trampoline
                    }),
                    concat!($docstr, "\0"),
                )
            })
        )*

    }
 }

macro_rules! generate_py_class {
    (
        #[docstring=$classdocstr:literal]
        struct $struct:tt {
            type PARENT_TYPE = $parent_type:ty;

            $(
                #[property_name=$property:ident]
                #[docstring=$docstr:literal]
                $field:ident: $field_ty:ty,
            )*
        },
        $(
            export $attr_name:ident as $wrapper_ty:tt,
        )*
    ) => {
        pub struct $struct {
            pub(in crate::api) __id: Uuid,
            pub(in crate::api) __gerb: Py<Gerb>,
        }

        const _: () = {
            use ::pyo3 as _pyo3;
            unsafe impl _pyo3::type_object::PyTypeInfo for $struct {
                type AsRefTarget = _pyo3::PyCell<Self>;
                const NAME: &'static str = stringify!($struct);
                const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;

                #[inline]
                fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                    use _pyo3::type_object::LazyStaticType;
                    static TYPE_OBJECT: LazyStaticType = LazyStaticType::new();
                    TYPE_OBJECT.get_or_init::<Self>(py)
                }
            }

            impl _pyo3::PyClass for $struct {
                type Frozen = _pyo3::pyclass::boolean_struct::False;
            }

            impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a $struct {
                type Holder = ::std::option::Option<_pyo3::PyRef<'py, $struct>>;

                #[inline]
                fn extract(
                    obj: &'py _pyo3::PyAny,
                    holder: &'a mut Self::Holder,
                ) -> _pyo3::PyResult<Self> {
                    _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
                }
            }

            impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a mut $struct {
                type Holder = ::std::option::Option<_pyo3::PyRefMut<'py, $struct>>;

                #[inline]
                fn extract(
                    obj: &'py _pyo3::PyAny,
                    holder: &'a mut Self::Holder,
                ) -> _pyo3::PyResult<Self> {
                    _pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
                }
            }

            impl _pyo3::IntoPy<_pyo3::PyObject> for $struct {
                fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                    _pyo3::IntoPy::into_py(_pyo3::Py::new(py, self).unwrap(), py)
                }
            }

            impl _pyo3::impl_::pyclass::PyClassImpl for $struct {
                const DOC: &'static str = concat!($classdocstr,"\0");
                const IS_BASETYPE: bool = false;
                const IS_SUBCLASS: bool = false;
                const IS_MAPPING: bool = false;
                const IS_SEQUENCE: bool = false;
                type Layout = _pyo3::PyCell<Self>;
                type BaseType = _pyo3::PyAny;
                type ThreadChecker = _pyo3::impl_::pyclass::ThreadCheckerStub<$struct>;
                type PyClassMutability =
                    <<_pyo3::PyAny as
                    _pyo3::impl_::pyclass::PyClassBaseType>::PyClassMutability
                    as _pyo3::impl_::pycell::PyClassMutability>::MutableChild;
                type Dict = _pyo3::impl_::pyclass::PyClassDummySlot;
                type WeakRef = _pyo3::impl_::pyclass::PyClassDummySlot;
                type BaseNativeType = _pyo3::PyAny;

                fn items_iter() -> _pyo3::impl_::pyclass::PyClassItemsIter {
                    use _pyo3::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                        methods: &[_pyo3::class::PyMethodDefType::Getter({
                            _pyo3::class::PyGetterDef::new(
                                "__gerb\0",
                                _pyo3::impl_::pymethods::PyGetter({
                                    unsafe extern "C" fn trampoline(
                                        slf: *mut _pyo3::ffi::PyObject,
                                        closure: *mut ::std::os::raw::c_void,
                                    ) -> *mut _pyo3::ffi::PyObject {
                                        _pyo3::impl_::trampoline::getter(
                                            slf,
                                            closure,
                                            $struct::__pymethod_get___gerb__,
                                        )
                                    }
                                    trampoline
                                }),
                                "\0",
                            )
                        })],
                        slots: &[],
                    };
                    PyClassItemsIter::new(&INTRINSIC_ITEMS, collector.py_methods())
                }
            }

            #[doc(hidden)]
            #[allow(non_snake_case)]
            impl $struct {
                unsafe fn __pymethod_get___gerb__(
                    _py: _pyo3::Python<'_>,
                    _slf: *mut _pyo3::ffi::PyObject,
                ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                    let _cell = _py
                        .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
                        .downcast::<_pyo3::PyCell<$struct>>()?;
                    let _ref = _cell.try_borrow()?;
                    let _slf: &$struct = &*_ref;
                    let item = ::std::clone::Clone::clone(&(_slf.__gerb));
                    let item: _pyo3::Py<_pyo3::PyAny> = _pyo3::IntoPy::into_py(item, _py);
                    ::std::result::Result::Ok(_pyo3::conversion::IntoPyPointer::into_ptr(item))
                }
            }
        };

        const _: () = {
            use ::pyo3 as _pyo3;
            impl _pyo3::impl_::pyclass::PyMethods<$struct> for _pyo3::impl_::pyclass::PyClassImplCollector<$struct> {
                fn py_methods(self) -> &'static _pyo3::impl_::pyclass::PyClassItems {
                    static ITEMS: _pyo3::impl_::pyclass::PyClassItems =
                        _pyo3::impl_::pyclass::PyClassItems {
                            methods: &[
                                $(generate_getter_method_def!($struct, $field, $docstr, $field_ty),)*
                                $(generate_setter_method_def!($struct, $field, $docstr, $field_ty),)*
                                $(generate_getter_method_def!($struct, $attr_name, " ", $wrapper_ty),)*
                            ],
                            slots: &[{
                                unsafe extern "C" fn trampoline(
                                    _slf: *mut _pyo3::ffi::PyObject,
                                ) -> *mut _pyo3::ffi::PyObject {
                                    _pyo3::impl_::trampoline::reprfunc(
                                        _slf,
                                        $struct::__pymethod___repr____,
                                    )
                                }
                                _pyo3::ffi::PyType_Slot {
                                    slot: _pyo3::ffi::Py_tp_repr,
                                    pfunc: trampoline as _pyo3::ffi::reprfunc as _,
                                }
                            }],
                        };
                    &ITEMS
                }
            }

            $(
                /* Hey what's this module doing here??
                 *
                 * Rust can't create arbitrary identifiers in macros.
                 * Which sucks, because we can't generate functions like this:
                 *
                 * ```
                 * fn get_$field(_) -> _ {
                 *  ...
                 * }
                 * ```
                 *
                 * But we need a trampoline function *and* a get/set function for every field,
                 * so their names must be unique.
                 *
                 * There are two options:
                 *
                 * - pass those unique names in the macro invocation, e.g.
                 *   generate_those_funcs!(my_name, get_my_name, get_my_name_trampoline)
                 * - or just use an identifier you already know is unique for each field: it's
                 *   own identity.
                 *
                 * Since $field is an :ident fragment specifier it can be used to identify
                 * things, obviously. So the generated functions are namespaced in a module
                 * called $field and problem solved.
                 */
                #[doc(hidden)]
                mod $field {
                    use super::*;
                    use ::pyo3 as _pyo3;

                    pub(super) unsafe fn get_tramp (
                        _py: _pyo3::Python<'_>,
                        _slf: *mut _pyo3::ffi::PyObject,
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let _cell = _py
                            .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
                            .downcast::<_pyo3::PyCell<$struct>>()?;
                        let _ref = _cell.try_borrow()?;
                        let _slf: &$struct = &*_ref;
                        let item = self::getter(_slf, _py);
                        _pyo3::callback::convert(_py, item)
                    }

                    pub(super) unsafe fn set_tramp (
                        _py: _pyo3::Python<'_>,
                        _slf: *mut _pyo3::ffi::PyObject,
                        _value: *mut _pyo3::ffi::PyObject,
                    ) -> _pyo3::PyResult<::core::ffi::c_int> {
                        let _cell = _py
                            .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
                            .downcast::<_pyo3::PyCell<$struct>>()?;
                        let mut _ref = _cell.try_borrow_mut()?;
                        let _slf: &mut $struct = &mut *_ref;
                        let _value = _py
                            .from_borrowed_ptr_or_opt(_value)
                            .ok_or_else(|| {
                                _pyo3::exceptions::PyAttributeError::new_err("can't delete attribute")
                            })?;
                        let _val = _pyo3::FromPyObject::extract(_value)?;

                        let item = self::setter(_slf, _val, _py);
                        _pyo3::callback::convert(_py, item)
                    }

                    fn getter(self_: &$struct, py: Python<'_>) -> PyResult<$field_ty> {
                        self_.__gerb
                            .as_ref(py)
                            .borrow()
                            .__send_rcv(
                                serde_json::to_string(&{
                                    Request::ObjectProperty {
                                        type_name: <$parent_type>::static_type().name().to_string(),
                                        id: self_.__id,
                                        property: <$parent_type>::$property.to_string(),
                                        action: Action::Get,
                                    }
                                })
                                .unwrap(),
                                py,
                            )?
                            .extract(py)
                    }

                    fn setter(self_: &mut $struct, value: $field_ty, py: Python<'_>) -> PyResult<()> {
                        self_.__gerb
                            .as_ref(py)
                            .borrow()
                            .__send_rcv(
                                serde_json::json!{
                                    Request::ObjectProperty {
                                        type_name: <$parent_type>::static_type().name().to_string(),
                                        id: self_.__id,
                                        property: <$parent_type>::$property.to_string(),
                                        action: Action::Set {
                                            value: serde_json::to_string(&serde_json::json! { value }).unwrap(),
                                        },
                                    }
                                }.to_string()
                                ,
                                py,
                            )?;
                        Ok(())
                    }
                }
            )*

            $(
                #[doc(hidden)]
                mod $attr_name {
                    use super::*;
                    use ::pyo3 as _pyo3;

                    pub(super) unsafe fn get_tramp (
                        _py: _pyo3::Python<'_>,
                        _slf: *mut _pyo3::ffi::PyObject,
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let _cell = _py
                            .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
                            .downcast::<_pyo3::PyCell<$struct>>()?;
                        let _ref = _cell.try_borrow()?;
                        let _slf: &$struct = &*_ref;
                        let item = self::getter(_slf, _py);
                        _pyo3::callback::convert(_py, item)
                    }

                    fn getter(self_: &$struct, py: Python<'_>) -> PyResult<$wrapper_ty> {
                        let __id: Uuid = Gerb::get_field_id(
                            &self_.__gerb.as_ref(py).borrow(),
                            self_.__id,
                             <$parent_type>::static_type().name(),
                            stringify!($attr_name),
                            py,
                        )?;
                        Ok($wrapper_ty {
                            __id,
                            __gerb: self_.__gerb.clone(),
                        })
                    }
                }
            )*

            #[doc(hidden)]
            #[allow(non_snake_case)]
            impl $struct {
                unsafe fn __pymethod___repr____(
                    _py: _pyo3::Python<'_>,
                    _raw_slf: *mut _pyo3::ffi::PyObject,
                ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                    let _slf = _raw_slf;
                    let _cell = _py
                        .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
                        .downcast::<_pyo3::PyCell<$struct>>()?;
                    let _ref = _cell.try_borrow()?;
                    let _slf: &$struct = &*_ref;
                    _pyo3::callback::convert(_py, $struct::__repr__(_slf))
                }
            }
        };

        impl $struct {
            fn __repr__(&self) -> PyResult<String> {
                Ok(format!("<{} instance, id: {}>", stringify!($struct), self.__id))
            }
        }
    };
}

generate_py_class!(
    #[docstring = "The currently loaded project."]
    struct Project {
        type PARENT_TYPE = ProjectParent;

        #[property_name=NAME]
        #[docstring = " "]
        name: String,
        #[property_name=MODIFIED]
        #[docstring = " "]
        modified: bool,
    },
    export font_info as FontInfo,
    export default_layer as Layer,
);

generate_py_class!(
    #[docstring = "Global settings."]
    struct Settings {
        type PARENT_TYPE = SettingsParent;

        #[property_name=HANDLE_SIZE]
        #[docstring = " "]
        handle_size: f64,
        #[property_name=LINE_WIDTH]
        #[docstring = " "]
        line_width: f64,
        #[property_name=GUIDELINE_WIDTH]
        #[docstring = " "]
        guideline_width: f64,
        #[property_name=WARP_CURSOR]
        #[docstring = " "]
        warp_cursor: bool,
    },
);

generate_py_class!(
    #[docstring = "Font info"]
    struct FontInfo {
        type PARENT_TYPE = FontInfoParent;

        #[property_name=FAMILY_NAME]
        #[docstring = " "]
        family_name: String,
        #[property_name=STYLE_NAME]
        #[docstring = " "]
        style_name: String,
        #[property_name=STYLE_MAP_FAMILY_NAME]
        #[docstring = " "]
        style_map_family_name: String,
        #[property_name=STYLE_MAP_STYLE_NAME]
        #[docstring = " "]
        style_map_style_name: String,
        #[property_name=COPYRIGHT]
        #[docstring = " "]
        copyright: String,
        #[property_name=TRADEMARK]
        #[docstring = " "]
        trademark: String,
        #[property_name=NOTE]
        #[docstring = " "]
        note: String,
        #[property_name=UNITS_PER_EM]
        #[docstring = " "]
        units_per_em: f64,
        #[property_name=X_HEIGHT]
        #[docstring = " "]
        x_height: f64,
        #[property_name=ASCENDER]
        #[docstring = " "]
        ascender: f64,
        #[property_name=DESCENDER]
        #[docstring = " "]
        descender: f64,
        #[property_name=CAP_HEIGHT]
        #[docstring = " "]
        cap_height: f64,
        #[property_name=ITALIC_ANGLE]
        #[docstring = " "]
        italic_angle: f64,
        #[property_name=YEAR]
        #[docstring = " "]
        year: u64,
        #[property_name=VERSION_MAJOR]
        #[docstring = " "]
        version_major: i64,
        #[property_name=VERSION_MINOR]
        #[docstring = " "]
        version_minor: u64,
    },
);

generate_py_class!(
    #[docstring = "Layers. <https://unifiedfontobject.org/versions/ufo3/layercontents.plist/>"]
    struct Layer {
        type PARENT_TYPE = LayerParent;

        #[property_name=NAME]
        #[docstring = " "]
        name: String,
        #[property_name=DIR_NAME]
        #[docstring = " "]
        dir_name: String,
    },
);
