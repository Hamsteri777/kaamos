// SPDX-License-Identifier: GPL-3.0-or-later
//! Wayland integration that Slint does not do by itself.

use slint::PlatformError;

/// Call once at the start of `main`, before creating any window.
///
/// - Selects the winit backend so that the first window uses the Plasma/xdg activation token
///   (`XDG_ACTIVATION_TOKEN`). Without it, launching from the app menu leaves an empty
///   "starting" entry next to the window in the task bar until KWin's timeout.
/// - Sets the Wayland app id to `app_id`. Use the name of your `.desktop` file (without
///   `.desktop`), otherwise the task bar does not match the window to the app and shows the
///   window and the launcher separately.
pub fn init(app_id: &str) -> Result<(), PlatformError> {
    use slint::winit_030::winit::platform::startup_notify::WindowAttributesExtStartupNotify;
    use slint::winit_030::winit::window::ActivationToken;
    let token = std::cell::Cell::new(std::env::var("XDG_ACTIVATION_TOKEN").ok());
    slint::BackendSelector::new()
        .with_winit_window_attributes_hook(move |attributes| match token.take() {
            Some(t) => attributes.with_activation_token(ActivationToken::from_raw(t)),
            None => attributes,
        })
        .select()?;
    slint::set_xdg_app_id(app_id)
}
