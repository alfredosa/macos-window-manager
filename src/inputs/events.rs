use crate::macos::Position;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField, KeyCode,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct KeyboardMonitor {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    down_keys: Arc<Mutex<Vec<i64>>>,
}

impl KeyboardMonitor {
    pub fn new() -> Self {
        KeyboardMonitor {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            down_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) {
        let running = self.running.clone();
        let down_keys = self.down_keys.clone();
        running.store(true, Ordering::SeqCst);

        self.handle = Some(thread::spawn(move || {
            let current = CFRunLoop::get_current();
            match CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ], // Add FlagsChanged
                // [Ref](https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h)
                move |_proxy, event_type, event| {
                    if !running.load(Ordering::SeqCst) {
                        CFRunLoop::get_current().stop();
                        return None;
                    }

                    if let Ok(mut keys) = down_keys.lock() {
                        match event_type {
                            CGEventType::KeyDown => {
                                let keycode = event
                                    .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                                if !keys.contains(&keycode) {
                                    keys.push(keycode);
                                }
                            }
                            CGEventType::KeyUp => {
                                let keycode = event
                                    .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                                if let Some(index) = keys.iter().position(|&k| k == keycode) {
                                    keys.remove(index);
                                }
                            }
                            CGEventType::FlagsChanged => {
                                // Handle modifier keys
                                let flags = event.get_flags();
                                let keycode = event
                                    .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);

                                // Command key (0x37)
                                if keycode == KeyCode::COMMAND.into() {
                                    if flags.contains(CGEventFlags::CGEventFlagCommand) {
                                        if !keys.contains(&KeyCode::COMMAND.into()) {
                                            keys.push(KeyCode::COMMAND.into());
                                        }
                                    } else {
                                        if let Some(index) =
                                            keys.iter().position(|&k| k == KeyCode::COMMAND.into())
                                        {
                                            keys.remove(index);
                                        }
                                    }
                                }

                                // Control key (0x3B)
                                if keycode == KeyCode::CONTROL.into() {
                                    if flags.contains(CGEventFlags::CGEventFlagControl) {
                                        if !keys.contains(&KeyCode::CONTROL.into()) {
                                            keys.push(KeyCode::CONTROL.into());
                                        }
                                    } else {
                                        if let Some(index) =
                                            keys.iter().position(|&k| k == KeyCode::CONTROL.into())
                                        {
                                            keys.remove(index);
                                        }
                                    }
                                }

                                println!(
                                    "Modifier flags changed: {:?}, keycode: {:#x}",
                                    flags, keycode
                                );
                            }
                            _ => (),
                        }
                    }
                    None
                },
            ) {
                Ok(tap) => unsafe {
                    let loop_source = tap
                        .mach_port
                        .create_runloop_source(0)
                        .expect("Failed to create run loop source");
                    current.add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                },
                Err(err) => panic!("Failed to create event tap: {:?}", err),
            }
        }));
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn get_current_position(&self) -> Option<Position> {
        if let Ok(keys) = self.down_keys.lock() {
            self.get_position_from_keys(&keys)
        } else {
            None
        }
    }

    fn get_position_from_keys(&self, pressed_keys: &[i64]) -> Option<Position> {
        println!("Current pressed keys: {:?}", pressed_keys);

        // TODO: Refactor to accept user defined configs
        let contains_cmd = pressed_keys.contains(&0x37); // Command key
        let contains_ctrl = pressed_keys.contains(&0x3B); // Control key
        let contains_left = pressed_keys.contains(&0x7B); // Left arrow
        let contains_right = pressed_keys.contains(&0x7C); // Right arrow

        let contains_down = pressed_keys.contains(&0x7D);
        let contains_up = pressed_keys.contains(&0x7E);

        println!(
            "Key states: CMD={}, CTRL={}, LEFT={}, RIGHT={}",
            contains_cmd, contains_ctrl, contains_left, contains_right
        );

        let result = match (
            contains_cmd,
            contains_ctrl,
            contains_left,
            contains_right,
            contains_down,
            contains_up,
        ) {
            // cmd, ctrl, left, right, down, up
            (true, true, true, false, false, false) => Some(Position::Left),
            // Command + Right = Right half of screen
            (true, true, false, true, false, false) => Some(Position::Right),
            // No matching combination
            (true, true, false, false, false, true) => Some(Position::Max),
            _ => None,
        };

        println!("Returning position: {:?}", result);
        result
    }
}
