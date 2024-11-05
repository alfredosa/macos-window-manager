use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSImage, NSMenu, NSMenuItem,
};
use cocoa::base::{id, nil, selector};
use cocoa::foundation::{NSAutoreleasePool, NSDictionary, NSProcessInfo, NSString};
use objc::{
    msg_send,
    runtime::{Class, Object},
    sel, sel_impl,
};
use std::ffi::CStr;

pub fn get_frontmost_application() -> *mut Object {
    unsafe {
        // Create an autorelease pool for this operation
        let pool = NSAutoreleasePool::new(nil);

        let workspace_class = Class::get("NSWorkspace").unwrap();
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];

        // Get the active application - this returns a retained object
        let active_app: *mut Object = msg_send![workspace, activeApplication];

        // Instead of autoreleasing, we should retain and later release when done
        let _: () = msg_send![active_app, retain];

        // Drain the pool
        pool.drain();

        active_app
    }
}

pub fn get_app_name(app: *mut Object) -> String {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        // Create NSString properly with autorelease pool
        let key = NSString::alloc(nil).init_str("NSApplicationName");
        let value: *mut Object = msg_send![app, objectForKey:key];

        // Get UTF8String and convert to Rust String
        let utf8_str: *const i8 = msg_send![value, UTF8String];
        let result = CStr::from_ptr(utf8_str).to_string_lossy().into_owned();

        pool.drain();
        result
    }
}

pub fn create_app_bar() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
        // LETS ADD AN IMAGE :Dl
        let icon_bytes = crate::assets::Asset::get("ruwindowman.png")
            .expect("Icon should be embedded in binary");

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("ruwindowman_icon.png");
        std::fs::write(&temp_path, icon_bytes.data).expect("Failed to write temporary icon file");

        let ns_temp_path = NSString::alloc(nil)
            .autorelease()
            .init_str(&temp_path.to_string_lossy());

        let image: id = msg_send![NSImage::alloc(nil), initWithContentsOfFile: ns_temp_path];
        let _: () = msg_send![app, setApplicationIconImage: image];

        let menubar = NSMenu::new(nil).autorelease();
        let app_menu_item = NSMenuItem::new(nil).autorelease();
        menubar.addItem_(app_menu_item);
        app.setMainMenu_(menubar);

        // create Application menu
        let app_menu = NSMenu::new(nil).autorelease();
        let quit_prefix = NSString::alloc(nil).init_str("Quit ").autorelease();
        let quit_title =
            quit_prefix.stringByAppendingString_(NSProcessInfo::processInfo(nil).processName());
        let quit_action = selector("terminate:");
        let quit_key = NSString::alloc(nil).init_str("q").autorelease();
        let quit_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(quit_title, quit_action, quit_key)
            .autorelease();
        app_menu.addItem_(quit_item);
        app_menu_item.setSubmenu_(app_menu);
    }
}
