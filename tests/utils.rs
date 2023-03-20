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

#[allow(dead_code)]
/// Copied from: https://github.com/gtk-rs/gtk4-rs/blob/dafd16367d6782648616961d9f7ba0384cd7bffa/gtk4/src/lib.rs#L59
pub fn gtk_test_wrapper<F, R>(function: F) -> R
where
    F: FnOnce() -> R + Send + std::panic::UnwindSafe + 'static,
    R: Send + 'static,
{
    use std::panic;
    use std::sync::mpsc;

    #[allow(dead_code)]
    static TEST_THREAD_WORKER: once_cell::sync::Lazy<glib::ThreadPool> =
        once_cell::sync::Lazy::new(|| {
            let pool = glib::ThreadPool::exclusive(1).unwrap();
            pool.push(move || {
                gtk::init().expect("Tests failed to initialize gtk");
            })
            .expect("Failed to schedule a test call");
            pool
        });

    let (tx, rx) = mpsc::sync_channel(1);
    TEST_THREAD_WORKER
        .push(move || {
            tx.send(panic::catch_unwind(function))
                .unwrap_or_else(|_| panic!("Failed to return result from thread pool"));
        })
        .expect("Failed to schedule a test call");
    rx.recv()
        .expect("Failed to receive result from thread pool")
        .unwrap_or_else(|e| std::panic::resume_unwind(e))
}

#[allow(dead_code)]
/// Copied from: https://github.com/gtk-rs/gtk4-rs/blob/dafd16367d6782648616961d9f7ba0384cd7bffa/gtk4/src/lib.rs#L59
pub fn glib_test_wrapper<F, R>(function: F) -> R
where
    F: FnOnce() -> R + Send + std::panic::UnwindSafe + 'static,
    R: Send + 'static,
{
    use std::panic;
    use std::sync::mpsc;

    #[allow(dead_code)]
    static TEST_THREAD_WORKER: once_cell::sync::Lazy<glib::ThreadPool> =
        once_cell::sync::Lazy::new(|| glib::ThreadPool::exclusive(1).unwrap());

    let (tx, rx) = mpsc::sync_channel(1);
    TEST_THREAD_WORKER
        .push(move || {
            tx.send(panic::catch_unwind(function))
                .unwrap_or_else(|_| panic!("Failed to return result from thread pool"));
        })
        .expect("Failed to schedule a test call");
    rx.recv()
        .expect("Failed to receive result from thread pool")
        .unwrap_or_else(|e| std::panic::resume_unwind(e))
}
