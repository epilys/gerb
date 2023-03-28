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
}
