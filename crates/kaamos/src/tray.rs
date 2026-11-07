// SPDX-License-Identifier: GPL-3.0-or-later
//! StatusNotifierItem tray icon via ksni in its own thread.
//!
//! Slint's own `SystemTrayIcon` creates an extra empty Wayland window that shows up in the
//! task bar, so the tray is done with ksni (as in NARME and PulsarFe). Callbacks run on the
//! tray thread: forward them to the UI with `slint::invoke_from_event_loop` or
//! `Weak::upgrade_in_event_loop`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::StandardItem;
use ksni::{Icon, MenuItem, ToolTip};

/// The tray icon image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayImage {
    /// Freedesktop icon name, e.g. "input-mouse".
    Name(String),
    /// RGBA pixels, row by row.
    Rgba {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    },
}

/// One menu entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    /// Greyed-out information line.
    Info(String),
    /// Clickable entry; `id` is passed to the menu callback.
    Action {
        label: String,
        id: String,
    },
    Separator,
}

/// Everything the tray shows. Change it with [`update`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayState {
    /// D-Bus id, e.g. the app id.
    pub id: String,
    pub title: String,
    pub image: TrayImage,
    pub tooltip_title: String,
    pub tooltip_body: String,
    pub menu: Vec<MenuEntry>,
}

/// RGBA → ARGB32 in network byte order (what StatusNotifierItem expects).
pub fn rgba_to_argb(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| [a, r, g, b])
        .collect()
}

pub struct KitTray {
    state: TrayState,
    on_activate: Box<dyn Fn() + Send>,
    on_menu: Box<dyn Fn(&str) + Send>,
    online: Arc<AtomicBool>,
}

/// Whether a tray host (the panel's StatusNotifierWatcher) is showing the icon right now.
/// Cheap to clone and to read from the UI thread.
#[derive(Debug, Clone)]
pub struct TrayStatus(Arc<AtomicBool>);

impl TrayStatus {
    /// `false` while there is no tray host (e.g. GNOME without the AppIndicator extension, or
    /// the panel restarting). Then a hidden window cannot be brought back: closing the window
    /// should quit, and a window started hidden should be shown.
    pub fn online(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl ksni::Tray for KitTray {
    fn id(&self) -> String {
        self.state.id.clone()
    }

    fn title(&self) -> String {
        self.state.title.clone()
    }

    fn icon_name(&self) -> String {
        match &self.state.image {
            TrayImage::Name(n) => n.clone(),
            TrayImage::Rgba { .. } => String::new(),
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        match &self.state.image {
            TrayImage::Rgba {
                width,
                height,
                pixels,
            } => vec![Icon {
                width: *width as i32,
                height: *height as i32,
                data: rgba_to_argb(pixels),
            }],
            TrayImage::Name(_) => Vec::new(),
        }
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: self.state.tooltip_title.clone(),
            description: self.state.tooltip_body.clone(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        (self.on_activate)();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        self.state
            .menu
            .iter()
            .map(|e| match e {
                MenuEntry::Info(label) => StandardItem {
                    label: label.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
                MenuEntry::Action { label, id } => {
                    let id = id.clone();
                    StandardItem {
                        label: label.clone(),
                        activate: Box::new(move |t: &mut Self| (t.on_menu)(&id)),
                        ..Default::default()
                    }
                    .into()
                }
                MenuEntry::Separator => MenuItem::Separator,
            })
            .collect()
    }

    fn watcher_online(&self) {
        self.online.store(true, Ordering::Relaxed);
    }

    /// The panel's tray may start after the app (autostart at login): keep waiting.
    fn watcher_offline(&self, _reason: ksni::OfflineReason) -> bool {
        self.online.store(false, Ordering::Relaxed);
        true
    }
}

pub type TrayHandle = Handle<KitTray>;

/// Starts the tray. `on_activate`: left click on the icon; `on_menu`: a menu action's id.
///
/// `None` only if the D-Bus service itself could not be started. A missing tray host does
/// *not* give `None`: the service keeps waiting for one (the panel may start after the app),
/// so check [`TrayStatus::online`] before hiding the window (see [`spawn_with_status`]).
pub fn spawn(
    state: TrayState,
    on_activate: impl Fn() + Send + 'static,
    on_menu: impl Fn(&str) + Send + 'static,
) -> Option<TrayHandle> {
    spawn_with_status(state, on_activate, on_menu).map(|(handle, _)| handle)
}

/// [`spawn`] plus a [`TrayStatus`] that says whether a tray host currently shows the icon.
///
/// Typical use: on close, hide the window only if `status.online()`, otherwise quit; and a
/// few seconds after a `--tray` start, show the window if the status is still offline.
pub fn spawn_with_status(
    state: TrayState,
    on_activate: impl Fn() + Send + 'static,
    on_menu: impl Fn(&str) + Send + 'static,
) -> Option<(TrayHandle, TrayStatus)> {
    // Online until ksni says otherwise; without a host it calls `watcher_offline` right away.
    let online = Arc::new(AtomicBool::new(true));
    let handle = KitTray {
        state,
        on_activate: Box::new(on_activate),
        on_menu: Box::new(on_menu),
        online: Arc::clone(&online),
    }
    .assume_sni_available(true)
    .spawn()
    .ok()?;
    Some((handle, TrayStatus(online)))
}

/// Changes the tray state; nothing is sent over D-Bus if it did not change.
pub fn update(handle: &TrayHandle, f: impl FnOnce(&mut TrayState)) {
    handle.update(|t| {
        let mut next = t.state.clone();
        f(&mut next);
        if next != t.state {
            t.state = next;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argb_order() {
        assert_eq!(
            rgba_to_argb(&[1, 2, 3, 4, 5, 6, 7, 8]),
            vec![4, 1, 2, 3, 8, 5, 6, 7]
        );
    }
}
