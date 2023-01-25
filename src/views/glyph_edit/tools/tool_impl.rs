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

use crate::views::{Canvas, GlyphEditView};
use gtk::{
    glib::{self},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

pub type ToolImplInstance = ToolImpl;

/// GObject glue code for our ToolImplClass which holds the function pointers to our virtual functions.
#[repr(C)]
pub struct ToolImplClass {
    pub parent_class: glib::object::GObjectClass,
    // If these functions are meant to be called from C, you need to make these functions
    // `unsafe extern "C" fn` & use FFI-safe types (usually raw pointers).
    pub on_button_press_event:
        fn(&ToolImplInstance, GlyphEditView, &Canvas, &gtk::gdk::EventButton) -> Inhibit,
    pub on_button_release_event:
        fn(&ToolImplInstance, GlyphEditView, &Canvas, &gtk::gdk::EventButton) -> Inhibit,
    pub on_scroll_event:
        fn(&ToolImplInstance, GlyphEditView, &Canvas, &gtk::gdk::EventScroll) -> Inhibit,
    pub on_motion_notify_event:
        fn(&ToolImplInstance, GlyphEditView, &Canvas, &gtk::gdk::EventMotion) -> Inhibit,
    pub setup_toolbox: fn(&ToolImplInstance, &gtk::Toolbar, &GlyphEditView),
    pub on_activate: fn(&ToolImplInstance, &GlyphEditView),
    pub on_deactivate: fn(&ToolImplInstance, &GlyphEditView),
}

unsafe impl ClassStruct for ToolImplClass {
    type Type = ToolImplInner;
}

#[derive(Default)]
pub struct ToolImplInner {
    name: OnceCell<Cow<'static, str>>,
    description: OnceCell<Cow<'static, str>>,
    icon: OnceCell<gtk::Image>,
}

static TOOL_PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
    once_cell::sync::Lazy::new(|| {
        vec![
            glib::ParamSpecString::new(
                ToolImpl::NAME,
                ToolImpl::NAME,
                ToolImpl::NAME,
                None,
                glib::ParamFlags::READWRITE,
            ),
            glib::ParamSpecString::new(
                ToolImpl::DESCRIPTION,
                ToolImpl::DESCRIPTION,
                ToolImpl::DESCRIPTION,
                None,
                glib::ParamFlags::READWRITE,
            ),
            glib::ParamSpecObject::new(
                ToolImpl::ICON,
                ToolImpl::ICON,
                ToolImpl::ICON,
                gtk::Image::static_type(),
                glib::ParamFlags::READWRITE,
            ),
        ]
    });

impl std::fmt::Debug for ToolImplInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ToolImplInner").finish()
    }
}

// Virtual method default implementation trampolines
fn on_button_press_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventButton,
) -> Inhibit {
    this.imp().on_button_press_event(view, viewport, event)
}

fn on_button_release_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventButton,
) -> Inhibit {
    this.imp().on_button_release_event(view, viewport, event)
}

fn on_scroll_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventScroll,
) -> Inhibit {
    this.imp().on_scroll_event(view, viewport, event)
}

fn on_motion_notify_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventMotion,
) -> Inhibit {
    this.imp().on_motion_notify_event(view, viewport, event)
}

fn setup_toolbox_default_trampoline(
    this: &ToolImplInstance,
    toolbox: &gtk::Toolbar,
    view: &GlyphEditView,
) {
    this.imp().setup_toolbox(toolbox, view)
}

fn on_activate_default_trampoline(this: &ToolImplInstance, view: &GlyphEditView) {
    this.imp().on_activate(view)
}

fn on_deactivate_default_trampoline(this: &ToolImplInstance, view: &GlyphEditView) {
    this.imp().on_deactivate(view)
}

pub fn base_on_button_press_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventButton,
) -> Inhibit {
    let klass = this.class();
    (klass.as_ref().on_button_press_event)(this, view, viewport, event)
}

