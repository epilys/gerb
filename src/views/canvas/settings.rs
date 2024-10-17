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

glib::wrapper! {
    pub struct CanvasSettings(ObjectSubclass<CanvasSettingsInner>);
}

impl std::ops::Deref for CanvasSettings {
    type Target = CanvasSettingsInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

#[derive(Debug, Default)]
pub struct CanvasSettingsInner {
    pub handle_size: Cell<f64>,
    pub line_width: Cell<f64>,
    pub show_grid: Cell<bool>,
    pub show_guidelines: Cell<bool>,
    pub show_handles: Cell<bool>,
    pub show_direction: Cell<bool>,
    pub show_outline: Cell<bool>,
    pub inner_fill: Cell<bool>,
    pub show_total_area: Cell<bool>,
    pub show_rulers: Cell<bool>,
    pub warp_cursor: Cell<bool>,
    pub bg_color: Cell<Color>,
    pub glyph_bbox_bg_color: Cell<Color>,
    pub glyph_inner_fill_color: Cell<Color>,
    pub ruler_fg_color: Cell<Color>,
    pub ruler_indicator_color: Cell<Color>,
    pub ruler_bg_color: Cell<Color>,
    pub outline_options: Cell<DrawOptions>,
    pub handle_options: Cell<DrawOptions>,
    pub smooth_corner_options: Cell<DrawOptions>,
    pub corner_options: Cell<DrawOptions>,
    pub direction_options: Cell<DrawOptions>,
    pub handle_connection_options: Cell<DrawOptions>,
}

#[glib::object_subclass]
impl ObjectSubclass for CanvasSettingsInner {
    const NAME: &'static str = "CanvasSettings";
    type Type = CanvasSettings;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl CanvasSettingsInner {
    pub const HANDLE_SIZE_INIT_VAL: f64 = 5.0;
    pub const LINE_WIDTH_INIT_VAL: f64 = 0.85;
    pub const RULER_BREADTH: f64 = 13.0;
    pub const SHOW_GRID_INIT_VAL: bool = false;
    pub const SHOW_GUIDELINES_INIT_VAL: bool = true;
    pub const SHOW_HANDLES_INIT_VAL: bool = true;
    pub const SHOW_DIRECTION_INIT_VAL: bool = true;
    pub const INNER_FILL_INIT_VAL: bool = false;
    pub const SHOW_TOTAL_AREA_INIT_VAL: bool = true;
    pub const SHOW_RULERS_INIT_VAL: bool = true;
    pub const WARP_CURSOR_INIT_VAL: bool = false;
    pub const RULER_FG_COLOR_INIT_VAL: Color = Color::BLACK;
    pub const RULER_BG_COLOR_INIT_VAL: Color = Color::WHITE;
    pub const RULER_INDICATOR_COLOR_INIT_VAL: Color = Color::RED;

    fn get_opts(&self, retval: DrawOptions) -> DrawOptions {
        if let Some((inherit, true)) = retval.inherit_size {
            DrawOptions {
                size: self.instance().property(inherit),
                ..retval
            }
        } else {
            retval
        }
    }
}

impl ObjectImpl for CanvasSettingsInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.handle_size.set(Self::HANDLE_SIZE_INIT_VAL);
        self.line_width.set(Self::LINE_WIDTH_INIT_VAL);
        self.show_grid.set(Self::SHOW_GRID_INIT_VAL);
        self.show_guidelines.set(Self::SHOW_GUIDELINES_INIT_VAL);
        self.show_handles.set(Self::SHOW_HANDLES_INIT_VAL);
        self.inner_fill.set(Self::INNER_FILL_INIT_VAL);
        self.show_total_area.set(Self::SHOW_TOTAL_AREA_INIT_VAL);
        self.show_rulers.set(Self::SHOW_RULERS_INIT_VAL);
        self.show_direction.set(Self::SHOW_DIRECTION_INIT_VAL);
        self.direction_options
            .set((Color::from_hex("#0478A2").with_alpha_f64(0.9), 2.0).into()); // [ref:hardcoded_color_value]
        self.handle_connection_options.set(DrawOptions::from((
            // [ref:hardcoded_color_value]
            Color::BLACK.with_alpha_f64(0.9),
            1.0,
            CanvasSettings::LINE_WIDTH,
        )));
        self.handle_options.set(
            DrawOptions::from((
                // [ref:hardcoded_color_value]
                Color::from_hex("#333333").with_alpha_f64(0.6),
                5.0,
                CanvasSettings::HANDLE_SIZE,
            ))
            .with_bg(Color::WHITE),
        );
        self.smooth_corner_options.set(
            (
                Color::from_hex("#333333").with_alpha_f64(0.6), // [ref:hardcoded_color_value]
                5.0,
                CanvasSettings::HANDLE_SIZE,
            )
                .into(),
        );
        self.corner_options.set(
            (
                Color::from_hex("#333333").with_alpha_f64(0.6), // [ref:hardcoded_color_value]
                5.0,
                CanvasSettings::HANDLE_SIZE,
            )
                .into(),
        );
        self.outline_options.set(
            (
                Color::from_hex("#333333").with_alpha_f64(0.6), // [ref:hardcoded_color_value]
                5.0,
                CanvasSettings::LINE_WIDTH,
            )
                .into(),
        );
        self.warp_cursor.set(Self::WARP_CURSOR_INIT_VAL);
        self.bg_color.set(Color::WHITE);
        self.bg_color.set(Color::from_hex("#EEF8F8")); // [ref:hardcoded_color_value]
        self.glyph_bbox_bg_color
            .set(Color::new_alpha(210, 227, 252, 153)); // [ref:hardcoded_color_value]
        self.glyph_inner_fill_color.set(Color::from_hex("#E6E6E4")); // [ref:hardcoded_color_value]
        self.ruler_fg_color.set(Self::RULER_FG_COLOR_INIT_VAL); // [ref:hardcoded_color_value]
        self.ruler_bg_color.set(Self::RULER_BG_COLOR_INIT_VAL); // [ref:hardcoded_color_value]
        self.ruler_indicator_color
            .set(Self::RULER_INDICATOR_COLOR_INIT_VAL); // [ref:hardcoded_color_value]
        self.ruler_fg_color.set(Color::from_hex("#8B9494")); // [ref:hardcoded_color_value]
        self.ruler_bg_color.set(Color::from_hex("#F2F8F8")); // [ref:hardcoded_color_value]
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> = once_cell::sync::Lazy::new(
            || {
                vec![
                    ParamSpecDouble::new(
                        CanvasSettings::HANDLE_SIZE,
                        CanvasSettings::HANDLE_SIZE,
                        "Diameter of round control point handle.",
                        0.0001,
                        10.0,
                        CanvasSettingsInner::HANDLE_SIZE_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecDouble::new(
                        CanvasSettings::LINE_WIDTH,
                        CanvasSettings::LINE_WIDTH,
                        "Width of lines in pixels.",
                        0.0001,
                        10.0,
                        CanvasSettingsInner::LINE_WIDTH_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::SHOW_GRID,
                        CanvasSettings::SHOW_GRID,
                        "Show/hide grid.",
                        CanvasSettingsInner::SHOW_GRID_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::SHOW_GUIDELINES,
                        CanvasSettings::SHOW_GUIDELINES,
                        "Show/hide all guidelines.",
                        CanvasSettingsInner::SHOW_GUIDELINES_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::SHOW_HANDLES,
                        CanvasSettings::SHOW_HANDLES,
                        "Show/hide handles.",
                        CanvasSettingsInner::SHOW_HANDLES_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::INNER_FILL,
                        CanvasSettings::INNER_FILL,
                        "Show/hide inner glyph fill.",
                        CanvasSettingsInner::INNER_FILL_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::SHOW_TOTAL_AREA,
                        CanvasSettings::SHOW_TOTAL_AREA,
                        "Show/hide total glyph area.",
                        CanvasSettingsInner::SHOW_TOTAL_AREA_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::SHOW_RULERS,
                        CanvasSettings::SHOW_RULERS,
                        "Show/hide canvas rulers.",
                        CanvasSettingsInner::SHOW_RULERS_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::SHOW_DIRECTION,
                        CanvasSettings::SHOW_DIRECTION,
                        "Show/hide contour direction arrows.",
                        CanvasSettingsInner::SHOW_DIRECTION_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoolean::new(
                        CanvasSettings::WARP_CURSOR,
                        CanvasSettings::WARP_CURSOR,
                        CanvasSettings::WARP_CURSOR,
                        CanvasSettingsInner::WARP_CURSOR_INIT_VAL,
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::BG_COLOR,
                        CanvasSettings::BG_COLOR,
                        "Background color of canvas.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::GLYPH_INNER_FILL_COLOR,
                        CanvasSettings::GLYPH_INNER_FILL_COLOR,
                        "Color of glyph's inner fill.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::GLYPH_BBOX_BG_COLOR,
                        CanvasSettings::GLYPH_BBOX_BG_COLOR,
                        "Background color of glyph's bounding box (total area).",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::RULER_FG_COLOR,
                        CanvasSettings::RULER_FG_COLOR,
                        "Foreground color of canvas rulers.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::RULER_BG_COLOR,
                        CanvasSettings::RULER_BG_COLOR,
                        "Background color of canvas rulers.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::RULER_INDICATOR_COLOR,
                        CanvasSettings::RULER_INDICATOR_COLOR,
                        "Color of mouse pointer in ruler.",
                        Color::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::DIRECTION_OPTIONS,
                        CanvasSettings::DIRECTION_OPTIONS,
                        "Theming options of contour direction arrow.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::HANDLE_CONNECTION_OPTIONS,
                        CanvasSettings::HANDLE_CONNECTION_OPTIONS,
                        "Theming options of handle connections (lines between handle and on-curve points).",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::HANDLE_OPTIONS,
                        CanvasSettings::HANDLE_OPTIONS,
                        "Theming options of handles.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::SMOOTH_CORNER_OPTIONS,
                        CanvasSettings::SMOOTH_CORNER_OPTIONS,
                        "Theming options of smooth (non-positional continuity) on-curve points.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::CORNER_OPTIONS,
                        CanvasSettings::CORNER_OPTIONS,
                        "Theming options of positional continuity on-curve points.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),
                    ParamSpecBoxed::new(
                        CanvasSettings::OUTLINE_OPTIONS,
                        CanvasSettings::OUTLINE_OPTIONS,
                        "Theming options of glyph outline.",
                        DrawOptions::static_type(),
                        ParamFlags::READWRITE | UI_EDITABLE,
                    ),

            ]
            },
        );
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            CanvasSettings::HANDLE_SIZE => self.handle_size.get().to_value(),
            CanvasSettings::LINE_WIDTH => self.line_width.get().to_value(),
            CanvasSettings::SHOW_GRID => self.show_grid.get().to_value(),
            CanvasSettings::SHOW_GUIDELINES => self.show_guidelines.get().to_value(),
            CanvasSettings::SHOW_HANDLES => self.show_handles.get().to_value(),
            CanvasSettings::INNER_FILL => self.inner_fill.get().to_value(),
            CanvasSettings::SHOW_TOTAL_AREA => self.show_total_area.get().to_value(),
            CanvasSettings::SHOW_RULERS => self.show_rulers.get().to_value(),
            CanvasSettings::WARP_CURSOR => self.warp_cursor.get().to_value(),
            CanvasSettings::BG_COLOR => self.bg_color.get().to_value(),
            CanvasSettings::GLYPH_INNER_FILL_COLOR => self.glyph_inner_fill_color.get().to_value(),
            CanvasSettings::GLYPH_BBOX_BG_COLOR => self.glyph_bbox_bg_color.get().to_value(),
            CanvasSettings::RULER_BREADTH_PIXELS => Self::RULER_BREADTH.to_value(),
            CanvasSettings::RULER_FG_COLOR => self.ruler_fg_color.get().to_value(),
            CanvasSettings::RULER_BG_COLOR => self.ruler_bg_color.get().to_value(),
            CanvasSettings::RULER_INDICATOR_COLOR => self.ruler_indicator_color.get().to_value(),
            CanvasSettings::SHOW_DIRECTION => self.show_direction.get().to_value(),
            CanvasSettings::HANDLE_OPTIONS => {
                { self.get_opts(self.handle_options.get()) }.to_value()
            }
            CanvasSettings::SMOOTH_CORNER_OPTIONS => {
                { self.get_opts(self.smooth_corner_options.get()) }.to_value()
            }
            CanvasSettings::CORNER_OPTIONS => {
                { self.get_opts(self.corner_options.get()) }.to_value()
            }
            CanvasSettings::DIRECTION_OPTIONS => {
                { self.get_opts(self.direction_options.get()) }.to_value()
            }
            CanvasSettings::HANDLE_CONNECTION_OPTIONS => {
                { self.get_opts(self.handle_connection_options.get()) }.to_value()
            }
            CanvasSettings::OUTLINE_OPTIONS => {
                { self.get_opts(self.outline_options.get()) }.to_value()
            }
            /*CanvasSettings::RULER_BREADTH_UNITS => {
                let ppu = self
                    .transformation
                    .property::<f64>(Transformation::PIXELS_PER_UNIT);
                let scale: f64 = self.transformation.property::<f64>(Transformation::SCALE);
                (RULER_BREADTH / (scale * ppu)).to_value()
            }*/
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
            CanvasSettings::HANDLE_SIZE => {
                self.handle_size.set(value.get().unwrap());
            }
            CanvasSettings::LINE_WIDTH => {
                self.line_width.set(value.get().unwrap());
            }
            CanvasSettings::SHOW_GRID => {
                self.show_grid.set(value.get().unwrap());
            }
            CanvasSettings::SHOW_GUIDELINES => {
                self.show_guidelines.set(value.get().unwrap());
            }
            CanvasSettings::SHOW_HANDLES => {
                self.show_handles.set(value.get().unwrap());
            }
            CanvasSettings::INNER_FILL => {
                self.inner_fill.set(value.get().unwrap());
            }
            CanvasSettings::SHOW_TOTAL_AREA => {
                self.show_total_area.set(value.get().unwrap());
            }
            CanvasSettings::SHOW_RULERS => {
                self.show_rulers.set(value.get().unwrap());
            }
            CanvasSettings::WARP_CURSOR => {
                self.warp_cursor.set(value.get().unwrap());
            }
            CanvasSettings::BG_COLOR => {
                self.bg_color.set(value.get().unwrap());
            }
            CanvasSettings::GLYPH_INNER_FILL_COLOR => {
                self.glyph_inner_fill_color.set(value.get().unwrap());
            }
            CanvasSettings::GLYPH_BBOX_BG_COLOR => {
                self.glyph_bbox_bg_color.set(value.get().unwrap());
            }
            CanvasSettings::RULER_FG_COLOR => {
                self.ruler_fg_color.set(value.get().unwrap());
            }
            CanvasSettings::RULER_BG_COLOR => {
                self.ruler_bg_color.set(value.get().unwrap());
            }
            CanvasSettings::RULER_INDICATOR_COLOR => {
                self.ruler_indicator_color.set(value.get().unwrap());
            }
            CanvasSettings::SHOW_DIRECTION => {
                self.show_direction.set(value.get().unwrap());
            }
            CanvasSettings::HANDLE_OPTIONS => {
                self.handle_options.set(value.get().unwrap());
            }
            CanvasSettings::SMOOTH_CORNER_OPTIONS => {
                self.smooth_corner_options.set(value.get().unwrap());
            }
            CanvasSettings::CORNER_OPTIONS => {
                self.corner_options.set(value.get().unwrap());
            }
            CanvasSettings::DIRECTION_OPTIONS => {
                self.direction_options.set(value.get().unwrap());
            }
            CanvasSettings::HANDLE_CONNECTION_OPTIONS => {
                self.handle_connection_options.set(value.get().unwrap());
            }
            CanvasSettings::OUTLINE_OPTIONS => {
                self.outline_options.set(value.get().unwrap());
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasSettings {
    pub const HANDLE_SIZE: &'static str = "handle-size";
    pub const LINE_WIDTH: &'static str = "line-width";
    pub const INNER_FILL: &'static str = "inner-fill";
    pub const VIEW_HEIGHT: &'static str = "view-height";
    pub const VIEW_WIDTH: &'static str = "view-width";
    pub const SHOW_GRID: &'static str = "show-grid";
    pub const SHOW_GUIDELINES: &'static str = "show-guidelines";
    pub const SHOW_HANDLES: &'static str = "show-handles";
    pub const SHOW_DIRECTION: &'static str = "show-direction";
    pub const HANDLE_OPTIONS: &'static str = "handle-options";
    pub const SMOOTH_CORNER_OPTIONS: &'static str = "smooth-corner-options";
    pub const CORNER_OPTIONS: &'static str = "corner-options";
    pub const DIRECTION_OPTIONS: &'static str = "direction-options";
    pub const HANDLE_CONNECTION_OPTIONS: &'static str = "handle-connection-options";
    pub const OUTLINE_OPTIONS: &'static str = "outline-options";
    pub const SHOW_TOTAL_AREA: &'static str = "show-total-area";
    pub const SHOW_RULERS: &'static str = "show-rules";
    pub const TRANSFORMATION: &'static str = "transformation";
    pub const WARP_CURSOR: &'static str = "warp-cursor";
    pub const MOUSE: &'static str = "mouse";
    pub const BG_COLOR: &'static str = "bg-color";
    pub const GLYPH_INNER_FILL_COLOR: &'static str = "glyph-inner-fill-color";
    pub const GLYPH_BBOX_BG_COLOR: &'static str = "glyph-bbox-bg-color";
    pub const RULER_BREADTH_PIXELS: &'static str = "ruler-breadth-pixels";
    pub const RULER_FG_COLOR: &'static str = "ruler-fg-color";
    pub const RULER_BG_COLOR: &'static str = "ruler-bg-color";
    pub const RULER_INDICATOR_COLOR: &'static str = "ruler-indicator-color";
    pub const CONTENT_WIDTH: &'static str = "content-width";

    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}

impl_property_window!(CanvasSettings, { "Editor Canvas" });
