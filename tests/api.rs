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

#![allow(unused_imports)]

mod utils;
use utils::*;

#[test]
#[cfg(feature = "python")]
fn test_api_works() {
    use gerb::api::shell::*;
    use gerb::api::*;
    use gerb::prelude::*;

    glib_test_wrapper(|| {
        let runtime = Runtime::new();

        let shell = ShellInstance::new(
            runtime.clone(),
            glib::clone!(@weak runtime => @default-return Continue(false), move |tx: &std::sync::mpsc::Sender<String>, msg: String| {
                let response = process_api_request(&runtime, msg);
                let json = response.unwrap();
                tx.send(json.to_string()).unwrap();
                Continue(true)
            }),
            glib::clone!(@weak runtime => @default-return Continue(false), move |hist, (prefix, mut msg)| {
                if !msg.is_empty() {
                    while msg.ends_with('\n') {
                        msg.pop();
                    }
                    hist.borrow_mut().push((prefix, msg));
                }
                Continue(true)
            }),
        );

        // Get a main loop context handle in order to iterate on it manually
        let l = glib::MainContext::default();

        let hist = shell.hist.clone();

        let read_line = move |input: String| -> bool {
            if ["quit", "exit"].contains(&input.trim()) {
                return true;
            } else if let Err(err) = shell.shell_stdin.send({
                if input.trim().is_empty() {
                    "\n".to_string()
                } else {
                    format!("{}\n", input)
                }
            }) {
                eprintln!("Internal error: {err}");
            }
            false
        };

        if !read_line("help(gerb)\n".to_string()) {
            while l.iteration(true) {
                if !hist.borrow().history().is_empty() {
                    let r = hist.borrow();
                    let [(prefix1, name1), (prefix2, name2)] = r.history() else {
                        panic!("History length is not 2");
                    };
                    assert_eq!(prefix1, &LinePrefix::Ps1);
                    assert_eq!(name1, "help(gerb)");
                    assert_eq!(prefix2, &LinePrefix::Output);
                    assert!(
                        name2.starts_with("Help on Gerb"),
                        "Output should be a docstring but is:\n\n{name2}"
                    );
                    break;
                }
            }
        }
    });
}
