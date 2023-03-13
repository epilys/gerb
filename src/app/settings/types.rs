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

use serde::{Deserialize, Serialize};

pub trait EnumValue: glib::value::ToValue + Sized {
    fn name(&self) -> String {
        let value = self.to_value();
        let (_, v) = glib::EnumValue::from_value(&value).unwrap();
        v.nick().to_string()
    }

    fn deserialize<'de>(item: Option<&toml_edit::Item>) -> Option<Self>
    where
        Self: Deserialize<'de>,
    {
        use serde::de::IntoDeserializer;
        item.cloned()?
            .into_value()
            .ok()
            .map(toml_edit::Value::into_deserializer)
            .and_then(|p| <Self as Deserialize>::deserialize(p).ok())
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "MarkColor")]
#[serde(rename_all = "kebab-case")]
pub enum MarkColor {
    None,
    Background,
    #[default]
    Icon,
}

impl EnumValue for MarkColor {}

#[derive(Debug, Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "ShowMinimap")]
#[serde(rename_all = "kebab-case")]
pub enum ShowMinimap {
    Never,
    Always,
    #[default]
    WhenManipulating,
}

impl EnumValue for ShowMinimap {}

#[test]
fn test_parse_toml() {
    use crate::prelude::Color;
    use toml_edit::Document;

    const TOML: &str = r##"handle-size = 4.8500000000000005
line-width = 1.0
warp-cursor = false
guideline-width = 0.7999999999999998
mark-color = "none"

[canvas]
color = "#E6E6E4"
"##;
    let doc = TOML.parse::<Document>().unwrap();
    assert_eq!(doc["line-width"].as_float().unwrap(), 1.0);
    assert!(!doc["warp-cursor"].as_bool().unwrap());
    assert_eq!(
        <MarkColor as EnumValue>::deserialize(Some(&doc["mark-color"])).unwrap(),
        MarkColor::None
    );
    assert_eq!(
        Color::from_hex(doc["canvas"]["color"].as_str().unwrap()),
        Color::from_hex("#E6E6E4")
    );
}
