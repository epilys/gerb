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

use super::Repository;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct GitSpaceInner {
    app: OnceCell<Application>,
    project: OnceCell<Project>,
    settings: OnceCell<Settings>,
    treeview: gtk::TreeView,
    repo: OnceCell<Repository>,

    menubar: gtk::MenuBar,
    action_group: gio::SimpleActionGroup,
    shortcut_status: gtk::Box,
}

#[glib::object_subclass]
impl ObjectSubclass for GitSpaceInner {
    const NAME: &'static str = "GitSpace";
    type Type = GitSpace;
    type ParentType = gtk::Bin;
}

impl ObjectImpl for GitSpaceInner {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        let store = gtk::TreeStore::new(&[String::static_type(), String::static_type()]);
        self.treeview.set_model(Some(&store));
        self.treeview.set_headers_visible(true);
        for n in ["type", "name"] {
            let column = gtk::TreeViewColumn::new();
            column.set_title(n);
            let cell = gtk::CellRendererText::new();

            column.pack_start(&cell, true);
            column.add_attribute(&cell, "text", 0);
            self.treeview.append_column(&column);
        }
        obj.set_visible(true);
        obj.set_expand(true);
        obj.set_can_focus(true);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        GitSpace::TITLE,
                        GitSpace::TITLE,
                        GitSpace::TITLE,
                        Some("git"),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        GitSpace::CLOSEABLE,
                        GitSpace::CLOSEABLE,
                        GitSpace::CLOSEABLE,
                        true,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        GitSpace::IS_MENU_VISIBLE,
                        GitSpace::IS_MENU_VISIBLE,
                        GitSpace::IS_MENU_VISIBLE,
                        true,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        Workspace::MENUBAR,
                        Workspace::MENUBAR,
                        Workspace::MENUBAR,
                        gtk::MenuBar::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            GitSpace::TITLE => "git".to_value(),
            GitSpace::CLOSEABLE => true.to_value(),
            GitSpace::IS_MENU_VISIBLE => true.to_value(),
            GitSpace::MENUBAR => Some(self.menubar.clone()).to_value(),
            _ => unimplemented!("{}", pspec.name()),
        }
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> =
            Lazy::new(|| vec![Signal::builder("update", &[], <()>::static_type().into()).build()]);
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for GitSpaceInner {}
impl ContainerImpl for GitSpaceInner {}
impl BinImpl for GitSpaceInner {}

glib::wrapper! {
    pub struct GitSpace(ObjectSubclass<GitSpaceInner>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
}

impl std::ops::Deref for GitSpace {
    type Target = GitSpaceInner;
    fn deref(&self) -> &Self::Target {
        self.imp()
    }
}

impl GitSpace {
    pub const CLOSEABLE: &str = Workspace::CLOSEABLE;
    pub const TITLE: &str = Workspace::TITLE;
    pub const IS_MENU_VISIBLE: &str = Workspace::IS_MENU_VISIBLE;
    pub const MENUBAR: &str = Workspace::MENUBAR;

    pub fn new(app: Application, project: Project, repository: Repository) -> Self {
        let ret: Self = glib::Object::new(&[]).unwrap();
        ret.repo.set(repository).unwrap();
        ret.app.set(app.clone()).unwrap();
        ret.connect_map(|self_| {
            let status = self_.app.get().unwrap().statusbar().message_area().unwrap();
            status.pack_end(&self_.shortcut_status, false, false, 1);
        });
        ret.connect_unmap(|self_| {
            let status = self_.app.get().unwrap().statusbar().message_area().unwrap();
            status.remove(&self_.shortcut_status);
        });
        let settings = app.runtime.settings.clone();
        //settings.register_type(ret.viewport.clone().upcast());
        ret.settings.set(settings).unwrap();
        ret.insert_action_group("git", Some(&ret.action_group));
        ret.menubar
            .insert_action_group("git", Some(&ret.action_group));
        ret.project.set(project).unwrap();
        //ret.setup_menu(&ret);
        ret
    }
}
