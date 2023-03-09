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

use super::widgets;
use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum PropertyWindowButtons {
    Modify {
        reset: gtk::Button,
        close: gtk::Button,
    },
    Create {
        cancel: gtk::Button,
        save: gtk::Button,
    },
}

impl Default for PropertyWindowButtons {
    fn default() -> Self {
        Self::Modify {
            reset: gtk::Button::builder()
                .label("Reset")
                .relief(gtk::ReliefStyle::None)
                .visible(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build(),
            close: gtk::Button::builder()
                .label("Close")
                .relief(gtk::ReliefStyle::None)
                .visible(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build(),
        }
    }
}

#[derive(Default, Debug)]
pub struct PropertyWindowInner {
    pub obj: OnceCell<glib::Object>,
    pub app: OnceCell<crate::prelude::Application>,
    grid: gtk::Grid,
    rows: Cell<i32>,
    pub buttons: OnceCell<PropertyWindowButtons>,
    widgets: RefCell<IndexMap<String, gtk::Widget>>,
    initial_values: RefCell<IndexMap<String, glib::Value>>,
    title: gtk::Label,
}

#[glib::object_subclass]
impl ObjectSubclass for PropertyWindowInner {
    const NAME: &'static str = "PropertyWindow";
    type Type = PropertyWindow;
    type ParentType = gtk::Window;
}

impl ObjectImpl for PropertyWindowInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_deletable(true);
        obj.set_destroy_with_parent(true);
        obj.set_focus_on_map(true);
        obj.set_resizable(true);
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_type_hint(gtk::gdk::WindowTypeHint::Utility);
        obj.set_window_position(gtk::WindowPosition::CenterOnParent);
    }
}

impl WidgetImpl for PropertyWindowInner {}
impl ContainerImpl for PropertyWindowInner {}
impl BinImpl for PropertyWindowInner {}
impl WindowImpl for PropertyWindowInner {}

