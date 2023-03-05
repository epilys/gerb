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

use glib::{
    clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecString, Value,
};
use gtk::cairo::{Context, FontSlant, FontWeight};
use once_cell::unsync::OnceCell;
use std::collections::HashMap;

use crate::glyphs::{Glyph, GlyphDrawingOptions, GlyphKind};
use crate::prelude::*;
use crate::unicode::blocks::*;

const GLYPH_BOX_WIDTH: f64 = 110.0;
const GLYPH_BOX_HEIGHT: f64 = 140.0;

#[derive(Debug, Default)]
pub struct CollectionInner {
    app: OnceCell<Application>,
    project: OnceCell<Project>,
    flow_box: gtk::FlowBox,
    tree: gtk::TreeView,
    tree_store: OnceCell<gtk::TreeStore>,
    show_blocks: RefCell<HashMap<&'static str, bool>>,
    hide_empty: Cell<bool>,
    zoom_factor: Cell<f64>,
    filter_input: RefCell<Option<String>>,
    widgets: RefCell<Vec<GlyphBox>>,
    title: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for CollectionInner {
    const NAME: &'static str = "Collection";
    type Type = Collection;
    type ParentType = gtk::EventBox;
}

impl ObjectImpl for CollectionInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        *self.filter_input.borrow_mut() = None;

        self.flow_box.set_max_children_per_line(150);
        self.flow_box.set_expand(true);
        self.flow_box.set_visible(true);
        self.flow_box.set_can_focus(true);
        self.flow_box.set_column_spacing(0);
        self.flow_box.set_row_spacing(0);

        let overlay = gtk::Overlay::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .margin_top(5)
            .margin_start(5)
            .build();

        scrolled_window.set_child(Some(&self.flow_box));

