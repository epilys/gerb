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
pub use builder::*;
pub use widgets::*;

pub trait CreatePropertyWindow: glib::object::ObjectExt + FriendlyNameInSettings {
    fn new_property_window(&self, app: &Application, _create: bool) -> PropertyWindow
    where
        Self: glib::IsA<glib::Object>,
    {
        PropertyWindow::builder(
            self.downgrade().upgrade().unwrap().upcast::<glib::Object>(),
            app,
        )
        .title(self.friendly_name())
        .friendly_name(self.friendly_name())
        .type_(PropertyWindowType::Modify)
        .build()
    }
}

pub trait FriendlyNameInSettings: glib::object::ObjectExt + glib::IsA<glib::Object> {
    fn friendly_name(&self) -> Cow<'static, str> {
        self.type_().name().into()
    }

    fn static_friendly_name() -> Cow<'static, str> {
        Self::static_type().name().into()
    }
}

pub trait PropertyWidget {
    fn get(
        app: &Application,
        val: glib::Value,
        obj: &glib::Object,
        property: &glib::ParamSpec,
        create: bool,
        readwrite: bool,
        flags: glib::BindingFlags,
    ) -> gtk::Widget;
}

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
                .relief(gtk::ReliefStyle::Normal)
                .visible(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build(),
            close: gtk::Button::builder()
                .label("Close")
                .relief(gtk::ReliefStyle::Normal)
                .visible(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build(),
        }
    }
}