pub fn base_on_button_release_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventButton,
) -> Inhibit {
    let klass = this.class();
    (klass.as_ref().on_button_release_event)(this, view, viewport, event)
}

pub fn base_on_scroll_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventScroll,
) -> Inhibit {
    let klass = this.class();
    (klass.as_ref().on_scroll_event)(this, view, viewport, event)
}

pub fn base_on_motion_notify_event_default_trampoline(
    this: &ToolImplInstance,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventMotion,
) -> Inhibit {
    let klass = this.class();
    (klass.as_ref().on_motion_notify_event)(this, view, viewport, event)
}

pub fn base_setup_toolbox_default_trampoline(
    this: &ToolImplInstance,
    toolbox: &gtk::Toolbar,
    view: &GlyphEditView,
) {
    let klass = this.class();
    (klass.as_ref().setup_toolbox)(this, toolbox, view)
}

pub fn base_on_activate_default_trampoline(this: &ToolImplInstance, view: &GlyphEditView) {
    let klass = this.class();
    (klass.as_ref().on_activate)(this, view)
}

pub fn base_on_deactivate_default_trampoline(this: &ToolImplInstance, view: &GlyphEditView) {
    let klass = this.class();
    (klass.as_ref().on_deactivate)(this, view)
}

/// Default implementations.
impl ToolImplInner {
    fn on_button_press_event(
        &self,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_button_release_event(
        &self,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_scroll_event(
        &self,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn on_motion_notify_event(
        &self,
        _view: GlyphEditView,
        _viewport: &Canvas,
        _event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        Inhibit(false)
    }

    fn setup_toolbox(&self, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        let obj = self.instance();
        let name = obj.property::<String>(ToolImpl::NAME);
        let button = gtk::ToggleToolButton::builder()
            .icon_widget(&obj.property::<gtk::Image>(ToolImpl::ICON))
            .label(&name)
            .visible(true)
            .active(false)
            .tooltip_text(&name)
            .build();
        button.connect_clicked(
            clone!(@strong view, @strong obj as self_ => move |_button| {
                self_.on_activate(&view)
            }),
        );
        button.connect_toggled(clone!(@strong toolbar => move |button| {
            if button.is_active() {
                button.style_context().add_class("active");
                for child in toolbar.children() {
                    if &child == button {
                        continue;
                    }
                    if let Ok(child) = child.downcast::<gtk::ToggleToolButton>() {
                        child.set_active(false);
                    }
                }
            } else {
                button.style_context().remove_class("active");
            }
        }));
        toolbar.add(&button);
        toolbar.set_item_homogeneous(&button, false);
    }

    fn on_activate(&self, view: &GlyphEditView) {
        let t = self.instance().type_();
        let active_tool = view.imp().glyph_state.get().unwrap().borrow().active_tool;
        if active_tool != t {
            if let Some(previous_tool) = view
                .imp()
                .glyph_state
                .get()
                .unwrap()
                .borrow()
                .tools
                .get(&active_tool)
                .map(Clone::clone)
            {
                previous_tool.on_deactivate(view);
            }

            view.imp()
                .glyph_state
                .get()
                .unwrap()
                .borrow_mut()
                .active_tool = t;
        }
    }

    fn on_deactivate(&self, _view: &GlyphEditView) {}
}

#[glib::object_subclass]
impl ObjectSubclass for ToolImplInner {
    const NAME: &'static str = "ToolImpl";
    type ParentType = glib::Object;
    type Type = ToolImpl;
    type Class = ToolImplClass;

    fn class_init(klass: &mut Self::Class) {
        klass.on_button_press_event = on_button_press_event_default_trampoline;
        klass.on_button_release_event = on_button_release_event_default_trampoline;
        klass.on_scroll_event = on_scroll_event_default_trampoline;
        klass.on_motion_notify_event = on_motion_notify_event_default_trampoline;
        klass.on_button_press_event = on_button_press_event_default_trampoline;
        klass.setup_toolbox = setup_toolbox_default_trampoline;
        klass.on_activate = on_activate_default_trampoline;
        klass.on_deactivate = on_deactivate_default_trampoline;
    }
}

impl ObjectImpl for ToolImplInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        TOOL_PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            ToolImpl::NAME => self.name.get().unwrap().as_ref().to_value(),
            ToolImpl::DESCRIPTION => self.description.get().map(Cow::as_ref).to_value(),
            ToolImpl::ICON => self.icon.get().unwrap().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            ToolImpl::NAME => self
                .name
                .set(value.get::<String>().unwrap().into())
                .unwrap(),
            ToolImpl::DESCRIPTION => self
                .description
                .set(value.get::<String>().unwrap().into())
                .unwrap(),
            ToolImpl::ICON => self.icon.set(value.get().unwrap()).unwrap(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

glib::wrapper! {
    pub struct ToolImpl(ObjectSubclass<ToolImplInner>);
}

impl ToolImpl {
    pub const NAME: &str = "name";
    pub const DESCRIPTION: &str = "description";
    pub const ICON: &str = "icon";

    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl Default for ToolImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Public trait that implements our functions for everything that derives from ToolImpl
pub trait ToolImplExt {
    fn on_button_press_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit;
    fn on_button_release_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit;
    fn on_scroll_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit;
    fn on_motion_notify_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit;
    fn setup_toolbox(&self, toolbar: &gtk::Toolbar, view: &GlyphEditView);
    fn on_activate(&self, view: &GlyphEditView);
    fn on_deactivate(&self, view: &GlyphEditView);
}

/// We call into ToolImplInner_$method_name for each function. These will retrieve the
/// correct class (the base class for the ToolImpl or the derived class for DerivedButton)
/// and call the correct implementation of the function.
impl<O: IsA<ToolImpl>> ToolImplExt for O {
    fn on_button_press_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        base_on_button_press_event_default_trampoline(
            self.upcast_ref::<ToolImpl>(),
            view,
            viewport,
            event,
        )
    }

    fn on_button_release_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        base_on_button_release_event_default_trampoline(
            self.upcast_ref::<ToolImpl>(),
            view,
            viewport,
            event,
        )
    }

    fn on_scroll_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        base_on_scroll_event_default_trampoline(
            self.upcast_ref::<ToolImpl>(),
            view,
            viewport,
            event,
        )
    }

    fn on_motion_notify_event(
        &self,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        base_on_motion_notify_event_default_trampoline(
            self.upcast_ref::<ToolImpl>(),
            view,
            viewport,
            event,
        )
    }

    fn setup_toolbox(&self, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        base_setup_toolbox_default_trampoline(self.upcast_ref::<ToolImpl>(), toolbar, view)
    }

    fn on_activate(&self, view: &GlyphEditView) {
        base_on_activate_default_trampoline(self.upcast_ref::<ToolImpl>(), view)
    }

    fn on_deactivate(&self, view: &GlyphEditView) {
        base_on_deactivate_default_trampoline(self.upcast_ref::<ToolImpl>(), view)
    }
}

/// The ToolImplImpl that each derived private struct has to implement. See derived_button/imp.rs for how
/// to override functions.
pub trait ToolImplImpl: ObjectImpl + 'static {
    fn on_button_press_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        self.parent_on_button_press_event(obj, view, viewport, event)
    }

