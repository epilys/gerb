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

use gdk_sys::*;
use gtk::prelude::StyleContextExt;
use gtk::prelude::WidgetExt;
use gtk::{gdk, gdk::ffi as gdk_sys};
use std::borrow::Cow;

use gtk::gdk::keys::{constants as key_constants, Key};
use smallvec::SmallVec;

const VOID: u32 = gdk_sys::GDK_KEY_VoidSymbol as u32;

pub type ShortcutCb = dyn Fn(&gtk::gio::SimpleActionGroup) -> bool;

// [ref:needs_user_doc]
// [ref:needs_dev_doc]
pub struct Shortcut {
    pub keys: SmallVec<[u32; 8]>,
}

impl std::fmt::Debug for Shortcut {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Shortcut")
            .field("keys", &self.to_string())
            .finish()
    }
}

pub struct ShortcutAction {
    shortcut: Shortcut,
    on_press_fn: Option<Box<ShortcutCb>>,
    on_release_fn: Option<Box<ShortcutCb>>,
    label: gtk::Label,
    desc_label: gtk::Label,
    desc: Cow<'static, str>,
}

impl std::fmt::Debug for ShortcutAction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ShortcutAction")
            .field("shortcut", &self.shortcut)
            .finish()
    }
}

impl ShortcutAction {
    pub fn new(
        desc: Cow<'static, str>,
        shortcut: Shortcut,
        on_press_fn: Box<ShortcutCb>,
        on_release_fn: Option<Box<ShortcutCb>>,
    ) -> Self {
        let desc_label = gtk::Label::builder()
            .label(&desc)
            .single_line_mode(true)
            .tooltip_text(&desc)
            .visible(true)
            .sensitive(false)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();
        desc_label.style_context().add_class("shortcut-label");
        Self {
            desc_label,
            label: shortcut.label(),
            shortcut,
            on_press_fn: on_press_fn.into(),
            on_release_fn,
            desc,
        }
    }

    pub fn new_on_release(
        desc: Cow<'static, str>,
        shortcut: Shortcut,
        on_release_fn: Box<ShortcutCb>,
        on_press_fn: Option<Box<ShortcutCb>>,
    ) -> Self {
        let desc_label = gtk::Label::builder()
            .label(&desc)
            .single_line_mode(true)
            .tooltip_text(&desc)
            .visible(true)
            .sensitive(false)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();
        desc_label.style_context().add_class("shortcut-label");
        Self {
            desc_label,
            label: shortcut.label(),
            shortcut,
            on_press_fn,
            on_release_fn: on_release_fn.into(),
            desc,
        }
    }

    pub fn try_press(
        &self,
        key: &Key,
        mut modifier_mask: gdk::ModifierType,
        group: &gtk::gio::SimpleActionGroup,
    ) -> bool {
        for &k in self.shortcut.keys.iter() {
            let k = Key::from(k);
            if let Some(ch) = k.to_unicode() {
                if ch.is_uppercase() && modifier_mask.contains(gdk::ModifierType::SHIFT_MASK) {
                    modifier_mask.set(gdk::ModifierType::SHIFT_MASK, false);
                }
            }
            match k {
                key_constants::Control_L
                    if modifier_mask.contains(gdk::ModifierType::CONTROL_MASK) =>
                {
                    modifier_mask.set(gdk::ModifierType::CONTROL_MASK, false);
                }
                key_constants::Meta_L if modifier_mask.contains(gdk::ModifierType::META_MASK) => {
                    modifier_mask.set(gdk::ModifierType::META_MASK, false);
                }
                key_constants::Shift_L if modifier_mask.contains(gdk::ModifierType::SHIFT_MASK) => {
                    modifier_mask.set(gdk::ModifierType::SHIFT_MASK, false);
                }
                key_constants::Shift_L | key_constants::Meta_L | key_constants::Control_L => break,
                _ if k == *key && modifier_mask.is_empty() => {
                    if let Some(f) = self.on_press_fn.as_ref() {
                        return (f)(group);
                    }
                }
                _ => break,
            }
        }
        false
    }

    pub fn try_release(
        &self,
        key: &Key,
        mut modifier_mask: gdk::ModifierType,
        group: &gtk::gio::SimpleActionGroup,
    ) -> bool {
        for &k in self.shortcut.keys.iter() {
            let k = Key::from(k);
            if let Some(ch) = k.to_unicode() {
                if ch.is_uppercase() && modifier_mask.contains(gdk::ModifierType::SHIFT_MASK) {
                    modifier_mask.set(gdk::ModifierType::SHIFT_MASK, false);
                }
            }
            match k {
                key_constants::Control_L
                    if modifier_mask.contains(gdk::ModifierType::CONTROL_MASK) =>
                {
                    modifier_mask.set(gdk::ModifierType::CONTROL_MASK, false);
                }
                key_constants::Meta_L if modifier_mask.contains(gdk::ModifierType::META_MASK) => {
                    modifier_mask.set(gdk::ModifierType::META_MASK, false);
                }
                key_constants::Shift_L if modifier_mask.contains(gdk::ModifierType::SHIFT_MASK) => {
                    modifier_mask.set(gdk::ModifierType::SHIFT_MASK, false);
                }
                key_constants::Shift_L | key_constants::Meta_L | key_constants::Control_L => break,
                _ if k == *key && modifier_mask.is_empty() => {
                    if let Some(f) = self.on_release_fn.as_ref() {
                        return (f)(group);
                    }
                }
                _ => break,
            }
        }
        false
    }

