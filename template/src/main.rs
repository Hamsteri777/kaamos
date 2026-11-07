//! __TITLE__.
//!
//! Structure (from slint-kit):
//! - a worker thread owns the slow resource and answers commands over a channel; the UI
//!   never waits for it (`poll::drain_every` delivers the replies on the UI thread);
//! - `platform::init` sets the Wayland app id and uses the activation token;
//! - a ksni tray (Show / Quit); closing the window hides it, `--tray` starts hidden;
//! - start at login via `autostart`.

use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use slint::ComponentHandle;
use slint_kit::tray::{MenuEntry, TrayImage, TrayState};
use slint_kit::{autostart, platform, poll, tray};

slint::include_modules!();

/// Also the name of the `.desktop` file (packaging/__APP__.desktop).
const APP_ID: &str = "__APP__";
const TITLE: &str = "__TITLE__";

enum Cmd {
    Load,
    Save { enabled: bool, level: i32 },
}

enum Reply {
    Loaded { enabled: bool, level: i32 },
    Saved,
}

/// The worker thread. Replace the body with your device, network or file access.
fn spawn_worker(replies: mpsc::Sender<Reply>) -> mpsc::Sender<Cmd> {
    let (tx, rx) = mpsc::channel::<Cmd>();
    thread::spawn(move || {
        let mut stored = (true, 3);
        for cmd in rx {
            thread::sleep(Duration::from_millis(300)); // pretend this is slow
            let reply = match cmd {
                Cmd::Load => Reply::Loaded {
                    enabled: stored.0,
                    level: stored.1,
                },
                Cmd::Save { enabled, level } => {
                    stored = (enabled, level);
                    Reply::Saved
                }
            };
            if replies.send(reply).is_err() {
                break;
            }
        }
    });
    tx
}

fn show_window(ui: &slint::Weak<MainWindow>) {
    let _ = ui.upgrade_in_event_loop(|ui| {
        let _ = ui.show();
    });
}

fn quit() {
    let _ = slint::invoke_from_event_loop(|| {
        let _ = slint::quit_event_loop();
    });
}

fn main() -> Result<(), slint::PlatformError> {
    let tray_only = std::env::args().any(|a| a == "--tray");
    platform::init(APP_ID)?;
    let ui = MainWindow::new()?;
    let (reply_tx, reply_rx) = mpsc::channel::<Reply>();
    let tx = spawn_worker(reply_tx);

    // Tray. Its callbacks run on the tray thread.
    let tray = Rc::new({
        let (w1, w2) = (ui.as_weak(), ui.as_weak());
        tray::spawn(
            TrayState {
                id: APP_ID.into(),
                title: TITLE.into(),
                image: TrayImage::Name("applications-system".into()),
                tooltip_title: TITLE.into(),
                tooltip_body: String::new(),
                menu: vec![
                    MenuEntry::Action {
                        label: "Show window".into(),
                        id: "show".into(),
                    },
                    MenuEntry::Separator,
                    MenuEntry::Action {
                        label: "Quit".into(),
                        id: "quit".into(),
                    },
                ],
            },
            move || show_window(&w1),
            move |id| match id {
                "show" => show_window(&w2),
                "quit" => quit(),
                _ => {}
            },
        )
    });
    // Without a tray, closing the window quits (otherwise it hides to the tray).
    if tray.is_none() {
        ui.window().on_close_requested(|| {
            let _ = slint::quit_event_loop();
            slint::CloseRequestResponse::HideWindow
        });
    }

    // Worker replies → UI.
    let _replies = {
        let (w, tray) = (ui.as_weak(), tray.clone());
        poll::drain_every(reply_rx, Duration::from_millis(50), move |reply| {
            let Some(ui) = w.upgrade() else {
                return;
            };
            ui.set_busy(false);
            match reply {
                Reply::Loaded { enabled, level } => {
                    ui.set_loaded_enabled(enabled);
                    ui.set_loaded_level(level);
                    ui.set_enabled(enabled);
                    ui.set_level(level);
                    ui.set_status("Loaded.".into());
                    if let Some(h) = tray.as_ref() {
                        tray::update(h, |s| s.tooltip_body = format!("Level {level}"));
                    }
                }
                Reply::Saved => {
                    ui.set_loaded_enabled(ui.get_enabled());
                    ui.set_loaded_level(ui.get_level());
                    ui.set_status("Saved.".into());
                }
            }
            ui.set_status_error(false);
        })
    };

    {
        let (w, tx) = (ui.as_weak(), tx.clone());
        ui.on_reload(move || {
            let ui = w.unwrap();
            ui.set_busy(true);
            let _ = tx.send(Cmd::Load);
        });
    }
    {
        let (w, tx) = (ui.as_weak(), tx.clone());
        ui.on_save(move || {
            let ui = w.unwrap();
            ui.set_busy(true);
            let _ = tx.send(Cmd::Save {
                enabled: ui.get_enabled(),
                level: ui.get_level(),
            });
        });
    }
    {
        let w = ui.as_weak();
        ui.on_revert(move || {
            let ui = w.unwrap();
            ui.set_enabled(ui.get_loaded_enabled());
            ui.set_level(ui.get_loaded_level());
        });
    }

    // Start at login.
    ui.set_autostart(autostart::is_enabled(APP_ID));
    {
        let w = ui.as_weak();
        ui.on_autostart_toggled(move |on| {
            let ui = w.unwrap();
            let entry = autostart::Entry {
                app_id: APP_ID,
                name: TITLE,
                comment: TITLE,
                icon: APP_ID,
                args: &["--tray"],
            };
            let result = if on {
                std::env::current_exe()
                    .and_then(|exe| autostart::enable(&entry, &exe))
                    .map(|p| format!("Start at login: {}", p.display()))
            } else {
                autostart::disable(APP_ID).map(|()| "Start at login off.".to_string())
            };
            match result {
                Ok(msg) => ui.set_status(msg.into()),
                Err(e) => {
                    ui.set_autostart(!on);
                    ui.set_status(format!("{e}").into());
                    ui.set_status_error(true);
                }
            }
        });
    }

    ui.set_busy(true);
    let _ = tx.send(Cmd::Load);
    if !tray_only || tray.is_none() {
        ui.show()?;
    }
    // Keeps running while the window is hidden; quit from the tray menu.
    slint::run_event_loop_until_quit()
}