glib::wrapper! {
    pub struct PropertyWindow(ObjectSubclass<PropertyWindowInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window;
}

#[derive(Default, Copy, Clone, Debug)]
pub enum PropertyWindowType {
    #[default]
    Modify,
    Create,
}

pub struct PropertyWindowBuilder {
    obj: glib::Object,
    app: crate::prelude::Application,
    title: Cow<'static, str>,
    type_: PropertyWindowType,
}

impl PropertyWindow {
    pub fn builder(obj: glib::Object, app: &crate::prelude::Application) -> PropertyWindowBuilder {
        PropertyWindowBuilder::new(obj, app)
    }

    pub fn widgets(&mut self) -> FieldRef<'_, IndexMap<String, gtk::Widget>> {
        self.imp().widgets.borrow().into()
    }

    fn object_to_property_grid(&self, obj: glib::Object, create: bool) {
        self.imp().grid.set_expand(true);
        self.imp().grid.set_visible(true);
        self.imp().grid.set_can_focus(true);
        self.imp().grid.set_column_spacing(5);
        self.imp().grid.set_margin(10);
        self.imp().grid.set_row_spacing(5);
        self.imp().title.set_label(&if create {
            format!("<big>New <i>{}</i></big>", obj.type_().name())
        } else {
            format!("<big>Options for <i>{}</i></big>", obj.type_().name())
        });
        self.imp().title.set_use_markup(true);
        self.imp().title.set_margin_top(5);
        self.imp().title.set_halign(gtk::Align::Start);
        self.imp().title.set_visible(true);
        self.imp().grid.attach(&self.imp().title, 0, 0, 1, 1);
        self.imp().grid.attach(
            &gtk::Separator::builder()
                .expand(true)
                .visible(true)
                .vexpand(false)
                .margin_bottom(10)
                .valign(gtk::Align::Start)
                .build(),
            0,
            1,
            2,
            1,
        );
        self.imp().rows.set(2);
        for prop in obj.list_properties().as_slice().iter().filter(|p| {
            (p.flags()
                .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
                || p.flags().contains(UI_READABLE))
                && p.owner_type() == obj.type_()
        }) {
            self.add_property(&obj, prop, create);
            self.add_separator();
        }
        if self.imp().rows.get() != 2 {
            self.imp().grid.remove_row(self.imp().rows.get() - 1);
        }
    }

    pub fn add(&self, name: &str, label: gtk::Label, widget: gtk::Widget) {
        let row = self.imp().rows.get();
        self.imp().grid.attach(&label, 0, row, 1, 1);
        self.imp().grid.attach(&widget, 1, row, 1, 1);
        self.imp()
            .widgets
            .borrow_mut()
            .insert(name.to_string(), widget);
        self.imp().rows.set(row + 1);
    }

    pub fn add_separator(&self) {
        let row = self.imp().rows.get();
        self.imp().grid.attach(
            &gtk::Separator::builder()
                .expand(true)
                .visible(true)
                .vexpand(false)
                .valign(gtk::Align::Start)
                .build(),
            0,
            row + 1,
            2,
            1,
        );
        self.imp().rows.set(row + 1);
    }

    fn add_property(&self, obj: &glib::Object, property: &glib::ParamSpec, create: bool) {
        let val: glib::Value = obj.property(property.name());
        self.imp()
            .initial_values
            .borrow_mut()
            .insert(property.name().to_string(), val.clone());
        let readwrite = property.flags().contains(glib::ParamFlags::READWRITE)
            && !(property.flags().contains(UI_READABLE));
        let flags = if readwrite {
            glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE
        } else {
            glib::BindingFlags::SYNC_CREATE
        };
        let widget: gtk::Widget = match val.type_().name() {
            "gboolean" => {
                let val = val.get::<bool>().unwrap();
                let entry = widgets::ToggleButton::new();
                entry.set_visible(true);
                entry.set_active(val);
                entry.set_sensitive(readwrite);
                entry.set_halign(gtk::Align::Start);
                entry.set_valign(gtk::Align::Start);
                obj.bind_property(property.name(), &entry, "active")
                    .flags(flags)
                    .build();

                entry.upcast()
            }
            "gchararray" => {
                let val = val.get::<Option<String>>().unwrap().unwrap_or_default();
                if readwrite {
                    let entry = gtk::Entry::builder()
                        .visible(true)
                        .sensitive(readwrite)
                        .halign(gtk::Align::Start)
                        .valign(gtk::Align::Center)
                        .build();
                    entry.buffer().set_text(&val);
                    obj.bind_property(property.name(), &entry.buffer(), "text")
                        .flags(flags)
                        .build();

                    entry.upcast()
                } else {
                    let l = gtk::Label::builder()
                        .label(&val)
                        .expand(false)
                        .visible(true)
                        .selectable(true)
                        .halign(gtk::Align::Start)
                        .valign(gtk::Align::Center)
                        .build();
                    obj.bind_property(property.name(), &l, "label")
                        .flags(flags)
                        .build();
                    if property.flags().contains(UI_PATH) && !create {
                        let b = gtk::Box::builder()
                            .visible(true)
                            .expand(false)
                            .sensitive(true)
                            .halign(gtk::Align::Start)
                            .valign(gtk::Align::Center)
                            .orientation(gtk::Orientation::Horizontal)
                            .build();
                        b.pack_start(&l, true, true, 15);
                        let image = gtk::Image::builder()
                            .icon_name("folder-open")
                            .visible(true)
                            .build();
                        let btn = gtk::Button::builder()
                            .image(&image)
                            .always_show_image(true)
                            .relief(gtk::ReliefStyle::None)
                            .visible(true)
                            .sensitive(true)
                            .tooltip_text("Open file location.")
                            .build();
                        btn.connect_clicked(clone!(@weak obj, @strong property => move |_self| {
                        //FIXME: show error to user, if any.
                        let Some(path) = obj.property::<Option<String>>(property.name()) else { return; };
                        let Ok(prefix) = std::env::current_dir() else { return; };
                        let mut abs_path = prefix.join(&path);
                        if abs_path.is_file() {
                            abs_path.pop();
                        }
                        let Ok(uri) = glib::filename_to_uri(&abs_path, None) else { return ; };
                        gtk::gio::AppInfo::launch_default_for_uri(&uri, gtk::gio::AppLaunchContext::NONE).unwrap();
                    }));
                        b.pack_end(&btn, false, false, 15);
                        b.upcast()
                    } else {
                        l.upcast()
                    }
                }
            }
            "gint64" => {
                let val = val.get::<i64>().unwrap();
                let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecInt64>()
                {
                    (spec.minimum(), spec.maximum())
                } else {
                    (i64::MIN, i64::MAX)
                };
                let entry = gtk::SpinButton::new(
                    Some(&gtk::Adjustment::new(
                        val as f64, min as f64, max as f64, 1.00, 1.00, 1.00,
                    )),
                    1.0,
                    0,
                );
                entry.set_halign(gtk::Align::Start);
                entry.set_valign(gtk::Align::Center);
                entry.set_input_purpose(gtk::InputPurpose::Digits);
                entry.set_sensitive(readwrite);
                entry.set_visible(true);
                obj.bind_property(property.name(), &entry, "value")
                    .transform_to(|_, value| {
                        let val = value.get::<i64>().ok()?;
                        Some((val as f64).to_value())
                    })
                    .transform_from(|_, value| {
                        let val = value.get::<f64>().ok()?;
                        Some((val as i64).to_value())
                    })
                    .flags(flags)
                    .build();
                entry.upcast()
            }
            "guint64" => {
                let val = val.get::<u64>().unwrap();
                let (min, max) =
                    if let Some(spec) = property.downcast_ref::<glib::ParamSpecUInt64>() {
                        (0, spec.maximum())
                    } else {
                        (0, u64::MAX)
                    };
                let entry = gtk::SpinButton::new(
                    Some(&gtk::Adjustment::new(
                        val as f64, min as f64, max as f64, 1.00, 1.00, 1.00,
                    )),
                    1.0,
                    0,
                );
                entry.set_halign(gtk::Align::Start);
                entry.set_valign(gtk::Align::Center);
                entry.set_input_purpose(gtk::InputPurpose::Digits);
                entry.set_sensitive(readwrite);
                entry.set_visible(true);
                obj.bind_property(property.name(), &entry, "value")
                    .transform_to(|_, value| {
                        let val = value.get::<u64>().ok()?;
                        Some((val as f64).to_value())
                    })
                    .transform_from(|_, value| {
                        let val = value.get::<f64>().ok()?;
                        Some((val as u64).to_value())
                    })
                    .flags(flags)
                    .build();
                entry.upcast()
            }
            "gdouble" => {
                let val = val.get::<f64>().unwrap();
                let (min, max) =
                    if let Some(spec) = property.downcast_ref::<glib::ParamSpecDouble>() {
                        (spec.minimum(), spec.maximum())
                    } else {
                        (f64::MIN, f64::MAX)
                    };
                let entry = gtk::SpinButton::new(
                    Some(&gtk::Adjustment::new(val, min, max, 0.05, 0.01, 0.01)),
                    1.0,
                    2,
                );
                entry.set_halign(gtk::Align::Start);
                entry.set_valign(gtk::Align::Center);
                entry.set_input_purpose(gtk::InputPurpose::Number);
                entry.set_sensitive(readwrite);
                entry.set_visible(true);
                obj.bind_property(property.name(), &entry, "value")
                    .flags(flags)
                    .build();
                entry.upcast()
            }
            "Color" => {
                let val = val.get::<Color>().unwrap();
                let entry = gtk::ColorButton::builder()
                    .rgba(&val.into())
                    .sensitive(readwrite)
                    .visible(true)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Center)
                    .use_alpha(true)
                    .show_editor(true)
                    .build();
                entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                    let new_val = self_.rgba();
                    _ = obj.try_set_property::<Color>(property.name(), new_val.into());
                }));
                entry.upcast()
            }
            "DrawOptions" => {
                let opts = val.get::<DrawOptions>().unwrap();
                let grid = gtk::Grid::builder()
                    .expand(true)
                    .visible(true)
                    .sensitive(readwrite)
                    .column_spacing(5)
                    .margin(10)
                    .row_spacing(5)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Start)
                    .build();
                let has_bg = opts.bg.is_some();

                let fg_entry = gtk::ColorButton::builder()
                    .rgba(&opts.color.into())
                    .sensitive(readwrite)
                    .visible(true)
                    .halign(gtk::Align::Fill)
                    .valign(gtk::Align::Start)
                    .use_alpha(true)
                    .show_editor(true)
                    .build();
                fg_entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let opts = obj.property::<DrawOptions>(property.name());
                let new_val = self_.rgba();
                _ = obj.try_set_property::<DrawOptions>(property.name(), DrawOptions { color: new_val.into(), ..opts });
            }));
                grid.attach(
                    &gtk::Label::builder()
                        .label(if has_bg { "fg color" } else { "color" })
                        .halign(gtk::Align::End)
                        .visible(true)
                        .build(),
                    0,
                    0,
                    1,
                    1,
                );
                grid.attach(&fg_entry, 1, 0, 1, 1);
                if let Some(bg) = opts.bg {
                    let bg_entry = gtk::ColorButton::builder()
                        .rgba(&bg.into())
                        .sensitive(readwrite)
                        .visible(true)
                        .halign(gtk::Align::Fill)
                        .valign(gtk::Align::Start)
                        .use_alpha(true)
                        .show_editor(true)
                        .build();
                    bg_entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                    let opts = obj.property::<DrawOptions>(property.name());
                    let new_val = self_.rgba();
                    _ = obj.try_set_property::<DrawOptions>(property.name(), DrawOptions { bg: Some(new_val.into()), ..opts });
                }));
                    grid.attach(
                        &gtk::Label::builder()
                            .label("bg color")
                            .visible(true)
                            .halign(gtk::Align::End)
                            .build(),
                        0,
                        1,
                        1,
                        1,
                    );
                    grid.attach(&bg_entry, 1, 1, 1, 1);
                }
                let listbox = gtk::ListBox::builder()
                    .visible(true)
                    .expand(true)
                    .sensitive(readwrite)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Start)
                    .build();
                let size_entry = gtk::SpinButton::new(
                    Some(&gtk::Adjustment::new(
                        opts.size,
                        0.0,
                        f64::MAX,
                        0.05,
                        0.01,
                        0.01,
                    )),
                    1.0,
                    2,
                );
                size_entry.set_halign(gtk::Align::Start);
                size_entry.set_valign(gtk::Align::Center);
                size_entry.set_input_purpose(gtk::InputPurpose::Number);
                size_entry.set_sensitive(readwrite);
                size_entry.set_visible(true);
                size_entry.connect_value_notify(
                    clone!(@weak obj, @strong property => move |self_| {
                        let opts = obj.property::<DrawOptions>(property.name());
                        let size = self_.value();
                        obj.set_property(property.name(), DrawOptions { size, ..opts });
                    }),
                );
                obj.bind_property(property.name(), &size_entry, "value")
                    .transform_to(|_, value| {
                        let opts = value.get::<DrawOptions>().ok()?;
                        Some(opts.size.to_value())
                    })
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                listbox.add(&size_entry);
                if let Some((from, val)) = opts.inherit_size {
                    if val {
                        size_entry.set_sensitive(false);
                    }
                    let inherit_entry = widgets::ToggleButton::new();
                    inherit_entry.set_label("Inherit global value");
                    inherit_entry.set_visible(true);
                    inherit_entry.set_active(val);
                    inherit_entry.set_relief(gtk::ReliefStyle::None);
                    inherit_entry.set_sensitive(readwrite);
                    inherit_entry.set_halign(gtk::Align::Start);
                    inherit_entry.set_valign(gtk::Align::Center);
                    obj.bind_property(property.name(), &inherit_entry, "active")
                        .transform_to(|_, value| {
                            let opts = value.get::<DrawOptions>().ok()?;
                            opts.inherit_size.map(|(_, b)| b.to_value())
                        })
                        .flags(glib::BindingFlags::SYNC_CREATE)
                        .build();
                    let inherit_value = gtk::Label::builder()
                        .label(&format!("{:.2}", obj.property::<f64>(from)))
                        .visible(val)
                        .width_chars(5)
                        .halign(gtk::Align::Start)
                        .valign(gtk::Align::Center)
                        .sensitive(false)
                        .wrap(true)
                        .build();
                    inherit_entry.connect_clicked(clone!(@weak obj, @strong property, @weak inherit_value, @weak size_entry => move |_| {
                    let opts = obj.property::<DrawOptions>(property.name());
                    if let Some((from, b)) = opts.inherit_size {
                        inherit_value.set_visible(!b);
                        size_entry.set_sensitive(b);
                        obj.set_property(property.name(), DrawOptions { inherit_size: Some((from, !b)), ..opts });
                    }
                }));
                    let inherit_box = gtk::Box::builder()
                        .visible(true)
                        .expand(true)
                        .sensitive(readwrite)
                        .halign(gtk::Align::Start)
                        .valign(gtk::Align::Start)
                        .orientation(gtk::Orientation::Horizontal)
                        .build();
                    inherit_box.add(
                        &gtk::ListBoxRow::builder()
                            .child(&inherit_entry)
                            .activatable(false)
                            .selectable(false)
                            .visible(true)
                            .build(),
                    );
                    inherit_box.add(
                        &gtk::ListBoxRow::builder()
                            .child(&inherit_value)
                            .activatable(false)
                            .selectable(false)
                            .visible(true)
                            .build(),
                    );
                    listbox.add(&inherit_box);
                }
                listbox.set_selection_mode(gtk::SelectionMode::None);
                grid.attach(
                    &gtk::Label::builder()
                        .label("width/length")
                        .visible(true)
                        .build(),
                    0,
                    if has_bg { 2 } else { 1 },
                    1,
                    1,
                );
                grid.attach(&listbox, 1, if has_bg { 2 } else { 1 }, 1, 1);
                grid.upcast()
            }
            _other => gtk::Label::builder()
                .label(&format!("{:?}", val))
                .visible(true)
                .width_chars(5)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .wrap(true)
                .build()
                .upcast(),
        };
        let label = get_label_for_property(property);
        self.add(property.name(), label, widget);
    }
}

