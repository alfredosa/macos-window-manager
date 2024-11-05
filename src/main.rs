mod assets;
mod inputs;
mod macos;

use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;

// use core_foundation::base::CFRelease;
// CFRelease is an important memory release
use inputs::events::KeyboardMonitor;
use macos::*;
use objc::{msg_send, sel, sel_impl};

fn main() {
    println!("Monitoring active windows... Press Ctrl+C to stop");
    let _ = check_and_request_screen_access();
    if !query_accessibility_permissions() {
        println!("Sadly your app is not accessible");
        std::process::exit(1);
    }

    create_app_bar();
    let mut monitor = KeyboardMonitor::new();
    monitor.start();

    let pool = unsafe { NSAutoreleasePool::new(nil) };

    loop {
        if !monitor.is_running() {
            println!("Keyboard Monitor is no longer running");
            break;
        }

        unsafe {
            // Create a new pool for each iteration
            let iteration_pool = NSAutoreleasePool::new(nil);

            let app = get_frontmost_application();
            let displ_cgrect = get_main_display();

            if let Some(pos) = monitor.get_current_position() {
                let rect = get_window_rect(pos, displ_cgrect);
                if let Err(res) = position_window(get_app_name(app), &rect) {
                    println!("Failed: {}", res);
                }
            }

            // Release the application object we retained
            let _: () = msg_send![app, release];

            // Drain the pool for this iteration
            iteration_pool.drain();
        }
    }

    // Clean up the main pool
    unsafe {
        pool.drain();
    }
    monitor.stop();
}
