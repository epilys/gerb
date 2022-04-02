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
use imp::Event;
use std::cell::RefCell;

glib::wrapper! {
    pub struct UndoDatabase(ObjectSubclass<imp::UndoDatabase>);
}

mod imp {
    use super::*;
    #[derive(Debug)]
    pub struct Event {
        pub timestamp: u64,
        pub action: super::Action,
    }

    #[derive(Debug, Default)]
    pub struct UndoDatabase {
        pub database: RefCell<Vec<Event>>,
        pub timestamp: RefCell<u64>,
        pub cursor: RefCell<usize>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UndoDatabase {
        const NAME: &'static str = "UndoDatabase";
        type Type = super::UndoDatabase;
        type ParentType = glib::Object;
        type Interfaces = ();
    }

    impl ObjectImpl for UndoDatabase {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
                once_cell::sync::Lazy::new(|| {
                    vec![
                        ParamSpecBoolean::new(
                            "can-undo",
                            "can-undo",
                            "can-undo",
                            false,
                            ParamFlags::READWRITE,
                        ),
                        ParamSpecBoolean::new(
                            "can-redo",
                            "can-redo",
                            "can-redo",
                            false,
                            ParamFlags::READWRITE,
                        ),
                    ]
                });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "can-undo" => {
                    let db = self.database.borrow();
                    let cursor = self.cursor.borrow();
                    (!(db.is_empty() || *cursor == 0)).to_value()
                }
                "can-redo" => {
                    let db = self.database.borrow();
                    let cursor = self.cursor.borrow();
                    (!(*cursor >= db.len())).to_value()
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            _value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "can-undo" => { /* ignore value*/ }
                "can-redo" => { /* ignore value*/ }
                _ => unimplemented!(),
            }
        }
    }
}

impl Default for UndoDatabase {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventStamp {
    pub t: std::any::TypeId,
    //FIXME: add also a weak object ref here
    pub property: &'static str,
    pub id: Box<[u8]>,
}

impl PartialEq for EventStamp {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.property == other.property && self.id == other.id
    }
}

impl Eq for EventStamp {}

impl UndoDatabase {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn event(&self, action: Action) {
        self.set_property("can-undo", true);
        self.set_property("can-redo", false);
        let mut cursor = self.imp().cursor.borrow_mut();
        let mut db = self.imp().database.borrow_mut();
        let mut timestamp = self.imp().timestamp.borrow_mut();
        *timestamp += 1;
        let timestamp = *timestamp - 1;
        db.drain(*cursor..);
        db.push(Event { timestamp, action });
        *cursor = db.len();
    }

    pub fn undo(&self) {
        let mut did_undo = false;
        {
            let mut cursor = self.imp().cursor.borrow_mut();
            let mut db = self.imp().database.borrow_mut();
            loop {
                if let Some(last) = db[..*cursor].last_mut() {
                    (last.action.undo)();
                    did_undo |= true;
                } else {
                    break;
                }
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
        if did_undo {
            self.set_property("can-undo", false);
            self.set_property("can-redo", true);
        }
    }

    pub fn redo(&self) {
        let mut did_redo = false;
        {
            let mut cursor = self.imp().cursor.borrow_mut();
            let mut db = self.imp().database.borrow_mut();
            loop {
                if let Some(last) = db.get_mut(*cursor) {
                    (last.action.redo)();
                    did_redo |= true;
                } else {
                    break;
                }
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
        if did_redo {
            self.set_property("can-undo", true);
            self.set_property("can-redo", false);
        }
    }
}

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