    fn on_button_release_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        self.parent_on_button_release_event(obj, view, viewport, event)
    }

    fn on_scroll_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        self.parent_on_scroll_event(obj, view, viewport, event)
    }

    fn on_motion_notify_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        self.parent_on_motion_notify_event(obj, view, viewport, event)
    }

    fn setup_toolbox(&self, obj: &ToolImpl, toolbar: &gtk::Toolbar, view: &GlyphEditView) {
        self.parent_setup_toolbox(obj, toolbar, view)
    }

    fn on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.parent_on_activate(obj, view)
    }

    fn on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        self.parent_on_deactivate(obj, view)
    }
}

pub trait ToolImplImplExt: ObjectSubclass {
    fn parent_on_button_press_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit;
    fn parent_on_button_release_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit;
    fn parent_on_scroll_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit;
    fn parent_on_motion_notify_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit;
    fn parent_setup_toolbox(&self, obj: &ToolImpl, toolbox: &gtk::Toolbar, view: &GlyphEditView);
    fn parent_on_activate(&self, obj: &ToolImpl, view: &GlyphEditView);
    fn parent_on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView);
}

impl<T: ToolImplImpl> ToolImplImplExt for T {
    fn parent_on_button_press_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.on_button_press_event)(obj, view, viewport, event)
        }
    }

    fn parent_on_button_release_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.on_button_release_event)(obj, view, viewport, event)
        }
    }

    fn parent_on_scroll_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventScroll,
    ) -> Inhibit {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.on_scroll_event)(obj, view, viewport, event)
        }
    }

    fn parent_on_motion_notify_event(
        &self,
        obj: &ToolImpl,
        view: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.on_motion_notify_event)(obj, view, viewport, event)
        }
    }

    fn parent_setup_toolbox(&self, obj: &ToolImpl, toolbox: &gtk::Toolbar, view: &GlyphEditView) {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.setup_toolbox)(obj, toolbox, view)
        }
    }

    fn parent_on_activate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.on_activate)(obj, view)
        }
    }

    fn parent_on_deactivate(&self, obj: &ToolImpl, view: &GlyphEditView) {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut ToolImplClass);
            (parent_class.on_deactivate)(obj, view)
        }
    }
}

