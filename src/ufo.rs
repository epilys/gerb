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

#[cfg(feature = "python")]
pub mod import;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct UFOInstance {
    pub directory_name: String,
    pub full_path: PathBuf,
    pub family_name: String,
    pub style_name: String,
}

/// fontinfo.plist
///
/// UFO3 Spec:
///
/// > This file contains information about the font itself, such as naming and dimensions.
/// > This file is optional. Not all values are required for a proper file.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    /* Generic Identification Information */
    #[serde(default)]
    pub family_name: String,
    #[serde(default)]
    pub style_name: String,
    #[serde(default)]
    pub style_map_family_name: String,
    #[serde(default)]
    pub style_map_style_name: String,
    #[serde(default)]
    pub year: Option<u64>,
    /* Generic Legal Information */
    #[serde(default)]
    pub copyright: String,
    #[serde(default)]
    pub trademark: String,
    /* Generic Dimension Information */
    #[serde(default)]
    pub units_per_em: Option<f64>,
    #[serde(default)]
    pub descender: Option<f64>,
    #[serde(default)]
    pub x_height: Option<f64>,
    #[serde(default)]
    pub cap_height: Option<f64>,
    #[serde(default)]
    pub ascender: Option<f64>,
    #[serde(default)]
    pub italic_angle: Option<f64>,
    /* Generic Miscellaneous Information */
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub version_major: Option<i64>,
    #[serde(default)]
    pub version_minor: Option<u64>,
    #[serde(default)]
    pub guidelines: Vec<GuidelineInfo>,
}

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidelineInfo {
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub angle: Option<f64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<crate::prelude::Color>,
    #[serde(default)]
    pub identifier: Option<String>,
}

impl FontInfo {
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // > This file is optional.
            return Ok(Self::default());
        }
        let retval: Self = plist::from_file(path)?;
        Ok(retval)
    }

    pub fn new_from_str(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let retval: Self = plist::from_reader_xml(std::io::Cursor::new(xml))?;
        Ok(retval)
    }

    pub fn save(&self, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
        #[allow(deprecated)]
        let opts = plist::XmlWriteOptions::default()
            .indent_string("  ")
            .root_element(true);

        let file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .open(destination)?;
        plist::to_writer_xml_with_options(file, self, &opts)?;
        Ok(())
    }
}

/// contents.plist
///
/// > `contents.plist` contains a dictionary that maps glyph names to GLIF file names.
/// > Glyph names may contain any character except they must not contain control characters.
/// > They must be at least one character long. There is no maximum name length. Glyph names must
/// > be unique within the layer.
///
/// # Specification
///
/// <https://unifiedfontobject.org/versions/ufo3/glyphs/contents.plist/>
///
/// > The property list data consists of a dictionary at the top level. The keys are glyph
/// > names and the values are file names.
///
/// > The file names must end with ".glif" and must begin with a string that is unique within
/// > the layer. The file names stored in the property list must be plain file names, not
/// > absolute or relative paths in the file system, and they must include the ".glif" extension.
/// > Care must be taken when choosing file names: glyph names are case sensitive, yet many file
/// > systems are not. There is no one standard glyph name to file name conversion. However, a
/// > common implementation is defined in the conventions.
///
/// > Authoring tools should preserve GLIF file names when writing into existing UFOs. This can
/// > be done by referencing the existing contents.plist before the write operation. The glyph
/// > name to file name mapping can then be referenced when creating new file names.
///
/// ## Example
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
/// "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
/// <plist version="1.0">
/// <dict>
///   <key>A</key>
///   <string>A_.glif</string>
///   <key>B</key>
///   <string>B_.glif</string>
/// </dict>
/// </plist>
/// ```
#[derive(Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contents {
    #[serde(flatten)]
    pub glyphs: HashMap<String, String>,
}

impl Contents {
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // This file is not optional.
            return Err(format!("Path {} does not exist: a valid UFOv3 project requires the presence of a contents.plist file.", path.display()).into());
        }
        let retval: Self = plist::from_file(path)?;
        Ok(retval)
    }

    pub fn new_from_str(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let retval: Self = plist::from_reader_xml(std::io::Cursor::new(xml))?;
        Ok(retval)
    }
}

/// metainfo.plist
///
/// > This file contains metadata about the UFO. This file is required.
///
/// # Specification
///
/// <https://unifiedfontobject.org/versions/ufo3/metainfo.plist/>
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaInfo {
    /// The application or library that created the UFO. This should follow a reverse domain
    /// naming scheme. For example, [`crate::APPLICATION_ID`]. Optional.
    #[serde(default)]
    pub creator: String,
    /// The major version number of the UFO format. 3 for UFO 3. Required.
    pub format_version: u64,
    /// The minor version number of the UFO format. Optional if the minor version is 0, must be
    /// present if the minor version is not 0.
    #[serde(default)]
    pub format_version_minor: u64,
}

