// SPDX-License-Identifier: GPL-3.0-or-later
//! Messages from worker threads to the UI thread.
//!
//! Pattern: the worker owns the slow resource (device, network, …) and sends results over
//! an `mpsc` channel; the UI never waits. `drain_every` empties the channel on the UI thread
//! with a Slint timer, so the handler can keep `Rc`/`RefCell` state and touch the window.

use std::sync::mpsc::Receiver;
use std::time::Duration;

use slint::{Timer, TimerMode};

/// Calls `handle` for every message in `rx` every `interval` on the UI thread.
/// Keep the returned timer alive (e.g. bind it to a variable in `main`).
#[must_use = "the timer stops when it is dropped"]
pub fn drain_every<T: 'static>(
    rx: Receiver<T>,
    interval: Duration,
    mut handle: impl FnMut(T) + 'static,
) -> Timer {
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, interval, move || {
        while let Ok(msg) = rx.try_recv() {
            handle(msg);
        }
    });
    timer
}
