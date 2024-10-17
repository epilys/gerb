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
    lock_guidelines: Cell<bool>,
    show_glyph_guidelines: Cell<bool>,
    show_project_guidelines: Cell<bool>,
    show_metrics_guidelines: Cell<bool>,
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
                vec![
                    glib::ParamSpecEnum::new(
                        EditorSettings::SHOW_MINIMAP,
                        EditorSettings::SHOW_MINIMAP,
                        "Show glyph minimap overview during modifications",
                        ShowMinimap::static_type(),
                        ShowMinimap::WhenManipulating as i32,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        EditorSettings::LOCK_GUIDELINES,
                        EditorSettings::LOCK_GUIDELINES,
                        EditorSettings::LOCK_GUIDELINES,
                        false,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        EditorSettings::SHOW_GLYPH_GUIDELINES,
                        EditorSettings::SHOW_GLYPH_GUIDELINES,
                        EditorSettings::SHOW_GLYPH_GUIDELINES,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        EditorSettings::SHOW_PROJECT_GUIDELINES,
                        EditorSettings::SHOW_PROJECT_GUIDELINES,
                        EditorSettings::SHOW_PROJECT_GUIDELINES,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        EditorSettings::SHOW_METRICS_GUIDELINES,
                        EditorSettings::SHOW_METRICS_GUIDELINES,
                        EditorSettings::SHOW_METRICS_GUIDELINES,
                        true,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            EditorSettings::SHOW_MINIMAP => self.show_minimap.get().to_value(),
            EditorSettings::LOCK_GUIDELINES => self.lock_guidelines.get().to_value(),
            EditorSettings::SHOW_GLYPH_GUIDELINES => self.show_glyph_guidelines.get().to_value(),
            EditorSettings::SHOW_PROJECT_GUIDELINES => {
                self.show_project_guidelines.get().to_value()
            }
            EditorSettings::SHOW_METRICS_GUIDELINES => {
                self.show_metrics_guidelines.get().to_value()
            }
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
            EditorSettings::SHOW_MINIMAP => {
                self.show_minimap.set(value.get().unwrap());
            }
            EditorSettings::LOCK_GUIDELINES => {
                self.lock_guidelines.set(value.get().unwrap());
            }
            EditorSettings::SHOW_GLYPH_GUIDELINES => {
                self.show_glyph_guidelines.set(value.get().unwrap());
            }
            EditorSettings::SHOW_PROJECT_GUIDELINES => {
                self.show_project_guidelines.set(value.get().unwrap());
            }
            EditorSettings::SHOW_METRICS_GUIDELINES => {
                self.show_metrics_guidelines.set(value.get().unwrap());
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
    pub const SHOW_MINIMAP: &'static str = "show-minimap";
    pub const LOCK_GUIDELINES: &'static str = "lock-guidelines";
    pub const SHOW_GLYPH_GUIDELINES: &'static str = "show-glyph-guidelines";
    pub const SHOW_PROJECT_GUIDELINES: &'static str = "show-project-guidelines";
    pub const SHOW_METRICS_GUIDELINES: &'static str = "show-metrics-guidelines";

    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}

impl_property_window!(EditorSettings, { "Editor" });
