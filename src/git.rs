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

pub mod tab;
pub use tab::*;

pub struct RepositoryInner {
    pub repository: OnceCell<git2::Repository>,
    absolute_path: RefCell<PathBuf>,
    workdir: RefCell<PathBuf>,
    state: Cell<RepositoryState>,
}

impl std::fmt::Debug for RepositoryInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Repository").finish()
    }
}

impl Default for RepositoryInner {
    fn default() -> Self {
        Self {
            repository: OnceCell::new(),
            state: Cell::new(RepositoryState::Clean),
            absolute_path: RefCell::new(PathBuf::default()),
            workdir: RefCell::new(PathBuf::default()),
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
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpecEnum::new(
                    Repository::STATE,
                    Repository::STATE,
                    Repository::STATE,
                    RepositoryState::static_type(),
                    RepositoryState::Clean as i32,
                    glib::ParamFlags::READABLE,
                )]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            Repository::STATE => {
                self.state
                    .set(self.repository.get().unwrap().state().into());

                self.state.get().to_value()
            }
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    /*
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
    pub const STATE: &'static str = "state";

    pub fn new(abs_path: &Path) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let ret: Self = glib::Object::new::<Self>(&[]).unwrap();
        let abs_path: PathBuf = abs_path.to_path_buf();
        let val = git2::Repository::discover(&abs_path)?;
        *ret.absolute_path.borrow_mut() = abs_path;
        *ret.workdir.borrow_mut() = if let Some(p) = val.workdir() {
            p.to_path_buf()
        } else {
            return Ok(None);
        };

        ret.state.set(val.state().into());
        /*{
            let diff = val.diff_index_to_workdir(
                None,
                Some(
                    git2::DiffOptions::new()
                        .include_ignored(false)
                        .ignore_filemode(true)
                        .skip_binary_check(true),
                ),
            )?;
            for d in diff.deltas() {
                dbg!(&d.old_file().path());
            }
        }
        */
        _ = ret.repository.set(val);
        Ok(Some(ret))
    }

    pub fn status_file(&self, path: &Path) -> Option<git2::Status> {
        if path.is_absolute() {
            let r = self.repository.get().unwrap();
            r.status_file(path.strip_prefix(&*self.workdir.borrow()).ok()?)
                .ok()
        } else {
            self.repository.get().unwrap().status_file(path).ok()
        }
    }
}

/// A listing of the possible states that a repository can be in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "RepositoryState")]
pub enum RepositoryState {
    Clean,
    Merge,
    Revert,
    RevertSequence,
    CherryPick,
    CherryPickSequence,
    Bisect,
    Rebase,
    RebaseInteractive,
    RebaseMerge,
    ApplyMailbox,
    ApplyMailboxOrRebase,
}

impl From<git2::RepositoryState> for RepositoryState {
    fn from(original: git2::RepositoryState) -> Self {
        match original {
            git2::RepositoryState::Clean => Self::Clean,
            git2::RepositoryState::Merge => Self::Merge,
            git2::RepositoryState::Revert => Self::Revert,
            git2::RepositoryState::RevertSequence => Self::RevertSequence,
            git2::RepositoryState::CherryPick => Self::CherryPick,
            git2::RepositoryState::CherryPickSequence => Self::CherryPickSequence,
            git2::RepositoryState::Bisect => Self::Bisect,
            git2::RepositoryState::Rebase => Self::Rebase,
            git2::RepositoryState::RebaseInteractive => Self::RebaseInteractive,
            git2::RepositoryState::RebaseMerge => Self::RebaseMerge,
            git2::RepositoryState::ApplyMailbox => Self::ApplyMailbox,
            git2::RepositoryState::ApplyMailboxOrRebase => Self::ApplyMailboxOrRebase,
        }
    }
}
