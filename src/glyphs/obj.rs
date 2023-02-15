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

#[derive(Debug, Default)]
pub struct GlyphStateInner {
    modified: Cell<bool>,
    mark_color: Cell<Color>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphStateInner {
    const NAME: &'static str = "Glyph";
    type Type = GlyphState;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for GlyphStateInner {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        GlyphState::MODIFIED,
                        GlyphState::MODIFIED,
                        GlyphState::MODIFIED,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoxed::new(
                        GlyphState::MARK_COLOR,
                        GlyphState::MARK_COLOR,
                        GlyphState::MARK_COLOR,
                        Color::static_type(),
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct GlyphState(ObjectSubclass<GlyphStateInner>);
}

impl std::ops::Deref for GlyphState {
    type Target = GlyphStateInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl GlyphState {
    pub const MODIFIED: &str = "modified";
    pub const MARK_COLOR: &str = "mark-color";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }
}
