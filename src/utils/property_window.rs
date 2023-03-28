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

mod types;
pub use types::*;

#[derive(Default, Debug)]
pub struct PropertyWindowInner {
    pub obj: OnceCell<glib::Object>,
    pub extra_objs: RefCell<Vec<glib::Object>>,
    pub app: OnceCell<crate::prelude::Application>,
    pub top_area: gtk::Box,
    pub top_button_area: gtk::Box,
    pub bottom_button_area: gtk::Box,
    pub main_area: gtk::Box,
    pub grid: gtk::Grid,
    rows: Cell<i32>,
    pub buttons: OnceCell<PropertyWindowButtons>,
    type_: OnceCell<PropertyWindowType>,
    widgets: RefCell<IndexMap<String, gtk::Widget>>,
    initial_values: RefCell<IndexMap<String, (Cell<bool>, glib::Value)>>,
    title_label: gtk::Label,
    title: RefCell<Cow<'static, str>>,
    friendly_name: RefCell<Cow<'static, str>>,
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
        for area in &[
            &self.top_area,
            &self.top_button_area,
            &self.bottom_button_area,
        ] {
            area.set_visible(true);
            area.set_expand(false);
        }
        self.top_area.set_halign(gtk::Align::Start);
        self.top_area.set_valign(gtk::Align::End);
        for area in &[&self.top_button_area, &self.bottom_button_area] {
            area.set_orientation(gtk::Orientation::Horizontal);
            area.set_halign(gtk::Align::Center);
            area.set_valign(gtk::Align::Fill);
            area.set_margin(0);
            area.set_margin_bottom(10);
            area.set_spacing(5);
            area.set_visible(true);
        }
        let sc = self.main_area.style_context();
        sc.add_class("vertical");
        sc.add_class("dialog-vbox");
        self.main_area.set_orientation(gtk::Orientation::Vertical);
        self.main_area.set_border_width(2);
        self.main_area.set_margin(5);
        self.main_area.set_margin_bottom(10);
        self.main_area.set_visible(true);
        self.main_area.set_halign(gtk::Align::Fill);
        self.main_area.set_valign(gtk::Align::Fill);
        self.main_area.set_expand(false);
        self.main_area
            .pack_start(&self.top_button_area, false, false, 0);
        self.main_area
            .pack_end(&self.bottom_button_area, false, false, 0);
        obj.style_context().add_class("property-window");
        self.title_label.set_use_markup(true);
        self.title_label.set_margin_top(5);
        self.title_label.set_halign(gtk::Align::Start);
        self.title_label.set_visible(true);
        obj.add_subsection(&self.title_label);
        self.grid.attach_next_to(
            &self.top_area,
            Some(&self.title_label),
            gtk::PositionType::Right,
            1,
            1,
        );
        obj.set_deletable(true);
        obj.set_destroy_with_parent(true);
        obj.set_focus_on_map(true);
        obj.set_resizable(true);
        obj.set_visible(true);
        obj.set_expand(false);
        obj.set_halign(gtk::Align::Fill);
        obj.set_valign(gtk::Align::Fill);
        obj.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
        obj.set_window_position(gtk::WindowPosition::Center);
        obj.connect_key_press_event(move |window, event| {
            if event.keyval() == gdk::keys::constants::Escape && !window.is_dirty() {
                window.close();

                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
        self.grid.style_context().add_class("horizontal");
        self.grid.set_expand(false);
        self.grid.set_visible(true);
        self.grid.set_can_focus(false);
        self.grid.set_column_spacing(5);
        self.grid.set_margin(10);
        self.grid.set_row_spacing(5);
        self.grid.set_orientation(gtk::Orientation::Horizontal);
        self.grid.set_halign(gtk::Align::Fill);
        self.grid.set_valign(gtk::Align::Fill);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecBoolean::new(
                    PropertyWindow::IS_DIRTY,
                    PropertyWindow::IS_DIRTY,
                    PropertyWindow::IS_DIRTY,
                    false,
                    glib::ParamFlags::READWRITE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            PropertyWindow::IS_DIRTY => self.instance().is_dirty().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            PropertyWindow::IS_DIRTY => {}
            _ => unimplemented!("{}", pspec.name()),
        }
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

impl PropertyWindow {
    const IS_DIRTY: &str = "is-dirty";

    pub fn builder(obj: glib::Object, app: &crate::prelude::Application) -> PropertyWindowBuilder {
        PropertyWindowBuilder::new(obj, app)
    }

    pub fn widgets(&mut self) -> FieldRef<'_, IndexMap<String, gtk::Widget>> {
        self.imp().widgets.borrow().into()
    }

    pub fn is_dirty(&self) -> bool {
        self.imp()
            .initial_values
            .borrow()
            .values()
            .any(|(d, _)| d.get())
    }

    fn object_to_property_grid(
        &mut self,
        obj: glib::Object,
        friendly_name: Option<Cow<'static, str>>,
        create: bool,
    ) {
        self.imp()
            .title_label
            .set_label(&if let Some(n) = friendly_name {
                format!("<big><i>{}</i></big>", n)
            } else if create {
                format!("<big>New <i>{}</i></big>", obj.type_().name())
            } else {
                format!("<big><i>{}</i></big>", obj.type_().name())
            });
        self.add_obj_properties(obj, create);
    }

    pub fn add_subsection(&self, label: &gtk::Label) {
        let row = self.imp().rows.get();
        self.imp().grid.attach(label, 0, row, 1, 1);
        self.imp().rows.set(row + 1);
        self.add_separator();
    }

    fn add_obj_properties(&self, obj: glib::Object, create: bool) {
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

    pub fn add(&self, name: &str, label: gtk::Widget, widget: gtk::Widget) {
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
        let row = self.imp().rows.get() + 1;
        self.imp().grid.attach(
            &gtk::Separator::builder()
                .expand(true)
                .visible(true)
                .vexpand(false)
                .valign(gtk::Align::Start)
                .build(),
            0,
            row,
            2,
            1,
        );
        self.imp().rows.set(row + 1);
    }

    fn add_property(&self, obj: &glib::Object, property: &glib::ParamSpec, create: bool) {
        let app = self.imp().app.get().unwrap();
        let val: glib::Value = obj.property(property.name());
        self.imp()
            .initial_values
            .borrow_mut()
            .insert(property.name().to_string(), (false.into(), val.clone()));
        let readwrite = property.flags().contains(glib::ParamFlags::READWRITE)
            && !(property.flags().contains(UI_READABLE));
        let flags = if readwrite {
            glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE
        } else {
            glib::BindingFlags::SYNC_CREATE
        };
        macro_rules! check_dirty_on_change {
            ($ty:ty) => {
                if readwrite {
                    obj.connect_notify_local(
                        Some(property.name()),
                        clone!(@weak self as win => move |obj, pspec| {
                            let Ok(init_values) = win.imp().initial_values.try_borrow() else { return; };
                            let Some(init_val) = init_values.get(pspec.name()) else { return; };
                            init_val.0.set(
                                init_val.1.get::<$ty>().ok()
                                    != Some(obj.property::<$ty>(pspec.name()))
                            );
                            win.notify(PropertyWindow::IS_DIRTY);
                        }),
                    );
                }
            };
        }
        let widget: gtk::Widget = match val.type_().name() {
            "gboolean" => {
                check_dirty_on_change!(bool);
                // Special case for bool checkboxes: toggle them by pressing the label as well.
                let widget = <bool>::get(app, val, obj, property, create, readwrite, flags);
                let label = get_label_for_property(property);
                let event_box = gtk::EventBox::builder()
                    .events(gtk::gdk::EventMask::BUTTON_PRESS_MASK)
                    .above_child(true)
                    .child(&label)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Start)
                    .visible(true)
                    .build();
                event_box.connect_button_press_event(clone!(@weak widget => @default-return Inhibit(false), move |_, event| {
                    if event.button() == gtk::gdk::BUTTON_PRIMARY && event.event_type() == gtk::gdk::EventType::ButtonPress {
                        let prop = widget.property::<bool>("active");
                        widget.set_property("active", !prop);
                    }
                    Inhibit(false)
                }));
                self.add(property.name(), event_box.upcast(), widget);
                return;
            }
            "gchararray" => {
                if readwrite {
                    check_dirty_on_change!(Option<String>);
                }
                <Option<String>>::get(app, val, obj, property, create, readwrite, flags)
            }
            "gint64" => {
                check_dirty_on_change!(i64);
                <i64>::get(app, val, obj, property, create, readwrite, flags)
            }
            "guint64" => {
                check_dirty_on_change!(u64);
                <u64>::get(app, val, obj, property, create, readwrite, flags)
            }
            "gdouble" => {
                check_dirty_on_change!(f64);
                <f64>::get(app, val, obj, property, create, readwrite, flags)
            }
            "Color" => {
                check_dirty_on_change!(Color);
                <Color>::get(app, val, obj, property, create, readwrite, flags)
            }
            "DrawOptions" => {
                check_dirty_on_change!(DrawOptions);
                <DrawOptions>::get(app, val, obj, property, create, readwrite, flags)
            }
            "Layer" if property.value_type() == ufo::objects::Layer::static_type() => {
                use ufo::objects::Layer;

                check_dirty_on_change!(Option<Layer>);
                <Option<Layer>>::get(app, val, obj, property, create, readwrite, flags)
            }
            "Theme" => {
                check_dirty_on_change!(Theme);
                <Theme>::get(app, val, obj, property, create, readwrite, flags)
            }
            "ShowMinimap" => {
                check_dirty_on_change!(ShowMinimap);
                <ShowMinimap>::get(app, val, obj, property, create, readwrite, flags)
            }
            "MarkColor" => {
                check_dirty_on_change!(MarkColor);
                <MarkColor>::get(app, val, obj, property, create, readwrite, flags)
            }
            _other => gtk::Label::builder()
                .label(&format!("{:?}", val))
                .visible(true)
                .expand(false)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Start)
                .wrap(true)
                .wrap_mode(gtk::pango::WrapMode::Char)
                .ellipsize(gtk::pango::EllipsizeMode::None)
                .build()
                .upcast(),
        };
        let label = get_label_for_property(property);
        self.add(property.name(), label.upcast(), widget);
    }

    pub fn add_extra_obj(&self, obj: glib::Object, friendly_name: Option<Cow<'static, str>>) {
        self.add_separator();
        self.add_subsection(
            &gtk::Label::builder()
                .label(&format!(
                    "<big><i>{}</i></big>",
                    friendly_name
                        .as_deref()
                        .unwrap_or_else(|| obj.type_().name())
                ))
                .use_markup(true)
                .margin_top(5)
                .halign(gtk::Align::Start)
                .visible(true)
                .build(),
        );
        self.add_obj_properties(
            obj.clone(),
            matches!(self.imp().type_.get().unwrap(), PropertyWindowType::Create),
        );
        self.imp().extra_objs.borrow_mut().push(obj);
    }

    pub fn set_buttons(&self, buttons: &[&gtk::Button]) {
        for b in buttons {
            let b = *b;
            let top_btn = gtk::Button::builder()
                .label(b.label().unwrap().as_str())
                .relief(gtk::ReliefStyle::Normal)
                .visible(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build();
            top_btn.connect_clicked(clone!(@weak  b => move |_| {
                b.emit_clicked();
            }));
            self.imp()
                .top_button_area
                .pack_end(&top_btn, false, false, 5);
            self.imp().bottom_button_area.pack_end(b, false, false, 5);
        }
    }
}

pub fn get_label_for_property(prop: &glib::ParamSpec) -> gtk::Label {
    let blurb = prop.blurb();
    let name = prop.name();
    let type_name: &str = match prop.value_type().name() {
        "gboolean" => "bool",
        "gchararray" if prop.flags().contains(UI_PATH) => "path",
        "gchararray" => "string",
        "guint64" | "gint64" => "int",
        "gdouble" => "float",
        "Color" => "color",
        "DrawOptions" => "theme options",
        _other => _other,
    };
    let label = if blurb == name {
        format!("Key: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
    } else {
        format!("<span insert_hyphens=\"true\" allow_breaks=\"true\">{blurb}</span>\n\nKey: <tt>{name}</tt>\nType: <span background=\"cornflowerblue\" foreground=\"white\"><tt> {type_name} </tt></span>")
    };
    gtk::Label::builder()
        .label(&label)
        .visible(true)
        .selectable(true)
        .wrap_mode(gtk::pango::WrapMode::Char)
        .use_markup(true)
        .max_width_chars(30)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .wrap(true)
        .build()
}
