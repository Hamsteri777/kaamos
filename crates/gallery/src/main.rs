// SPDX-License-Identifier: GPL-3.0-or-later
//! Kaamos gallery. Run: `cargo run -p kaamos-gallery`.

use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // Wayland app id + activation token (see kaamos::platform).
    kaamos::platform::init("kaamos-gallery")?;
    let ui = Gallery::new()?;
    let w = ui.as_weak();
    ui.on_notify(move |text| {
        if let Some(ui) = w.upgrade() {
            ui.set_status(text);
        }
    });
    ui.run()
}
