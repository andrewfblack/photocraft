// PhotoCraft for iPadOS — the app's C entry point.
//
// winit's iOS backend calls `UIApplicationMain` itself (see rust-windowing/winit, iOS
// `EventLoop::run`), so this `main` is not a UIKit `@main`/`UIApplicationMain` app: it simply
// hands control to Rust, which runs the eframe/winit event loop and never returns.
//
// `start_photocraft` is defined in `apps/photocraft-ios/src/lib.rs`.

#import <Foundation/Foundation.h>

extern void start_photocraft(void);

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;
    start_photocraft();
    return 0;
}
