//! Where the iOS app keeps its data.
//!
//! In an iOS app the process's `HOME` is the app's sandbox container
//! (`/var/mobile/Containers/Data/Application/<uuid>`), so preferences and logs live under
//! `Library/Application Support/PhotoCraft` there — the same layout the desktop app uses under the
//! user's home, just inside the sandbox.

use std::path::PathBuf;

/// The app's settings directory (`$HOME/Library/Application Support/PhotoCraft`), if `HOME` is
/// set. iOS always sets it; `None` only when the environment is missing (which the app treats as
/// "settings are session-only").
pub fn config_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    Some(PathBuf::from(home).join("Library/Application Support/PhotoCraft"))
}

/// The saved preferences file (`preferences.json`).
pub fn prefs_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("preferences.json"))
}
