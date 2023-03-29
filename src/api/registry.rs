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

/// Wraps a an `expose_field` function pointer of the [`ObjRef`] trait.
///
/// It's necessary in order to declare an inventory of all available exposed types. This is done
/// using the [`inventory`](https://crates.io/crates/inventory/0.3.5) crate.
///
/// Example:
///
/// ```text
/// inventory::submit! { ExposeFn(Runtime::expose_field) }
/// ```
#[allow(clippy::type_complexity)]
#[derive(Copy, Clone)]
pub struct ExposeFn(
    pub  fn(
        _type_name: &str,
        _obj: &glib::Object,
        _identifier: Option<PyUuid>,
        _field_name: &str,
        _runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>>,
);

inventory::collect!(ExposeFn);

inventory::submit! { ExposeFn(ProjectParent::expose_field) }
inventory::submit! { ExposeFn(SettingsParent::expose_field) }
inventory::submit! { ExposeFn(Runtime::expose_field) }
inventory::submit! { ExposeFn(crate::ufo::objects::FontInfo::expose_field) }
inventory::submit! { ExposeFn(crate::ufo::objects::Layer::expose_field) }
inventory::submit! { ExposeFn(crate::prelude::GlyphMetadata::expose_field) }
inventory::submit! { ExposeFn(crate::glyphs::Contour::expose_field) }
inventory::submit! { ExposeFn(crate::prelude::Bezier::expose_field) }

#[inline(always)]
fn downcast<'a, T: glib::ObjectType + glib::IsA<glib::Object>>(
    _runtime: &Runtime,
    type_name: &str,
    obj: &'a glib::Object,
    _id: Option<PyUuid>,
) -> Result<&'a T, Box<dyn std::error::Error>> {
    debug_assert_eq!(
        _runtime.register_obj(obj),
        ObjectRegistry::opt_id(obj).unwrap()
    );

    obj.downcast_ref::<T>().ok_or_else(|| format!("Fatal API error: requested object of type {type_name:?} from Runtime registry but got something else instead: {}", obj.type_().name()).into())
}

/// Trait to convert fields of an object into serializable data for the API.
pub trait ObjRef: glib::ObjectExt {
    fn obj_ref(identifier: Option<PyUuid>, runtime: &Runtime) -> Self;

    fn expose_field(
        _type_name: &str,
        _obj: &glib::Object,
        _identifier: Option<PyUuid>,
        _field_name: &str,
        _runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        None
    }
}

/// Trait to get/set properties using [`serde_json::Value`]
pub trait AttributeGetSet: glib::ObjectExt {
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

impl ObjRef for crate::prelude::Runtime {
    fn obj_ref(_id: Option<PyUuid>, runtime: &Runtime) -> Self {
        #[cfg(debug_assertions)]
        if let Some(id) = _id {
            assert_eq!(id, runtime.register_obj(runtime.upcast_ref()));
        }

        runtime.clone()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }

        match field_name {
            "project" => Some(Either::A(
                runtime.register_obj(
                    downcast::<Self>(runtime, type_name, obj, id)
                        .unwrap()
                        .project
                        .borrow()
                        .upcast_ref(),
                ),
            )),
            "settings" => Some(Either::A(
                runtime.register_obj(
                    downcast::<Self>(runtime, type_name, obj, id)
                        .unwrap()
                        .settings
                        .upcast_ref(),
                ),
            )),
            _ => None,
        }
    }
}

impl ObjRef for ProjectParent {
    fn obj_ref(_: Option<PyUuid>, runtime: &Runtime) -> Self {
        runtime.project.borrow().clone()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "font_info" => Some(Either::A(
                runtime.register_obj(
                    downcast::<Self>(runtime, type_name, obj, id)
                        .unwrap()
                        .fontinfo
                        .borrow()
                        .upcast_ref(),
                ),
            )),
            "default_layer" => Some(Either::A(
                runtime.register_obj(
                    downcast::<Self>(runtime, type_name, obj, id)
                        .unwrap()
                        .default_layer
                        .upcast_ref(),
                ),
            )),
            "path" => Some(Either::B(ObjectValue {
                py_type: PyType::String,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .path
                    .borrow()
                    .to_path_buf()),
            })),
            _ => None,
        }
    }
}

impl ObjRef for crate::app::Settings {
    fn obj_ref(_: Option<PyUuid>, runtime: &Runtime) -> Self {
        runtime.settings.clone()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "path" => Some(Either::B(ObjectValue {
                py_type: PyType::String,
                // [ref:settings_path()_sync_return_value]
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .path()),
            })),
            _ => None,
        }
    }
}

impl ObjRef for crate::ufo::objects::FontInfo {
    fn obj_ref(_: Option<PyUuid>, runtime: &Runtime) -> Self {
        runtime.project.borrow().fontinfo.borrow().clone()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "path" => Some(Either::B(ObjectValue {
                py_type: PyType::String,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .path()
                    .to_path_buf()),
            })),
            "modified" => Some(Either::B(ObjectValue {
                py_type: PyType::Bool,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .modified()),
            })),
            _ => None,
        }
    }
}

