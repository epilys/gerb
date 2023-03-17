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

//! # Placeholder module for an anyhow Error type.

pub struct Error;

impl Error {
    pub fn suggest_bug_report(err: &str) -> String {
        format!("Application error: {err}\n\nIf you wish to report this bug to <{}>, you can include the following build info string:\n\n{}", crate::ISSUE_TRACKER, crate::BUILD_INFO)
    }
}
