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

#[derive(Debug, Clone)]
pub struct State {
    pub app: Application,
    pub glyph: Rc<RefCell<Glyph>>,
    pub viewport: Canvas,
    pub tools: IndexMap<glib::types::Type, ToolImpl>,
    pub active_tool: glib::types::Type,
    pub panning_tool: glib::types::Type,
    pub selection: Vec<GlyphPointIndex>,
    pub selection_set: HashSet<uuid::Uuid>,
    pub kd_tree: Rc<RefCell<crate::utils::range_query::KdTree>>,
}

impl State {
    pub fn new(glyph: &Rc<RefCell<Glyph>>, app: Application, viewport: Canvas) -> Self {
        let ret = Self {
            app,
            glyph: Rc::clone(glyph),
            viewport,
            tools: IndexMap::default(),
            active_tool: glib::types::Type::INVALID,
            panning_tool: PanningTool::static_type(),
            selection: vec![],
            selection_set: HashSet::new(),
            kd_tree: Rc::new(RefCell::new(crate::utils::range_query::KdTree::new(&[]))),
        };

        for (contour_index, contour) in glyph.borrow().contours.iter().enumerate() {
            (ret.add_contour(contour, contour_index).redo)();
        }
        ret
    }

    pub fn add_contour(&self, contour: &Contour, contour_index: usize) -> Action {
        Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Contour::static_type().name(),
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[contour_index]).into() },
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.kd_tree as kd_tree, @weak contour as contour  => move || {
                    let mut kd_tree = kd_tree.borrow_mut();
                    for (curve_index, curve) in contour.curves().iter().enumerate() {
                        for (idx, pos) in curve.points().iter().map(|p| (p.glyph_index(contour_index, curve_index), p.position)) {
                            kd_tree.add(idx, pos);
                        }
                    }
                }),
            ),
            undo: Box::new(
                clone!(@weak self.kd_tree as kd_tree, @weak contour as contour => move || {
                    let mut kd_tree = kd_tree.borrow_mut();
                    for (curve_index, curve) in contour.curves().iter().enumerate() {
                        for idx in curve.points().iter().map(|p| p.glyph_index(contour_index, curve_index)) {
                            kd_tree.remove(idx);
                        }
                    }
                }),
            ),
        }
    }

    pub fn reverse_contour(&self, contour: &Contour, contour_index: usize) -> Action {
        let cl = Box::new(
            clone!(@weak self.kd_tree as kd_tree, @weak contour as contour  => move || {
                let mut kd_tree = kd_tree.borrow_mut();
                for (curve_index, curve) in contour.curves().iter().enumerate() {
                    for idx in curve.points().iter().map(|p| p.glyph_index(contour_index, curve_index)) {
                        kd_tree.remove(idx);
                    }
                }
                contour.reverse_direction();
                for (curve_index, curve) in contour.curves().iter().enumerate() {
                    for (idx, pos) in curve.points().iter().map(|p| (p.glyph_index(contour_index, curve_index), p.position)) {
                        kd_tree.add(idx, pos);
                    }
                }
            }),
        );
        Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Contour::static_type().name(),
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[contour_index]).into() },
            },
            compress: false,
            redo: cl.clone(),
            undo: cl,
        }
    }

    pub fn new_guideline(&self, angle: f64, p: Point) -> Action {
        let x = Some(p.x).filter(|&v| v != 0.0);
        let y = Some(p.y).filter(|&v| v != 0.0);
        let angle = Some(angle).filter(|&v| v != 0.0);
        let identifier = crate::ufo::make_random_identifier();

        Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Guideline::static_type().name(),
                id: Box::new([]),
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak self.viewport as viewport, @weak self.app as app => move || {
                    let guideline = Guideline::builder()
                        .angle(angle)
                        .x(x)
                        .y(y)
                        .identifier(Some(identifier.clone()))
                        .build();
                    app.runtime.project.borrow().link(&guideline);
                    glyph.borrow_mut().add_guideline(guideline);
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self.glyph as glyph, @weak self.viewport as viewport => move || {
                    glyph.borrow_mut().pop_guideline();
                    viewport.queue_draw();
                }),
            ),
        }
    }

    pub fn delete_guideline(&self, idx: usize) -> Action {
        let viewport = self.viewport.clone();
        let json: serde_json::Value =
            { serde_json::to_value(self.glyph.borrow().guidelines()[idx].imp()).unwrap() };
        Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Guideline::static_type().name(),
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[idx]).into() },
            },
            compress: false,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow_mut().remove_guideline(Either::B(idx));
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    glyph.borrow_mut().add_guideline(Guideline::try_from(json.clone()).unwrap());
                    viewport.queue_draw();
                }),
            ),
        }
    }

    pub fn add_undo_action(&self, action: Action) {
        self.app.undo_db.borrow_mut().event(action);
    }

    pub fn transform_guideline(&self, idx: usize, m: Matrix, dangle: f64) {
        let viewport = self.viewport.clone();
        let mut action = Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Guideline::static_type().name(),
                id: unsafe { std::mem::transmute::<&[usize], &[u8]>(&[idx]).into() },
            },
            compress: true,
            redo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    let glyph = glyph.borrow();
                    let g = &glyph.guidelines()[idx];
                    let x = g.property(Guideline::X);
                    let y = g.property(Guideline::Y);
                    let angle: f64 = g.property(Guideline::ANGLE);
                    let (x, y) = m.transform_point(x, y);
                    g.set_property(Guideline::X, x);
                    g.set_property(Guideline::Y, y);
                    g.set_property(Guideline::ANGLE, angle + dangle);
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@weak self.glyph as glyph, @weak viewport => move || {
                    let m = if let Ok(m) = m.try_invert() {m} else {return;};
                    let glyph = glyph.borrow();
                    let g = &glyph.guidelines()[idx];
                    let x = g.property(Guideline::X);
                    let y = g.property(Guideline::Y);
                    let angle: f64 = g.property(Guideline::ANGLE);
                    let (x, y) = m.transform_point(x, y);
                    g.set_property(Guideline::X, x);
                    g.set_property(Guideline::Y, y);
                    g.set_property(Guideline::ANGLE, angle - dangle);
                    viewport.queue_draw();
                }),
            ),
        };
        (action.redo)();
        self.add_undo_action(action);
    }

    pub fn transform_selection(&self, m: Matrix, compress: bool) {
        let mut action = self.transform_points(&self.selection, m);
        action.compress = compress;
        (action.redo)();
        self.add_undo_action(action);
    }

    pub fn transform_points(&self, idxs_: &[GlyphPointIndex], m: Matrix) -> Action {
        let viewport = self.viewport.clone();
        let idxs = Rc::new(idxs_.to_vec());
        Action {
            stamp: EventStamp {
                t: std::any::TypeId::of::<Self>(),
                property: Point::static_type().name(),
                id: idxs_
                    .iter()
                    .map(GlyphPointIndex::as_bytes)
                    .flat_map(<_>::into_iter)
                    .collect::<Vec<u8>>()
                    .into(),
            },
            compress: false,
            redo: Box::new(
                clone!(@strong idxs, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak viewport => move || {
                    let mut kd_tree = kd_tree.borrow_mut();
                    let glyph = glyph.borrow();
                    for contour_index in idxs.iter().map(|i| i.contour_index) {
                        let contour = &glyph.contours[contour_index];
                        for (idx, new_pos) in contour.transform_points(contour_index, &idxs, m) {
                            kd_tree.add(idx, new_pos);
                        }
                    }
                    viewport.queue_draw();
                }),
            ),
            undo: Box::new(
                clone!(@strong idxs, @weak self.kd_tree as kd_tree, @weak self.glyph as glyph, @weak viewport => move || {
                    let m = if let Ok(m) = m.try_invert() {m} else {return;};
                    let mut kd_tree = kd_tree.borrow_mut();
                    let glyph = glyph.borrow();
                    for contour_index in idxs.iter().map(|i| i.contour_index) {
                        let contour = &glyph.contours[contour_index];
                        for (idx, new_pos) in contour.transform_points(contour_index, &idxs, m) {
                            /* update kd_tree */
                            kd_tree.add(idx, new_pos);
                        }
                    }
                    viewport.queue_draw();
                }),
            ),
        }
    }

    pub fn set_selection(&mut self, selection: &[GlyphPointIndex], modifier: SelectionModifier) {
        use SelectionModifier::*;
        match modifier {
            Replace => {
                self.selection.clear();
                self.selection_set.clear();
                self.selection.extend(selection.iter());
                for v in &self.selection {
                    self.selection_set.insert(v.uuid);
                }
            }
            Add => {
                self.selection.extend(selection.iter());
                for v in &self.selection {
                    self.selection_set.insert(v.uuid);
                }
            }
            Remove => {
                self.selection.retain(|e| !selection.contains(e));
                for v in selection {
                    self.selection_set.remove(&v.uuid);
                }
            }
        }
    }

    pub fn get_selection_set(&self) -> &HashSet<uuid::Uuid> {
        &self.selection_set
    }

    pub fn get_selection(&self) -> &[GlyphPointIndex] {
        &self.selection
    }
}