impl ObjRef for crate::ufo::objects::Layer {
    fn obj_ref(id: Option<PyUuid>, runtime: &Runtime) -> Self {
        // [ref:TODO] return Option
        runtime.get_obj(id.unwrap()).unwrap().downcast().unwrap()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "path" => Some(Either::B(ObjectValue {
                py_type: PyType::String,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .path()
                    .to_path_buf()),
            })),
            "modified" => Some(Either::B(ObjectValue {
                py_type: PyType::Bool,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .modified()),
            })),
            "glyphs" => Some(Either::B(ObjectValue {
                py_type: PyType::Dict,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .glyphs()
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            runtime.register_obj(v.borrow().metadata.upcast_ref()),
                        )
                    })
                    .collect::<IndexMap<String, PyUuid>>()),
            })),
            _ => None,
        }
    }
}

impl ObjRef for crate::prelude::GlyphMetadata {
    fn obj_ref(id: Option<PyUuid>, runtime: &Runtime) -> Self {
        // [ref:TODO] return Option
        runtime.get_obj(id.unwrap()).unwrap().downcast().unwrap()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "modified" => Some(Either::B(ObjectValue {
                py_type: PyType::Bool,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .modified()),
            })),
            "contours" => Some(Either::B(ObjectValue {
                py_type: PyType::List,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .glyph_ref
                    .get()
                    .unwrap()
                    .borrow()
                    .contours
                    .iter()
                    .map(|v| { runtime.register_obj(v.upcast_ref()) })
                    .collect::<Vec<PyUuid>>()),
            })),
            /*"unicode" => Some(Either::B(ObjectValue {
                py_type: PyType::List,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .unicode()
                    .iter()
                    .map(|u| { u.hex().to_string() })
                    .collect::<Vec<String>>()),
            })),*/
            _ => None,
        }
    }
}

impl ObjRef for crate::glyphs::Contour {
    fn obj_ref(id: Option<PyUuid>, runtime: &Runtime) -> Self {
        // [ref:TODO] return Option
        runtime.get_obj(id.unwrap()).unwrap().downcast().unwrap()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "curves" => Some(Either::B(ObjectValue {
                py_type: PyType::List,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .curves()
                    .iter()
                    .map(|v| { runtime.register_obj(v.upcast_ref()) })
                    .collect::<Vec<PyUuid>>()),
            })),
            _ => None,
        }
    }
}

impl ObjRef for crate::prelude::Bezier {
    fn obj_ref(id: Option<PyUuid>, runtime: &Runtime) -> Self {
        // [ref:TODO] return Option
        runtime.get_obj(id.unwrap()).unwrap().downcast().unwrap()
    }

    fn expose_field(
        type_name: &str,
        obj: &glib::Object,
        id: Option<PyUuid>,
        field_name: &str,
        runtime: &Runtime,
    ) -> Option<Either<PyUuid, ObjectValue>> {
        if type_name != Self::static_type().name() {
            return None;
        }
        match field_name {
            "continuity_in" => Some(Either::B(ObjectValue {
                py_type: PyType::Class,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .continuity_in
                    .get()),
            })),
            "continuity_out" => Some(Either::B(ObjectValue {
                py_type: PyType::Class,
                value: serde_json::json!(downcast::<Self>(runtime, type_name, obj, id)
                    .unwrap()
                    .continuity_out
                    .get()),
            })),
            _ => None,
        }
    }
}

impl AttributeGetSet for glib::Object {}

/// Collection of live objects, each given a UUID. Holds weak references so that deallocating an
/// object doesn't cause any problem.
#[derive(Debug, Default)]
pub struct ObjectRegistry {
    index: IndexMap<PyUuid, glib::object::WeakRef<glib::Object>>,
}

impl ObjectRegistry {
    const QUARK_KEY: &str = "api-uuid";

    pub fn add(&mut self, obj: &glib::Object) -> PyUuid {
        Self::opt_id(obj).unwrap_or_else(|| {
            let id = PyUuid(Uuid::new_v4());
            self.index.insert(id, obj.downgrade());
            unsafe { obj.set_qdata(glib::Quark::from_str(Self::QUARK_KEY), id.0.as_u128()) };
            id
        })
    }

    pub fn get(&self, id: PyUuid) -> Option<glib::Object> {
        self.index.get(&id).and_then(glib::object::WeakRef::upgrade)
    }

    /// Check if `obj` has a set UUID
    pub fn opt_id(obj: &glib::Object) -> Option<PyUuid> {
        let id = unsafe { obj.qdata(glib::Quark::from_str(Self::QUARK_KEY)) }?;
        Some(PyUuid(Uuid::from_u128(unsafe { *id.as_ptr() })))
    }
}