/// Make the ToolImpl subclassable
unsafe impl<T: ToolImplImpl> IsSubclassable<T> for ToolImpl {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.on_button_press_event = on_button_press_event_trampoline::<T>;
        klass.on_button_release_event = on_button_release_event_trampoline::<T>;
        klass.on_scroll_event = on_scroll_event_trampoline::<T>;
        klass.on_motion_notify_event = on_motion_notify_event_trampoline::<T>;
        klass.setup_toolbox = setup_toolbox_trampoline::<T>;
        klass.on_activate = on_activate_trampoline::<T>;
        klass.on_deactivate = on_deactivate_trampoline::<T>;
    }
}

// Virtual method implementation trampolines

fn on_button_press_event_trampoline<T>(
    this: &ToolImpl,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventButton,
) -> Inhibit
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.on_button_press_event(this, view, viewport, event)
}

fn on_button_release_event_trampoline<T>(
    this: &ToolImpl,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventButton,
) -> Inhibit
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.on_button_release_event(this, view, viewport, event)
}

fn on_scroll_event_trampoline<T>(
    this: &ToolImpl,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventScroll,
) -> Inhibit
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.on_scroll_event(this, view, viewport, event)
}

fn on_motion_notify_event_trampoline<T>(
    this: &ToolImpl,
    view: GlyphEditView,
    viewport: &Canvas,
    event: &gtk::gdk::EventMotion,
) -> Inhibit
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.on_motion_notify_event(this, view, viewport, event)
}

fn setup_toolbox_trampoline<T>(this: &ToolImpl, toolbox: &gtk::Toolbar, view: &GlyphEditView)
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.setup_toolbox(this, toolbox, view)
}

fn on_activate_trampoline<T>(this: &ToolImpl, view: &GlyphEditView)
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.on_activate(this, view)
}

fn on_deactivate_trampoline<T>(this: &ToolImpl, view: &GlyphEditView)
where
    T: ObjectSubclass + ToolImplImpl,
{
    let imp = this.dynamic_cast_ref::<T::Type>().unwrap().imp();
    imp.on_deactivate(this, view)
}
