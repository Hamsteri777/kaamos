// SPDX-License-Identifier: GPL-3.0-or-later
//! Linux desktop helpers for Slint apps, collected from PulsarFe and NARME:
//!
//! - [`platform::init`]: Wayland app id and the Plasma activation token, so the task bar
//!   matches the window to its `.desktop` file and no empty "starting" entry is left behind.
//! - [`tray`]: a StatusNotifierItem tray icon via ksni in its own thread (Slint's own
//!   `SystemTrayIcon` creates an extra empty window on Wayland).
//! - [`autostart`]: start at login via `~/.config/autostart/<app>.desktop`.
//! - [`paths`]: XDG directories.
//! - [`poll`]: deliver messages from worker threads to the UI thread.
//!
//! The Slint components live in the `kaamos-ui` crate (`@kaamos/kaamos.slint`).

pub mod autostart;
pub mod paths;
pub mod platform;
pub mod poll;
#[cfg(feature = "tray")]
pub mod tray;
