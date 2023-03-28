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

glib::wrapper! {
    pub struct EditorSettings(ObjectSubclass<EditorSettingsInner>);
}

impl std::ops::Deref for EditorSettings {
    type Target = EditorSettingsInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

#[derive(Debug, Default)]
pub struct EditorSettingsInner {
    show_minimap: Cell<ShowMinimap>,
}

#[glib::object_subclass]
impl ObjectSubclass for EditorSettingsInner {
    const NAME: &'static str = "EditorSettings";
    type Type = EditorSettings;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for EditorSettingsInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecEnum::new(
                    Editor::SHOW_MINIMAP,
                    Editor::SHOW_MINIMAP,
                    "Show glyph minimap overview during modifications",
                    ShowMinimap::static_type(),
                    ShowMinimap::WhenManipulating as i32,
                    glib::ParamFlags::READWRITE | UI_EDITABLE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            Editor::SHOW_MINIMAP => self.show_minimap.get().to_value(),
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
            Editor::SHOW_MINIMAP => {
                self.show_minimap.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorSettings {
    pub const SHOW_MINIMAP: &str = "show-minimap";

    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}

impl_property_window!(EditorSettings, { "Editor" });
