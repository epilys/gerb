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

use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::{Lazy, OnceCell};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

type ThemeKey = Cow<'static, str>;

glib::wrapper! {
    pub struct ThemeValue(ObjectSubclass<ThemeValueInner>);
}

#[derive(Debug, Default)]
pub struct ThemeValueInner {}

#[glib::object_subclass]
impl ObjectSubclass for ThemeValueInner {
    const NAME: &'static str = "ThemeValue";
    type Type = ThemeValue;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for ThemeValueInner {}

macro_rules! declare_theme_value_type {
    ($ident:tt, $inner:ident, $rusttype:ty) => {
        glib::wrapper! {
            pub struct $ident(ObjectSubclass<$inner>);
        }

        #[derive(Debug, Default)]
        pub struct $inner {
            attribute_name: RefCell<ThemeKey>,
            value: Cell<$rusttype>,
            default_value: Cell<$rusttype>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for $inner {
            const NAME: &'static str = "$ident";
            type Type = $ident;
            type ParentType = glib::Object;
            type Interfaces = ();
        }

        impl ObjectImpl for $inner {
            fn properties() -> &'static [glib::ParamSpec] {
                static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
                    once_cell::sync::Lazy::new(|| {
                        vec![
                            glib::ParamSpecString::new(
                                Theme::NAME,
                                Theme::NAME,
                                Theme::NAME,
                                None,
                                glib::ParamFlags::READABLE,
                            ),
                            glib::ParamSpecBoxed::new(
                                Theme::VALUE,
                                Theme::VALUE,
                                Theme::VALUE,
                                <$rusttype>::static_type(),
                                glib::ParamFlags::READWRITE | crate::UI_EDITABLE,
                            ),
                            glib::ParamSpecBoxed::new(
                                Theme::DEFAULT_VALUE,
                                Theme::DEFAULT_VALUE,
                                Theme::DEFAULT_VALUE,
                                <$rusttype>::static_type(),
                                glib::ParamFlags::READABLE,
                            ),
                        ]
                    });
                PROPERTIES.as_ref()
            }

            fn property(
                &self,
                _obj: &Self::Type,
                _id: usize,
                pspec: &glib::ParamSpec,
            ) -> glib::Value {
                match pspec.name() {
                    Theme::NAME => self.attribute_name.borrow().to_value(),
                    Theme::VALUE => self.value.get().to_value(),
                    Theme::DEFAULT_VALUE => self.default_value.get().to_value(),
                    _ => unimplemented!("{}", pspec.name()),
                }
            }

            fn set_property(
                &self,
                _obj: &Self::Type,
                _id: usize,
                value: &glib::Value,
                pspec: &glib::ParamSpec,
            ) {
                match pspec.name() {
                    Theme::VALUE => {
                        self.value.set(value.get().unwrap());
                    }
                    Theme::DEFAULT_VALUE => self.default_value.set(value.get().unwrap()),
                    _ => unimplemented!("{}", pspec.name()),
                }
            }
        }

        impl $ident {
            pub fn new(name: ThemeKey, value: $rusttype) -> Self {
                let ret = glib::Object::new::<Self>(&[]).unwrap();
                ret.imp().value.set(value);
                ret.imp().default_value.set(value);
                *ret.imp().attribute_name.borrow_mut() = name.into();
                ret
            }
        }
    };
}

declare_theme_value_type!(ThemeValueColor, ThemeValueColorInner, Color);
declare_theme_value_type!(ThemeValueBoolean, ThemeValueBooleanInner, bool);
declare_theme_value_type!(ThemeValueDouble, ThemeValueDoubleInner, f64);

pub use crate::utils::colors::*;

glib::wrapper! {
    pub struct Theme(ObjectSubclass<ThemeInner>);
}

#[derive(Debug)]
pub struct ThemeInner {
    pub target: OnceCell<glib::Object>,
    pub section: RefCell<String>,
    pub color_map: RefCell<HashMap<ThemeKey, ThemeValueColor>>,
    pub boolean_map: RefCell<HashMap<ThemeKey, ThemeValueBoolean>>,
    pub double_map: RefCell<HashMap<ThemeKey, ThemeValueDouble>>,
}

impl Default for ThemeInner {
    fn default() -> ThemeInner {
        ThemeInner {
            target: OnceCell::default(),
            section: RefCell::default(),
            color_map: RefCell::default(),
            boolean_map: RefCell::default(),
            double_map: RefCell::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ThemeInner {
    const NAME: &'static str = "Theme";
    type Type = Theme;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for ThemeInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        Theme::TARGET,
                        Theme::TARGET,
                        Theme::TARGET,
                        glib::Object::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        Theme::COLOR_ENTRIES,
                        Theme::COLOR_ENTRIES,
                        Theme::COLOR_ENTRIES,
                        gio::ListStore::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Theme::BOOLEAN_ENTRIES,
                        Theme::BOOLEAN_ENTRIES,
                        Theme::BOOLEAN_ENTRIES,
                        gio::ListStore::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        Theme::DOUBLE_ENTRIES,
                        Theme::DOUBLE_ENTRIES,
                        Theme::DOUBLE_ENTRIES,
                        gio::ListStore::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        Theme::SECTION,
                        Theme::SECTION,
                        Theme::SECTION,
                        None,
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Theme::TARGET => self.target.get().unwrap().to_value(),
            //Theme::COLOR_ENTRIES => self.color_entries.borrow().to_value(),
            //Theme::BOOLEAN_ENTRIES => self.boolean_entries.borrow().to_value(),
            //Theme::DOUBLE_ENTRIES => self.double_entries.borrow().to_value(),
            Theme::SECTION => self.section.borrow().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![
                Signal::builder(
                    Theme::ENTRY_CHANGED,
                    &[
                        glib::types::Type::static_type().into(),
                        glib::GString::static_type().into(),
                    ],
                    <()>::static_type().into(),
                )
                .build(),
                Signal::builder(
                    Theme::ENTRIES_RELOADED,
                    &[glib::types::Type::static_type().into()],
                    <()>::static_type().into(),
                )
                .build(),
            ]
        });
        SIGNALS.as_ref()
    }
}

impl Theme {
    pub const TARGET: &str = "target";
    pub const ENTRY_CHANGED: &str = "entry-changed";
    pub const ENTRIES_RELOADED: &str = "entries-reloaded";
    pub const COLOR_ENTRIES: &str = "color-entries";
    pub const BOOLEAN_ENTRIES: &str = "boolean-entries";
    pub const DOUBLE_ENTRIES: &str = "double-entries";
    pub const SECTION: &str = "section";
    pub const VALUE: &str = "value";
    pub const DEFAULT_VALUE: &str = "default-value";
    pub const NAME: &str = "name";

