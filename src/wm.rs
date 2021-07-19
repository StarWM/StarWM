// Wm.rs - This is where all the magic happens
use crate::key::{Key, XKeyEvent, get_lookup, KeyCode, META_SHIFT, META};
use std::collections::HashMap;
use xcb::Connection;

// Ignore the David Bowie reference, this is the struct that controls X
pub struct StarMan {
    conn: Connection,
    keymap: HashMap<u8, Vec<String>>,
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
            xcb::MOD_MASK_ANY as u16,
            xcb::GRAB_ANY as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );
        // Write buffer to server
        conn.flush();
        // Instantiate and return
        Self { 
            keymap: get_lookup(&conn),
            conn,
        }
    }

    pub fn run(&mut self) {
        // Start event loop
        loop {
            // Wait for event
            let event = self.conn.wait_for_event().unwrap();
            match event.response_type() {
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
            // Start terminal on [Meta] + [T]
            (META, KeyCode::Char('t')) => {
                std::thread::spawn(move || {
                    run_cmd!(alacritty).unwrap();
                });
            }
            // Screenshot on [Meta] + [S]
            (META, KeyCode::Char('s')) => {
                std::thread::spawn(move || {
                    run_cmd!(maim --delay=0.1 > /home/luke/pic/capture.png).unwrap();
                });
            }
            _ => (),
        }
    }
}
