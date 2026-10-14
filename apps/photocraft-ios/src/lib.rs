//! PhotoCraft for iPadOS — the iOS app shell.
//!
//! This crate is the iOS counterpart of `apps/photocraft` (desktop) and `apps/photocraft-web`
//! (browser). It reuses the same [`photocraft_ui_egui::PhotocraftApp`] and
//! [`photocraft_engine`] as the other shells; only the platform [`Services`] and the app entry
//! point differ.
//!
//! # How it starts
//!
//! The Xcode app's `main` (`ios/Sources/main.m`) calls [`start_photocraft`]. That runs
//! [`eframe::run_native`], which on iOS ends up in winit's UIKit backend, and winit calls
//! `UIApplicationMain` itself — so this function does not return; the process ends with the app.
//!
//! # Build (Phase 1)
//!
//! Built with the **Xcode 26 / iOS 26 SDK** on purpose: the iOS 27 SDK makes the UIKit
//! *scene* life cycle mandatory, and winit's iOS backend does not adopt it yet
//! (rust-windowing/winit#4224). Linking the iOS 26 SDK keeps the classic life cycle legal while
//! still running on iOS 27 devices. When winit lands scene support (milestone 0.31.0) or we move
//! to a hand-rolled UIKit + Metal shell, only this crate's shell changes — the engine, the UI and
//! [`services`] are unaffected.
//!
//! [`Services`]: photocraft_ui_egui::Services

mod app_dirs;
mod services;

use photocraft_engine::Session;
use photocraft_ui_egui::{theme::ThemeKind, PhotocraftApp};

/// Starts PhotoCraft. Called from the Xcode app's `main` (see `ios/Sources/main.m`).
///
/// `#[no_mangle]` publishes the symbol the Objective-C `main` links against; it is safe here
/// because no other function in the process shares this name (the `#[allow]` only silences the
/// `unsafe_code` lint for this one attribute).
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn start_photocraft() {
    // The system font scan can take seconds on some devices (as on desktop): run it off the UI
    // thread, before the window opens.
    photocraft_ui_egui::type_tool::warm_fonts();

    let options = eframe::NativeOptions {
        // iOS ignores size/position; the system fills the screen. `app_id` names the Wayland
        // class on Linux and is harmless elsewhere; keep it for consistency with the other shells.
        viewport: egui::ViewportBuilder::default().with_title("PhotoCraft").with_app_id("ai.storyteller.photocraft"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "PhotoCraft",
        options,
        Box::new(|cc| {
            // Install fonts and theme tokens before the first frame, as the web shell does.
            PhotocraftApp::setup_context(&cc.egui_ctx, ThemeKind::Pro);
            let mut app = PhotocraftApp::new(Session::new(), services::ios());
            app.set_theme(&cc.egui_ctx, ThemeKind::Pro);
            // Draw the document canvas on the GPU (Metal through wgpu), not via egui textures.
            if let Some(rs) = cc.wgpu_render_state.clone() {
                log::info!("photocraft-ios: wgpu backend {:?}", rs.adapter.get_info().backend);
                app.set_wgpu(rs);
            }
            Ok(Box::new(app))
        }),
    );

    if let Err(error) = result {
        log::error!("PhotoCraft failed to start: {error}");
    }
}
