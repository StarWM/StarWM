// Wm.rs - This is where all the magic happens
use crate::key::{get_lookup, Key, KeyCode, META, META_SHIFT};
use crate::utils::{XEnterEvent, XKeyEvent, XLeaveEvent, XMapEvent};
use std::collections::HashMap;
use xcb::Connection;

// Ignore the David Bowie reference, this is the struct that controls X
pub struct StarMan {
    conn: Connection,
    keymap: HashMap<u8, Vec<String>>,
    floating: Vec<u32>,
}

impl StarMan {
    pub fn new() -> Self {
        // Establish connection with X
        let (conn, _) = Connection::connect(None).expect("Failed to connect to X");
        let setup = conn.get_setup();
        let screen = setup.roots().nth(0).unwrap();
        // Establish a grab on the keyboard
        xcb::grab_key(
            &conn,
            false,
            screen.root(),
            META as u16,
            xcb::GRAB_ANY as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );
        // Establish a grab for notification events
        xcb::change_window_attributes(
            &conn,
            screen.root(),
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY as u32,
            )],
        );
        // Write buffer to server
        conn.flush();
        // Instantiate and return
        Self {
            keymap: get_lookup(&conn),
            floating: vec![],
            conn,
        }
    }

    pub fn run(&mut self) {
        // Start event loop
        loop {
            // Wait for event
            let event = self.conn.wait_for_event().unwrap();
            match event.response_type() {
                // On Window map (show event)
                xcb::MAP_NOTIFY => {
                    // Retrieve window
                    let map_notify: XMapEvent = unsafe { xcb::cast_event(&event) };
                    let window = map_notify.window();
                    // Push to floating windows
                    self.floating.push(window);
                    // Ensure the window recieves events
                    xcb::change_window_attributes(
                        &self.conn,
                        window,
                        &[(
                            xcb::CW_EVENT_MASK,
                            xcb::EVENT_MASK_ENTER_WINDOW | xcb::EVENT_MASK_LEAVE_WINDOW,
                        )],
                    );
                }
                // On Window mouse enter
                xcb::ENTER_NOTIFY => {
                    let enter_notify: XEnterEvent = unsafe { xcb::cast_event(&event) };
                    let window = enter_notify.event();
                    xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, window, 0);
                }
                // On Window mouse leave
                xcb::LEAVE_NOTIFY => {
                    let leave_notify: XLeaveEvent = unsafe { xcb::cast_event(&event) };
                }
                // On Keypress
                xcb::KEY_PRESS => {
                    // Retrieve key code
                    let key_press: XKeyEvent = unsafe { xcb::cast_event(&event) };
                    let code = st!(self.keymap[&key_press.detail()][0]);
                    let modifiers = key_press.state();
                    // Form a key and send it to be handled
                    if let Some(key) = Key::from_x(modifiers.into(), &code) {
                        self.key_input(key);
                    }
                }
                // Otherwise
                _ => (),
            }
            // Write buffer to server
            self.conn.flush();
        }
    }

    fn key_input(&mut self, key: Key) {
        // For handling key inputs
        match (key.mods, key.code) {
            // Exit on [Meta] + [Shift] + [Q]
            (META_SHIFT, KeyCode::Char('q')) => {
                std::process::exit(0);
            }
            // Start application launcher on [Meta] + [T]
            (META, KeyCode::Char('t')) => {
                std::thread::spawn(move || {
                    run_cmd!(rofi -show run).unwrap();
                });
            }
            // Screenshot on [Meta] + [S]
            (META, KeyCode::Char('s')) => {
                std::thread::spawn(move || {
                    run_cmd!(maim --delay=0.1 > ~/pic/capture.png).unwrap();
                });
            }
            _ => (),
        }
    }
}