impl Default for MetaInfo {
    fn default() -> Self {
        MetaInfo {
            creator: crate::APPLICATION_ID.to_string(),
            format_version: 3,
            format_version_minor: 0,
        }
    }
}

impl MetaInfo {
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // This file is not optional.
            return Err(format!("Path {} does not exist: a valid UFOv3 project requires the presence of a metainfo.plist file.", path.display()).into());
        }
        let retval: Self = plist::from_file(path)?;
        Ok(retval)
    }

    pub fn new_from_str(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let retval: Self = plist::from_reader_xml(std::io::Cursor::new(xml))?;
        Ok(retval)
    }
}

/// layercontents.plist
///
/// > This file maps the layer names to the glyph directory names. This file is required.
///
/// # Specification
///
/// <https://unifiedfontobject.org/versions/ufo3/layercontents.plist/>
#[derive(Debug)]
pub struct LayerContents {
    pub layers: IndexMap<String, String>,
}

impl Default for LayerContents {
    fn default() -> Self {
        let mut layers = IndexMap::new();
        layers.insert("public.default".to_string(), "glyphs".to_string());
        LayerContents { layers }
    }
}

impl LayerContents {
    fn inner_from_vec(vec: Vec<(String, String)>) -> Result<Self, Box<dyn std::error::Error>> {
        if vec.is_empty() {
            return Err("Input contains no layers: a valid UFOv3 project requires the presence of at least one layer, the default layer with name `public.default` and directory name `glyphs`.".into());
        }
        if &vec[0].1 != "glyphs" {
            return Err("Input contains an invalid default layer: a valid UFOv3 project requires the default layer (i.e. the first one) to have its directory name equal to `glyphs`.".into());
        }
        let vec_len = vec.len();
        let layers: IndexMap<String, String> = vec.into_iter().collect();
        if layers.len() != vec_len {
            return Err("Input contains duplicate layer names.".into());
        }
        let mut directories = layers
            .values()
            .skip(1)
            .collect::<indexmap::IndexSet<&String>>();
        if directories.len() != vec_len - 1 {
            return Err("Input contains duplicate layer directory values.".into());
        }
        directories.retain(|d| !d.starts_with("glyphs."));
        if !directories.is_empty() {
            return Err(format!(
                "Input contains layer directory values that don't start with `glyphs.`: {:?}.",
                &directories
            )
            .into());
        }
        Ok(Self { layers })
    }

    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // This file is not optional.
            return Err(format!("Path {} does not exist: a valid UFOv3 project requires the presence of a layercontents.plist file.", path.display()).into());
        }
        Self::inner_from_vec(plist::from_file(path)?)
            .map_err(|err| format!("Path {}: {err}", path.display()).into())
    }

    pub fn new_from_str(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::inner_from_vec(plist::from_reader_xml(std::io::Cursor::new(xml))?)
    }
}

#[test]
fn test_plist_parse() {
    let p: FontInfo = FontInfo::new_from_str(PLIST).unwrap();
    assert_eq!(&p.family_name, "Source Sans 3");
    assert_eq!(&p.style_name, "Regular");
    assert_eq!(&p.style_map_family_name, "");
    assert_eq!(&p.style_map_style_name, "");
    assert_eq!(p.year, None);
    assert_eq!(&p.copyright, "Copyright 2010–2021 Adobe Systems Incorporated (http://www.adobe.com/), with Reserved Font Name 'Source'.");
    assert_eq!(&p.trademark, "Source is a trademark of Adobe Systems Incorporated in the United States and/or other countries.");
    assert_eq!(p.units_per_em, Some(1000.0));
    assert_eq!(p.descender, Some(-205.0));
    assert_eq!(p.x_height, Some(486.0));
    assert_eq!(p.cap_height, Some(656.0));
    assert_eq!(p.ascender, Some(712.0));
    assert_eq!(p.italic_angle, Some(0.0));
    assert_eq!(p.note, None);
    assert_eq!(p.version_major, Some(3));
    assert_eq!(p.version_minor, Some(38));
    assert!(p.guidelines.is_empty());
    let c: Contents = Contents::new_from_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>A</key>
  <string>A_.glif</string>
  <key>B</key>
  <string>B_.glif</string>
</dict>
</plist>"#,
    )
    .unwrap();
    assert_eq!(
        &c.glyphs,
        &[
            ("A".to_string(), "A_.glif".to_string()),
            ("B".to_string(), "B_.glif".to_string())
        ]
        .into()
    );
    let m: MetaInfo = MetaInfo::new_from_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>creator</key>
  <string>com.epilys.gerb</string>
  <key>formatVersion</key>
  <integer>3</integer>
  <key>formatVersionMinor</key>
  <integer>0</integer>
