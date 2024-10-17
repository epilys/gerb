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
use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};

use crate::prelude::*;

pub type LayerCallback = dyn Fn(&Canvas, ContextRef<'_, '_>) -> Inhibit;

pub struct LayerInner {
    active: Cell<bool>,
    hidden: Cell<bool>,
    callback: Rc<RefCell<Rc<LayerCallback>>>,
    name: Rc<RefCell<Cow<'static, str>>>,
}

impl Default for LayerInner {
    fn default() -> Self {
        Self {
            callback: Rc::new(RefCell::new(Rc::new(|_canvas, _context| Inhibit(false)))),
            active: Cell::new(true),
            hidden: Cell::new(false),
            name: Rc::new(RefCell::new("".into())),
        }
    }
}

impl std::fmt::Debug for LayerInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("LayerInner").finish()
    }
}

#[glib::object_subclass]
impl ObjectSubclass for LayerInner {
    const NAME: &'static str = "CanvasLayer";
    type Type = Layer;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for LayerInner {
    fn constructed(&self, _obj: &Self::Type) {}

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        Layer::ACTIVE,
                        Layer::ACTIVE,
                        Layer::ACTIVE,
                        true,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        Layer::HIDDEN,
                        Layer::HIDDEN,
                        Layer::HIDDEN,
                        false,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecString::new(
                        Layer::NAME,
                        Layer::NAME,
                        Layer::NAME,
                        None,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            Layer::ACTIVE => self.active.get().to_value(),
            Layer::HIDDEN => self.hidden.get().to_value(),
            Layer::NAME => self.name.borrow().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Layer::ACTIVE => self.active.set(value.get().unwrap()),
            Layer::HIDDEN => self.hidden.set(value.get().unwrap()),
            Layer::NAME => *self.name.borrow_mut() = value.get::<String>().unwrap().into(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct Layer(ObjectSubclass<LayerInner>);
}

impl std::ops::Deref for Layer {
    type Target = LayerInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Layer {
    pub const ACTIVE: &'static str = "active";
    pub const HIDDEN: &'static str = "hidden";
    pub const NAME: &'static str = "name";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Layer");
        ret
    }

    #[inline]
    pub fn is_active(l: &&Self) -> bool {
        l.property::<bool>(Self::ACTIVE)
    }

    pub fn set_callback(&self, callback: Box<LayerCallback>) {
        *self.callback.borrow_mut() = callback.into();
    }

    pub fn get_callback(&self) -> std::cell::Ref<Rc<LayerCallback>> {
        self.callback.borrow()
    }

    pub fn reset_callback(&self) {
        *self.callback.borrow_mut() = Rc::new(|_canvas, _context| Inhibit(false));
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LayerBuilder {
    active: bool,
    hidden: bool,
    name: Option<Cow<'static, str>>,
    cb: Option<Box<LayerCallback>>,
}

impl LayerBuilder {
    pub fn new() -> Self {
        Self {
            active: true,
            hidden: false,
            name: None,
            cb: None,
        }
    }

    pub fn set_active(self, active: bool) -> Self {
        Self { active, ..self }
    }

    pub fn set_hidden(self, hidden: bool) -> Self {
        Self { hidden, ..self }
    }

    pub fn set_callback(self, cb: Option<Box<LayerCallback>>) -> Self {
        Self { cb, ..self }
    }

    pub fn set_name(self, name: Option<impl Into<Cow<'static, str>>>) -> Self {
        Self {
            name: name.map(Into::into),
            ..self
        }
    }

    pub fn build(self) -> Layer {
        let retval = Layer::new();
        retval.set_property::<bool>(Layer::ACTIVE, self.active);
        retval.set_property::<bool>(Layer::HIDDEN, self.hidden);
        if let Some(name) = self.name {
            *retval.name.borrow_mut() = name;
        }
        if let Some(cb) = self.cb {
            retval.set_callback(cb);
        }
        retval
    }
}

impl Default for LayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