        let tool_palette = gtk::Toolbar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .expand(true)
            .halign(gtk::Align::End)
            .valign(gtk::Align::End)
            .visible(true)
            .can_focus(true)
            .build();
        let zoom_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.1, 2.0, 0.1);
        zoom_scale.set_visible(true);
        zoom_scale.set_value(1.0);
        zoom_scale.connect_value_changed(clone!(@weak obj => move |_self| {
            let value = _self.value();
            obj.set_property(Collection::ZOOM_FACTOR, value);
            obj.update_flow_box();
        }));

        let zoom_pop_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .expand(false)
            .visible(true)
            .can_focus(true)
            .build();

        let close_zoom_pop_box = gtk::Button::builder()
            .label("Close")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .visible(true)
            .build();

        zoom_pop_box.pack_start(&close_zoom_pop_box, false, false, 0);
        zoom_pop_box.pack_start(&zoom_scale, true, false, 0);

        let zoom_pop = gtk::Popover::builder()
            .expand(false)
            .visible(false)
            .modal(true)
            .child(&zoom_pop_box)
            .relative_to(&tool_palette)
            .width_request(200)
            .build();
        let show_zoom_pop = gtk::ToolButton::builder()
            .label("Scale")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Start)
            .visible(true)
            .build();
        show_zoom_pop.connect_clicked(clone!(@strong zoom_pop => move |_| {
            zoom_pop.show();
        }));
        close_zoom_pop_box.connect_clicked(clone!(@strong zoom_pop => move |_| {
            zoom_pop.hide();
        }));

        tool_palette.add(&show_zoom_pop);
        tool_palette.set_item_homogeneous(&show_zoom_pop, false);

        let hide_empty_button = gtk::ToggleToolButton::builder()
            .label("Hide empty glyphs")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Start)
            .visible(true)
            .active(false)
            .build();
        hide_empty_button.connect_toggled(clone!(@weak obj => move |_| {
            let hide_empty = !obj.imp().hide_empty.get();
            obj.imp().hide_empty.set(hide_empty);
            obj.update_flow_box();
            obj.imp().flow_box.queue_draw();
        }));

        tool_palette.add(&hide_empty_button);
        tool_palette.set_item_homogeneous(&hide_empty_button, false);

        let add_glyph_button = gtk::ToolButton::builder()
            .label("Add glyph")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Start)
            .visible(true)
            .build();

        add_glyph_button.connect_clicked(clone!(@weak obj => move |_| {
            let metadata = GlyphMetadata::new();
            let w = metadata.new_property_window(obj.app());
            if let PropertyWindowButtons::Create { cancel: _, ref save } = w.imp().buttons.get().unwrap() {
                save.connect_clicked(clone!(@weak metadata, @weak w, @weak obj => move |_| {
                    let project = obj.project();
                    let name = metadata.name().to_string();
                    let glyph = Rc::new(RefCell::new(metadata.clone().into()));
                    metadata.glyph_ref.set(glyph.clone()).unwrap();
                    project.glyphs.borrow_mut().insert(name, glyph);
                    obj.emit_by_name::<()>(Collection::NEW_GLYPH, &[&metadata]);
                    w.close();
                }));
            }
            w.present();
        }));

        tool_palette.add(&add_glyph_button);
        tool_palette.set_item_homogeneous(&add_glyph_button, false);

        let search_entry = gtk::Entry::builder()
            .expand(true)
            .visible(true)
            .placeholder_text("Filter glyph name")
            .build();

        search_entry.connect_changed(clone!(@weak obj => move |_self| {
            let filter_input = if _self.buffer().length() == 0 {
                None
            } else {
                Some(_self.buffer().text())
            };
            let mut cur_input = obj.imp().filter_input.borrow_mut();
            if cur_input.as_ref() != filter_input.as_ref() {
                *cur_input = filter_input;
                drop(cur_input);
                obj.update_flow_box();
                obj.imp().flow_box.queue_draw();
            }
        }));

        tool_palette.add(
            &gtk::ToolItem::builder()
                .visible(true)
                .child(&search_entry)
                .build(),
        );

        self.tree.set_visible(true);
        self.tree.set_grid_lines(gtk::TreeViewGridLines::Both);
        let store = gtk::TreeStore::new(&[
            bool::static_type(),
            String::static_type(),
            i64::static_type(),
        ]);

        let append_text_column = |tree: &gtk::TreeView, store: &gtk::TreeStore| {
            let column = gtk::TreeViewColumn::new();
            let cell = gtk::CellRendererToggle::new();
            cell.set_radio(false);
            cell.set_active(true);
            cell.set_activatable(true);
            cell.connect_toggled(clone!(@strong store, @weak obj => move |_self, treepath| {
                if let Some(iter) = store.iter(&treepath) {
                    let prev_value: bool = store.value(&iter, 0).get().unwrap();
                    let cat_value = store.value(&iter, 1);
                    let block_category: &str = cat_value.get().unwrap();
                    let new_value = !prev_value;
                    store.set_value(&iter, 0, &new_value.to_value());
                    let mut update = false;
                    if let Some(v) = obj.imp().show_blocks.borrow_mut().get_mut(block_category) {
                        *v = new_value;
                        update = true;
                    }
                    if update {
                        obj.update_flow_box();
                    }
                }
            }));
            column.pack_start(&cell, true);
            column.add_attribute(&cell, "active", 0);
            tree.append_column(&column);

            let column = gtk::TreeViewColumn::new();
            let cell = gtk::CellRendererText::new();

            column.pack_start(&cell, true);
            column.add_attribute(&cell, "text", 1);
            tree.append_column(&column);

            let column = gtk::TreeViewColumn::new();
            let cell = gtk::CellRendererText::new();

            column.pack_start(&cell, true);
            column.add_attribute(&cell, "text", 2);
            tree.append_column(&column);
        };
        self.tree.set_model(Some(&store));
        self.tree.set_headers_visible(false);
        append_text_column(&self.tree, &store);

        let filter_pop_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .expand(true)
            .visible(true)
            .can_focus(true)
            .build();
        let tree_scrolled_window = gtk::ScrolledWindow::builder()
            .expand(true)
            .visible(true)
            .can_focus(true)
            .min_content_height(30)
            .min_content_width(50)
            .margin_top(5)
            .margin_start(5)
            .build();

        tree_scrolled_window.set_child(Some(&self.tree));
        let close_filter_pop_box = gtk::Button::builder()
            .label("Close")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .visible(true)
            .build();

        filter_pop_box.pack_start(&close_filter_pop_box, false, false, 0);
        filter_pop_box.pack_start(&tree_scrolled_window, true, true, 0);

        let filter_pop = gtk::Popover::builder()
            .expand(true)
            .visible(false)
            .modal(true)
            .child(&filter_pop_box)
            .relative_to(&tool_palette)
            .build();
        let show_filter_pop = gtk::ToolButton::builder()
            .label("Filter...")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Start)
            .visible(true)
            .build();
        show_filter_pop.connect_clicked(clone!(@strong filter_pop => move |_| {
            filter_pop.show();
        }));
        close_filter_pop_box.connect_clicked(clone!(@strong filter_pop => move |_| {
            filter_pop.hide();
        }));

        tool_palette.add(&show_filter_pop);
        tool_palette.set_item_homogeneous(&show_filter_pop, false);

        tool_palette
            .style_context()
            .add_class("glyphs_area_toolbar");

        overlay.set_child(Some(&scrolled_window));
        overlay.add_overlay(
            &gtk::Expander::builder()
                .child(&tool_palette)
                .expanded(true)
                .visible(true)
                .can_focus(true)
                .tooltip_text("Overview tools")
                .halign(gtk::Align::End)
                .valign(gtk::Align::End)
                .build(),
        );
        obj.set_child(Some(&overlay));
        self.hide_empty.set(false);
        obj.set_property(Collection::ZOOM_FACTOR, 1.0);
        self.tree_store.set(store).unwrap();
        obj.connect_local(
            Collection::NEW_GLYPH,
            false,
            clone!(@weak obj => @default-return None, move |v: &[gtk::glib::Value]| {
                let metadata = v[1].get::<GlyphMetadata>().unwrap();
                let glyph = metadata.glyph_ref.get().unwrap().clone();
                {
                    let glyph_box = GlyphBox::new(obj.app().clone(), obj.project().clone(), glyph);
                    obj.bind_property(Collection::ZOOM_FACTOR, &glyph_box, GlyphBox::ZOOM_FACTOR)
                        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
                        .build();
                    obj.imp().flow_box.add(&glyph_box);
                    obj.imp().widgets.borrow_mut().push(glyph_box);
                }
                obj.update_flow_box();
                obj.update_tree_store();
                obj.imp().flow_box.queue_draw();
                obj.queue_draw();

                None
            }),
        );
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        Collection::TITLE,
                        Collection::TITLE,
                        Collection::TITLE,
                        Some("collection"),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        Collection::CLOSEABLE,
                        Collection::CLOSEABLE,
                        Collection::CLOSEABLE,
                        false,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecDouble::new(
                        Collection::ZOOM_FACTOR,
                        Collection::ZOOM_FACTOR,
                        Collection::ZOOM_FACTOR,
                        0.0,
                        std::f64::MAX,
                        1.0,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            Collection::TITLE => self.title.borrow().to_value(),
            Collection::CLOSEABLE => false.to_value(),
            Collection::ZOOM_FACTOR => self.zoom_factor.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            Collection::TITLE => {
                *self.title.borrow_mut() = value.get().unwrap();
            }
            Collection::ZOOM_FACTOR => self.zoom_factor.set(value.get().unwrap()),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                Collection::NEW_GLYPH,
                &[GlyphMetadata::static_type().into()],
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for CollectionInner {}
impl ContainerImpl for CollectionInner {}
impl BinImpl for CollectionInner {}
impl EventBoxImpl for CollectionInner {}

impl CollectionInner {
    pub fn app(&self) -> &Application {
        self.app.get().unwrap()
    }

    pub fn project(&self) -> &Project {
        self.project.get().unwrap()
    }
}

impl std::ops::Deref for Collection {
    type Target = CollectionInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

glib::wrapper! {
    pub struct Collection(ObjectSubclass<CollectionInner>)
        @extends gtk::Widget, gtk::Container, gtk::EventBox;
}

impl Collection {
    pub const TITLE: &str = Workspace::TITLE;
    pub const CLOSEABLE: &str = Workspace::CLOSEABLE;
    pub const ZOOM_FACTOR: &str = "zoom-factor";
    pub const NEW_GLYPH: &str = "new-glyph";

    pub fn new(app: Application, project: Project) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        project
            .bind_property(Project::STYLE_NAME, &ret, Self::TITLE)
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
            .build();
        let flow_box = &ret.imp().flow_box;
        let mut widgets = vec![];
        {
            let glyphs_b = project.imp().glyphs.borrow();
            let mut glyphs = glyphs_b.values().collect::<Vec<&Rc<RefCell<Glyph>>>>();
            glyphs.sort();
            for glyph in glyphs {
                let glyph_box = GlyphBox::new(app.clone(), project.clone(), glyph.clone());
                ret.bind_property(Self::ZOOM_FACTOR, &glyph_box, GlyphBox::ZOOM_FACTOR)
                    .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
                    .build();
                flow_box.add(&glyph_box);
                widgets.push(glyph_box);
            }
        }
        ret.imp().app.set(app).unwrap();
        ret.imp().project.set(project).unwrap();
        *ret.imp().widgets.borrow_mut() = widgets;
        ret.update_flow_box();
        ret.update_tree_store();
        ret
    }

    fn update_tree_store(&self) {
        let tree_store = self.imp().tree_store.get().unwrap();
        let mut show_blocks = self.imp().show_blocks.borrow_mut();
        show_blocks.clear();
        tree_store.clear();
        let mut blocks_set: std::collections::HashMap<&'static str, usize> = Default::default();
        for c in self.imp().widgets.borrow().iter() {
            let glyph = c.imp().glyph.get().unwrap().borrow();
            if let GlyphKind::Char(c) = glyph.kinds().0 {
                if let Some(idx) = c.char_block() {
                    *blocks_set.entry(UNICODE_BLOCKS[idx].1).or_default() += 1;
                } else {
                    *blocks_set.entry("Unknown").or_default() += 1;
                }
                *blocks_set.entry("Unicode").or_default() += 1;
            } else {
                *blocks_set.entry("Components").or_default() += 1;
            };
        }
        if let Some(c) = blocks_set.remove("Components") {
            show_blocks.insert("Components", true);
            tree_store.insert_with_values(
                None,
                None,
                &[(0, &true), (1, &"Components"), (2, &(c as i64))],
            );
        }
        let unicode = if let Some(c) = blocks_set.remove("Unicode") {
            show_blocks.insert("Unicode", true);
            Some(tree_store.insert_with_values(
                None,
                None,
                &[(0, &true), (1, &"Unicode"), (2, &(c as i64))],
            ))
        } else {
            None
        };
        let mut block_set: Vec<(&'static str, usize)> = blocks_set.into_iter().collect();
        block_set.sort_by_key(|e| e.1);
        block_set.reverse();
        for (block_name, count) in block_set {
            show_blocks.insert(block_name, true);
            tree_store.insert_with_values(
                unicode.as_ref(),
                None,
                &[(0, &true), (1, &block_name), (2, &(count as i64))],
            );
        }
    }

    fn update_flow_box(&self) {
        let hide_empty: bool = self.imp().hide_empty.get();
        let zoom_factor: f64 = self.imp().zoom_factor.get();
        let show_blocks = self.imp().show_blocks.clone();
        let filter_input = self.imp().filter_input.clone();
        let filter_input_uppercase: Option<String> = filter_input
            .borrow()
            .as_ref()
            .map(|s| s.to_ascii_uppercase());
        let filter_input_char: Option<char> = filter_input.borrow().as_ref().and_then(|s| {
            let mut iter = s.chars();
            let first = iter.next()?;
            if iter.next().is_some() {
                None
            } else {
                Some(first)
            }
        });
        let flow_box = &self.imp().flow_box;
        flow_box.set_filter_func(Some(Box::new(
            clone!(@weak self as _self => @default-return true, move |flowbox: &gtk::FlowBoxChild| {
                let child = flowbox.child();
                if let Some(c) = child
                    .as_ref()
                    .and_then(|w| w.downcast_ref::<GlyphBox>())
                {
                    c.set_height_request((zoom_factor * GLYPH_BOX_HEIGHT) as i32);
                    c.set_width_request((zoom_factor * GLYPH_BOX_WIDTH) as i32);
                    c.queue_draw();
                    let show_blocks = show_blocks.borrow();
                    let filter_input = filter_input.borrow();
                    let glyph = c.imp().glyph.get().unwrap().borrow();
                    if hide_empty && glyph.is_empty() {
                        return false;
                    }
                    if !match glyph.kinds().0 {
                        GlyphKind::Component(_) => *show_blocks.get("Components").unwrap_or(&true),
                        GlyphKind::Char(c) => {
                            *show_blocks.get("Unicode").unwrap_or(&true)
                                && *c
                                    .char_block()
                                    .and_then(|idx| show_blocks.get(UNICODE_BLOCKS[idx].1))
                                    .unwrap_or(&true)
                        }
                    } {
                        return false;
                    }

                    if let (Some(f), Some(fu)) =
                        (filter_input.as_ref(), filter_input_uppercase.as_ref())
                    {
                        if !(glyph.name().contains(f.as_str())
                            && !glyph
                                .name()
                                .contains(fu.as_str())
                            && !filter_input_char
                                .as_ref()
                                .map(|c| glyph.kinds().0 == GlyphKind::Char(*c))
                                .unwrap_or(false))
                        {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }),
        )));
        flow_box.queue_draw();
    }
}

#[derive(Debug, Default)]
pub struct GlyphBoxInner {
    pub app: OnceCell<Application>,
    pub project: OnceCell<Project>,
    pub glyph: OnceCell<Rc<RefCell<Glyph>>>,
    pub focused: Cell<bool>,
    modified: Cell<bool>,
    mark_color: Cell<Color>,
    pub zoom_factor: Cell<f64>,
    pub show_details: Cell<bool>,
    pub drawing_area: gtk::DrawingArea,
}

#[glib::object_subclass]
impl ObjectSubclass for GlyphBoxInner {
    const NAME: &'static str = "GlyphBox";
    type Type = GlyphBox;
    type ParentType = gtk::EventBox;
}

impl ObjectImpl for GlyphBoxInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_height_request(GLYPH_BOX_HEIGHT as _);
        obj.set_width_request(GLYPH_BOX_WIDTH as _);
        obj.set_can_focus(true);
        obj.set_expand(false);
        obj.set_halign(gtk::Align::Start);
        obj.set_valign(gtk::Align::Start);

        obj.connect_button_press_event(
            clone!(@weak obj => @default-return Inhibit(false), move |_self, event| {
                    match event.button() {
                        gtk::gdk::BUTTON_SECONDARY => {
                            let context_menu = crate::utils::menu::Menu::new()
                                .add_button_cb(
                                    "Edit in canvas",
                                    clone!(@strong obj => move |_| {
                                        obj.emit_open_glyph_edit();
                                    }),
                                )
                                .add_button("Edit properties")
                                .add_button("Delete glyph")
                                .add_button("Export SVG");
                            context_menu.popup(event.time());
                        }
                        gtk::gdk::BUTTON_PRIMARY => {
                            obj.emit_open_glyph_edit();
                        }
                        _ => return Inhibit(false),
                    }
                    Inhibit(true)
            }),
        );
        self.drawing_area.set_expand(true);
        self.drawing_area.set_visible(true);
        self.drawing_area.set_can_focus(true);
        self.drawing_area.set_has_tooltip(true);
        self.drawing_area.connect_query_tooltip(
            clone!(@weak obj => @default-return false, move |_self, _x: i32, _y: i32, _by_keyboard: bool, tooltip| {
                let glyph = obj.imp().glyph.get().unwrap().borrow();
                if let GlyphKind::Char(c) = glyph.kinds().0 {
                    let block_name = if let Some(idx) = c.char_block() {
                        UNICODE_BLOCKS[idx].1
                    } else {
                        "Unknown"
                    };
                    let unicode = format!("U+{:04X}", c as u32);

                    tooltip.set_text(Some(&format!("Name: {}\nUnicode: {}\nBlock: {}", glyph.name(), unicode, block_name)));
                } else {
                    tooltip.set_text(Some(&format!("Name: {}\nComponent", glyph.name())));
                }
                true
            }));
        self.drawing_area.connect_draw(clone!(@weak obj => @default-return Inhibit(false), move |viewport: &gtk::DrawingArea, mut ctx: &Context| {
            let mut cr = ctx.push();
            cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
            let is_focused: bool = obj.imp().focused.get();
            let zoom_factor: f64 = obj.imp().zoom_factor.get();
            let units_per_em = obj.imp().project.get().unwrap().property(Project::UNITS_PER_EM);

            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint().expect("Invalid cairo surface state");

            let (x, y) = (0.01, 0.01);
            let glyph = obj.imp().glyph.get().unwrap().borrow();
            let label = match glyph.kinds().0 {
                GlyphKind::Char(c) => c.to_string(),
                GlyphKind::Component(ref n) => n.to_string(),
            };
            let label = label.replace('\0', "").trim().to_string();
            cr.set_line_width(1.5);
            let (point, (width, height)) = crate::utils::draw_round_rectangle(cr.push(), (x, y).into(), (zoom_factor * GLYPH_BOX_WIDTH, zoom_factor * GLYPH_BOX_HEIGHT), 1.0, 1.5);
            let glyph_width = glyph.width().unwrap_or(units_per_em) * (width * 0.8) / units_per_em;
            if is_focused {
                cr.set_source_rgb(1.0, 250.0 / 255.0, 141.0 / 255.0);
            } else {
                cr.set_source_rgb(1.0, 1.0, 1.0);
            }
            cr.fill_preserve().expect("Invalid cairo surface state");
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.stroke_preserve().expect("Invalid cairo surface state");
            cr.clip();
            cr.new_path();
            let mark_color = obj.imp().mark_color.get();
            if mark_color.is_visible() {
                use crate::app::settings::types::MarkColor;
                let app = obj.imp().app.get().unwrap();
                let settings = &app.settings.borrow();
                match settings.property::<MarkColor>(Settings::MARK_COLOR) {
                    MarkColor::None => {},
                    MarkColor::Background => {
                        let cr1 = cr.push();
                        cr1.set_source_color_alpha(mark_color);
                        cr1.rectangle(width - width / 10.0, width / 10.0, width / 10.0, width / 10.0);
                        cr1.fill().unwrap();
                    },
                    MarkColor::Icon => {
                        let cr1 = cr.push();
                        cr1.set_source_color_alpha(mark_color);
                        let scale_factor = viewport.scale_factor();
                        if let Some(icon) = crate::resources::icons::MARK
                            .to_pixbuf()
                                .and_then(|p| p .scale_simple(32, 32, gtk::gdk_pixbuf::InterpType::Bilinear)) {
                            cr1.mask_surface(
                                &icon.create_surface(scale_factor, viewport.window().as_ref())
                                .unwrap(),
                                width * 0.8,
                                0.0,
                            ).unwrap();
                            cr1.clip_preserve();
                            cr1.paint().unwrap();
                        }
                    },
                }
            }
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
            // View height.
            let vh = viewport.allocated_height() as f64;
            cr.set_font_size(zoom_factor * 62.0);


            /* Draw glyph. */

            let sextents = cr
                .text_extents(&label)
                .expect("Invalid cairo surface state");
            if glyph.is_empty() {
                cr.move_to(point.x + width / 2.0 - sextents.width / 2.0, point.y + (height / 3.0) + 20.0);
                cr.show_text(&label).expect("Invalid cairo surface state");
            } else {
                let mut matrix = gtk::cairo::Matrix::identity();
                matrix.translate((width - glyph_width) / 2.0, 4.5 * vh / 8.0);
                matrix.scale((width * 0.8) / units_per_em, -(width * 0.8) / units_per_em);
                let options = GlyphDrawingOptions {
                    outline: (Color::new_alpha(0, 0, 0, 0), 1.5).into(),
                    inner_fill: Some((Color::new(89, 89, 89), 1.5).into()),
                    highlight: None,
                    matrix,
                    units_per_em,
                    ..Default::default()
                };
                glyph.draw(cr.push(), options);
            }

            /* Draw glyph label */

            cr.set_line_width(2.0);
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.move_to(x, point.y + 2.0 * (height / 3.0));
            cr.line_to(x + width * 1.2, point.y + 2.0 * (height / 3.0));
            cr.stroke().expect("Invalid cairo surface state");
            cr.set_source_rgb(196.0 / 255.0, 196.0 / 255.0, 196.0 / 255.0);
            cr.new_path();
            cr.rectangle(x, point.y + 2.0 * (height / 3.0), width * 1.2, 1.2 * height / 3.0);
            cr.fill().expect("Invalid cairo surface state");
            cr.reset_clip();

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_font_size(zoom_factor * 12.0);
            let sextents = cr
                .text_extents(&label)
                .expect("Invalid cairo surface state");
            cr.move_to(point.x + width / 2.0 - sextents.width / 2.0, point.y + 2.4 * height / 3.0);
            cr.show_text(&label).expect("Invalid cairo surface state");

            let label = match glyph.kinds().0 {
                GlyphKind::Char(c) => format!("U+{:04X}", c as u32),
                GlyphKind::Component(ref n) => n.to_string(),
            };
            let extents = cr
                .text_extents(&label)
                .expect("Invalid cairo surface state");
            cr.move_to(point.x + width / 2.0 - extents.width  / 2.0, point.y + 2.4 * height / 3.0 + 1.5 * sextents.height);
            cr.show_text(&label).expect("Invalid cairo surface state");

            Inhibit(false)
        }
        ));
        obj.set_child(Some(&self.drawing_area));

        obj.set_events(
            gtk::gdk::EventMask::POINTER_MOTION_MASK
                | gtk::gdk::EventMask::ENTER_NOTIFY_MASK
                | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK,
        );
        obj.connect_enter_notify_event(|_self, _event| -> Inhibit {
            if let Some(screen) = _self.window() {
                let display = screen.display();
                screen.set_cursor(Some(
                    &gtk::gdk::Cursor::from_name(&display, "pointer").unwrap(),
                ));
            }
            _self.imp().focused.set(true);
            _self.imp().drawing_area.queue_draw();
            Inhibit(false)
        });

        obj.connect_leave_notify_event(|_self, _event| -> Inhibit {
            if let Some(screen) = _self.window() {
                let display = screen.display();
                screen.set_cursor(Some(
                    &gtk::gdk::Cursor::from_name(&display, "default").unwrap(),
                ));
            }
            _self.imp().focused.set(false);
            _self.imp().drawing_area.queue_draw();

            Inhibit(false)
        });

        self.focused.set(false);
        self.zoom_factor.set(1.0);
        obj.set_visible(true);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        GlyphBox::SHOW_DETAILS,
                        GlyphBox::SHOW_DETAILS,
                        GlyphBox::SHOW_DETAILS,
                        true,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        GlyphBox::ZOOM_FACTOR,
                        GlyphBox::ZOOM_FACTOR,
                        GlyphBox::ZOOM_FACTOR,
                        0.0,
                        std::f64::MAX,
                        1.0,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        GlyphBox::FOCUSED,
                        GlyphBox::FOCUSED,
                        GlyphBox::FOCUSED,
                        false,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        GlyphMetadata::MARK_COLOR,
                        GlyphMetadata::MARK_COLOR,
                        GlyphMetadata::MARK_COLOR,
                        Color::static_type(),
                        glib::ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        GlyphMetadata::MODIFIED,
                        GlyphMetadata::MODIFIED,
                        GlyphMetadata::MODIFIED,
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            GlyphBox::SHOW_DETAILS => self.show_details.get().to_value(),
            GlyphBox::ZOOM_FACTOR => self.zoom_factor.get().to_value(),
            GlyphBox::FOCUSED => self.focused.get().to_value(),
            GlyphMetadata::MARK_COLOR => self.mark_color.get().to_value(),
            GlyphMetadata::MODIFIED => self.modified.get().to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            GlyphBox::SHOW_DETAILS => self.show_details.set(value.get().unwrap()),
            GlyphBox::ZOOM_FACTOR => self.zoom_factor.set(value.get().unwrap()),
            GlyphBox::FOCUSED => self.focused.set(value.get().unwrap()),
            GlyphMetadata::MARK_COLOR => {
                self.mark_color.set(value.get().unwrap());
            }
            GlyphMetadata::MODIFIED => {
                self.modified.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl WidgetImpl for GlyphBoxInner {}
impl ContainerImpl for GlyphBoxInner {}
impl BinImpl for GlyphBoxInner {}
impl EventBoxImpl for GlyphBoxInner {}

impl GlyphBox {
    pub const SHOW_DETAILS: &str = "show-details";
    pub const ZOOM_FACTOR: &str = Collection::ZOOM_FACTOR;
    pub const FOCUSED: &str = "focused";
    pub const MODIFIED: &str = GlyphMetadata::MODIFIED;
    pub const MARK_COLOR: &str = GlyphMetadata::MARK_COLOR;

    fn emit_open_glyph_edit(&self) {
        self.imp()
            .app
            .get()
            .unwrap()
            .window
            .emit_by_name::<()>("open-glyph-edit", &[&self])
    }
}

glib::wrapper! {
    pub struct GlyphBox(ObjectSubclass<GlyphBoxInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::EventBox;
}

impl GlyphBox {
    pub fn new(app: Application, project: Project, glyph: Rc<RefCell<Glyph>>) -> Self {
        let ret: Self = glib::Object::new(&[]).expect("Failed to create Main Window");
        ret.imp().app.set(app).unwrap();
        ret.imp().project.set(project).unwrap();
        {
            let metadata = &glyph.borrow().metadata;
            metadata
                .bind_property(GlyphMetadata::MARK_COLOR, &ret, Self::MARK_COLOR)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            metadata
                .bind_property(GlyphMetadata::MODIFIED, &ret, Self::MODIFIED)
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
        ret.imp().glyph.set(glyph).unwrap();
        ret
    }
}
