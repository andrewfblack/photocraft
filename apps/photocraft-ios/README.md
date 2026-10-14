# PhotoCraft for iPadOS

The iOS app shell for [PhotoCraft](https://github.com/storytold/photocraft), the pure-Rust
image editor. It reuses the shared engine and egui UI and adds only the platform layer and the
app entry point — the iOS counterpart of `apps/photocraft` (desktop) and `apps/photocraft-web`
(browser).

This is a **fork** (`andrewfblack/photocraft`) with an added `apps/photocraft-ios` crate and an
`ios/` Xcode project. Upstream does not (yet) target iOS.

## Why it can be ported at all

PhotoCraft's GUI is thin and command-driven: `photocraft_ui_egui::PhotocraftApp` drives
everything through `photocraft_engine::Session`, and the platform is injected through a `Services`
struct of function pointers. Adding iOS is a new `Services` implementation plus an entry point —
nothing below `ui-egui` changes.

## The one hard constraint: UIScene vs winit

iOS 27 (built with the **iOS 27 SDK**) makes the UIKit *scene* life cycle mandatory: an app
without a `UIApplicationSceneManifest` is killed at launch. winit's iOS backend does **not** adopt
UIScene yet (tracking: [rust-windowing/winit#4224](https://github.com/rust-windowing/winit/issues/4224),
milestone 0.31.0, no ETA), and eframe/egui sit on winit.

**Phase 1 therefore builds against the iOS 26 SDK** (Xcode 26.6 on the GitHub `macos-26` runner).
The mandate is SDK-gated, so an iOS 26 SDK build runs fine on iOS 27 devices.

**Phase 2** (when Xcode 27 becomes mandatory for submissions) is one of:
- adopt winit 0.31 once it supports UIScene, or
- replace this crate's shell with a hand-rolled UIKit + `CAMetalLayer` + `egui-wgpu` loop
  (a scene-based `SceneDelegate`).

Either way only the shell changes; `services.rs`, the engine and the UI are untouched.

## Layout

```
apps/photocraft-ios/    Rust: the entry point (#[no_mangle] start_photocraft) and iOS Services
ios/project.yml         XcodeGen spec (the .xcodeproj is generated, not committed)
ios/Sources/main.m      the C main that calls into Rust
.github/workflows/ios.yml
```

## Build

On macOS (or let CI do it):

```sh
rustup target add aarch64-apple-ios
cargo build --release -p photocraft-ios --target aarch64-apple-ios
brew install xcodegen && (cd ios && xcodegen generate)
cd ios && xcodebuild build -project PhotoCraft.xcodeproj -scheme PhotoCraft \
  -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO
```

PowerShell users: the workflow in `.github/workflows/ios.yml` runs the same steps on push to the
`ios` branch.

## Signing & TestFlight

Signing uses **fastlane match** with the encrypted repo `andrewfblack/match-signing`, an App Store
Connect API key, and the same secrets layout as the other repos.

**One-time setup**

1. **App ID:** register `ai.storyteller.photocraft` at developer.apple.com (or `fastlane produce`).
   `match` fails if the App ID does not exist.
2. **App Store Connect app record:** create the app for that bundle ID (needed for TestFlight).
3. **App Store Connect API key** (Users and Access → Integrations → App Store Connect API → Team
   Keys), with **App Manager** access. Download the `.p8` once.
4. **GitHub secrets** (Settings → Secrets and variables → Actions):

   | Secret | Value |
   | --- | --- |
   | `APP_ID` | `ai.storyteller.photocraft` |
   | `TEAM_ID` | your Apple Developer team id |
   | `APPLE_KEY_ID` | the API key's Key ID |
   | `APPLE_ISSUER_ID` | the API key's Issuer ID |
   | `APPLE_KEY_CONTENT` | `base64 -i AuthKey_XXXX.p8` |
   | `MATCH_REPOSITORY` | `https://github.com/andrewfblack/match-signing.git` |
   | `MATCH_PASSWORD` | the match repo's encryption passphrase |
   | `MATCH_GIT_BASIC_AUTHORIZATION` | `printf 'x-access-token:<PAT>' \| base64` |

**Resetting the match repo** (the current `match-signing` was encrypted with a lost password, so its
contents are unrecoverable):

```sh
# 1. Empty the repo (the encrypted files are useless without the old password).
git clone https://github.com/andrewfblack/match-signing.git && cd match-signing
git rm -r certs profiles match_version.txt
git commit -m "reset: lost match password" && git push

# 2. Save a NEW passphrase somewhere safe and set it as the MATCH_PASSWORD secret.
#    (The previous one is what was lost — keep this one.)
# 3. The next `push_to_testflight` run regenerates the distribution certificate and App Store
#    profile (match runs with readonly: false) and stores them encrypted in the repo.
```

If the account has hit its certificate limit, revoke the stale ones in the portal, or run
`fastlane match nuke distribution` (it uses the API key and does not need the old password).
`match-signing` is shared, so emptying it also re-provisions Alt-Mobile-Play on its next run.

## Status / roadmap

- [x] Fork + scaffold: Rust crate, Xcode project, CI
- [x] CI green: staticlib builds for `aarch64-apple-ios`; Xcode links an (unsigned) `.app`
      (run [38622509008](https://github.com/andrewfblack/photocraft/actions/runs/38622509008),
      produced a 56 MB arm64 `PhotoCraft.app`)
- [ ] Boot on the iPad simulator / device
- [ ] Native services: `UIDocumentPicker` (Open/Save), `UIPasteboard`, `UIApplication.openURL`
- [ ] Apple Pencil → `PhotocraftApp::stylus` feed (pressure / tilt / roll)
- [ ] Touch UX: pinch-zoom, pan, on-screen modifiers, safe areas
- [ ] Signing + TestFlight (App Store Connect API key)

## Licensing / branding

PhotoCraft is dual-licensed MIT OR Apache-2.0. The ArtCraft name and logos are trademarks and,
per `docs/brand/LICENSE-brand.txt`, **must be removed from forks** — the iOS build must ship
without them. The desktop control server and MCP are not part of this shell.
