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

use glib::{ParamSpec, Value};
use gtk::glib;
use gtk::subclass::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::glyphs::{Glyph, Guideline};

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct Project {
        pub name: RefCell<String>,
        pub modified: RefCell<bool>,
        pub last_saved: RefCell<Option<u64>>,
        pub glyphs: RefCell<HashMap<String, Rc<RefCell<Glyph>>>>,
        pub path: RefCell<Option<PathBuf>>,
        pub family_name: RefCell<String>,
        pub style_name: RefCell<String>,
        pub version_major: RefCell<i64>,
        pub version_minor: RefCell<u64>,
        /// Copyright statement.
        pub copyright: RefCell<String>,
        /// Trademark statement.
        pub trademark: RefCell<String>,
        /// Units per em.
        pub units_per_em: RefCell<f64>,
        /// Descender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
        pub descender: RefCell<f64>,
        /// x-height value.
        pub x_height: RefCell<f64>,
        /// Cap height value.
        pub cap_height: RefCell<f64>,
        /// Ascender value. Note: The specification is agnostic about the relationship to the more specific vertical metric values.
        pub ascender: RefCell<f64>,
        /// Italic angle. This must be an angle in counter-clockwise degrees from the vertical.
        pub italic_angle: RefCell<f64>,
        /// Arbitrary note about the font.
        pub note: RefCell<String>,
        /// A list of guideline definitions that apply to all glyphs in all layers in the font. This attribute is optional.
        pub guidelines: RefCell<Vec<Guideline>>,
    }

    impl Default for Project {
        fn default() -> Self {
            Project {
                name: RefCell::new("New project".to_string()),
                modified: RefCell::new(false),
                last_saved: RefCell::new(None),
                glyphs: RefCell::new(HashMap::default()),
                path: RefCell::new(None),
                family_name: RefCell::new("New project".to_string()),
                style_name: RefCell::new(String::new()),
                version_major: RefCell::new(0),
                version_minor: RefCell::new(0),
                copyright: RefCell::new(String::new()),
                trademark: RefCell::new(String::new()),
                units_per_em: RefCell::new(1000.0),
                descender: RefCell::new(-200.),
                x_height: RefCell::new(450.),
                cap_height: RefCell::new(650.),
                ascender: RefCell::new(700.),
                italic_angle: RefCell::new(0.),
                note: RefCell::new(String::new()),
                guidelines: RefCell::new(vec![]),
            }
        }
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Project {
        const NAME: &'static str = "Project";
        type Type = super::Project;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    // Trait shared by all GObjects
    impl ObjectImpl for Project {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
                once_cell::sync::Lazy::new(|| vec![]);
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Project(ObjectSubclass<imp::Project>);
}

impl Project {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn from_path(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let glyphs = Glyph::from_ufo(path);
        let mut path: PathBuf = Path::new(path).into();
        if !path.exists() {
            return Err(format!("Directory <i>{}</i> does not exist.", path.display()).into());
        }
        if !path.is_dir() {
            return Err(format!("Path {} is not a directory.", path.display()).into());
        }
        path.push("fontinfo.plist");
        let mut file = match File::open(&path) {
            Err(err) => return Err(format!("couldn't open {}: {}", path.display(), err).into()),
            Ok(file) => file,
        };

        let mut s = String::new();
        if let Err(err) = file.read_to_string(&mut s) {
            return Err(format!("couldn't read {}: {}", path.display(), err).into());
        }
        let mut plist = fontinfo::Plist::from_str(&s)
            .map_err(|err| format!("couldn't read fontinfo.plist {}: {}", path.display(), err))?;
        let family_name =
            if let Some(fontinfo::DictValue::String(s)) = plist.dict.remove("familyName") {
                s
            } else {
                String::new()
            };
        let style_name =
            if let Some(fontinfo::DictValue::String(s)) = plist.dict.remove("styleName") {
                s
            } else {
                String::new()
            };
        let copyright = if let Some(fontinfo::DictValue::String(s)) = plist.dict.remove("copyright")
        {
            s
        } else {
            String::new()
        };
        let trademark = if let Some(fontinfo::DictValue::String(s)) = plist.dict.remove("trademark")
        {
            s
        } else {
            String::new()
        };
        let units_per_em =
            if let Some(fontinfo::DictValue::Integer(u)) = plist.dict.remove("unitsPerEm") {
                u as f64
            } else {
                1000.0
            };
        let x_height = if let Some(fontinfo::DictValue::Integer(x)) = plist.dict.remove("xHeight") {
            x as f64
        } else {
            500.0
        };
        let ascender = if let Some(fontinfo::DictValue::Integer(a)) = plist.dict.remove("ascender")
        {
            a as f64
        } else {
            700.0
        };
        let descender =
            if let Some(fontinfo::DictValue::Integer(d)) = plist.dict.remove("descender") {
                d as f64
            } else {
                -200.0
            };
        let cap_height =
            if let Some(fontinfo::DictValue::Integer(c)) = plist.dict.remove("capHeight") {
                c as f64
            } else {
                600.0
            };
        let italic_angle =
            if let Some(fontinfo::DictValue::Integer(i)) = plist.dict.remove("italicAngle") {
                i as f64
            } else {
                0.0
            };
        let version_major =
            if let Some(fontinfo::DictValue::Integer(i)) = plist.dict.remove("versionMajor") {
                i
            } else {
                0
            };
        let version_minor =
            if let Some(fontinfo::DictValue::Integer(i)) = plist.dict.remove("versionMinor") {
                i as u64
            } else {
                0
            };
        let ret: Self = Self::new();
        *ret.imp().name.borrow_mut() = family_name.clone();
        *ret.imp().modified.borrow_mut() = false;
        *ret.imp().last_saved.borrow_mut() = None;
        *ret.imp().glyphs.borrow_mut() = glyphs?;
        *ret.imp().path.borrow_mut() = Some(path);
        *ret.imp().family_name.borrow_mut() = family_name;
        *ret.imp().style_name.borrow_mut() = style_name;
        *ret.imp().version_major.borrow_mut() = version_major;
        *ret.imp().version_minor.borrow_mut() = version_minor;
        *ret.imp().copyright.borrow_mut() = copyright;
        *ret.imp().trademark.borrow_mut() = trademark;
        *ret.imp().units_per_em.borrow_mut() = units_per_em;
        *ret.imp().descender.borrow_mut() = descender;
        *ret.imp().x_height.borrow_mut() = x_height;
        *ret.imp().cap_height.borrow_mut() = cap_height;
        *ret.imp().ascender.borrow_mut() = ascender;
        *ret.imp().italic_angle.borrow_mut() = italic_angle;
        *ret.imp().note.borrow_mut() = String::new();
        *ret.imp().guidelines.borrow_mut() = vec![];
        Ok(ret)
    }
}

impl Default for Project {
    fn default() -> Self {
        let ret: Self = Self::new();
        *ret.imp().name.borrow_mut() = "New project".to_string();
        *ret.imp().modified.borrow_mut() = false;
        *ret.imp().last_saved.borrow_mut() = None;
        *ret.imp().glyphs.borrow_mut() = HashMap::default();
        *ret.imp().path.borrow_mut() = None;
        *ret.imp().family_name.borrow_mut() = "New project".to_string();
        *ret.imp().style_name.borrow_mut() = String::new();
        *ret.imp().version_major.borrow_mut() = 0;
        *ret.imp().version_minor.borrow_mut() = 0;
        *ret.imp().copyright.borrow_mut() = String::new();
        *ret.imp().trademark.borrow_mut() = String::new();
        *ret.imp().units_per_em.borrow_mut() = 1000.0;
        *ret.imp().descender.borrow_mut() = -200.;
        *ret.imp().x_height.borrow_mut() = 450.;
        *ret.imp().cap_height.borrow_mut() = 650.;
        *ret.imp().ascender.borrow_mut() = 700.;
        *ret.imp().italic_angle.borrow_mut() = 0.;
        *ret.imp().note.borrow_mut() = String::new();
        *ret.imp().guidelines.borrow_mut() = vec![];
        ret
    }
}

mod fontinfo {
    use std::collections::HashMap;

    extern crate quick_xml;
    extern crate serde;

    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub struct Plist {
        pub dict: HashMap<String, DictValue>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum DictValue {
        Integer(i64),
        String(String),
        Array(Vec<DictValue>),
        Real(f64),
    }

    impl Plist {
        pub fn from_str(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
            use quick_xml::events::Event;
            use quick_xml::Reader;

            let mut ret = Self {
                dict: HashMap::default(),
            };

            let mut reader = Reader::from_str(xml);
            reader.trim_text(true);

            let mut buf = Vec::new();

            #[derive(Debug)]
            enum ArrayType {
                Integer,
                Real,
                String,
            }

            #[derive(Debug)]
            enum State {
                Start,
                InDict,
                InKey,
                Key(String),
                String(String),
                Integer(String),
                Real(String),
                Array(String, Vec<DictValue>, ArrayType),
            }
            let mut state = State::Start;
            loop {
                match (&mut state, reader.read_event(&mut buf)) {
                    (State::Start, Ok(Event::Start(ref e))) => match e.name() {
                        b"dict" => {
                            state = State::InDict;
                        }
                        _ => (),
                    },
                    (State::InDict, Ok(Event::Start(ref e))) => match e.name() {
                        b"key" => {
                            state = State::InKey;
                        }
                        _ => (),
                    },
                    (State::Key(_), Ok(Event::End(ref e))) => match e.name() {
                        b"key" => {}
                        _ => (),
                    },
                    (State::Key(keyval), Ok(Event::Start(ref e))) => match e.name() {
                        b"integer" => {
                            let keyval = std::mem::replace(keyval, String::new());
                            state = State::Integer(keyval);
                        }
                        b"array" => {
                            let keyval = std::mem::replace(keyval, String::new());
                            state = State::Array(keyval, vec![], ArrayType::Integer);
                        }
                        b"real" => {
                            let keyval = std::mem::replace(keyval, String::new());
                            state = State::Real(keyval);
                        }
                        b"string" => {
                            let keyval = std::mem::replace(keyval, String::new());
                            state = State::String(keyval);
                        }
                        _ => (),
                    },
                    (State::InDict, Ok(Event::End(ref e))) => match e.name() {
                        b"dict" => {
                            break;
                        }
                        b"string" | b"real" | b"integer" | b"array" => {}
                        _ => (),
                    },
                    (State::Array(keyval, values, _array_type), Ok(Event::End(ref e))) => {
                        match e.name() {
                            b"array" => {
                                let keyval = std::mem::replace(keyval, String::new());
                                let values = std::mem::replace(values, vec![]);
                                ret.dict.insert(keyval, DictValue::Array(values));
                                state = State::InDict;
                            }
                            b"string" | b"real" | b"integer" => {}
                            _ => (),
                        }
                    }
                    (State::Array(_, _, array_type), Ok(Event::Start(ref e))) => match e.name() {
                        b"integer" => {
                            *array_type = ArrayType::Integer;
                        }
                        b"string" => {
                            *array_type = ArrayType::String;
                        }
                        b"real" => {
                            *array_type = ArrayType::Real;
                        }
                        _ => (),
                    },
                    (State::Array(_, values, array_type), Ok(Event::Text(e))) => match array_type {
                        ArrayType::Integer => {
                            values.push(DictValue::Integer(
                                std::str::from_utf8(&e.unescaped()?)?.parse()?,
                            ));
                        }
                        ArrayType::String => {
                            values.push(DictValue::String(e.unescape_and_decode(&mut reader)?));
                        }
                        ArrayType::Real => {
                            values.push(DictValue::Real(
                                std::str::from_utf8(&e.unescaped()?)?.parse()?,
                            ));
                        }
                    },
                    (State::InKey, Ok(Event::Text(e))) => {
                        state = State::Key(e.unescape_and_decode(&mut reader)?);
                    }
                    (State::String(keyval), Ok(Event::Text(e))) => {
                        let keyval = std::mem::replace(keyval, String::new());
                        state = State::InDict;
                        ret.dict.insert(
                            keyval,
                            DictValue::String(e.unescape_and_decode(&mut reader)?),
                        );
                    }
                    (State::Integer(keyval), Ok(Event::Text(e))) => {
                        let keyval = std::mem::replace(keyval, String::new());
                        state = State::InDict;
                        ret.dict.insert(
                            keyval,
                            DictValue::Integer(std::str::from_utf8(&e.unescaped()?)?.parse()?),
                        );
                    }
                    (State::Real(keyval), Ok(Event::Text(e))) => {
                        let keyval = std::mem::replace(keyval, String::new());
                        state = State::InDict;
                        ret.dict.insert(
                            keyval,
                            DictValue::Real(std::str::from_utf8(&e.unescaped()?)?.parse()?),
                        );
                    }
                    (State::InDict, Ok(Event::Eof)) => break, // exits the loop when reaching end of file
                    (_, Err(e)) => {
                        panic!("Error at position {}: {:?}", reader.buffer_position(), e)
                    }
                    _ => (), // There are several other `Event`s we do not consider here
                }

                //buf.clear();
            }
            Ok(ret)
        }
    }

    #[test]
    fn test_plist_parse() {
        //let p: Plist = quick_xml::de::from_str(_PLIST).unwrap();
        //println!("{:#?}", p);
        let p: Plist = Plist::from_str(_PLIST).unwrap();
        println!("{:#?}", p);
    }

    const _PLIST: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
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
}