    pub fn new(target: glib::Object) -> Self {
        let ret = glib::Object::new::<Self>(&[]).unwrap();
        ret.imp().target.set(target).unwrap();
        ret
    }

    pub fn builder(target: glib::Object) -> ThemeBuilder {
        ThemeBuilder::new(target)
    }
}

pub struct ThemeBuilder {
    target: glib::Object,
    section: String,
    color_map: HashMap<ThemeKey, ThemeValueColor>,
    boolean_map: HashMap<ThemeKey, ThemeValueBoolean>,
    double_map: HashMap<ThemeKey, ThemeValueDouble>,
}

impl ThemeBuilder {
    pub fn new(target: glib::Object) -> Self {
        Self {
            target,
            section: String::new(),
            color_map: HashMap::default(),
            boolean_map: HashMap::default(),
            double_map: HashMap::default(),
        }
    }

    pub fn set_section(self, section: String) -> Self {
        Self { section, ..self }
    }

    pub fn add_color(mut self, key: impl Into<ThemeKey>, value: Color) -> Self {
        let key = key.into();
        self.color_map
            .insert(key.clone(), ThemeValueColor::new(key, value));
        self
    }

    pub fn add_boolean(mut self, key: impl Into<ThemeKey>, value: bool) -> Self {
        let key = key.into();
        self.boolean_map
            .insert(key.clone(), ThemeValueBoolean::new(key, value));
        self
    }

    pub fn add_double(mut self, key: impl Into<ThemeKey>, value: f64) -> Self {
        let key = key.into();
        self.double_map
            .insert(key.clone(), ThemeValueDouble::new(key, value));
        self
    }

    pub fn build(self) -> Theme {
        let Self {
            target,
            section,
            color_map,
            boolean_map,
            double_map,
        } = self;
        let retval = Theme::new(target);
        *retval.imp().section.borrow_mut() = section;
        macro_rules! connect_values {
            ($field:ident, $ty:tt) => {
                for v in $field.values() {
                    v.connect_notify_local(
                        Some(Theme::VALUE),
                        clone!(@weak retval as theme => move |entry, _| {
                            theme.emit_by_name::<()>(Theme::ENTRY_CHANGED, &[&entry.imp().value.get().to_value(), &$ty::static_type()]);
                        }),
                    );
                }
                *retval.imp().$field.borrow_mut() = $field;
            }
        }
        connect_values!(color_map, Color);
        connect_values!(double_map, f64);
        connect_values!(boolean_map, bool);
        retval
    }
}
