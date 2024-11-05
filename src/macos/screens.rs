use std::process::Command;

use cocoa::appkit::NSScreen;
use cocoa::base::nil;
use cocoa::foundation::NSArray;
use core_graphics::display::{CGDisplay, CGRect};
use objc::{msg_send, runtime::Object, sel, sel_impl};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Screen {
    index: u64,
    visible_frame: Rect,
    frame: Rect,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

// Position is used to determine where to push from the current screen, the beautiful App you are running
#[derive(Debug)]
pub enum Position {
    Left,
    BottomLeft,
    TopLeft,
    Right,
    BottomRight,
    TopRight,
    Max, // Maximize
    Min, // Minimize
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        return Rect {
            x,
            y,
            width,
            height,
        };
    }
}

pub fn get_main_display() -> CGRect {
    let display = CGDisplay::main();
    display.bounds()
}

pub fn get_screens() -> Vec<Screen> {
    let mut rv = Vec::new();
    unsafe {
        let screens: *mut Object = NSScreen::screens(nil);
        for index in 0..NSArray::count(screens) {
            let screen: *mut Object = msg_send![screens, objectAtIndex: index];
            let visible_frame = screen.visibleFrame();
            let frame = NSScreen::frame(screen);
            rv.push(Screen {
                index,
                visible_frame: Rect {
                    x: visible_frame.origin.x as i32,
                    y: visible_frame.origin.y as i32,
                    width: visible_frame.size.width as i32,
                    height: visible_frame.size.height as i32,
                },
                frame: Rect {
                    x: frame.origin.x as i32,
                    y: frame.origin.y as i32,
                    width: frame.size.width as i32,
                    height: frame.size.height as i32,
                },
            })
        }
    };
    // The window frames have their origins in the bottom left of the screen, y going upwards.
    // However, screen bounds have the origin at the top left going down. We need to convert here
    // to get them in the screen space.
    for idx in 1..rv.len() {
        let y = rv[0].frame.height - rv[idx].visible_frame.height - rv[idx].visible_frame.y;
        rv[idx].visible_frame.y = y;
    }
    rv
}

// Helper function to position window based on enum
// Sorry I need to move away from this project, adding osascript as a dependency.
pub fn position_window(app_name: String, coord: &Rect) -> Result<(), String> {
    let script = format!(
        r#"tell application "System Events" to tell process "{}"
            set position of window 1 to {{{}, {}}}
            set size of window 1 to {{{}, {}}}
        end tell"#,
        app_name, coord.x, coord.y, coord.width, coord.height
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("AppleScript error: {}", error))
    }
}

pub fn get_window_rect(position: Position, rect: CGRect) -> Rect {
    let screen_height = rect.size.height as i32;
    let screen_width = rect.size.width as i32;
    let half_width = screen_width / 2;

    let rect = match position {
        Position::Left => Rect {
            x: 0,
            y: 0,
            width: half_width,
            height: screen_height,
        },
        Position::Right => Rect {
            x: half_width,
            y: 0,
            width: half_width,
            height: screen_height,
        },
        Position::TopLeft => Rect {
            x: 0,
            y: 0,
            width: half_width,
            height: screen_height / 2,
        },
        Position::TopRight => Rect {
            x: half_width,
            y: 0,
            width: half_width,
            height: screen_height / 2,
        },
        Position::BottomLeft => Rect {
            x: 0,
            y: screen_height / 2,
            width: half_width,
            height: screen_height / 2,
        },
        Position::BottomRight => Rect {
            x: half_width,
            y: screen_height / 2,
            width: half_width,
            height: screen_height / 2,
        },
        Position::Max => Rect {
            x: 0,
            y: 0,
            width: screen_width,
            height: screen_height,
        },
        Position::Min => Rect {
            x: screen_width / 4,  // Center of half width
            y: screen_height / 4, // Center of half height
            width: half_width,
            height: screen_height / 2,
        },
    };

    rect
}
