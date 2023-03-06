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

pub mod constants;
pub mod glif;
pub mod objects;

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
    /// A list of gasp Range Records. These must be sorted in ascending order based on the `range_max_PPEM` value of the record.
    #[serde(default)]
    pub open_type_gasp_range_records: Option<Vec<OpenTypeGaspRangeRecord>>,
    // OpenType head Table Fields
    /// Creation date. Expressed as a string of the format “YYYY/MM/DD HH:MM:SS”. “YYYY/MM/DD” is
    /// year/month/day. The month must be in the range 1-12 and the day must be in the range
    /// 1-end of month. “HH:MM:SS” is hour:minute:second. The hour must be in the range 0:23. The
    /// minute and second must each be in the range 0-59. The timezone is UTC.
    #[serde(default)]
    pub open_type_head_created: Option<String>,
    /// Smallest readable size in pixels. Corresponds to the OpenType head table lowestRecPPEM field.
    #[serde(default)]
    #[serde(rename = "openTypeHeadLowestRecPPEM")]
    pub open_type_head_lowest_rec_ppem: Option<i64>,
    /// A list of bit numbers indicating the flags. The bit numbers are listed in the OpenType head
    /// specification. Corresponds to the OpenType head table flags field.
    #[serde(default)]
    pub open_type_head_flags: Vec<u32>,
    // OpenType hhea Table Fields
    /// Ascender value. Corresponds to the OpenType hhea table Ascender field.
    #[serde(default)]
    pub open_type_hhea_ascender: Option<f64>,
    /// Descender value. Corresponds to the OpenType hhea table Descender field.
    #[serde(default)]
    pub open_type_hhea_descender: Option<f64>,
    /// Line gap value. Corresponds to the OpenType hhea table LineGap field.
    #[serde(default)]
    pub open_type_hhea_line_gap: Option<f64>,
    /// Caret slope rise value. Corresponds to the OpenType hhea table caretSlopeRise field.
    #[serde(default)]
    pub open_type_hhea_caret_slope_rise: Option<f64>,
    /// Caret slope run value. Corresponds to the OpenType hhea table caretSlopeRun field.
    #[serde(default)]
    pub open_type_hhea_caret_slope_run: Option<f64>,
    /// Caret offset value. Corresponds to the OpenType hhea table caretOffset field.
    #[serde(default)]
    pub open_type_hhea_caret_offset: Option<f64>,
    // OpenType Name Table Fields
    /// Designer name. Corresponds to the OpenType name table name ID 9.
    #[serde(default)]
    pub open_type_name_designer: Option<String>,
    /// URL for the designer. Corresponds to the OpenType name table name ID 12.
    #[serde(default)]
    #[serde(rename = "openTypeNameDesignerURL")]
    pub open_type_name_designer_url: Option<String>,
    /// Manufacturer name. Corresponds to the OpenType name table name ID 8.
    #[serde(default)]
    pub open_type_name_manufacturer: Option<String>,
    /// Manufacturer URL. Corresponds to the OpenType name table name ID 11.
    #[serde(default)]
    #[serde(rename = "openTypeNameManufacturerURL")]
    pub open_type_name_manufacturer_url: Option<String>,
    /// License text. Corresponds to the OpenType name table name ID 13.
    #[serde(default)]
    pub open_type_name_license: Option<String>,
    /// URL for the license. Corresponds to the OpenType name table name ID 14.
    #[serde(default)]
    #[serde(rename = "openTypeNameLicenseURL")]
    pub open_type_name_license_url: Option<String>,
    /// Version string. Corresponds to the OpenType name table name ID 5.
    #[serde(default)]
    pub open_type_name_version: Option<String>,
    /// Unique ID string. Corresponds to the OpenType name table name ID 3.
    #[serde(default)]
    #[serde(rename = "openTypeNameUniqueID")]
    pub open_type_name_unique_id: Option<String>,
    /// Description of the font. Corresponds to the OpenType name table name ID 10.
    #[serde(default)]
    pub open_type_name_description: Option<String>,
    /// Preferred family name. Corresponds to the OpenType name table name ID 16.
    #[serde(default)]
    pub open_type_name_preferred_family_name: Option<String>,
    /// Preferred subfamily name. Corresponds to the OpenType name table name ID 17.
    #[serde(default)]
    pub open_type_name_preferred_subfamily_name: Option<String>,
    /// Compatible full name. Corresponds to the OpenType name table name ID 18.
    #[serde(default)]
    pub open_type_name_compatible_full_name: Option<String>,
    /// Sample text. Corresponds to the OpenType name table name ID 19.
    #[serde(default)]
    pub open_type_name_sample_text: Option<String>,
    /// WWS family name. Corresponds to the OpenType name table name ID 21.
    #[serde(default, rename = "openTypeNameWWSFamilyName")]
    pub open_type_name_wws_family_name: Option<String>,
    /// WWS Subfamily name. Corresponds to the OpenType name table name ID 22.
    #[serde(default, rename = "openTypeNameWWSSubFamilyName")]
    pub open_type_name_wws_sub_family_name: Option<String>,
    /// A list of name records. This name record storage area is intended for records that require
    /// platform, encoding and or language localization.
    #[serde(default)]
    pub open_type_name_records: Option<Vec<NameRecord>>,

    // PostScript Specific Data
    /// Name to be used for the FontName field in Type 1/CFF table.
    #[serde(default)]
    pub postscript_font_name: Option<String>,
    /// Name to be used for the FullName field in Type 1/CFF table.
    #[serde(default)]
    pub postscript_full_name: Option<String>,
    /// Artificial slant angle. This must be an angle in counter-clockwise degrees from the
    /// vertical. This value is not the same as the italic angle. Font authoring tools may use this
    /// value to set the FontMatrix in Type 1/CFF table.
    #[serde(default)]
    pub postscript_slant_angle: Option<f64>,
    /// A unique ID number as defined in the Type 1/CFF specification.
    #[serde(default, rename = "postscriptUniqueID")]
    pub postscript_unique_id: Option<i64>,
    /// Underline thickness value. Corresponds to the Type 1/CFF/post table UnderlineThickness field.
    #[serde(default)]
    pub postscript_underline_thickness: Option<f64>,
    /// Underline position value. Corresponds to the Type 1/CFF/post table UnderlinePosition field.
    #[serde(default)]
    pub postscript_underline_position: Option<f64>,
    /// Indicates if the font is monospaced. An authoring tool could calculate this automatically,
    /// but the designer may wish to override this setting. This corresponds to the Type 1/CFF
    /// isFixedPitched field
    #[serde(default)]
    pub postscript_is_fixed_pitch: Option<bool>,
    /// A list of up to 14 integers or floats specifying the values that should be in the Type
    /// 1/CFF BlueValues field. This list must contain an even number of integers following the
    /// rules defined in the Type 1/CFF specification.
    #[serde(default)]
    pub postscript_blue_values: Option<Vec<f64>>,
    /// A list of up to 10 integers or floats specifying the values that should be in the Type
    /// 1/CFF OtherBlues field. This list must contain an even number of integers following the
    /// rules defined in the Type 1/CFF specification.
    #[serde(default)]
    pub postscript_other_blues: Option<Vec<f64>>,
    /// A list of up to 14 integers or floats specifying the values that should be in the Type
    /// 1/CFF FamilyBlues field. This list must contain an even number of integers following the
    /// rules defined in the Type 1/CFF specification.
    #[serde(default)]
    pub postscript_family_blues: Option<Vec<f64>>,
    /// A list of up to 10 integers or floats specifying the values that should be in the Type
    /// 1/CFF FamilyOtherBlues field. This list must contain an even number of integers following
    /// the rules defined in the Type 1/CFF specification.
    #[serde(default)]
    pub postscript_family_other_blues: Option<Vec<f64>>,
    /// List of horizontal stems sorted in the order specified in the Type 1/CFF specification. Up
    /// to 12 integers or floats are possible. This corresponds to the Type 1/CFF StemSnapH field.
    #[serde(default)]
    pub postscript_stem_snap_h: Option<Vec<f64>>,
    /// List of vertical stems sorted in the order specified in the Type 1/CFF specification. Up to
    /// 12 integers or floats are possible. This corresponds to the Type 1/CFF StemSnapV field.
    #[serde(default)]
    pub postscript_stem_snap_v: Option<Vec<f64>>,
    /// BlueFuzz value. This corresponds to the Type 1/CFF BlueFuzz field.
    #[serde(default)]
    pub postscript_blue_fuzz: Option<f64>,
    /// BlueShift value. This corresponds to the Type 1/CFF BlueShift field.
    #[serde(default)]
    pub postscript_blue_shift: Option<f64>,
    /// BlueScale value. This corresponds to the Type 1/CFF BlueScale field.
    #[serde(default)]
    pub postscript_blue_scale: Option<f64>,
    /// Indicates how the Type 1/CFF ForceBold field should be set.
    #[serde(default)]
    pub postscript_force_bold: Option<bool>,
    /// Default width for glyphs.
    #[serde(default)]
    pub postscript_default_width_x: Option<f64>,
    /// Nominal width for glyphs.
    #[serde(default)]
    pub postscript_nominal_width_x: Option<f64>,
    /// A string indicating the overall weight of the font. This corresponds to the Type 1/CFF
    /// Weight field. It should have a reasonable value that reflects the openTypeOS2WeightClass
    /// value.
    #[serde(default)]
    pub postscript_weight_name: Option<String>,
    /// The name of the glyph that should be used as the default character in PFM files.
    #[serde(default)]
    pub postscript_default_character: Option<String>,
    /// The Windows character set. The values are defined below.
    #[serde(default)]
    pub postscript_windows_character_set: Option<PostscriptWindowsCharacterSet>,

    //OpenType OS/2 Table Fields↩
    ///  non-negative integer  Width class value. Must be in the range 1-9. Corresponds to the OpenType OS/2 table usWidthClass field.
    #[serde(default, rename = "openTypeOS2WidthClass")]
    pub open_type_os2_width_class: Option<u64>,
    /// non-negative integer  Weight class value. Corresponds to the OpenType OS/2 table usWeightClass field.
    #[serde(default, rename = "openTypeOS2WeightClass")]
    pub open_type_os2_weight_class: Option<u64>,
    /// list  A list of bit numbers indicating the bits that should be set in fsSelection. The bit numbers are listed in the OpenType OS/2 specification. Corresponds to the OpenType OS/2 table selection field. Note: Bits 0 (italic), 5 (bold) and 6 (regular) must not be set here. These bits should be taken from the generic styleMapStyleName attribute.
    #[serde(default, rename = "openTypeOS2Selection")]
    pub open_type_os2_selection: Option<Vec<u32>>,
    ///  string  Four character identifier for the creator of the font. Corresponds to the OpenType OS/2 table achVendID field.
    #[serde(default, rename = "openTypeOS2VendorID")]
    pub open_type_os2_vendor_id: Option<String>,
    ///  list  The list must contain 10 non-negative integers that represent the setting for each category in the Panose specification. The integers correspond with the option numbers in each of the Panose categories. This corresponds to the OpenType OS/2 table Panose field.
    #[serde(default, rename = "openTypeOS2Panose")]
    pub open_type_os2_panose: Option<Vec<u64>>,
    /// list  Two integers representing the IBM font class and font subclass of the font. The first number, representing the class ID, must be in the range 0-14. The second number, representing the subclass, must be in the range 0-15. The numbers are listed in the OpenType OS/2 specification. Corresponds to the OpenType OS/2 table sFamilyClass field.
    #[serde(default, rename = "openTypeOS2FamilyClass")]
    pub open_type_os2_family_class: Option<Vec<u64>>,
    /// list  A list of bit numbers that are supported Unicode ranges in the font. The bit numbers are listed in the OpenType OS/2 specification. Corresponds to the OpenType OS/2 table ulUnicodeRange1, ulUnicodeRange2, ulUnicodeRange3 and ulUnicodeRange4 fields.
    #[serde(default, rename = "openTypeOS2UnicodeRanges")]
    pub open_type_os2_unicode_ranges: Option<Vec<u32>>,
    ///  list  A list of bit numbers that are supported code page ranges in the font. The bit numbers are listed in the OpenType OS/2 specification. Corresponds to the OpenType OS/2 table ulCodePageRange1 and ulCodePageRange2 fields.
    #[serde(default, rename = "openTypeOS2CodePageRanges")]
    pub open_type_os2_code_page_ranges: Option<Vec<u32>>,
    // Ascender value. Corresponds to the OpenType OS/2 table sTypoAscender field.
    #[serde(default, rename = "openTypeOS2TypoAscender")]
    pub open_type_os2_typo_ascender: Option<i64>,
    /// integer   Descender value. Corresponds to the OpenType OS/2 table sTypoDescender field.
    #[serde(default, rename = "openTypeOS2TypoDescender")]
    pub open_type_os2_typo_descender: Option<i64>,
    /// integer   Line gap value. Corresponds to the OpenType OS/2 table sTypoLineGap field.
    #[serde(default, rename = "openTypeOS2TypoLineGap")]
    pub open_type_os2_typo_line_gap: Option<i64>,
    /// non-negative integer  Ascender value. Corresponds to the OpenType OS/2 table usWinAscent field.
    #[serde(default, rename = "openTypeOS2WinAscent")]
    pub open_type_os2_win_ascent: Option<u64>,
    ///  non-negative integer  Descender value. Corresponds to the OpenType OS/2 table usWinDescent field.
    #[serde(default, rename = "openTypeOS2WinDescent")]
    pub open_type_os2_win_descent: Option<u64>,
    ///  list  A list of bit numbers indicating the embedding type. The bit numbers are listed in the OpenType OS/2 specification. Corresponds to the OpenType OS/2 table fsType field.
    #[serde(default, rename = "openTypeOS2Type")]
    pub open_type_os2_type: Option<Vec<u32>>,
    ///  integer   Subscript horizontal font size. Corresponds to the OpenType OS/2 table ySubscriptXSize field.
    #[serde(default, rename = "openTypeOS2SubscriptXSize")]
    pub open_type_os2_subscript_xsize: Option<i64>,
    ///  integer   Subscript vertical font size. Corresponds to the OpenType OS/2 table ySubscriptYSize field.
    #[serde(default, rename = "openTypeOS2SubscriptYSize")]
    pub open_type_os2_subscript_ysize: Option<i64>,
    ///  integer   Subscript x offset. Corresponds to the OpenType OS/2 table ySubscriptXOffset field.
    #[serde(default, rename = "openTypeOS2SubscriptXOffset")]
    pub open_type_os2_subscript_xoffset: Option<i64>,
    ///  integer   Subscript y offset. Corresponds to the OpenType OS/2 table ySubscriptYOffset field.
    #[serde(default, rename = "openTypeOS2SubscriptYOffset")]
    pub open_type_os2_subscript_yoffset: Option<i64>,
    ///  integer   Superscript horizontal font size. Corresponds to the OpenType OS/2 table ySuperscriptXSize field.
    #[serde(default, rename = "openTypeOS2SuperscriptXSize")]
    pub open_type_os2_superscript_xsize: Option<i64>,
    ///  integer   Superscript vertical font size. Corresponds to the OpenType OS/2 table ySuperscriptYSize field.
    #[serde(default, rename = "openTypeOS2SuperscriptYSize")]
    pub open_type_os2_superscript_ysize: Option<i64>,
    ///  integer   Superscript x offset. Corresponds to the OpenType OS/2 table ySuperscriptXOffset field.
    #[serde(default, rename = "openTypeOS2SuperscriptXOffset")]
    pub open_type_os2_superscript_xoffset: Option<i64>,
    ///  integer   Superscript y offset. Corresponds to the OpenType OS/2 table ySuperscriptYOffset field.
    #[serde(default, rename = "openTypeOS2SuperscriptYOffset")]
    pub open_type_os2_superscript_yoffset: Option<i64>,
    /// integer   Strikeout size. Corresponds to the OpenType OS/2 table yStrikeoutSize field.
    #[serde(default, rename = "openTypeOS2StrikeoutSize")]
    pub open_type_os2_strikeout_size: Option<i64>,
    /// integer   Strikeout position. Corresponds to the OpenType OS/2 table yStrikeoutPosition field.
    #[serde(default, rename = "openTypeOS2StrikeoutPosition")]
    pub open_type_os2_strikeout_position: Option<i64>,
}

