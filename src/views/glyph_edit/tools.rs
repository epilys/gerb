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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ControlPointMode {
    None,
    Drag,
    DragGuideline(usize),
}

impl Default for ControlPointMode {
    fn default() -> ControlPointMode {
        ControlPointMode::None
    }
}

#[derive(Debug, Clone)]
pub enum Tool {
    Panning,
    Manipulate { mode: ControlPointMode },
}

impl Default for Tool {
    fn default() -> Tool {
        Tool::Manipulate {
            mode: ControlPointMode::default(),
        }
    }
}

impl Tool {
    pub fn is_manipulate(&self) -> bool {
        matches!(self, Tool::Manipulate { .. })
    }

    pub fn is_panning(&self) -> bool {
        matches!(self, Tool::Panning)
    }

    pub fn can_highlight(&self) -> bool {
        !matches!(
            self,
            Tool::Manipulate {
                mode: ControlPointMode::Drag
            }
        )
    }

    pub fn on_button_press_event(
        obj: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
        match glyph_state.tool {
            Tool::Manipulate { .. } => match event.button() {
                gtk::gdk::BUTTON_MIDDLE => {
                    glyph_state.tool = Tool::Panning;
                }
                gtk::gdk::BUTTON_PRIMARY => {
                    let event_position = event.position();
                    let UnitPoint(position) =
                        viewport.view_to_unit_point(ViewPoint(event_position.into()));
                    let ruler_breadth = viewport.property::<f64>(Canvas::RULER_BREADTH_PIXELS);
                    if event_position.0 < ruler_breadth || event_position.1 < ruler_breadth {
                        let angle = if event_position.0 < ruler_breadth
                            && event_position.1 < ruler_breadth
                        {
                            -45.0
                        } else if event_position.0 < ruler_breadth {
                            90.0
                        } else {
                            0.0
                        };
                        let mut action = glyph_state.new_guideline(angle, position);
                        (action.redo)();
                        let app: &crate::Application = crate::Application::from_instance(
                            &obj.imp()
                                .app
                                .get()
                                .unwrap()
                                .downcast_ref::<crate::GerbApp>()
                                .unwrap(),
                        );
                        let undo_db = app.undo_db.borrow_mut();
                        undo_db.event(action);
                    }
                    let mut is_guideline: bool = false;
                    let GlyphState {
                        ref mut tool,
                        ref glyph,
                        ..
                    } = *glyph_state;
                    for (i, g) in glyph.borrow().guidelines.iter().enumerate() {
                        if g.imp().on_line_query(position, None) {
                            obj.imp()
                                .select_object(Some(g.clone().upcast::<gtk::glib::Object>()));
                            *tool = Tool::Manipulate {
                                mode: ControlPointMode::DragGuideline(i),
                            };
                            is_guideline = true;
                            break;
                        }
                    }
                    if !is_guideline {
                        let pts = glyph_state.kd_tree.borrow().query(position, 10);
                        glyph_state.tool = Tool::Manipulate {
                            mode: ControlPointMode::Drag,
                        };
                        glyph_state.set_selection(&pts);
                    }
                    return Inhibit(true);
                }
                _ => {}
            },
            Tool::Panning => match event.button() {
                gtk::gdk::BUTTON_MIDDLE => {
                    glyph_state.tool = Tool::Panning;
                    return Inhibit(true);
                }
                _ => {}
            },
        }

        Inhibit(false)
    }

    pub fn on_button_release_event(
        obj: GlyphEditView,
        _viewport: &Canvas,
        event: &gtk::gdk::EventButton,
    ) -> Inhibit {
        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
        match glyph_state.tool {
            Tool::Manipulate { ref mut mode } => {
                *mode = ControlPointMode::None;
                return Inhibit(true);
            }
            Tool::Panning => match event.button() {
                gtk::gdk::BUTTON_MIDDLE => {
                    glyph_state.tool = Tool::default();
                    return Inhibit(true);
                }
                _ => {}
            },
        }

        Inhibit(false)
    }

    pub fn on_motion_notify_event(
        obj: GlyphEditView,
        viewport: &Canvas,
        event: &gtk::gdk::EventMotion,
    ) -> Inhibit {
        let mut glyph_state = obj.imp().glyph_state.get().unwrap().borrow_mut();
        if glyph_state.tool.is_panning() {
            let mouse: ViewPoint = viewport.get_mouse();
            let delta = <_ as Into<Point>>::into(event.position()) - mouse.0;
            viewport
                .imp()
                .transformation
                .move_camera_by_delta(ViewPoint(delta));
        } else {
            let UnitPoint(position) =
                viewport.view_to_unit_point(ViewPoint(event.position().into()));
            if let Tool::Manipulate {
                mode: ControlPointMode::Drag,
            } = glyph_state.tool
            {
                glyph_state.update_positions(position);
            } else if let Tool::Manipulate {
                mode: ControlPointMode::DragGuideline(idx),
            } = glyph_state.tool
            {
                let mut action = glyph_state.update_guideline(idx, position);
                (action.redo)();
                let app: &crate::Application = crate::Application::from_instance(
                    &obj.imp()
                        .app
                        .get()
                        .unwrap()
                        .downcast_ref::<crate::GerbApp>()
                        .unwrap(),
                );
                let undo_db = app.undo_db.borrow_mut();
                undo_db.event(action);
            }
            let pts = glyph_state.kd_tree.borrow().query(position, 10);
            if pts.is_empty() {
                obj.imp().hovering.set(None);
                if let Some(screen) = viewport.window() {
                    let display = screen.display();
                    screen.set_cursor(Some(
                        &//if glyph_state.tool.is_manipulate() {
                                        gtk::gdk::Cursor::from_name(&display, "default").unwrap(), //} else if glyph_state.tool.is_bezier_pen() {
                                                                                                   //    gtk::gdk::Cursor::from_name(&display, "crosshair").unwrap()
                                                                                                   //} else {
                                                                                                   //    gtk::gdk::Cursor::from_name(&display, "default").unwrap()
                                                                                                   //}
                    ));
                }
            } else if let Some(screen) = viewport.window() {
                let display = screen.display();
                screen.set_cursor(Some(
                    &gtk::gdk::Cursor::from_name(&display, "grab").unwrap(),
                ));
            }

            let glyph = glyph_state.glyph.borrow();
            if let Some(((i, j), curve)) = glyph.on_curve_query(position, &pts) {
                obj.imp().new_statusbar_message(&format!("{:?}", curve));
                obj.imp().hovering.set(Some((i, j)));
            }
        }

        Inhibit(true)
    }
}
