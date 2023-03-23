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

use gtk::prelude::*;

/// Error dialog util
///
/// ## Usage:
/// ```no_run
/// # use gtk::prelude::*;
/// # use gerb::utils::widgets::new_simple_error_dialog;
/// fn doctest(window: &gtk::Window) {
///     let dialog = new_simple_error_dialog(
///         None,
///         "Oops",
///         None,
///         &window,
///     );
///     dialog.run();
///     dialog.emit_close();
/// }
/// ```
#[must_use]
pub fn new_simple_error_dialog(
    title: Option<&str>,
    text: &str,
    secondary_text: Option<&str>,
    window: &gtk::Window,
) -> gtk::MessageDialog {
    let dialog = gtk::MessageDialog::new(
        Some(window),
        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        text,
    );
    dialog.set_secondary_text(secondary_text);
    dialog.set_secondary_use_markup(true);
    dialog.set_title(title.unwrap_or("Error"));
    dialog.set_use_markup(true);
    dialog
}

/// Info dialog util
///
/// ## Usage:
/// ```no_run
/// # use gtk::prelude::*;
/// # use gerb::utils::widgets::new_simple_info_dialog;
/// fn doctest(window: &gtk::Window) {
///     let dialog = new_simple_info_dialog(
///         None,
///         "Id",
///         None,
///         &window,
///     );
///     dialog.run();
///     dialog.emit_close();
/// }
/// ```
#[must_use]
pub fn new_simple_info_dialog(
    title: Option<&str>,
    text: &str,
    secondary_text: Option<&str>,
    window: &gtk::Window,
) -> gtk::MessageDialog {
    let dialog = gtk::MessageDialog::new(
        Some(window),
        gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::Close,
        text,
    );
    dialog.set_secondary_text(secondary_text);
    dialog.set_secondary_use_markup(true);
    dialog.set_title(title.unwrap_or("Information"));
    dialog.set_use_markup(true);
    dialog
}
