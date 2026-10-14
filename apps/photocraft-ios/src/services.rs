//! The iOS platform [`Services`].
//!
//! `Services` is the seam between the shared UI (`photocraft-ui-egui`) and the platform: file
//! import/export, preferences, clipboard, dialogs and so on. The desktop shell fills it in
//! `apps/photocraft/src/services.rs` and the web shell in `apps/photocraft-web/src/web.rs`; this
//! is the iOS one.
//!
//! # Status
//!
//! Milestone 1 wires the **pure-Rust** services that need no native UI: import, export, writing
//! files inside the sandbox, PNG encoding and preferences. The native pieces arrive next and each
//! is a single field:
//!
//! * `file_dialog` — `UIDocumentPickerViewController` (Open/Save As)
//! * `clipboard_set_image` / `clipboard_get_image` — `UIPasteboard`
//! * `open_url` — `UIApplication.openURL`
//! * `inbox` — files handed to the app by other apps
//! * `beep` — `AudioServicesPlaySystemSound`
//!
//! Until then those fields stay `None`: the app runs, and an action that needs a missing service
//! reports it instead of crashing.

use std::path::Path;

use photocraft_codecs::{ChannelLayout, EncodeOptions, Image};
use photocraft_doc::Document;
use photocraft_ui_egui::{ExportSettings, Services};

use crate::app_dirs;

/// Builds the iOS [`Services`].
pub fn ios() -> Services {
    Services {
        // Decode any format the engine reads (PSD, PNG, JPEG, .pcraft, …): the same pure-Rust
        // path the desktop and web shells use.
        import: Some(Box::new(|name: &str, bytes: &[u8], max_svg_group_depth: usize| {
            photocraft_io::import_with_svg_group_depth(name, bytes, max_svg_group_depth).map(|r| (r.document, r.warnings)).map_err(|e| e.to_string())
        })),
        // Encode a document to the format the file name's extension asks for (Export As, Save As).
        export: Some(Box::new(|doc: &Document, path: &str, settings: &ExportSettings| {
            let opts = export_options(settings);
            photocraft_io::export(doc, path, &opts).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
        })),
        // Write bytes to a path inside the sandbox (used by Save/Export and autosave).
        write: Some(Box::new(|path: &str, bytes: &[u8]| write_atomic(Path::new(path), bytes))),
        // PNG encoding for screenshots and `ui.render` (the control channel is disabled on iOS,
        // but the UI still uses this for thumbnails and quick export).
        encode_png: Some(Box::new(|w: u32, h: u32, rgba: &[u8]| {
            let img = Image::from_u8(w, h, ChannelLayout::Rgba, rgba.to_vec()).map_err(|e| e.to_string())?;
            photocraft_codecs::encode(&img, photocraft_codecs::Format::Png, &EncodeOptions::default()).map_err(|e| e.to_string())
        })),
        // Preferences in the sandbox container.
        load_prefs: Some(Box::new(|| std::fs::read_to_string(app_dirs::prefs_file()?).ok())),
        save_prefs: Some(Box::new(|text: &str| {
            let path = app_dirs::prefs_file().ok_or("no config directory")?;
            write_atomic(&path, text.as_bytes())
        })),
        // History Log: append to a file.
        append_text: Some(Box::new(|path: &str, text: &str| {
            use std::io::Write as _;
            let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| e.to_string())?;
            file.write_all(text.as_bytes()).map_err(|e| e.to_string())
        })),
        ..Services::default()
    }
}

/// Maps the UI's export settings to the encoder's options (the same mapping the desktop shell uses).
fn export_options(settings: &ExportSettings) -> photocraft_io::ExportOptions {
    let mut opts = photocraft_io::ExportOptions::default();
    if let Some(q) = settings.jpeg_quality {
        opts.encode.jpeg_quality = q;
    }
    opts.encode.webp_lossless = settings.webp_lossless;
    if let Some(q) = settings.webp_quality {
        opts.encode.webp_quality = q;
    }
    opts.tiff_layers = settings.tiff_layers;
    opts.xmp = if settings.xmp_all { photocraft_io::XmpEmbed::All } else { photocraft_io::XmpEmbed::None };
    opts.tga_bits = settings.tga_bits;
    opts.image_previews = settings.image_previews;
    opts.psd_maximize_compatibility = settings.psd_maximize_compatibility;
    opts
}

/// Writes `bytes` to `path` atomically: to a sibling temp file, then rename over the target, after
/// creating the parent directory. A crash mid-write leaves the original intact.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp~");
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}
