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
use ufo::objects::Layer;

#[derive(Debug, Default)]
pub struct GlyphMetadataInner {
    modified: Cell<bool>,
    pub mark_color: Cell<Color>,
    layer: RefCell<Option<Layer>>,
    pub image: RefCell<Option<ImageRef>>,
    pub advance: Cell<Option<Advance>>,
    pub unicode: RefCell<Vec<Unicode>>,
    pub anchors: RefCell<Vec<Anchor>>,
    pub width: Cell<Option<f64>>,
    pub name: RefCell<String>,
    pub kinds: RefCell<(GlyphKind, Vec<GlyphKind>)>,
    pub filename: RefCell<String>,
    pub glif_source: RefCell<String>,
    pub glyph_ref: OnceCell<Rc<RefCell<Glyph>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphMetadataInner {
    const NAME: &'static str = "Glyph";
    type Type = GlyphMetadata;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for GlyphMetadataInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.mark_color.set(Color::TRANSPARENT);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        GlyphMetadata::NAME,
                        GlyphMetadata::NAME,
                        "Glyph name.",
                        None,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        GlyphMetadata::MODIFIED,
                        GlyphMetadata::MODIFIED,
                        GlyphMetadata::MODIFIED,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoxed::new(
                        GlyphMetadata::MARK_COLOR,
                        GlyphMetadata::MARK_COLOR,
                        GlyphMetadata::MARK_COLOR,
                        Color::static_type(),
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecString::new(
                        GlyphMetadata::RELATIVE_PATH,
                        GlyphMetadata::RELATIVE_PATH,
                        "Filesystem path.",
                        None,
                        glib::ParamFlags::READABLE | UI_READABLE | UI_PATH,
                    ),
                    glib::ParamSpecString::new(
                        GlyphMetadata::FILENAME,
                        GlyphMetadata::FILENAME,
                        "Filename.",
                        None,
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    def_param!(f64 GlyphMetadata::WIDTH, f64::MIN, 0.0),
                    glib::ParamSpecObject::new(
                        GlyphMetadata::LAYER,
                        GlyphMetadata::LAYER,
                        GlyphMetadata::LAYER,
                        Layer::static_type(),
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            GlyphMetadata::NAME => Some(self.name.borrow().to_string()).to_value(),
            GlyphMetadata::MARK_COLOR => self.mark_color.get().to_value(),
            GlyphMetadata::MODIFIED => self.modified.get().to_value(),
            GlyphMetadata::RELATIVE_PATH => {
                if let Some(layer) = self.layer.borrow().as_ref() {
                    Some(format!(
                        "{}/{}",
                        layer.property::<String>(Layer::DIR_NAME),
                        self.filename.borrow()
                    ))
                    .to_value()
                } else {
                    Some(format!("glyphs/{}", self.filename.borrow())).to_value()
                }
            }
            GlyphMetadata::FILENAME => Some(self.filename.borrow().to_string()).to_value(),
            GlyphMetadata::WIDTH => self.width.get().unwrap_or_default().to_value(),
            GlyphMetadata::LAYER => self.layer.borrow().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            GlyphMetadata::NAME => {
                if let Ok(Some(name)) = value.get::<Option<String>>() {
                    *self.name.borrow_mut() = name;
                } else {
                    *self.name.borrow_mut() = String::new();
                }
            }
            GlyphMetadata::MARK_COLOR => {
                self.mark_color.set(value.get().unwrap());
            }
            GlyphMetadata::MODIFIED => {
                self.modified.set(value.get().unwrap());
            }
            GlyphMetadata::FILENAME => {
                if let Ok(Some(filename)) = value.get::<Option<String>>() {
                    *self.filename.borrow_mut() = filename;
                } else {
                    *self.filename.borrow_mut() = String::new();
                }
                self.instance().notify(GlyphMetadata::RELATIVE_PATH);
            }
            GlyphMetadata::LAYER => {
                if let Some(layer) = value
                    .get::<Option<Layer>>()
                    .ok()
                    .flatten()
                    .or_else(|| value.get::<Layer>().ok())
                {
                    *self.layer.borrow_mut() = Some(layer);
                    self.instance().notify(GlyphMetadata::RELATIVE_PATH);
                }
            }
            GlyphMetadata::WIDTH => {
                let val: f64 = value.get().unwrap();
                if val.is_normal() {
                    self.width.set(if val < 0.0 { None } else { Some(val) })
                }
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct GlyphMetadata(ObjectSubclass<GlyphMetadataInner>);
}

impl std::ops::Deref for GlyphMetadata {
    type Target = GlyphMetadataInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl GlyphMetadata {
    pub const MODIFIED: &'static str = "modified";
    pub const MARK_COLOR: &'static str = "mark-color";
    pub const RELATIVE_PATH: &'static str = "relative-path";
    pub const FILENAME: &'static str = "filename";
    pub const NAME: &'static str = "name";
    pub const LAYER: &'static str = "layer";
    pub const WIDTH: &'static str = "width";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn name(&self) -> FieldRef<'_, String> {
        self.name.borrow().into()
    }

    pub fn filename(&self) -> FieldRef<'_, String> {
        self.filename.borrow().into()
    }

    pub fn kinds(&self) -> FieldRef<'_, (GlyphKind, Vec<GlyphKind>)> {
        self.kinds.borrow().into()
    }

    pub fn unicode(&self) -> FieldRef<'_, Vec<Unicode>> {
        self.unicode.borrow().into()
    }

    pub fn width(&self) -> Option<f64> {
        self.width.get()
    }

    #[inline(always)]
    pub fn modified(&self) -> bool {
        self.imp().modified.get()
    }

    pub fn layer(&self) -> FieldRef<'_, Option<Layer>> {
        self.layer.borrow().into()
    }
}

impl Default for GlyphMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl_modified!(GlyphMetadata);

impl From<GlyphMetadata> for Glyph {
    fn from(metadata: GlyphMetadata) -> Self {
        Self {
            metadata,
            ..Self::default()
        }
    }
}

impl_friendly_name!(GlyphMetadata, "Glyph Metadata");

impl CreatePropertyWindow for GlyphMetadata {
    fn new_property_window(&self, app: &Application, create: bool) -> PropertyWindow {
        if create {
            self.set_property(Self::FILENAME, "glyph_name.glif");
        }
        let w = PropertyWindow::builder(self.clone().upcast(), app)
            .title(if create {
                "Add glyph".into()
            } else {
                "Edit glyph".into()
            })
            .type_(if create {
                PropertyWindowType::Create
            } else {
                PropertyWindowType::Modify
            })
            .build();
        {
            {
                let widgets = w.widgets();
                let filename = &widgets[Self::FILENAME];
                let name = &widgets[Self::NAME];
                filename.set_sensitive(false);
                filename.style_read_only(false);
                filename.style_monospace();
                name.bind_property("text", self, Self::FILENAME)
                    .transform_to(|_, val| {
                        let Some(n) = val.get::<Option<String>>().ok()?.filter(|n| !n.is_empty())
                        else {
                            return Some("glyph_name.glif".to_value());
                        };
                        Some(format!("{n}.glif").to_value())
                    })
                    .build();
            }
            let unicode_label = gtk::Label::builder().label(&{
                    let blurb = "Unicode codepoint e.g. U+67";
                    let name = "unicode";
                    let type_name: &str = "unicode codepoint";
                    format!("<span insert_hyphens=\"true\" allow_breaks=\"true\">{blurb}</span>\n\nKey: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
                }).visible(true)
                .selectable(true)
                .wrap_mode(gtk::pango::WrapMode::Char)
                .use_markup(true)
                .max_width_chars(30)
                .halign(gtk::Align::Start)
                .wrap(true)
                .build();
            let unicode_entry = gtk::Entry::builder()
                .editable(true)
                .visible(true)
                .placeholder_text("^[uU]\\+[[:xdigit:]]{1,6}$")
                .max_length(8)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Center)
                .build();

            // U+0 to U+10FFFF
            let uni_hex = regex::Regex::new("^[uU]\\+[[:xdigit:]]{1,6}$").unwrap();
            unicode_entry.connect_insert_text(move |entry, _new_text, _pos| {
                let text = entry.text();
                let text = text.trim();
                if uni_hex.is_match(text) {
                    entry.style_context().remove_class("invalid");
                } else {
                    entry.style_context().add_class("invalid");
                }
            });
            let uni_hex = regex::Regex::new("^[uU]\\+[[:xdigit:]]{1,6}$").unwrap();
            unicode_entry.connect_delete_text(move |entry, _start_pos, _end_pos| {
                let text = entry.text();
                let text = text.trim();
                if uni_hex.is_match(text) {
                    entry.style_context().remove_class("invalid");
                } else {
                    entry.style_context().add_class("invalid");
                }
            });
            let uni_hex = regex::Regex::new("^[uU]\\+[[:xdigit:]]{1,6}$").unwrap();
            unicode_entry.connect_changed(move |entry| {
                let text = entry.text();
                let text = text.trim();
                if uni_hex.is_match(text) {
                    entry.style_context().remove_class("invalid");
                } else {
                    entry.style_context().add_class("invalid");
                }
            });
            unicode_entry.buffer().connect_notify_local(
                Some("text"),
                clone!(@strong self as obj => move |entry, _| {
                    let mut unicodes = obj.imp().unicode.borrow_mut();
                    let mut kinds = obj.imp().kinds.borrow_mut();
                    let text = entry.text();
                    if let Some(t) = text.strip_prefix("u+").or_else(|| text.strip_prefix("U+")) {
                        let val = Unicode::new(t.to_string());
                        // [ref:TODO] show error to user
                        if let Ok(kind) = GlyphKind::try_from(&val) {
                            kinds.0 = kind;
                            unicodes.clear();
                            unicodes.push(val);
                        }
                    }
                }),
            );
            let codepoint = PropertyChoice::new(
                "codepoint",
                gtk::RadioButton::new(),
                unicode_entry.clone().upcast(),
            );
            let name_entry = gtk::Entry::builder()
                .visible(true)
                .expand(false)
                .placeholder_text("component name")
                .build();
            name_entry.buffer().connect_notify_local(
                Some("text"),
                clone!(@strong self as obj => move |entry, _| {
                    let mut unicodes = obj.imp().unicode.borrow_mut();
                    unicodes.clear();
                    let mut kinds = obj.imp().kinds.borrow_mut();
                    let text = entry.text();
                    kinds.0 = GlyphKind::Component(text);
                }),
            );

            let component = PropertyChoice::new(
                "component",
                gtk::RadioButton::from_widget(codepoint.button()),
                name_entry.clone().upcast(),
            );
            let kind_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(15)
                .expand(true)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .visible(true)
                .can_focus(true)
                .build();
            kind_box.pack_start(&codepoint, false, false, 0);
            kind_box.pack_start(&component, false, false, 0);
            kind_box.show_all();
            let kind = self.kinds.borrow().0.clone();
            match kind {
                GlyphKind::Component(ref n) => {
                    codepoint.button().set_active(false);
                    component.button().set_active(true);
                    name_entry.buffer().set_text(n);
                }
                GlyphKind::Char(c) => {
                    codepoint.button().set_active(true);
                    component.button().set_active(false);
                    unicode_entry
                        .buffer()
                        .set_text(&format!("U+{:X}", c as u32));
                }
            }
            w.add_separator();
            w.add("unicode", unicode_label.upcast(), kind_box.upcast());
        }
        w
    }
}
