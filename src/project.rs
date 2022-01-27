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

use std::collections::HashMap;
use std::path::PathBuf;

use crate::glyphs::Glyph;

#[derive(Debug)]
pub struct Guideline {
    name: Option<String>,
    identifier: Option<String>,
    color: Option<(f64, f64, f64, f64)>,
    x: Option<f64>,
    y: Option<f64>,
    angle: Option<f64>,
}

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub modified: bool,
    pub last_saved: Option<u64>,
    pub glyphs: HashMap<u32, Glyph>,
    pub path: Option<PathBuf>,
    pub family_name: String,
    pub style_name: String,
    pub version_major: i64,
    pub version_minor: u64,
    /// Copyright statement.
    pub copyright: String,
    /// Trademark statement.
    pub trademark: String,
    /// Units per em.
    pub units_per_em: f64,
    /// Descender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
    pub descender: f64,
    /// x-height value.
    pub x_height: f64,
    /// Cap height value.
    pub cap_height: f64,
    /// Ascender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
    pub ascender: f64,
    /// Italic angle. This must be an angle in counter-clockwise degrees from the vertical.
    pub italic_angle: f64,
    /// Arbitrary note about the font.
    pub note: String,
    /// A list of guideline definitions that apply to all glyphs in all layers in the font. This attribute is optional.
    pub guidelines: Vec<Guideline>,
}

impl Default for Project {
    fn default() -> Self {
        let glyphs = Glyph::from_ufo("./font.ufo");
        Project {
            name: "test project".to_string(),
            modified: false,
            last_saved: None,
            /*glyphs: crate::utils::CODEPOINTS
                .chars()
                .map(|c| {
                    if c == 'b' {
                        ('b' as u32, Glyph::default())
                    } else {
                        (c as u32, Glyph::new_empty("", c))
                    }
                })
                .collect::<HashMap<u32, Glyph>>(),
            ,*/
            glyphs: glyphs
                .into_iter()
                .map(|g| (g.char as u32, g))
                .collect::<HashMap<u32, Glyph>>(),
            path: None,
            family_name: "Test Sans".to_string(),
            style_name: String::new(),
            version_major: 3,
            version_minor: 38,
            copyright: String::new(),
            trademark: String::new(),
            units_per_em: 1000.0,
            descender: -205.,
            x_height: 486.,
            cap_height: 656.,
            ascender: 712.,
            italic_angle: 0.,
            note: String::new(),
            guidelines: vec![],
        }
    }
}