    pub fn shortcut(&self) -> &Shortcut {
        &self.shortcut
    }

    pub fn label(&self) -> &gtk::Label {
        &self.label
    }

    pub fn desc_label(&self) -> &gtk::Label {
        &self.desc_label
    }

    pub fn desc(&self) -> &str {
        self.desc.as_ref()
    }
}

/// Helper macro to declare const shortcuts
#[macro_export]
macro_rules! decl_shortcut {
    ($a:expr, $b:expr, $c:expr,$d:expr,$e:expr,$f:expr,$g:expr,$_8:expr) => {
        $crate::utils::shortcuts::Shortcut {
            keys: ::smallvec::SmallVec::from_const([
                $a as u32, $b as u32, $c as u32, $d as u32, $e as u32, $f as u32, $g as u32,
                $_8 as u32,
            ]),
        }
    };
    ($a:expr, $b:expr, $c:expr,$d:expr,$e:expr,$f:expr,$g:expr) => {
        $crate::decl_shortcut!($a, $b, $c, $d, $e, $f, $g, VOID)
    };
    ($a:expr, $b:expr, $c:expr,$d:expr,$e:expr,$f:expr) => {
        $crate::decl_shortcut!($a, $b, $c, $d, $e, $f, VOID)
    };
    ($a:expr, $b:expr, $c:expr,$d:expr,$e:expr) => {
        $crate::decl_shortcut!($a, $b, $c, $d, $e, VOID)
    };
    ($a:expr, $b:expr, $c:expr,$d:expr) => {
        $crate::decl_shortcut!($a, $b, $c, $d, VOID)
    };
    ($a:expr, $b:expr, $c:expr) => {
        $crate::decl_shortcut!($a, $b, $c, VOID)
    };
    ($a:expr, $b:expr) => {
        $crate::decl_shortcut!($a, $b, VOID)
    };
    ($a:expr) => {
        $crate::decl_shortcut!($a, VOID)
    };
}

pub const GRAVE: Shortcut = decl_shortcut!(GDK_KEY_grave);
pub const MINISCULE_X: Shortcut = decl_shortcut!(GDK_KEY_x);
pub const MINISCULE_Y: Shortcut = decl_shortcut!(GDK_KEY_y);
pub const A: Shortcut = decl_shortcut!(GDK_KEY_A);
pub const G: Shortcut = decl_shortcut!(GDK_KEY_G);
pub const L: Shortcut = decl_shortcut!(GDK_KEY_L);
pub const M: Shortcut = decl_shortcut!(GDK_KEY_M);

impl std::fmt::Display for Shortcut {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for key in &self.keys {
            if *key == GDK_KEY_VoidSymbol as u32 {
                break;
            }
            let key = Key::from(*key);
            if key == key_constants::Control_L {
                fmt.write_fmt(format_args!("Ctrl+"))?;
            } else if key == key_constants::Meta_L {
                #[cfg(target_os = "macos")]
                fmt.write_fmt(format_args!("Cmd+"))?;
                #[cfg(unix)]
                fmt.write_fmt(format_args!("Super+"))?;
                #[cfg(target_os = "windows")]
                fmt.write_fmt(format_args!("Win+"))?;
                #[cfg(not(any(unix, target_os = "macos", target_os = "windows")))]
                fmt.write_fmt(format_args!("Meta+"))?
            } else if key == key_constants::Shift_L {
                fmt.write_fmt(format_args!("Shift+"))?;
            } else if let Some(c) = key.to_unicode() {
                fmt.write_fmt(format_args!("{}", c))?;
            } else if let Some(name) = key.name() {
                fmt.write_fmt(format_args!("{}", &name))?;
            }
        }
        Ok(())
    }
}

impl Shortcut {
    pub fn label(&self) -> gtk::Label {
        let text = self.to_string();
        let ret = gtk::Label::builder()
            .label(&text)
            .single_line_mode(true)
            .tooltip_text(&text)
            .visible(true)
            .sensitive(false)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

        ret.style_context().add_class("shortcut-kbd");
        ret
    }

    pub const fn empty() -> Self {
        decl_shortcut!(VOID)
    }

    pub fn char(mut self, c: char) -> Self {
        self.trim();
        self.keys.push(*Key::from_unicode(c));
        self
    }

    pub fn key(mut self, key: Key) -> Self {
        self.trim();
        self.keys.push(*key);
        self
    }

    pub fn meta(mut self) -> Self {
        self.trim();
        self.keys.push(*key_constants::Meta_L);
        self
    }

    pub fn control(mut self) -> Self {
        self.trim();
        self.keys.push(*key_constants::Control_L);
        self
    }

    pub fn shift(mut self) -> Self {
        self.trim();
        self.keys.push(*key_constants::Shift_L);
        self
    }

    fn trim(&mut self) {
        while self.keys.ends_with(&[VOID]) {
            self.keys.pop();
        }
    }
}

#[test]
fn test_shortcut_display() {
    assert_eq!("`", &GRAVE.to_string());
    assert_eq!("x", &MINISCULE_X.to_string());
}

pub mod constants {
    use super::*;

    macro_rules! const_ {
        ($name:ident, $constant:ident) => {
            pub const $name: u32 = $constant as u32;
        };
    }

    const_!(F1, GDK_KEY_F1);
}