impl PropertyWindowBuilder {
    pub fn new(obj: glib::Object, app: &crate::prelude::Application) -> Self {
        Self {
            obj,
            app: app.clone(),
            title: "".into(),
            type_: PropertyWindowType::default(),
        }
    }

    pub fn title(mut self, title: Cow<'static, str>) -> Self {
        self.title = title;
        self
    }

    pub fn type_(mut self, type_: PropertyWindowType) -> Self {
        self.type_ = type_;
        self
    }

    pub fn build(self) -> PropertyWindow {
        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .min_content_height(400)
            .min_content_width(400)
            .build();
        let b = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin(5)
            .margin_bottom(10)
            .visible(true)
            .build();
        let ret: PropertyWindow = glib::Object::new(&[]).unwrap();
        ret.object_to_property_grid(
            self.obj.clone(),
            matches!(self.type_, PropertyWindowType::Create),
        );
        b.pack_start(&ret.imp().grid, true, true, 0);

        ret.set_transient_for(Some(&self.app.window));
        ret.set_attached_to(Some(&self.app.window));
        ret.set_application(Some(&self.app));
        ret.set_title(&self.title);
        ret.imp()
            .buttons
            .set(match self.type_ {
                PropertyWindowType::Modify => {
                    let btns = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .margin(5)
                        .margin_bottom(10)
                        .visible(true)
                        .build();
                    let reset = gtk::Button::builder()
                        .label("Reset")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    reset.connect_clicked(clone!(@weak ret => move |_| {
                        let Some(obj) = ret.imp().obj.get() else { return; } ;
                        for (prop, val) in ret.imp().initial_values.borrow().iter() {
                            _= obj.try_set_property_from_value(prop.as_str(), val);
                        }
                    }));
                    let close = gtk::Button::builder()
                        .label("Close")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    btns.pack_end(&close, false, false, 5);
                    btns.pack_end(&reset, false, false, 5);
                    b.pack_end(&btns, true, false, 5);
                    close.connect_clicked(clone!(@weak ret => move |_| {
                        ret.close();
                    }));
                    PropertyWindowButtons::Modify { reset, close }
                }
                PropertyWindowType::Create => {
                    let btns = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .margin(5)
                        .margin_bottom(10)
                        .visible(true)
                        .build();
                    let cancel = gtk::Button::builder()
                        .label("Cancel")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    cancel.connect_clicked(clone!(@weak ret => move |_| {
                        ret.close();
                    }));
                    let save = gtk::Button::builder()
                        .label("Save")
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    btns.pack_end(&save, false, false, 5);
                    btns.pack_end(&cancel, false, false, 5);
                    b.pack_end(&btns, true, false, 5);

                    PropertyWindowButtons::Create { cancel, save }
                }
            })
            .unwrap();
        ret.imp().obj.set(self.obj).unwrap();

        scrolled_window.set_child(Some(&b));
        ret.set_child(Some(&scrolled_window));

        ret
    }
}

