// SPDX-License-Identifier: GPL-3.0-or-later
//! Start at login: a freedesktop autostart file `~/.config/autostart/<app_id>.desktop`
//! (KDE, GNOME and most desktops read it). Only the user's own config directory is touched.

use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
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

/// Escapes a value of type string (Desktop Entry spec: `\\` for a backslash). Control
/// characters (a newline would start a new key) are refused.
fn escape_string(key: &str, s: &str) -> io::Result<String> {
    if s.chars().any(char::is_control) {
        return Err(invalid(format!("{key} contains a control character")));
    }
    Ok(s.replace('\\', "\\\\"))
}

/// Quotes one Exec argument. The spec escapes in two layers: inside quotes `"`, `` ` ``, `$`
/// and `\` get a backslash, and then the whole value is escaped as a string, which doubles
/// every backslash. `%` starts a field code, so a literal one is written `%%`.
fn quote_exec_arg(s: &str) -> io::Result<String> {
    if s.chars().any(char::is_control) {
        return Err(invalid(format!(
            "Exec argument contains a control character: {s:?}"
        )));
    }
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '`' | '$' => {
                out.push_str("\\\\");
                out.push(c);
            }
            '\\' => out.push_str("\\\\\\\\"),
            '%' => out.push_str("%%"),
            _ => out.push(c),
        }
    }
    out.push('"');
    Ok(out)
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, msg)
}

/// The autostart file's text. Fails on control characters and on a path that is not UTF-8.
pub fn desktop_entry(entry: &Entry, exe: &Path) -> io::Result<String> {
    let exe = exe
        .to_str()
        .ok_or_else(|| invalid(format!("executable path is not UTF-8: {}", exe.display())))?;
    let mut exec = quote_exec_arg(exe)?;
    for a in entry.args {
        exec.push(' ');
        exec.push_str(&quote_exec_arg(a)?);
    }
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nExec={exec}\nIcon={}\n\
         Terminal=false\nX-GNOME-Autostart-enabled=true\n",
        escape_string("Name", entry.name)?,
        escape_string("Comment", entry.comment)?,
        escape_string("Icon", entry.icon)?,
    ))
}

/// `current_exe()` of a binary that was replaced while running (e.g. by a package upgrade)
/// ends in " (deleted)"; the new binary is at the path without it.
fn strip_deleted(exe: &Path) -> &Path {
    exe.as_os_str()
        .as_bytes()
        .strip_suffix(b" (deleted)")
        .map_or(exe, |b| Path::new(OsStr::from_bytes(b)))
}

/// Writes the autostart file for `exe` (usually `std::env::current_exe()`). Fails if `exe`
/// does not exist (after removing a " (deleted)" suffix).
pub fn enable(entry: &Entry, exe: &Path) -> io::Result<PathBuf> {
    let exe = strip_deleted(exe);
    if !exe.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("executable not found: {}", exe.display()),
        ));
    }
    let text = desktop_entry(entry, exe)?;
    let path = desktop_path(entry.app_id).ok_or_else(|| io::Error::other("no $HOME"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, text)?;
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
        let e = desktop_entry(&ENTRY, Path::new("/usr/bin/myapp")).unwrap();
        assert!(e.starts_with("[Desktop Entry]\n"));
        assert!(e.contains("\nExec=\"/usr/bin/myapp\" \"--tray\"\n"), "{e}");
        assert!(e.contains("\nName=My App\n"));
        assert!(e.contains("\nIcon=myapp\n"));
    }

    #[test]
    fn exec_is_escaped_in_both_layers() {
        // Path: /tmp/a b/$x"y\z%u`w
        let e = desktop_entry(&ENTRY, Path::new("/tmp/a b/$x\"y\\z%u`w")).unwrap();
        let want = r#"Exec="/tmp/a b/\\$x\\"y\\\\z%%u\\`w" "--tray""#;
        assert!(e.contains(&format!("\n{want}\n")), "{e}");
    }

    #[test]
    fn backslash_in_name_is_escaped() {
        let entry = Entry {
            name: r"A\B",
            ..ENTRY
        };
        let e = desktop_entry(&entry, Path::new("/usr/bin/myapp")).unwrap();
        assert!(e.contains("\nName=A\\\\B\n"), "{e}");
    }

    #[test]
    fn control_characters_are_refused() {
        let entry = Entry {
            name: "My App\nExec=evil",
            ..ENTRY
        };
        assert!(desktop_entry(&entry, Path::new("/usr/bin/myapp")).is_err());
        let entry = Entry {
            args: &["--tray\n"],
            ..ENTRY
        };
        assert!(desktop_entry(&entry, Path::new("/usr/bin/myapp")).is_err());
    }

    #[test]
    fn non_utf8_path_is_refused() {
        let exe = Path::new(OsStr::from_bytes(b"/usr/bin/my\xffapp"));
        assert!(desktop_entry(&ENTRY, exe).is_err());
    }

    #[test]
    fn deleted_suffix_is_removed() {
        assert_eq!(
            strip_deleted(Path::new("/usr/bin/myapp (deleted)")),
            Path::new("/usr/bin/myapp")
        );
        assert_eq!(
            strip_deleted(Path::new("/usr/bin/myapp")),
            Path::new("/usr/bin/myapp")
        );
    }

    #[test]
    fn missing_executable_is_refused() {
        let err = enable(&ENTRY, Path::new("/nonexistent/myapp (deleted)")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
