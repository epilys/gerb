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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

glib::wrapper! {
    pub struct UndoDatabase(ObjectSubclass<UndoDatabaseInner>);
}

#[derive(Debug)]
pub struct Event {
    pub timestamp: u64,
    pub action: Action,
}

#[derive(Debug)]
#[repr(C)]
pub struct EventStamp {
    pub t: std::any::TypeId,
    pub property: &'static str,
    pub id: Box<[u8]>,
}

impl PartialEq for EventStamp {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.property == other.property && self.id == other.id
    }
}

impl Eq for EventStamp {}

pub struct Action {
    pub stamp: EventStamp,
    pub compress: bool,
    pub redo: Box<dyn FnMut()>,
    pub undo: Box<dyn FnMut()>,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Action")
            .field("compress", &self.compress)
            .field(
                "stamp",
                &format!(
                    "{:?} {} {:?}",
                    self.stamp.t, self.stamp.property, self.stamp.id
                ),
            )
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct UndoDatabaseInner {
    pub database: RefCell<Vec<Event>>,
    pub timestamp: RefCell<u64>,
    pub cursor: RefCell<usize>,
}

#[glib::object_subclass]
impl ObjectSubclass for UndoDatabaseInner {
    const NAME: &'static str = "UndoDatabase";
    type Type = UndoDatabase;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for UndoDatabaseInner {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        UndoDatabase::CAN_UNDO,
                        UndoDatabase::CAN_UNDO,
                        UndoDatabase::CAN_UNDO,
                        false,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        UndoDatabase::CAN_REDO,
                        UndoDatabase::CAN_REDO,
                        UndoDatabase::CAN_REDO,
                        false,
                        ParamFlags::READABLE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            UndoDatabase::CAN_UNDO => {
                let db = self.database.borrow();
                let cursor = self.cursor.borrow();
                (!(db.is_empty() || *cursor == 0)).to_value()
            }
            UndoDatabase::CAN_REDO => {
                let db = self.database.borrow();
                let cursor = self.cursor.borrow();
                (*cursor < db.len()).to_value()
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }
}

impl Default for UndoDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoDatabase {
    pub const CAN_UNDO: &str = "can-undo";
    pub const CAN_REDO: &str = "can-redo";

    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn event(&self, action: Action) {
        {
            let mut cursor = self.imp().cursor.borrow_mut();
            let mut db = self.imp().database.borrow_mut();
            let mut timestamp = self.imp().timestamp.borrow_mut();
            *timestamp += 1;
            let timestamp = *timestamp - 1;
            db.drain(*cursor..);
            db.push(Event { timestamp, action });
            *cursor = db.len();
        }
        self.notify(Self::CAN_UNDO);
    }

    pub fn undo(&self) {
        let mut did = false;
        {
            let mut cursor = self.imp().cursor.borrow_mut();
            let mut db = self.imp().database.borrow_mut();
            while let Some(last) = db.get_mut(..*cursor).and_then(<[Event]>::last_mut) {
                (last.action.undo)();
                did |= true;
                if *cursor == 0 {
                    break;
                }
                *cursor -= 1;
                match (db[..*cursor].last(), db[..*cursor + 1].last()) {
                    (Some(prev), Some(cur))
                        if prev.action.stamp == cur.action.stamp
                            && prev.action.compress
                            && cur.action.compress => {}
                    _ => break,
                }
            }
        }
        if did {
            self.notify(Self::CAN_UNDO);
            self.notify(Self::CAN_REDO);
        }
    }

    pub fn redo(&self) {
        let mut did = false;
        {
            let mut cursor = self.imp().cursor.borrow_mut();
            let mut db = self.imp().database.borrow_mut();
            while let Some(last) = db.get_mut(*cursor) {
                (last.action.redo)();
                did |= true;
                *cursor += 1;
                if *cursor >= db.len() {
                    *cursor = std::cmp::min(db.len(), *cursor);
                    break;
                }
                match (db.get(*cursor - 1), db.get(*cursor)) {
                    (Some(prev), Some(cur))
                        if prev.action.stamp == cur.action.stamp
                            && prev.action.compress
                            && cur.action.compress => {}
                    _ => break,
                }
            }
        }
        if did {
            self.notify(Self::CAN_UNDO);
            self.notify(Self::CAN_REDO);
        }
    }
}
