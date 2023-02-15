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

pub struct RepositoryInner {
    pub repository: RefCell<Option<Result<git2::Repository, Box<dyn std::error::Error>>>>,
}

impl std::fmt::Debug for RepositoryInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Repository")
            .field("repository", &self.repository.borrow().is_some())
            .finish()
    }
}

impl Default for RepositoryInner {
    fn default() -> Self {
        RepositoryInner {
            repository: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RepositoryInner {
    const NAME: &'static str = "Repository";
    type Type = Repository;
    type ParentType = glib::Object;
    type Interfaces = ();
}

impl ObjectImpl for RepositoryInner {
    /*
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| vec![]);
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        unimplemented!("{}", pspec.name())
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &glib::Value, pspec: &ParamSpec) {
        unimplemented!("{}", pspec.name());
    }
    */
}

glib::wrapper! {
    pub struct Repository(ObjectSubclass<RepositoryInner>);
}

impl std::ops::Deref for Repository {
    type Target = RepositoryInner;

    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl Repository {
    pub fn new() -> Self {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        ret
    }

    pub fn discover(&self, path: &Path) {
        let abs_path: PathBuf = std::fs::canonicalize(path).unwrap();
        let val = git2::Repository::discover(&abs_path).map_err(Into::into);
        match &val {
            Err(ref err) => {
                println!("git2 err: {}", err);
            }
            Ok(ref v) => {
                println!("git2 succ: {:?}", v.status_file(path));
            }
        }
        *self.repository.borrow_mut() = Some(val);
    }

    pub fn status_file(&self, path: &Path) -> Option<git2::Status> {
        dbg!(self
            .repository
            .borrow()
            .as_ref()?
            .as_ref()
            .ok()?
            .status_file(path))
        .ok()
    }
}

impl Default for Repository {
    fn default() -> Self {
        let ret: Self = Self::new();
        ret
    }
}
