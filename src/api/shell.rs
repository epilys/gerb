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

//! Python shell

use super::*;

const SYS_PS1: &str = ">>> ";
const SYS_PS2: &str = "... ";
const BANNER: &str = "Exported objects: 'gerb'. Use 'help(gerb)' for more information.";

// [ref:needs_user_doc]
// [ref:needs_dev_doc]
// [ref:FIXME]: show errors in UI instead of unwrapping

// [ref:FIXME]: Ctrl-C not working when issuing `help(gerb)`?

// [ref:TODO]: Serialize messages and exceptions in JSON.
//
// [ref:TODO]: Typing hints?

#[derive(Clone, Copy)]
pub enum LinePrefix {
    Output,
    Ps1,
    Ps2,
}

/// Python shell history
pub struct ShellHistory {
    history: Vec<(LinePrefix, String)>,
}

/// Setup python shell window
pub fn new_shell_window(app: Application) -> gtk::Window {
    let w = gtk::Window::builder()
        .deletable(true)
        .transient_for(&app.window)
        .attached_to(&app.window)
        .destroy_with_parent(true)
        .application(&app)
        .focus_on_map(true)
        .resizable(true)
        .title("python shell")
        .visible(true)
        .expand(true)
        .type_hint(gtk::gdk::WindowTypeHint::Utility)
        .window_position(gtk::WindowPosition::Center)
        .build();
    w.set_default_size(640, 480);
    w.set_size_request(640, 480);
    let scrolled_window = gtk::ScrolledWindow::builder()
        .expand(true)
        .visible(true)
        .can_focus(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .build();
    let adj = scrolled_window.vadjustment();
    adj.set_value(adj.upper());
    let list = gtk::ListBox::builder()
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .visible(true)
        .build();
    list.style_context().add_class("terminal-box");
    {
        let label = gtk::Label::new(Some(BANNER));
        label.set_wrap(true);
        label.set_selectable(true);
        label.set_visible(true);
        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::End);
        list.add(&label);
        list.queue_draw();
    }
    {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin(0)
            .margin_bottom(0)
            .visible(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .expand(true)
            .build();
        container.style_context().add_class("terminal-box");
        container.pack_end(&list, false, false, 0);
        scrolled_window.set_child(Some(&container));
    }
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin(5)
        .spacing(6)
        .margin_bottom(10)
        .visible(true)
        .build();
    let entry = gtk::Entry::new();
    entry.style_context().add_class("terminal-entry");
    entry.set_visible(true);
    let locals_dict: Py<PyDict> = Python::with_gil(|py| {
        let dict: Py<PyDict> = PyDict::new(py).into();
        dict
    });
    let globals_dict: Py<PyDict> = Python::with_gil(|py| setup_globals(py, &locals_dict).unwrap());

    let hist = Rc::new(RefCell::new(ShellHistory { history: vec![] }));

    // shell stdout channel
    let (tx, rx) = MainContext::channel::<(LinePrefix, String)>(PRIORITY_DEFAULT);
    // shell stdin channel
    let (tx_shell, rx_shell) = std::sync::mpsc::channel::<String>();
    // shell -> app channel
    let (tx_py, rx_py) = MainContext::channel(PRIORITY_DEFAULT);
    // app -> shell channel
    let (tx_py2, rx_py2) = std::sync::mpsc::channel::<String>();
    rx_py.attach(
        None,
        clone!(@weak app, @weak list, @weak adj => @default-return Continue(false), move |msg: String| {
            match msg.as_str() {
                "project-name" => tx_py2.send(app.window.project().property::<String>(Project::NAME)).unwrap(),
                // [ref:FIXME]: send serialized error
                _ => tx_py2.send(String::new()).unwrap(),
            }
            Continue(true)
        }),
    );
    rx.attach(
        None,
        clone!(@weak app, @weak list, @weak adj, @strong hist => @default-return Continue(false), move |(prefix, mut msg)| {
            if !msg.is_empty() {
                while msg.ends_with('\n') {
                    msg.pop();
                }
                let label = gtk::Label::new(Some(&format!("{}{msg}", match prefix {
                    LinePrefix::Output => "",
                    LinePrefix::Ps1 => SYS_PS1,
                    LinePrefix::Ps2 => SYS_PS2,
                })));
                hist.borrow_mut().history.push((prefix, msg));
                label.set_wrap(true);
                label.set_selectable(true);
                label.set_visible(true);
                label.set_halign(gtk::Align::Start);
                label.set_valign(gtk::Align::End);
                list.add(&label);
                list.queue_draw();
                adj.set_value(adj.upper());
            }
            Continue(true)
        }),
    );
    std::thread::spawn(move || {
        shell_thread(tx, globals_dict, locals_dict, tx_py, rx_py2, rx_shell).unwrap()
    });

    entry.connect_activate(clone!(@weak app, @weak list, @weak adj => move |entry| {
        let buffer = entry.buffer();
        let text = buffer.text();
        buffer.set_text("");
        if let Err(err) = tx_shell.send(if text.is_empty() { "\n".to_string() } else { text }) {
            eprintln!("Internal error: {err}");
        }
    }));

    {
        let clear_btn = gtk::Button::builder()
            .label("Clear")
            .relief(gtk::ReliefStyle::None)
            .visible(true)
            .halign(gtk::Align::End)
            .valign(gtk::Align::Center)
            .build();
        clear_btn.connect_clicked(clone!(@weak list, @strong hist => move |_| {
            for c in list.children() {
                list.remove(&c);
            }
            hist.borrow_mut().history.clear();
        }));
        let save_history_btn = gtk::Button::builder()
            .label("Save history")
            .relief(gtk::ReliefStyle::None)
            .visible(true)
            .sensitive(false)
            .halign(gtk::Align::End)
            .valign(gtk::Align::Center)
            .build();
        save_history_btn.connect_clicked(clone!(@weak list => move |_| {
        }));
        let copy_history_btn = gtk::Button::builder()
            .label("Copy history")
            .relief(gtk::ReliefStyle::None)
            .visible(true)
            .halign(gtk::Align::End)
            .valign(gtk::Align::Center)
            .build();
        copy_history_btn.connect_clicked(clone!(@weak list, @strong hist => move |copy_history_btn| {
            if let Some(clip) = gtk::Clipboard::default(&copy_history_btn.display()) {
                let output = hist.borrow().history.iter().fold(String::new(), |mut acc, (p, l)| {
                    match p {
                        LinePrefix::Output =>{},
                        LinePrefix::Ps1 => acc.push_str(SYS_PS1),
                        LinePrefix::Ps2 => acc.push_str(SYS_PS2),
                    }
                    acc.push_str(l);
                    acc.push('\n');
                    acc
                });
                clip.set_text(&output);
            }
        }));
        let btn_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .margin(0)
            .visible(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .hexpand(true)
            .vexpand(false)
            .build();
        btn_container.pack_end(&save_history_btn, false, false, 0);
        btn_container.pack_end(&copy_history_btn, false, false, 0);
        btn_container.pack_end(&clear_btn, false, false, 0);
        let close_btn = gtk::Button::builder()
            .label("Close")
            .relief(gtk::ReliefStyle::None)
            .visible(true)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .build();
        close_btn.connect_clicked(clone!(@weak w => move |_| {
            w.close();
        }));
        btn_container.pack_start(&close_btn, false, false, 0);
        b.pack_start(&btn_container, false, true, 0);
    }
    b.pack_start(&scrolled_window, true, true, 0);
    {
        let entry_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .margin(0)
            .spacing(4)
            .visible(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();
        let enter_btn = gtk::Button::builder()
            .label("enter")
            .relief(gtk::ReliefStyle::None)
            .visible(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();
        enter_btn.connect_clicked(clone!(@weak w => move |_| {
        }));
        entry_container.pack_start(&entry, true, true, 0);
        entry_container.pack_end(&enter_btn, false, false, 0);
        b.pack_start(&entry_container, false, true, 0);
    }
    w.set_child(Some(&b));
    w.show_all();
    w
}

/// Handle input inside of shell instance
fn handle_input(
    text: String,
    needed_more: bool,
    py: Python<'_>,
    locals: &PyDict,
    globals: &PyDict,
    tx: &glib::Sender<(LinePrefix, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    locals.set_item("gerb", globals.get_item("gerb").unwrap())?;
    let shell = globals.get_item("gerb").unwrap().getattr("__shell")?;
    Ok(
        match shell
            .getattr("push")?
            .call1((if text.is_empty() { "\n" } else { &text },))
        {
            Ok(result) => {
                let needs_more_input: bool = result.extract()?;
                if !needs_more_input {
                    let r = py
                        .eval("gerb.__stdout.getvalue()", Some(globals), None)?
                        .str()?
                        .to_string_lossy();
                    py.run(
                        "gerb.__stdout.seek(0); gerb.__stdout.truncate(0)",
                        Some(globals),
                        None,
                    )?;
                    if !r.is_empty() {
                        if needed_more {
                            tx.send((LinePrefix::Ps2, format!("{}\n", &text,)))?;
                            tx.send((LinePrefix::Output, r.to_string()))?;
                        } else {
                            tx.send((LinePrefix::Ps1, format!("{}\n", &text,)))?;
                            tx.send((LinePrefix::Output, r.to_string()))?;
                        }
                    } else if needed_more {
                        tx.send((LinePrefix::Ps2, text))?;
                    } else {
                        tx.send((LinePrefix::Ps1, text))?;
                    }
                    needs_more_input
                } else {
                    if needed_more {
                        tx.send((
                            LinePrefix::Ps2,
                            if text.is_empty() {
                                "\n".to_string()
                            } else {
                                text
                            },
                        ))?;
                    } else {
                        tx.send((
                            LinePrefix::Ps1,
                            if text.is_empty() {
                                "\n".to_string()
                            } else {
                                text
                            },
                        ))?;
                    }
                    needs_more_input
                }
            }
            Err(err) => {
                tx.send((LinePrefix::Output, err.to_string()))?;
                false
            }
        },
    )
}

/// Helper function to setup python globals.
fn setup_globals<'py>(
    py: Python<'py>,
    locals_dict: &Py<PyDict>,
) -> Result<Py<PyDict>, Box<dyn std::error::Error + 'py>> {
    let globals = PyDict::new(py);
    // Import and get sys.modules
    let sys = PyModule::import(py, "sys")?;
    let py_modules: &PyDict = sys.getattr("modules").unwrap().downcast()?;

    let io = PyModule::import(py, "io")?;
    let code = PyModule::import(py, "code")?;
    py_modules.set_item("sys", sys)?;
    py.import("io")?;
    py.import("sys")?;
    let gerb = Gerb {
        __stdout: io.getattr("StringIO")?.call0()?.into(),
        __shell: code
            .getattr("InteractiveConsole")?
            .call1((locals_dict.as_ref(py),))?
            .into(),
        __send: Py::new(py, Sender(None))?,
        __rcv: Py::new(py, Receiver(None))?,
        __types_dict: Gerb::types(py),
    };
    let gerb = PyCell::new(py, gerb)?;
    globals.set_item("gerb", gerb)?;
    globals.set_item("sys", sys)?;
    py.run(
        r#"""
sys.stdout = gerb.__stdout
sys.stderr = gerb.__stdout
"""#,
        Some(globals),
        None,
    )?;
    Ok(globals.into())
}

fn shell_thread(
    tx: glib::Sender<(LinePrefix, String)>,
    globals_dict: Py<PyDict>,
    locals_dict: Py<PyDict>,
    tx_py: glib::Sender<String>,
    rx_py2: mpsc::Receiver<String>,
    rx_shell: mpsc::Receiver<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let needs_more_input = Cell::new(false);
    let globals = &globals_dict;
    let locals = &locals_dict;
    let res: Result<(), Box<dyn std::error::Error>> = Python::with_gil(|py| {
        {
            let c: &PyCell<Sender> = globals
                .as_ref(py)
                .get_item("gerb")
                .unwrap()
                .getattr("__send")?
                .extract()?;
            let mut guard: PyRefMut<'_, Sender> = c.borrow_mut();
            let n_mutable: &mut Sender = &mut guard;
            n_mutable.0 = Some(tx_py);
        }
        {
            let c: &PyCell<Receiver> = globals
                .as_ref(py)
                .get_item("gerb")
                .unwrap()
                .getattr("__rcv")?
                .extract()?;
            let mut guard: PyRefMut<'_, Receiver> = c.borrow_mut();
            let n_mutable: &mut Receiver = &mut guard;
            n_mutable.0 = Some(rx_py2);
        }
        locals
            .as_ref(py)
            .set_item("gerb", globals.as_ref(py).get_item("gerb").unwrap())?;
        Ok(())
    });
    res?;

    while let Ok(text) = rx_shell.recv() {
        if text.is_empty() && !needs_more_input.get() {
            let res: Result<(), Box<dyn std::error::Error>> = Python::with_gil(|py| {
                let c: &PyCell<Sender> = globals
                    .as_ref(py)
                    .get_item("gerb")
                    .unwrap()
                    .getattr("__send")?
                    .extract()?;
                let mut guard: PyRefMut<'_, Sender> = c.borrow_mut();
                let n_mutable: &mut Sender = &mut guard;
                n_mutable.0.as_ref().unwrap().send(String::new())?;
                Ok(())
            });
            res?;
            continue;
        }
        needs_more_input.set(Python::with_gil(|py| {
            handle_input(
                text,
                needs_more_input.get(),
                py,
                locals.as_ref(py),
                globals.as_ref(py),
                &tx,
            )
            .unwrap()
        }));
    }
    Ok(())
}
