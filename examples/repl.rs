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

#![deny(clippy::dbg_macro)]

#[cfg(not(feature = "python"))]
fn main() {
    std::compile_error!("The `repl` binary requires the `python` feature to be enabled.")
}

#[cfg(feature = "python")]
fn main() {
    use gerb::api::shell::*;
    use gerb::api::*;
    use gerb::prelude::*;

    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::sync::mpsc::Sender;

    let runtime = Runtime::new();
    match std::env::args()
        .nth(1)
        .map(|d| (d.clone(), Project::from_path(&d)))
    {
        Some((ufo_dir, Ok(project))) => {
            #[cfg(feature = "python")]
            runtime.register_obj(project.upcast_ref());
            *runtime.project.borrow_mut() = project;
            println!("Loaded {}.", ufo_dir);
        }
        Some((ufo_dir, Err(err))) => {
            eprintln!("Could not load {ufo_dir}: {err}");
            return;
        }
        None => {}
    }

    let shell = ShellInstance::new(
        runtime.clone(),
        glib::clone!(@weak runtime => @default-return Continue(false), move |tx: &std::sync::mpsc::Sender<String>, msg: String| {
            let response = process_api_request(&runtime, msg);
            let (Err(json) | Ok(json)) = response;
            tx.send(json.to_string()).unwrap();
            Continue(true)
        }),
        glib::clone!(@weak runtime => @default-return Continue(false), move |hist, (prefix, mut msg)| {
            if !msg.is_empty() {
                while msg.ends_with('\n') {
                    msg.pop();
                }
                for line in msg.lines() {
                    println!("{} {line}", prefix.as_str());
                }
                print!("> ");
                std::io::stdout().flush().unwrap();
                hist.borrow_mut().push((prefix, msg));
            }
            Continue(true)
        }),
    );

    // take stdin and poll() it in glib loop
    // if we used stdin().read_line() instead, the main loop would be blocked
    let input_stream = unsafe { gio::UnixInputStream::take_fd(std::io::stdin().as_raw_fd()) };
    // -> DataInputStream which has a read line in utf8 method
    let data_stream = gio::DataInputStream::new(&input_stream);

    // Get a main loop context handle in order to iterate on it manually
    let l = glib::MainContext::default();

    // Channel between asynchronous closure that reads stdin input and the loop {} below,
    // if it sends 'true' then the loop {} exits
    let (tx, rx) = std::sync::mpsc::channel::<bool>();

    let read_line = move |shell_stdin: Sender<String>, tx: Sender<bool>| {
        data_stream.read_line_utf8_async(
            glib::source::PRIORITY_DEFAULT,
            gio::Cancellable::NONE,
            move |result| {
                tx.send(if let Ok(input) = result {
                    let input = input.map(|g| g.to_string()).unwrap_or_default();
                    if ["quit", "exit"].contains(&input.trim()) {
                        true
                    } else {
                        print!("\r");
                        std::io::stdout().flush().unwrap();
                        if let Err(err) = shell_stdin.send(if input.trim().is_empty() {
                            "\n".to_string()
                        } else {
                            format!("{}\n", input)
                        }) {
                            eprintln!("Internal error: {err}");
                        }
                        false
                    }
                } else {
                    false
                })
                .unwrap()
            },
        );
    };

    eprintln!("{}", BANNER);
    eprint!("> ");
    loop {
        read_line(shell.shell_stdin.clone(), tx.clone());
        while l.iteration(true) {}

        if rx.try_recv() == Ok(true) {
            break;
        }
    }
}
