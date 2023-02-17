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

use super::tools::{constraints::Precision, MoveDirection, SelectionAction};
use super::*;
use gtk::gdk::keys::constants as keys;

impl GlyphEditView {
    pub const LOCK: &str = "lock";
    pub const SNAP: &str = "snap";
    pub const PRECISION: &str = "precision";
    pub const PREVIEW_ACTION: &str = Self::PREVIEW;
    pub const ZOOM_IN_ACTION: &str = "zoom.in";
    pub const ZOOM_OUT_ACTION: &str = "zoom.out";
    pub const LOCK_ACTION: &str = Self::LOCK;
    pub const LOCK_X_ACTION: &str = "lock.x";
    pub const LOCK_Y_ACTION: &str = "lock.y";
    pub const LOCK_LOCAL_ACTION: &str = "lock.local";
    pub const LOCK_CONTROLS_ACTION: &str = "lock.controls";
    pub const PRECISION_ACTION: &str = Self::PRECISION;
    pub const SNAP_ACTION: &str = Self::SNAP;
    pub const SNAP_CLEAR_ACTION: &str = "snap.clear";
    pub const SNAP_ANGLE_ACTION: &str = "snap.angle";
    pub const SNAP_GRID_ACTION: &str = "snap.grid";
    pub const SNAP_GUIDELINES_ACTION: &str = "snap.guidelines";
    pub const SNAP_METRICS_ACTION: &str = "snap.metrics";
    pub const MOVE_UP_ACTION: &str = "move.up";
    pub const MOVE_DOWN_ACTION: &str = "move.down";
    pub const MOVE_RIGHT_ACTION: &str = "move.right";
    pub const MOVE_LEFT_ACTION: &str = "move.left";
    pub const SELECT_ALL_ACTION: &str = "select.all";
    pub const SELECT_NONE_ACTION: &str = "select.none";
    pub const SELECT_INVERT_ACTION: &str = "select.invert";
}