pub fn object_to_property_grid(obj: glib::Object, create: bool) -> gtk::Grid {
    let grid = gtk::Grid::builder()
        .expand(true)
        .visible(true)
        .can_focus(true)
        .column_spacing(5)
        .margin(10)
        .row_spacing(5)
        .build();
    grid.attach(
        &gtk::Label::builder()
            .label(&if create {
                format!("<big>New <i>{}</i></big>", obj.type_().name())
            } else {
                format!("<big>Options for <i>{}</i></big>", obj.type_().name())
            })
            .use_markup(true)
            .margin_top(5)
            .halign(gtk::Align::Start)
            .visible(true)
            .build(),
        0,
        0,
        1,
        1,
    );
    grid.attach(
        &gtk::Separator::builder()
            .expand(true)
            .visible(true)
            .vexpand(false)
            .margin_bottom(10)
            .valign(gtk::Align::Start)
            .build(),
        0,
        1,
        2,
        1,
    );
    let mut row: i32 = 2;
    for prop in obj.list_properties().as_slice().iter().filter(|p| {
        (p.flags()
            .contains(glib::ParamFlags::READWRITE | UI_EDITABLE)
            || p.flags().contains(UI_READABLE))
            && p.owner_type() == obj.type_()
    }) {
        grid.attach(&get_label_for_property(prop), 0, row, 1, 1);
        grid.attach(&get_widget_for_value(&obj, prop, create), 1, row, 1, 1);
        grid.attach(
            &gtk::Separator::builder()
                .expand(true)
                .visible(true)
                .vexpand(false)
                .valign(gtk::Align::Start)
                .build(),
            0,
            row + 1,
            2,
            1,
        );
        row += 2;
    }
    if row != 2 {
        grid.remove_row(row - 1);
    }
    grid
}

