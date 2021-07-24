// Wm.rs - This is where all the magic happens
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
use crate::config::{Config, Handler};
use crate::key::{get_lookup, Key, META, META_SHIFT};
use crate::mouse::MouseInfo;
use std::collections::HashMap;
use xcb::Connection;

// Shorthand for an X events
pub type XMapEvent<'a> = &'a xcb::MapNotifyEvent;
pub type XDestroyEvent<'a> = &'a xcb::DestroyNotifyEvent;
pub type XKeyEvent<'a> = &'a xcb::KeyPressEvent;
pub type XEnterEvent<'a> = &'a xcb::EnterNotifyEvent;
pub type XLeaveEvent<'a> = &'a xcb::LeaveNotifyEvent;
pub type XButtonPressEvent<'a> = &'a xcb::ButtonPressEvent;
pub type XMotionEvent<'a> = &'a xcb::MotionNotifyEvent;

// Ignore the David Bowie reference, this is the struct that controls X
pub struct StarMan {
    conn: Connection,
    conf: Config,
    keymap: HashMap<u8, Vec<String>>,
    floating: Vec<u32>,
    focus: usize,
    mouse: Option<MouseInfo>,
}

impl StarMan {
    pub fn new() -> Self {
        // Establish connection with X
        let (conn, _) = Connection::connect(None).expect("Failed to connect to X");
        let setup = conn.get_setup();
        let screen = setup.roots().next().unwrap();
        // Establish a grab for mouse events
        xcb::randr::select_input(
            &conn,
            screen.root(),
            xcb::randr::NOTIFY_MASK_CRTC_CHANGE as u16,
        );
        for button in [0, 3] {
            StarMan::grab_button(&conn, &screen, button, META as u16);
            StarMan::grab_button(&conn, &screen, button, META_SHIFT as u16);
        }
        // Set root cursor
        let font = conn.generate_id();
        xcb::open_font(&conn, font, "cursor");
        let cursor = conn.generate_id();
        xcb::create_glyph_cursor(
            &conn, cursor, font, font, 68, 69, 0, 0, 0, 0xffff, 0xffff, 0xffff,
        );
        xcb::change_window_attributes(&conn, screen.root(), &[(xcb::CW_CURSOR, cursor)]);
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
            mouse: None,
            focus: 0,
        }
    }

    pub fn run(&mut self) {
        // Start event loop
        loop {
            // Wait for event
            let event = self.conn.wait_for_event().unwrap();
            match event.response_type() {
                // On window map (window appears)
                xcb::MAP_NOTIFY => {
                    let map_notify: XMapEvent = unsafe { xcb::cast_event(&event) };
                    self.map_event(map_notify);
                }
                // On window destroy (window closes)
                xcb::DESTROY_NOTIFY => {
                    let destroy_notify: XDestroyEvent = unsafe { xcb::cast_event(&event) };
                    self.destroy_event(destroy_notify);
                }
                // On mouse entering a window
                xcb::ENTER_NOTIFY => {
                    let enter_notify: XEnterEvent = unsafe { xcb::cast_event(&event) };
                    self.enter_event(enter_notify);
                }
                // On mouse leaving a window
                xcb::LEAVE_NOTIFY => {
                    let _: XLeaveEvent = unsafe { xcb::cast_event(&event) };
                }
                // On mouse button press
                xcb::BUTTON_PRESS => {
                    let button_press: XButtonPressEvent = unsafe { xcb::cast_event(&event) };
                    self.button_press_event(button_press);
                }
                // On mouse movement
                xcb::MOTION_NOTIFY => {
                    let motion_event: XMotionEvent = unsafe { xcb::cast_event(&event) };
                    self.motion_event(motion_event);
                }
                // On mouse button release
                xcb::BUTTON_RELEASE => {
                    self.mouse = None;
                }
                // On key press
                xcb::KEY_PRESS => {
                    // Retrieve key code
                    let key_press: XKeyEvent = unsafe { xcb::cast_event(&event) };
                    self.key_event(key_press);
                }
                // Otherwise
                _ => (),
            }
            // Write buffer to server
            self.conn.flush();
        }
    }

    fn map_event(&mut self, map_notify: XMapEvent) {
        // Handle window map event
        let window = map_notify.window();
        // Push to floating windows
        self.floating.push(window);
        // Grab the events where the cursor leaves and enters the window
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
        self.focus = self.floating.len().saturating_sub(1);
    }

    fn destroy_event(&mut self, destroy_notify: XDestroyEvent) {
        // Handle window destroy event
        let window = destroy_notify.window();
        self.floating.retain(|&w| w != window);
        // Fix focus if need be
        if self.focus >= self.floating.len() {
            self.focus = self.floating.len().saturating_sub(1);
        }
        if let Some(&target) = self.floating.get(self.focus) {
            xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, target, 0);
        }
    }

    fn enter_event(&mut self, enter_notify: XEnterEvent) {
        // Handle window enter event
        let window = enter_notify.event();
        xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, window, 0);
        self.focus = self.floating.iter().position(|w| w == &window).unwrap();
    }

    fn button_press_event(&mut self, button_press: XButtonPressEvent) {
        // Handle mouse button click event
        //let resize = button_press.state() == 65;
        let geo = xcb::get_geometry(&self.conn, button_press.child())
            .get_reply()
            .ok();
        self.mouse = Some(MouseInfo::new(button_press, geo));
    }

    fn motion_event(&mut self, motion_event: XMotionEvent) {
        // Handle mouse motion event
        let resize = motion_event.state() == 321;
        if let Some(start) = self.mouse.as_ref() {
            let end = MouseInfo::motion(motion_event);
            // Calculate deltas
            let delta_x = end.root_x - start.root_x;
            let delta_y = end.root_y - start.root_y;
            if delta_x == 0 && delta_y == 0 {
                // Exit if only a click
                return;
            } else if start.detail == 1 {
                // Move window if drag was performed (with the left mouse button)
                if let Some(geo) = start.geo {
                    if resize {
                        let w = geo.2 as i16 + delta_x;
                        let h = geo.3 as i16 + delta_y;
                        if w > 0 && h > 0 {
                            xcb::configure_window(
                                &self.conn,
                                start.child,
                                &[
                                    (xcb::CONFIG_WINDOW_WIDTH as u16, w as u32),
                                    (xcb::CONFIG_WINDOW_HEIGHT as u16, h as u32),
                                ],
                            );
                        }
                    } else {
                        let x = geo.0 as i16 + delta_x;
                        let y = geo.1 as i16 + delta_y;
                        xcb::configure_window(
                            &self.conn,
                            start.child,
                            &[
                                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                            ],
                        );
                    }
                }
            }
        }
    }

    fn key_event(&mut self, key_press: XKeyEvent) {
        // Handle key press events
        let code = st!(self.keymap[&key_press.detail()][0]);
        let modifiers = key_press.state();
        // Create key and call handler
        let key = Key::new(modifiers.into(), &code);
        if let Some(handler) = self.conf.key(&key) {
            handler(self);
        }
    }

    pub fn bind<K: Into<Key>>(&mut self, key: K, handler: Handler) {
        // Bind a key to a handler
        let key = key.into();
        let setup = self.conn.get_setup();
        let screen = setup.roots().next().unwrap();
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

    pub fn destroy(&mut self, target: u32) {
        // Set up a destroy event
        let protocols = xcb::intern_atom(&self.conn, false, "WM_PROTOCOLS")
            .get_reply()
            .unwrap()
            .atom();
        let delete = xcb::intern_atom(&self.conn, false, "WM_DELETE_WINDOW")
            .get_reply()
            .unwrap()
            .atom();
        let data = xcb::ClientMessageData::from_data32([delete, xcb::CURRENT_TIME, 0, 0, 0]);
        let event = xcb::ClientMessageEvent::new(32, target, protocols, data);
        // Send the event
        xcb::send_event(&self.conn, false, target, xcb::EVENT_MASK_NO_EVENT, &event);
    }

    pub fn destroy_focus(&mut self) {
        // Destroy the window that is currently focussed on
        if let Some(&target) = self.floating.get(self.focus) {
            self.destroy(target);
        }
    }

    fn grab_button(conn: &xcb::Connection, screen: &xcb::Screen, button: u8, mods: u16) {
        xcb::grab_button(
            conn,
            false,
            screen.root(),
            (xcb::EVENT_MASK_BUTTON_PRESS
                | xcb::EVENT_MASK_BUTTON_RELEASE
                | xcb::EVENT_MASK_POINTER_MOTION) as u16,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::NONE,
            xcb::NONE,
            button,
            mods,
        );
    }
}
