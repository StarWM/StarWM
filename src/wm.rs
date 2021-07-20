// Wm.rs - This is where all the magic happens
use crate::config::{Config, Handler};
use crate::key::{get_lookup, Key};
use std::collections::HashMap;
use xcb::Connection;

// Shorthand for an X events
pub type XMapEvent<'a> = &'a xcb::MapNotifyEvent;
pub type XKeyEvent<'a> = &'a xcb::KeyPressEvent;
pub type XEnterEvent<'a> = &'a xcb::EnterNotifyEvent;
pub type XLeaveEvent<'a> = &'a xcb::LeaveNotifyEvent;

// Ignore the David Bowie reference, this is the struct that controls X
pub struct StarMan {
    conn: Connection,
    conf: Config,
    keymap: HashMap<u8, Vec<String>>,
    floating: Vec<u32>,
}

impl StarMan {
    pub fn new() -> Self {
        // Establish connection with X
        let (conn, _) = Connection::connect(None).expect("Failed to connect to X");
        let setup = conn.get_setup();
        let screen = setup.roots().nth(0).unwrap();
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
            conf: Config::new(),
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
                    // Focus on this window
                    xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, window, 0);
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
                    self.key_input(Key::new(modifiers.into(), &code));
                }
                // Otherwise
                _ => (),
            }
            // Write buffer to server
            self.conn.flush();
        }
    }

    pub fn bind<K: Into<Key>>(&mut self, key: K, handler: Handler) {
        // Bind a key to a handler
        let key = key.into();
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(0).unwrap();
        // Establish a grab on this shortcut
        for code in key.xcode(&self.keymap) {
            xcb::grab_key(
                &self.conn,
                false,
                screen.root(),
                key.mods as u16,
                code,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
        // Perform the bind
        self.conf.bind_handler(key, handler);
    }

    fn key_input(&mut self, key: Key) {
        // For handling key inputs
        if let Some(handler) = self.conf.key(key) {
            handler();
        }
    }
}
