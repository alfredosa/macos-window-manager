#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub fn check_and_request_screen_access() -> bool {
    unsafe {
        let has_access = CGPreflightScreenCaptureAccess();
        if !has_access {
            CGRequestScreenCaptureAccess()
        } else {
            has_access
        }
    }
}

#[cfg(target_os = "macos")]
pub fn query_accessibility_permissions() -> bool {
    let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    if trusted {
        println!("Application is totally trusted!");
    } else {
        println!("Application isn't trusted :(");
    }
    return trusted;
}

#[cfg(not(target_os = "macos"))]
pub fn query_accessibility_permissions() -> bool {
    print!("Who knows... 🤷‍♀️");
    return true;
}