pub fn get_widget_for_value(
    obj: &glib::Object,
    property: &glib::ParamSpec,
    create: bool,
) -> gtk::Widget {
    let val: glib::Value = obj.property(property.name());
    let readwrite = property.flags().contains(glib::ParamFlags::READWRITE)
        && !(property.flags().contains(UI_READABLE));
    let flags = if readwrite {
        glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE
    } else {
        glib::BindingFlags::SYNC_CREATE
    };
    match val.type_().name() {
        "gboolean" => {
            let val = val.get::<bool>().unwrap();
            let entry = widgets::ToggleButton::new();
            entry.set_visible(true);
            entry.set_active(val);
            entry.set_sensitive(readwrite);
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Start);
            obj.bind_property(property.name(), &entry, "active")
                .flags(flags)
                .build();

            entry.upcast()
        }
        "gchararray" => {
            let val = val.get::<Option<String>>().unwrap().unwrap_or_default();
            if readwrite {
                let entry = gtk::Entry::builder()
                    .visible(true)
                    .sensitive(readwrite)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Center)
                    .build();
                entry.buffer().set_text(&val);
                obj.bind_property(property.name(), &entry.buffer(), "text")
                    .flags(flags)
                    .build();

                entry.upcast()
            } else {
                let l = gtk::Label::builder()
                    .label(&val)
                    .expand(false)
                    .visible(true)
                    .selectable(true)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Center)
                    .build();
                obj.bind_property(property.name(), &l, "label")
                    .flags(flags)
                    .build();
                if property.flags().contains(UI_PATH) && !create {
                    let b = gtk::Box::builder()
                        .visible(true)
                        .expand(false)
                        .sensitive(true)
                        .halign(gtk::Align::Start)
                        .valign(gtk::Align::Center)
                        .orientation(gtk::Orientation::Horizontal)
                        .build();
                    b.pack_start(&l, true, true, 15);
                    let image = gtk::Image::builder()
                        .icon_name("folder-open")
                        .visible(true)
                        .build();
                    let btn = gtk::Button::builder()
                        .image(&image)
                        .always_show_image(true)
                        .relief(gtk::ReliefStyle::None)
                        .visible(true)
                        .sensitive(true)
                        .tooltip_text("Open file location.")
                        .build();
                    btn.connect_clicked(clone!(@weak obj, @strong property => move |_self| {
                        //FIXME: show error to user, if any.
                        let Some(path) = obj.property::<Option<String>>(property.name()) else { return; };
                        let Ok(prefix) = std::env::current_dir() else { return; };
                        let mut abs_path = prefix.join(&path);
                        if abs_path.is_file() {
                            abs_path.pop();
                        }
                        let Ok(uri) = glib::filename_to_uri(&abs_path, None) else { return ; };
                        gtk::gio::AppInfo::launch_default_for_uri(&uri, gtk::gio::AppLaunchContext::NONE).unwrap();
                    }));
                    b.pack_end(&btn, false, false, 15);
                    b.upcast()
                } else {
                    l.upcast()
                }
            }
        }
        "gint64" => {
            let val = val.get::<i64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecInt64>() {
                (spec.minimum(), spec.maximum())
            } else {
                (i64::MIN, i64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    val as f64, min as f64, max as f64, 1.00, 1.00, 1.00,
                )),
                1.0,
                0,
            );
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Center);
            entry.set_input_purpose(gtk::InputPurpose::Digits);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
                .transform_to(|_, value| {
                    let val = value.get::<i64>().ok()?;
                    Some((val as f64).to_value())
                })
                .transform_from(|_, value| {
                    let val = value.get::<f64>().ok()?;
                    Some((val as i64).to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "guint64" => {
            let val = val.get::<u64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecUInt64>() {
                (0, spec.maximum())
            } else {
                (0, u64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    val as f64, min as f64, max as f64, 1.00, 1.00, 1.00,
                )),
                1.0,
                0,
            );
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Center);
            entry.set_input_purpose(gtk::InputPurpose::Digits);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
                .transform_to(|_, value| {
                    let val = value.get::<u64>().ok()?;
                    Some((val as f64).to_value())
                })
                .transform_from(|_, value| {
                    let val = value.get::<f64>().ok()?;
                    Some((val as u64).to_value())
                })
                .flags(flags)
                .build();
            entry.upcast()
        }
        "gdouble" => {
            let val = val.get::<f64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecDouble>() {
                (spec.minimum(), spec.maximum())
            } else {
                (f64::MIN, f64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(val, min, max, 0.05, 0.01, 0.01)),
                1.0,
                2,
            );
            entry.set_halign(gtk::Align::Start);
            entry.set_valign(gtk::Align::Center);
            entry.set_input_purpose(gtk::InputPurpose::Number);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
                .flags(flags)
                .build();
            entry.upcast()
        }
        "Color" => {
            let val = val.get::<Color>().unwrap();
            let entry = gtk::ColorButton::builder()
                .rgba(&val.into())
                .sensitive(readwrite)
                .visible(true)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Center)
                .use_alpha(true)
                .show_editor(true)
                .build();
            entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let new_val = self_.rgba();
                _ = obj.try_set_property::<Color>(property.name(), new_val.into());
            }));
            entry.upcast()
        }
        "DrawOptions" => {
            let opts = val.get::<DrawOptions>().unwrap();
            let grid = gtk::Grid::builder()
                .expand(true)
                .visible(true)
                .sensitive(readwrite)
                .column_spacing(5)
                .margin(10)
                .row_spacing(5)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            let has_bg = opts.bg.is_some();

            let fg_entry = gtk::ColorButton::builder()
                .rgba(&opts.color.into())
                .sensitive(readwrite)
                .visible(true)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .use_alpha(true)
                .show_editor(true)
                .build();
            fg_entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let opts = obj.property::<DrawOptions>(property.name());
                let new_val = self_.rgba();
                _ = obj.try_set_property::<DrawOptions>(property.name(), DrawOptions { color: new_val.into(), ..opts });
            }));
            grid.attach(
                &gtk::Label::builder()
                    .label(if has_bg { "fg color" } else { "color" })
                    .halign(gtk::Align::End)
                    .visible(true)
                    .build(),
                0,
                0,
                1,
                1,
            );
            grid.attach(&fg_entry, 1, 0, 1, 1);
            if let Some(bg) = opts.bg {
                let bg_entry = gtk::ColorButton::builder()
                    .rgba(&bg.into())
                    .sensitive(readwrite)
                    .visible(true)
                    .halign(gtk::Align::Fill)
                    .valign(gtk::Align::Start)
                    .use_alpha(true)
                    .show_editor(true)
                    .build();
                bg_entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                    let opts = obj.property::<DrawOptions>(property.name());
                    let new_val = self_.rgba();
                    _ = obj.try_set_property::<DrawOptions>(property.name(), DrawOptions { bg: Some(new_val.into()), ..opts });
                }));
                grid.attach(
                    &gtk::Label::builder()
                        .label("bg color")
                        .visible(true)
                        .halign(gtk::Align::End)
                        .build(),
                    0,
                    1,
                    1,
                    1,
                );
                grid.attach(&bg_entry, 1, 1, 1, 1);
            }
            let listbox = gtk::ListBox::builder()
                .visible(true)
                .expand(true)
                .sensitive(readwrite)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            let size_entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    opts.size,
                    0.0,
                    f64::MAX,
                    0.05,
                    0.01,
                    0.01,
                )),
                1.0,
                2,
            );
            size_entry.set_halign(gtk::Align::Start);
            size_entry.set_valign(gtk::Align::Center);
            size_entry.set_input_purpose(gtk::InputPurpose::Number);
            size_entry.set_sensitive(readwrite);
            size_entry.set_visible(true);
            size_entry.connect_value_notify(clone!(@weak obj, @strong property => move |self_| {
                let opts = obj.property::<DrawOptions>(property.name());
                let size = self_.value();
                obj.set_property(property.name(), DrawOptions { size, ..opts });
            }));
            obj.bind_property(property.name(), &size_entry, "value")
                .transform_to(|_, value| {
                    let opts = value.get::<DrawOptions>().ok()?;
                    Some(opts.size.to_value())
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            listbox.add(&size_entry);
            if let Some((from, val)) = opts.inherit_size {
                if val {
                    size_entry.set_sensitive(false);
                }
                let inherit_entry = widgets::ToggleButton::new();
                inherit_entry.set_label("Inherit global value");
                inherit_entry.set_visible(true);
                inherit_entry.set_active(val);
                inherit_entry.set_relief(gtk::ReliefStyle::None);
                inherit_entry.set_sensitive(readwrite);
                inherit_entry.set_halign(gtk::Align::Start);
                inherit_entry.set_valign(gtk::Align::Center);
                obj.bind_property(property.name(), &inherit_entry, "active")
                    .transform_to(|_, value| {
                        let opts = value.get::<DrawOptions>().ok()?;
                        opts.inherit_size.map(|(_, b)| b.to_value())
                    })
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                let inherit_value = gtk::Label::builder()
                    .label(&format!("{:.2}", obj.property::<f64>(from)))
                    .visible(val)
                    .width_chars(5)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Center)
                    .sensitive(false)
                    .wrap(true)
                    .build();
                inherit_entry.connect_clicked(clone!(@weak obj, @strong property, @weak inherit_value, @weak size_entry => move |_| {
                    let opts = obj.property::<DrawOptions>(property.name());
                    if let Some((from, b)) = opts.inherit_size {
                        inherit_value.set_visible(!b);
                        size_entry.set_sensitive(b);
                        obj.set_property(property.name(), DrawOptions { inherit_size: Some((from, !b)), ..opts });
                    }
                }));
                let inherit_box = gtk::Box::builder()
                    .visible(true)
                    .expand(true)
                    .sensitive(readwrite)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Start)
                    .orientation(gtk::Orientation::Horizontal)
                    .build();
                inherit_box.add(
                    &gtk::ListBoxRow::builder()
                        .child(&inherit_entry)
                        .activatable(false)
                        .selectable(false)
                        .visible(true)
                        .build(),
                );
                inherit_box.add(
                    &gtk::ListBoxRow::builder()
                        .child(&inherit_value)
                        .activatable(false)
                        .selectable(false)
                        .visible(true)
                        .build(),
                );
                listbox.add(&inherit_box);
            }
            listbox.set_selection_mode(gtk::SelectionMode::None);
            grid.attach(
                &gtk::Label::builder()
                    .label("width/length")
                    .visible(true)
                    .build(),
                0,
                if has_bg { 2 } else { 1 },
                1,
                1,
            );
            grid.attach(&listbox, 1, if has_bg { 2 } else { 1 }, 1, 1);
            grid.upcast()
        }
        _other => gtk::Label::builder()
            .label(&format!("{:?}", val))
            .visible(true)
            .width_chars(5)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Start)
            .wrap(true)
            .build()
            .upcast(),
    }
}