/// gasp Range Record Format
///
/// UFO3 Spec:
///
/// > This file contains information about the font itself, such as naming and dimensions.
/// > This file is optional. Not all values are required for a proper file.
#[derive(Default, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTypeGaspRangeRecord {
    #[serde(rename = "rangeMaxPPEM")]
    range_max_ppem: u8,
    #[serde(default)]
    range_gasp_behavior: Vec<RangeGaspBehavior>,
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RangeGaspBehavior {
    /// 0x0001
    GASP_GRIDFIT = 0,
    /// 0x0002
    GASP_DOGRAY = 1,
    /// 0x0004
    GASP_SYMMETRIC_GRIDFIT = 2,
    /// 0x0008
    GASP_SYMMETRIC_SMOOTHING = 3,
}

/// Name Record Format
#[derive(Default, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameRecord {
    /// The name ID.
    #[serde(default)]
    name_id: Option<u64>,
    /// The platform ID.
    #[serde(default)]
    platform_id: Option<u64>,
    /// The encoding ID.
    #[serde(default)]
    encoding_id: Option<u64>,
    /// The language ID.
    #[serde(default)]
    language_id: Option<u64>,
    /// The string value for the record.
    #[serde(default)]
    string: Option<String>,
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostscriptWindowsCharacterSet {
    ANSI = 1,
    Default = 2,
    Symbol = 3,
    Macintosh = 4,
    ShiftJIS = 5,
    Hangul = 6,
    HangulJohab = 7,
    GB2312 = 8,
    ChineseBIG5 = 9,
    Greek = 10,
    Turkish = 11,
    Vietnamese = 12,
    Hebrew = 13,
    Arabic = 14,
    Baltic = 15,
    Bitstream = 16,
    Cyrillic = 17,
    Thai = 18,
    EuropeanEastern = 19,
    OEM = 20,
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
            .truncate(true)
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
    assert_eq!(p.open_type_name_license.as_deref(), Some("This Font Software is licensed under the SIL Open Font License, Version 1.1. This license is available with a FAQ at: http://scripts.sil.org/OFL. This Font Software is distributed on an 'AS IS' BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the SIL Open Font License for the specific language, permissions and limitations governing your use of this Font Software."));
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