</dict>
</plist>
"#,
    )
    .unwrap();
    assert_eq!(&m.creator, crate::APPLICATION_ID);
    assert_eq!(m.format_version, 3);
    assert_eq!(m.format_version_minor, 0);
    let m: MetaInfo = MetaInfo::new_from_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>formatVersion</key>
  <integer>3</integer>
</dict>
</plist>
"#,
    )
    .unwrap();
    assert!(m.creator.is_empty());
    assert_eq!(m.format_version, 3);
    assert_eq!(m.format_version_minor, 0);
    assert!(MetaInfo::new_from_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>creator</key>
  <string>goofball</string>
</dict>
</plist>
"#,
    )
    .is_err());
    let l: LayerContents = LayerContents::new_from_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <array>
    <string>public.default</string>
    <string>glyphs</string>
  </array>
  <array>
    <string>Sketches</string>
    <string>glyphs.S_ketches</string>
  </array>
  <array>
    <string>public.background</string>
    <string>glyphs.public.background</string>
  </array>
</array>
</plist>
"#,
    )
    .unwrap();
    assert_eq!(
        &l.layers,
        &[
            ("public.default".to_string(), "glyphs".to_string()),
            ("Sketches".to_string(), "glyphs.S_ketches".to_string()),
            (
                "public.background".to_string(),
                "glyphs.public.background".to_string()
            ),
        ]
        .into()
    );
    for (input, err_msg) in [
        (r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <array>
    <string>public.default</string>
    <string>glyphss</string>
  </array>
</array>
</plist>
"#,"Input contains an invalid default layer: a valid UFOv3 project requires the default layer (i.e. the first one) to have its directory name equal to `glyphs`."),
(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
</array>
</plist>
"#,"Input contains no layers: a valid UFOv3 project requires the presence of at least one layer, the default layer with name `public.default` and directory name `glyphs`."),
(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <array>
    <string>public.default</string>
    <string>glyphs</string>
  </array>
  <array>
    <string>public.default</string>
    <string>glyphs</string>
  </array>
</array>
</plist>
"#,"Input contains duplicate layer names."),
(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <array>
    <string>public.default</string>
    <string>glyphs</string>
  </array>
  <array>
    <string>two</string>
    <string>glyphs.two</string>
  </array>
  <array>
    <string>three</string>
    <string>glyphs.two</string>
  </array>
</array>
</plist>
"#,"Input contains duplicate layer directory values."),
(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <array>
    <string>public.default</string>
    <string>glyphs</string>
  </array>
  <array>
    <string>public.default.2</string>
    <string>2glyphs</string>
  </array>
</array>
</plist>
"#,"Input contains layer directory values that don't start with `glyphs.`: {\"2glyphs\"}."),
] {
    assert_eq!(&LayerContents::new_from_str(input).unwrap_err().to_string(), err_msg);
        }
}

#[test]
fn test_plist_write() {
    let p: FontInfo = FontInfo::new_from_str(PLIST).unwrap();
    #[allow(deprecated)]
    let opts = plist::XmlWriteOptions::default()
        .indent_string("  ")
        .root_element(true);

    let mut s = vec![];
    plist::to_writer_xml_with_options(std::io::Cursor::new(&mut s), &p, &opts).unwrap();
    let p2: FontInfo = FontInfo::new_from_str(&String::from_utf8(s).unwrap()).unwrap();
    assert_eq!(p, p2);
}

#[cfg(test)]
const PLIST: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
	<dict>
		<key>ascender</key>
		<integer>712</integer>
		<key>capHeight</key>
		<integer>656</integer>
		<key>copyright</key>
		<string>Copyright 2010–2021 Adobe Systems Incorporated (http://www.adobe.com/), with Reserved Font Name 'Source'.</string>
		<key>descender</key>
		<integer>-205</integer>
		<key>familyName</key>
		<string>Source Sans 3</string>
		<key>guidelines</key>
		<array>
		</array>
		<key>italicAngle</key>
		<integer>0</integer>
		<key>openTypeHheaAscender</key>
		<integer>1024</integer>
		<key>openTypeHheaDescender</key>
		<integer>-400</integer>
		<key>openTypeHheaLineGap</key>
		<integer>0</integer>
		<key>openTypeNameDesigner</key>
		<string>Paul D. Hunt</string>
		<key>openTypeNameLicense</key>
		<string>This Font Software is licensed under the SIL Open Font License, Version 1.1. This license is available with a FAQ at: http://scripts.sil.org/OFL. This Font Software is distributed on an 'AS IS' BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the SIL Open Font License for the specific language, permissions and limitations governing your use of this Font Software.</string>
		<key>openTypeNameLicenseURL</key>
		<string>http://scripts.sil.org/OFL</string>
		<key>openTypeNameManufacturer</key>
		<string>Adobe Systems Incorporated</string>
		<key>openTypeNameManufacturerURL</key>
		<string>http://www.adobe.com/type</string>
		<key>openTypeOS2CodePageRanges</key>
		<array>
			<integer>0</integer>
			<integer>1</integer>
			<integer>2</integer>
			<integer>3</integer>
			<integer>4</integer>
			<integer>7</integer>
			<integer>8</integer>
			<integer>29</integer>
		</array>
		<key>openTypeOS2Panose</key>
		<array>
			<integer>2</integer>
			<integer>11</integer>
			<integer>5</integer>
			<integer>3</integer>
			<integer>3</integer>
			<integer>4</integer>
			<integer>3</integer>
			<integer>2</integer>
			<integer>2</integer>
			<integer>4</integer>
		</array>
		<key>openTypeOS2TypoAscender</key>
		<integer>750</integer>
		<key>openTypeOS2TypoDescender</key>
		<integer>-250</integer>
		<key>openTypeOS2TypoLineGap</key>
		<integer>0</integer>
		<key>openTypeOS2UnicodeRanges</key>
		<array>
			<integer>0</integer>
			<integer>1</integer>
			<integer>2</integer>
			<integer>4</integer>
			<integer>5</integer>
			<integer>6</integer>
			<integer>7</integer>
			<integer>9</integer>
			<integer>29</integer>
			<integer>30</integer>
			<integer>32</integer>
			<integer>57</integer>
		</array>
		<key>openTypeOS2VendorID</key>
		<string>ADBO</string>
		<key>openTypeOS2WinAscent</key>
		<integer>984</integer>
		<key>openTypeOS2WinDescent</key>
		<integer>273</integer>
		<key>postscriptBlueFuzz</key>
		<integer>0</integer>
		<key>postscriptBlueScale</key>
		<real>0.0625</real>
		<key>postscriptBlueValues</key>
		<array>
			<integer>-12</integer>
			<integer>0</integer>
			<integer>486</integer>
			<integer>498</integer>
			<integer>518</integer>
			<integer>530</integer>
			<integer>574</integer>
			<integer>586</integer>
			<integer>638</integer>
			<integer>650</integer>
			<integer>656</integer>
			<integer>668</integer>
			<integer>712</integer>
			<integer>724</integer>
		</array>
		<key>postscriptFamilyBlues</key>
		<array>
			<integer>-12</integer>
			<integer>0</integer>
			<integer>486</integer>
			<integer>498</integer>
			<integer>518</integer>
			<integer>530</integer>
			<integer>574</integer>
			<integer>586</integer>
			<integer>638</integer>
			<integer>650</integer>
			<integer>656</integer>
			<integer>668</integer>
			<integer>712</integer>
			<integer>724</integer>
		</array>
		<key>postscriptFamilyOtherBlues</key>
		<array>
			<integer>-217</integer>
			<integer>-205</integer>
		</array>
		<key>postscriptFontName</key>
		<string>SourceSans3-Regular</string>
		<key>postscriptOtherBlues</key>
		<array>
			<integer>-217</integer>
			<integer>-205</integer>
		</array>
		<key>postscriptStemSnapH</key>
		<array>
			<integer>67</integer>
			<integer>78</integer>
		</array>
		<key>postscriptStemSnapV</key>
		<array>
			<integer>84</integer>
			<integer>95</integer>
		</array>
		<key>postscriptUnderlinePosition</key>
		<integer>-75</integer>
		<key>postscriptUnderlineThickness</key>
		<integer>50</integer>
		<key>styleName</key>
		<string>Regular</string>
		<key>trademark</key>
		<string>Source is a trademark of Adobe Systems Incorporated in the United States and/or other countries.</string>
		<key>unitsPerEm</key>
		<integer>1000</integer>
		<key>versionMajor</key>
		<integer>3</integer>
		<key>versionMinor</key>
		<integer>38</integer>
		<key>xHeight</key>
		<integer>486</integer>
	</dict>
</plist>
"##;

/// A simple random identifier generator.
///
/// Follows the logic of the example in <https://unifiedfontobject.org/versions/ufo3/conventions/>
/// but does not check if exists in a given set.
pub fn make_random_identifier() -> String {
    use rand::seq::IteratorRandom;
    const CHARACTERS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    let mut rng = &mut rand::thread_rng();

    CHARACTERS
        .chars()
        .choose_multiple(&mut rng, 10)
        .into_iter()
        .collect()
}

#[test]
fn test_make_random_identifier() {
    assert_eq!(make_random_identifier().len(), 10);
}