mod builder {
    use super::*;

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
        friendly_name: Cow<'static, str>,
        type_: PropertyWindowType,
    }

    impl PropertyWindowBuilder {
        pub fn new(obj: glib::Object, app: &crate::prelude::Application) -> Self {
            Self {
                obj,
                app: app.clone(),
                title: "".into(),
                friendly_name: "".into(),
                type_: PropertyWindowType::default(),
            }
        }

        pub fn title(mut self, title: Cow<'static, str>) -> Self {
            self.title = title;
            self
        }

        pub fn friendly_name(mut self, friendly_name: Cow<'static, str>) -> Self {
            self.friendly_name = friendly_name;
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
                .min_content_height(600)
                .min_content_width(500)
                .build();
            let b = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .border_width(2)
                .margin(5)
                .margin_bottom(10)
                .visible(true)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Fill)
                .expand(false)
                .build();
            {
                let sc = b.style_context();
                sc.add_class("vertical");
                sc.add_class("dialog-vbox");
            }
            let mut ret: PropertyWindow = glib::Object::new(&[]).unwrap();
            ret.imp().app.set(self.app.clone()).unwrap();
            ret.object_to_property_grid(
                self.obj.clone(),
                if self.friendly_name.is_empty() {
                    None
                } else {
                    Some(self.friendly_name.clone())
                },
                matches!(self.type_, PropertyWindowType::Create),
            );
            b.pack_start(&ret.imp().grid, false, false, 0);
            ret.imp().grid.style_context().add_class("horizontal");

            ret.set_transient_for(Some(&self.app.window));
            ret.set_attached_to(Some(&self.app.window));
            ret.set_application(Some(&self.app));
            ret.imp()
            .buttons
            .set(match self.type_ {
                PropertyWindowType::Modify => {
                    let btns = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Fill)
                        .margin(0)
                        .margin_bottom(10)
                        .spacing(5)
                        .visible(true)
                        .build();
                    let reset = gtk::Button::builder()
                        .label("Reset")
                        .relief(gtk::ReliefStyle::Normal)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    reset.connect_clicked(clone!(@weak ret => move |_| {
                        let Some(obj) = ret.imp().obj.get() else { return; } ;
                        for (prop, (is_dirty, val)) in ret.imp().initial_values.borrow().iter() {
                            is_dirty.set(false);
                            ret.notify(PropertyWindow::IS_DIRTY);
                            for obj in std::iter::once(obj).chain(ret.imp().extra_objs.borrow().iter()) {
                                if obj.try_set_property_from_value(prop.as_str(), val).is_ok() {
                                    break;
                                }
                            }
                        }
                    }));
                    let close = gtk::Button::builder()
                        .label("Close")
                        .relief(gtk::ReliefStyle::Normal)
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
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Fill)
                        .margin(0)
                        .margin_bottom(10)
                        .spacing(5)
                        .visible(true)
                        .build();
                    let cancel = gtk::Button::builder()
                        .label("Cancel")
                        .relief(gtk::ReliefStyle::Normal)
                        .visible(true)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .build();
                    cancel.connect_clicked(clone!(@weak ret => move |_| {
                        ret.close();
                    }));
                    let save = gtk::Button::builder()
                        .label("Save")
                        .relief(gtk::ReliefStyle::Normal)
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
            ret.set_title(&self.title);
            *ret.imp().title.borrow_mut() = self.title;
            if !self.friendly_name.is_empty() {
                *ret.imp().friendly_name.borrow_mut() = self.friendly_name
            };
            ret.bind_property(PropertyWindow::IS_DIRTY, &ret, "title")
                .transform_to(|binding, is_dirty_val| {
                    let window = binding.source()?.downcast::<PropertyWindow>().ok()?;
                    let title = window.imp().title.borrow();
                    if is_dirty_val.get::<bool>().ok()? {
                        Some(format!("{}*", &title).to_value())
                    } else {
                        Some(title.to_value())
                    }
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            if matches!(self.type_, PropertyWindowType::Create) {
                if let Some((_, first_widget)) = ret.imp().widgets.borrow().get_index(0) {
                    first_widget.set_has_focus(true);
                }
            }
            ret.imp().type_.set(self.type_).unwrap();

            ret
        }
    }
}

mod widgets {
    use super::*;

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
            obj.set_halign(gtk::Align::End);
            obj.set_valign(gtk::Align::Start);
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
            let ret: Self = glib::Object::new(&[]).unwrap();
            let label = gtk::Label::builder()
                .label(label)
                .visible(true)
                .selectable(false)
                .max_width_chars(30)
                .halign(gtk::Align::End)
                .wrap(true)
                .expand(false)
                .build();
            let event_box = gtk::EventBox::builder()
                .events(gtk::gdk::EventMask::BUTTON_PRESS_MASK)
                .above_child(true)
                .child(&label)
                .halign(gtk::Align::End)
                .visible(true)
                .build();
            btn.set_halign(gtk::Align::End);
            widget.set_halign(gtk::Align::Fill);
            ret.pack_start(&event_box, false, false, 5);
            ret.pack_start(&btn, false, false, 5);
            ret.pack_start(&widget, false, false, 0);
            btn.bind_property("active", &widget, "sensitive")
                .transform_to(|b, val| {
                    let val = val.get::<bool>().ok()?;
                    let w = b.target()?.downcast::<gtk::Widget>().ok()?;
                    w.style_read_only(val);
                    Some(val.to_value())
                })
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

    impl PropertyWidget for bool {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            flags: glib::BindingFlags,
        ) -> gtk::Widget {
            let val = val.get::<bool>().unwrap();
            let entry = gtk::CheckButton::new();
            entry.set_visible(true);
            entry.set_active(val);
            entry.set_sensitive(readwrite);
            entry.style_read_only(readwrite);
            entry.set_expand(false);
            entry.set_halign(gtk::Align::Fill);
            entry.set_valign(gtk::Align::Start);
            obj.bind_property(property.name(), &entry, "active")
                .flags(flags)
                .build();
            entry.upcast()
        }
    }

    impl PropertyWidget for Option<String> {
        fn get(
            app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            create: bool,
            readwrite: bool,
            flags: glib::BindingFlags,
        ) -> gtk::Widget {
            let val = val.get::<Option<String>>().unwrap().unwrap_or_default();
            if readwrite {
                let entry = gtk::Entry::builder()
                    .visible(true)
                    .sensitive(readwrite)
                    .expand(false)
                    .halign(gtk::Align::Fill)
                    .valign(gtk::Align::Start)
                    .build();
                entry.buffer().set_text(&val);
                obj.bind_property(property.name(), &entry.buffer(), "text")
                    .flags(flags)
                    .build();

                entry.upcast()
            } else {
                let l = gtk::Label::builder()
                    .label(&val)
                    .visible(true)
                    .selectable(true)
                    .wrap(true)
                    .wrap_mode(gtk::pango::WrapMode::Char)
                    .ellipsize(gtk::pango::EllipsizeMode::None)
                    .expand(false)
                    .halign(gtk::Align::Fill)
                    .valign(gtk::Align::Start)
                    .build();
                l.style_read_only(false);
                if property.flags().contains(UI_PATH) {
                    l.style_monospace();
                    l.style_context().add_class("path");
                }
                obj.bind_property(property.name(), &l, "label")
                    .flags(flags)
                    .build();
                if property.flags().contains(UI_PATH) && !create {
                    let b = gtk::Box::builder()
                        .visible(true)
                        .sensitive(true)
                        .expand(false)
                        .halign(gtk::Align::Fill)
                        .valign(gtk::Align::Start)
                        .orientation(gtk::Orientation::Horizontal)
                        .build();
                    b.pack_start(&l, true, true, 0);
                    let image = gtk::Image::builder()
                        .icon_name("folder-open")
                        .visible(true)
                        .build();
                    let btn = gtk::Button::builder()
                        .image(&image)
                        .always_show_image(true)
                        .relief(gtk::ReliefStyle::Normal)
                        .visible(true)
                        .sensitive(true)
                        .tooltip_text("Open file location")
                        .build();
                    btn.connect_clicked(clone!(@weak obj, @strong property, @weak app.window as window => move |_self| {
                        let Some(path) = obj.property::<Option<String>>(property.name()) else { return; };
                        let Ok(prefix) = std::env::current_dir() else { return; };
                        let mut abs_path = prefix.join(path);
                        if abs_path.is_file() {
                            abs_path.pop();
                        }
                        if let Err(err) = glib::filename_to_uri(&abs_path, None).and_then(|uri| gtk::gio::AppInfo::launch_default_for_uri(&uri, gtk::gio::AppLaunchContext::NONE)) {
                            let dialog = crate::utils::widgets::new_simple_error_dialog(
                                Some("Error: Could not open location."),
                                &err.to_string(),
                                None,
                                window.upcast_ref(),
                            );
                            dialog.run();
                            dialog.emit_close();
                        };
                    }));
                    b.pack_end(&btn, false, false, 15);
                    b.upcast()
                } else {
                    l.upcast()
                }
            }
        }
    }
    impl PropertyWidget for i64 {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            flags: glib::BindingFlags,
        ) -> gtk::Widget {
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
            entry.set_expand(false);
            entry.set_halign(gtk::Align::Fill);
            entry.set_valign(gtk::Align::Start);
            entry.set_input_purpose(gtk::InputPurpose::Digits);
            entry.set_sensitive(readwrite);
            if !readwrite {
                entry.style_read_only(readwrite);
            }
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
    }

    impl PropertyWidget for u64 {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            flags: glib::BindingFlags,
        ) -> gtk::Widget {
            let val = val.get::<u64>().unwrap();
            let (min, max) = if let Some(spec) = property.downcast_ref::<glib::ParamSpecUInt64>() {
                (0, spec.maximum())
            } else {
                (0, u64::MAX)
            };
            let entry = gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(
                    val as f64,
                    f64::from(min),
                    max as f64,
                    1.00,
                    1.00,
                    1.00,
                )),
                1.0,
                0,
            );
            entry.set_expand(false);
            entry.set_halign(gtk::Align::Fill);
            entry.set_valign(gtk::Align::Start);
            entry.set_input_purpose(gtk::InputPurpose::Digits);
            entry.set_sensitive(readwrite);
            entry.set_visible(true);
            if !readwrite {
                entry.style_read_only(readwrite);
            }
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
    }

    impl PropertyWidget for f64 {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            flags: glib::BindingFlags,
        ) -> gtk::Widget {
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
            entry.set_expand(false);
            entry.set_halign(gtk::Align::Fill);
            entry.set_valign(gtk::Align::Start);
            entry.set_input_purpose(gtk::InputPurpose::Number);
            entry.set_sensitive(readwrite);
            if !readwrite {
                entry.style_read_only(readwrite);
            }
            entry.set_visible(true);
            obj.bind_property(property.name(), &entry, "value")
                .flags(flags)
                .build();
            entry.upcast()
        }
    }

    impl PropertyWidget for Color {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            _flags: glib::BindingFlags,
        ) -> gtk::Widget {
            let val = val.get::<Color>().unwrap();
            let entry = gtk::ColorButton::builder()
                .rgba(&val.into())
                .sensitive(readwrite)
                .visible(true)
                .expand(false)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .use_alpha(true)
                .show_editor(true)
                .build();
            entry.connect_color_set(clone!(@weak obj, @strong property => move |self_| {
                let new_val = self_.rgba();
                _ = obj.try_set_property::<Color>(property.name(), new_val.into());
            }));
            entry.upcast()
        }
    }

    impl PropertyWidget for DrawOptions {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            _flags: glib::BindingFlags,
        ) -> gtk::Widget {
            let opts = val.get::<DrawOptions>().unwrap();
            let grid = gtk::Grid::builder()
                .expand(true)
                .visible(true)
                .sensitive(readwrite)
                .column_spacing(5)
                .margin(10)
                .row_spacing(5)
                .halign(gtk::Align::Fill)
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
            size_entry.set_expand(true);
            size_entry.set_halign(gtk::Align::Fill);
            size_entry.set_valign(gtk::Align::Start);
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
            grid.attach(
                &gtk::Label::builder()
                    .label("width/length")
                    .visible(true)
                    .sensitive(readwrite)
                    .build(),
                0,
                if has_bg { 2 } else { 1 },
                1,
                1,
            );
            grid.attach(&size_entry, 1, if has_bg { 2 } else { 1 }, 1, 1);
            if let Some((from, val)) = opts.inherit_size {
                if val {
                    size_entry.set_sensitive(false);
                }
                let inherit_entry = gtk::CheckButton::new();
                inherit_entry.set_label("Inherit global value");
                inherit_entry.set_visible(true);
                inherit_entry.set_active(val);
                inherit_entry.set_relief(gtk::ReliefStyle::Normal);
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
                    .halign(gtk::Align::Fill)
                    .valign(gtk::Align::Start)
                    .orientation(gtk::Orientation::Horizontal)
                    .build();
                inherit_box.pack_start(&inherit_entry, true, true, 0);
                inherit_box.pack_start(&inherit_value, true, true, 5);
                grid.attach(&inherit_box, 1, if has_bg { 3 } else { 2 }, 1, 1);
            }
            grid.upcast()
        }
    }

    impl PropertyWidget for Option<ufo::objects::Layer> {
        fn get(
            app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            _flags: glib::BindingFlags,
        ) -> gtk::Widget {
            use ufo::objects::Layer;

            let val = val.get::<Option<Layer>>().unwrap();
            let project = app.runtime.project.borrow().clone();

            let entry = gtk::ComboBoxText::builder()
                .sensitive(readwrite)
                .visible(true)
                .expand(false)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .build();
            obj.bind_property(property.name(), &entry, "active-id")
                .transform_to(|_, val| {
                    let layer = val.get::<Option<Layer>>().ok()??;
                    let ret = Some(layer.name.borrow().to_string().to_value());
                    ret
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            if !readwrite {
                entry.style_read_only(readwrite);
            } else {
                entry.connect_changed(clone!(@weak obj, @strong project, @strong property => move |entry| {
                        let active_id = entry.active_id();
                        obj.set_property(property.name(), active_id.and_then(|n| project.all_layers.borrow().iter().find(|l| l.name.borrow().as_str() == n).map(Layer::clone)));
                    }));
            }
            let mut active_id: Cow<'static, str> = if let Some(l) = val {
                l.name.borrow().clone().into()
            } else {
                "public.default".into()
            };
            for l in project.all_layers.borrow().iter() {
                let name = l.name.borrow();
                if name.as_str() == active_id {
                    active_id = name.clone().into();
                }
                entry.append(Some(name.as_str()), name.as_str());
            }
            entry.set_active_id(Some(active_id.as_ref()));
            let layer_box = gtk::Box::builder()
                .visible(true)
                .sensitive(readwrite)
                .spacing(5)
                .expand(false)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .orientation(gtk::Orientation::Vertical)
                .build();
            layer_box.add(&entry);
            let l = gtk::Label::builder()
                .visible(true)
                .expand(false)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .wrap(true)
                .wrap_mode(gtk::pango::WrapMode::Char)
                .ellipsize(gtk::pango::EllipsizeMode::None)
                .use_markup(true)
                .build();
            obj.bind_property(property.name(), &l, "label")
                .transform_to(|_, val| {
                    let Some(l): Option<Layer> = val.get::<Option<Layer>>().ok()? else {
                            return Some(String::new().to_value());
                        };
                    Some(format!("Directory: <tt>{}/</tt>", l.dir_name.borrow()).to_value())
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            layer_box.add(&l);
            layer_box.upcast()
        }
    }

    impl PropertyWidget for Theme {
        fn get(
            _app: &Application,
            val: glib::Value,
            obj: &glib::Object,
            property: &glib::ParamSpec,
            _create: bool,
            readwrite: bool,
            _flags: glib::BindingFlags,
        ) -> gtk::Widget {
            let val = val.get::<Theme>().unwrap();

            let entry = gtk::ComboBoxText::builder()
                .sensitive(readwrite)
                .visible(true)
                .expand(false)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .build();
            obj.bind_property(property.name(), &entry, "active-id")
                .transform_to(|_, val| {
                    let theme = val.get::<Theme>().ok()?;
                    Some(theme.name().to_value())
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            if !readwrite {
                entry.style_read_only(readwrite);
            } else {
                entry.connect_changed(clone!(@weak obj, @strong property => move |entry| {
                    let Some(active_id) = entry.active_id() else { return; };
                    let Some(val)= Theme::kebab_str_deserialize(&active_id) else { return; };
                    obj.set_property(property.name(), val);
                }));
            }
            let active_id: Cow<'static, str> = val.name().into();
            for v in Theme::kebab_case_variants() {
                entry.append(Some(v), v);
            }
            entry.set_active_id(Some(active_id.as_ref()));
            entry.upcast()
        }
    }
}
