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

            self.shortcut_status
                .set_orientation(gtk::Orientation::Horizontal);
            self.shortcut_status.set_visible(true);
            let grid = gtk::Grid::builder()
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
                .relief(gtk::ReliefStyle::None)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build();
            btn.style_context().add_class("shortcuts-more");
            self.shortcut_status.pack_end(&btn, false, false, 1);
            for s in sh.iter() {
                let l = s.label();
                l.set_tooltip_text(Some(s.desc()));
                self.shortcut_status.pack_end(l, false, false, 1);
                grid.add(s.desc_label());
                grid.add(&s.shortcut().label());
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