pub fn get_label_for_property(prop: &glib::ParamSpec) -> gtk::Label {
    gtk::Label::builder().label(&{
                let blurb = prop.blurb();
                let name = prop.name();
                let type_name: &str = match prop.value_type().name() {
                    "gboolean" => "bool",
                    "gchararray" => "string",
                    "guint64"|"gint64" => "int",
                    "gdouble"=> "float",
                    "Color" => "color",
                    "DrawOptions" => "theme options",
                    _other => _other,
                };
                if blurb == name {
                    format!("Key: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
                } else {
                    format!("<span insert_hyphens=\"true\" allow_breaks=\"true\" foreground=\"#222222\">{blurb}</span>\n\nKey: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
                }
            })
            .visible(true)
                .selectable(true)
                .wrap_mode(gtk::pango::WrapMode::Char)
                .use_markup(true)
                .max_width_chars(30)
                .halign(gtk::Align::Start)
                .wrap(true)
                .build()
}

pub fn new_property_window(
    app: &crate::prelude::Application,
    obj: glib::Object,
    title: &str,
) -> gtk::Window {
    let w = gtk::Window::builder()
        .deletable(true)
        .transient_for(&app.window)
        .attached_to(&app.window)
        .application(app)
        .destroy_with_parent(true)
        .focus_on_map(true)
        .resizable(true)
        .title(title)
        .visible(true)
        .expand(true)
        .type_hint(gtk::gdk::WindowTypeHint::Utility)
        .window_position(gtk::WindowPosition::CenterOnParent)
        .build();
    let scrolled_window = gtk::ScrolledWindow::builder()
        .min_content_height(400)
        .min_content_width(400)
        .expand(true)
        .visible(true)
        .can_focus(true)
        .build();
    let grid = crate::utils::object_to_property_grid(obj, false);
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin(5)
        .margin_bottom(10)
        .visible(true)
        .build();
    b.pack_start(&grid, true, true, 0);
    let close_button = gtk::Button::builder()
        .label("Close")
        .relief(gtk::ReliefStyle::None)
        .visible(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    close_button.connect_clicked(clone!(@weak w => move |_| {
        w.close();
    }));
    b.pack_end(&close_button, false, false, 0);
    scrolled_window.set_child(Some(&b));
    w.set_child(Some(&scrolled_window));
    w
}

#[derive(Default, Debug)]
pub struct PropertyChoiceInner {
    pub btn: OnceCell<gtk::RadioButton>,
    pub widget: OnceCell<gtk::Widget>,
}

#[glib::object_subclass]
impl ObjectSubclass for PropertyChoiceInner {
    const NAME: &'static str = "PropertyChoice";
    type Type = PropertyChoice;
    type ParentType = gtk::Box;
}

impl ObjectImpl for PropertyChoiceInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.upcast_ref::<gtk::Box>()
            .set_orientation(gtk::Orientation::Horizontal);
        obj.set_spacing(1);
        obj.set_expand(false);
        obj.set_visible(true);
        obj.set_can_focus(true);
    }
}

impl WidgetImpl for PropertyChoiceInner {}
impl ContainerImpl for PropertyChoiceInner {}
impl BoxImpl for PropertyChoiceInner {}

glib::wrapper! {
    pub struct PropertyChoice(ObjectSubclass<PropertyChoiceInner>)
        @extends gtk::Widget, gtk::Container, gtk::Box;
}

impl PropertyChoice {
    pub fn new(label: &str, btn: gtk::RadioButton, widget: gtk::Widget) -> Self {
        let ret: PropertyChoice = glib::Object::new(&[]).unwrap();
        let label = gtk::Label::builder()
            .label(label)
            .visible(true)
            .selectable(false)
            .max_width_chars(30)
            .halign(gtk::Align::Start)
            .wrap(true)
            .expand(false)
            .build();
        let event_box = gtk::EventBox::builder()
            .events(gtk::gdk::EventMask::BUTTON_PRESS_MASK)
            .above_child(true)
            .child(&label)
            .visible(true)
            .build();
        ret.pack_start(&event_box, false, false, 5);
        ret.pack_start(&btn, false, false, 5);
        ret.pack_start(&widget, false, false, 5);
        btn.bind_property("active", &widget, "sensitive")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        event_box.connect_button_press_event(clone!(@weak btn => @default-return Inhibit(false), move |_, event| {
            if event.button() == gtk::gdk::BUTTON_PRIMARY && event.event_type() == gtk::gdk::EventType::ButtonPress {
                btn.set_active(true);
            }
            Inhibit(false)
        }));
        ret.btn.set(btn).unwrap();
        ret
    }

    pub fn button(&self) -> &gtk::RadioButton {
        self.btn.get().unwrap()
    }
}

impl_deref!(PropertyChoice, PropertyChoiceInner);
