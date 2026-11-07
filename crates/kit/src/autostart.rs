// SPDX-License-Identifier: GPL-3.0-or-later
//! Start at login: a freedesktop autostart file `~/.config/autostart/<app_id>.desktop`
//! (KDE, GNOME and most desktops read it). Only the user's own config directory is touched.

use std::io;
use std::path::{Path, PathBuf};

/// What the autostart entry says.
#[derive(Debug, Clone)]
pub struct Entry<'a> {
    /// File name without `.desktop`; use the same id as the app's `.desktop` file.
    pub app_id: &'a str,
    pub name: &'a str,
    pub comment: &'a str,
    pub icon: &'a str,
    /// Arguments after the executable, e.g. `["--tray"]`.
    pub args: &'a [&'a str],
}

pub fn desktop_path(app_id: &str) -> Option<PathBuf> {
    crate::paths::config_dir("autostart").map(|d| d.join(format!("{app_id}.desktop")))
}

pub fn is_enabled(app_id: &str) -> bool {
    desktop_path(app_id).is_some_and(|p| p.exists())
}

/// Quotes an Exec argument (Desktop Entry spec: escape `"`, `` ` ``, `$` and `\`).
fn quote_exec_arg(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

pub fn desktop_entry(entry: &Entry, exe: &Path) -> String {
    let mut exec = quote_exec_arg(&exe.to_string_lossy());
    for a in entry.args {
        exec.push(' ');
        exec.push_str(&quote_exec_arg(a));
    }
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nExec={exec}\nIcon={}\n\
         Terminal=false\nX-GNOME-Autostart-enabled=true\n",
        entry.name, entry.comment, entry.icon
    )
}

/// Writes the autostart file for `exe` (usually `std::env::current_exe()`).
pub fn enable(entry: &Entry, exe: &Path) -> io::Result<PathBuf> {
    let path = desktop_path(entry.app_id).ok_or_else(|| io::Error::other("no $HOME"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, desktop_entry(entry, exe))?;
    Ok(path)
}

/// Removes the autostart file if it exists.
pub fn disable(app_id: &str) -> io::Result<()> {
    match desktop_path(app_id) {
        Some(path) if path.exists() => std::fs::remove_file(path),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: Entry = Entry {
        app_id: "myapp",
        name: "My App",
        comment: "Does things",
        icon: "myapp",
        args: &["--tray"],
    };

    #[test]
    fn entry_starts_with_args() {
        let e = desktop_entry(&ENTRY, Path::new("/usr/bin/myapp"));
        assert!(e.starts_with("[Desktop Entry]\n"));
        assert!(e.contains("\nExec=\"/usr/bin/myapp\" \"--tray\"\n"), "{e}");
        assert!(e.contains("\nName=My App\n"));
        assert!(e.contains("\nIcon=myapp\n"));
    }

    #[test]
    fn exec_path_is_quoted_and_escaped() {
        let e = desktop_entry(&ENTRY, Path::new("/tmp/a b/$x\"y"));
        assert!(e.contains("Exec=\"/tmp/a b/\\$x\\\"y\""), "{e}");
    }
}
