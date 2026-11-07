// SPDX-License-Identifier: GPL-3.0-or-later
//! XDG base directories.

use std::ffi::OsString;
use std::path::PathBuf;

/// `$var`, or `$HOME/<home_fallback>` if it is unset or empty, with `sub` appended.
///
/// `xdg_dir("XDG_CONFIG_HOME", ".config", "myapp")` → `~/.config/myapp`.
pub fn xdg_dir(var: &str, home_fallback: &str, sub: &str) -> Option<PathBuf> {
    resolve(
        std::env::var_os(var),
        std::env::var_os("HOME"),
        home_fallback,
        sub,
    )
}

/// `$XDG_CONFIG_HOME/<sub>` or `~/.config/<sub>`.
pub fn config_dir(sub: &str) -> Option<PathBuf> {
    xdg_dir("XDG_CONFIG_HOME", ".config", sub)
}

/// `$XDG_DATA_HOME/<sub>` or `~/.local/share/<sub>`.
pub fn data_dir(sub: &str) -> Option<PathBuf> {
    xdg_dir("XDG_DATA_HOME", ".local/share", sub)
}

fn resolve(
    value: Option<OsString>,
    home: Option<OsString>,
    home_fallback: &str,
    sub: &str,
) -> Option<PathBuf> {
    let base = value
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.map(|h| PathBuf::from(h).join(home_fallback)))?;
    Some(base.join(sub))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_wins_and_empty_falls_back_to_home() {
        let r = |v: Option<&str>, h: Option<&str>| {
            resolve(
                v.map(OsString::from),
                h.map(OsString::from),
                ".config",
                "app",
            )
        };
        assert_eq!(r(Some("/x"), Some("/h")), Some(PathBuf::from("/x/app")));
        assert_eq!(
            r(Some(""), Some("/h")),
            Some(PathBuf::from("/h/.config/app"))
        );
        assert_eq!(r(None, Some("/h")), Some(PathBuf::from("/h/.config/app")));
        assert_eq!(r(None, None), None);
    }
}
