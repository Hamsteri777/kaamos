// SPDX-License-Identifier: GPL-3.0-or-later
//! Kaamos' `.slint` files, fonts and icons. Use this crate as a **build-dependency**:
//! it has no dependencies, so the build script does not compile Slint a second time.
//!
//! ```ignore
//! // build.rs
//! let config = slint_build::CompilerConfiguration::new()
//!     .with_style("fluent-dark".into())
//!     .with_library_paths(kaamos_ui::library_paths());
//! slint_build::compile_with_config("ui/main.slint", config).unwrap();
//! ```
//!
//! Then in Slint: `import { NButton, Section } from "@kaamos/kaamos.slint";`

use std::collections::HashMap;
use std::path::PathBuf;

/// Directory with `kaamos.slint`, `theme.slint`, `widgets.slint`, `layout.slint`, `info.slint`,
/// `fonts/` and `icons/`.
pub fn ui_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

/// `{"kaamos": ui_path()}` for `CompilerConfiguration::with_library_paths`, so that
/// `@kaamos/kaamos.slint` resolves.
pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("kaamos".to_string(), ui_path())])
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui_files_exist() {
        for f in [
            "kaamos.slint",
            "theme.slint",
            "widgets.slint",
            "layout.slint",
            "info.slint",
            "fonts/Inter-Regular.ttf",
            "icons/nav-settings.svg",
        ] {
            assert!(super::ui_path().join(f).exists(), "{f}");
        }
    }
}