impl GlyphEditViewInner {
    pub fn setup_shortcuts(&self, obj: &GlyphEditView) {
        {
            use GlyphEditView as A;
            let mut sh = self.shortcuts.borrow_mut();
            sh.push(ShortcutAction::new(
                "preview".into(),
                Shortcut::empty().char('`'),
                Box::new(|group| {
                    if group.is_action_enabled(A::PREVIEW_ACTION) {
                        group.change_action_state(A::PREVIEW_ACTION, &true.to_variant());
                        true
                    } else {
                        false
                    }
                }),
                Some(Box::new(|group| {
                    if group.is_action_enabled(A::PREVIEW_ACTION) {
                        group.change_action_state(A::PREVIEW_ACTION, &false.to_variant());
                        true
                    } else {
                        false
                    }
                })),
            ));
            sh.push(ShortcutAction::new(
                "lock x".into(),
                Shortcut::empty().char('x'),
                Box::new(|group| {
                    if group.is_action_enabled(A::LOCK_ACTION) {
                        group.activate_action(A::LOCK_X_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "lock y".into(),
                Shortcut::empty().char('y'),
                Box::new(|group| {
                    if group.is_action_enabled(A::LOCK_ACTION) {
                        group.activate_action(A::LOCK_Y_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "lock local".into(),
                Shortcut::empty().char('l'),
                Box::new(|group| {
                    if group.is_action_enabled(A::LOCK_ACTION) {
                        group.activate_action(A::LOCK_LOCAL_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "snap angle".into(),
                Shortcut::empty().char('A'),
                Box::new(|group| {
                    if group.is_action_enabled(A::SNAP_ACTION) {
                        group.activate_action(A::SNAP_ANGLE_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "snap grid".into(),
                Shortcut::empty().char('G'),
                Box::new(|group| {
                    if group.is_action_enabled(A::SNAP_ACTION) {
                        group.activate_action(A::SNAP_GRID_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "snap guidelines".into(),
                Shortcut::empty().char('L'),
                Box::new(|group| {
                    if group.is_action_enabled(A::SNAP_ACTION) {
                        group.activate_action(A::SNAP_GUIDELINES_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "snap metrics".into(),
                Shortcut::empty().char('M'),
                Box::new(|group| {
                    if group.is_action_enabled(A::SNAP_ACTION) {
                        group.activate_action(A::SNAP_METRICS_ACTION, None);
                        true
                    } else {
                        false
                    }
                }),
                None,
            ));
            for (desc, key, action_name) in [
                ("move up", keys::Up, A::MOVE_UP_ACTION),
                ("move down", keys::Down, A::MOVE_DOWN_ACTION),
                ("move right", keys::Right, A::MOVE_RIGHT_ACTION),
                ("move left", keys::Left, A::MOVE_LEFT_ACTION),
            ] {
                sh.push(ShortcutAction::new(
                    desc.into(),
                    Shortcut::empty().key(key),
                    Box::new(|group| {
                        group.activate_action(action_name, None);
                        true
                    }),
                    None,
                ));
            }
            sh.push(ShortcutAction::new(
                "select all".into(),
                Shortcut::empty().control().char('a'),
                Box::new(|group| {
                    group.activate_action(A::SELECT_ALL_ACTION, None);
                    true
                }),
                None,
            ));
            sh.push(ShortcutAction::new(
                "select none".into(),
                Shortcut::empty().control().shift().char('A'),
                Box::new(|group| {
                    group.activate_action(A::SELECT_NONE_ACTION, None);
                    true
                }),
                None,
            ));
            for (name, dir) in [
                (A::MOVE_UP_ACTION, MoveDirection::Up),
                (A::MOVE_DOWN_ACTION, MoveDirection::Down),
                (A::MOVE_RIGHT_ACTION, MoveDirection::Right),
                (A::MOVE_LEFT_ACTION, MoveDirection::Left),
            ] {
                let a = gtk::gio::SimpleAction::new(name, None);
                a.connect_activate(glib::clone!(@weak obj => move |_, _| {
                    let t = obj.property::<super::PanningTool>(A::PANNING_TOOL);
                    t.move_action(&obj, dir);
                }));
                obj.action_group.add_action(&a);
            }
            for (name, action) in [
                (A::SELECT_ALL_ACTION, SelectionAction::All),
                (A::SELECT_NONE_ACTION, SelectionAction::None),
            ] {
                let a = gtk::gio::SimpleAction::new(name, None);
                a.connect_activate(glib::clone!(@weak obj => move |_, _| {
                    let t = obj.property::<super::PanningTool>(A::PANNING_TOOL);
                    t.selection_action(&obj, action);
                }));
                obj.action_group.add_action(&a);
            }
            for (name, key, num) in [
                ("precision 1", '!', Precision::EMPTY),
                ("precision 3", '@', Precision::_1),
                ("precision 4", '#', Precision::_05),
                ("precision 5", '$', Precision::_01),
            ] {
                sh.push(ShortcutAction::new(
                    name.into(),
                    Shortcut::empty().shift().char(key),
                    Box::new(move |group| {
                        group.change_action_state(A::PRECISION_ACTION, &num.to_variant());
                        true
                    }),
                    None,
                ));
            }

            self.shortcut_status
                .set_orientation(gtk::Orientation::Horizontal);
            self.shortcut_status.set_visible(true);
            let grid = gtk::FlowBox::builder()
                .expand(false)
                .visible(true)
                .sensitive(false)
                .can_focus(true)
                .column_spacing(5)
                .margin(10)
                .row_spacing(5)
                .build();
            let btn = gtk::Button::builder()
                .label("⏶")
                .visible(true)
                .focus_on_click(false)
                .tooltip_text("shortcuts")
                .relief(gtk::ReliefStyle::None)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build();
            btn.style_context().add_class("shortcuts-more");
            self.shortcut_status.pack_end(&btn, false, false, 1);
            for s in sh.iter() {
                let b = gtk::Box::builder()
                    .expand(false)
                    .visible(true)
                    .sensitive(true)
                    .build();
                b.pack_start(s.desc_label(), false, false, 1);
                b.pack_end(&s.shortcut().label(), false, false, 1);
                grid.add(&b);
            }
            let pop = gtk::Popover::builder()
                .expand(false)
                .visible(false)
                .sensitive(true)
                .modal(true)
                .position(gtk::PositionType::Bottom)
                .child(&grid)
                .relative_to(&btn)
                .build();
            btn.connect_clicked(clone!(@strong pop => move |_| {
                pop.popup();
            }));
        }

        let ctrl = gtk::EventControllerKey::new(obj);
        ctrl.connect_key_pressed(
            clone!(@weak self.action_group as group, @weak self.shortcuts as shortcuts => @default-return false, move |_self, keyval, _, modifier_type: gdk::ModifierType| {
                use gtk::gdk::keys::Key;

                let key = Key::from(keyval);
                let sks = shortcuts.borrow();
                if sks.iter().any(|s| s.try_press(&key, modifier_type, &group)) {
                    return true;
                }
                false
            }),
        );
        ctrl.connect_key_released(
            clone!(@weak self.action_group as group, @weak self.shortcuts as shortcuts  => move |_self, keyval, _,  modifier_type: gdk::ModifierType| {
                use gtk::gdk::keys::Key;

                let key = Key::from(keyval);
                let sks = shortcuts.borrow();
                sks.iter().find(|s| s.try_release(&key, modifier_type, &group));
            }),
        );
        self.ctrl.set(ctrl).unwrap();
    }
}
