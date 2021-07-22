// Wm.rs - This is where all the magic happens
use crate::config::{Config, Handler};
use crate::key::{get_lookup, Key};
use std::collections::HashMap;
use xcb::{ffi, Connection, Event, Reply};

// Shorthand for an X events
pub type XMapEvent<'a> = &'a xcb::MapNotifyEvent;
pub type XDestroyEvent<'a> = &'a xcb::DestroyNotifyEvent;
pub type XKeyEvent<'a> = &'a xcb::KeyPressEvent;
pub type XEnterEvent<'a> = &'a xcb::EnterNotifyEvent;
pub type XLeaveEvent<'a> = &'a xcb::LeaveNotifyEvent;
pub type XButtonPressEvent<'a> = &'a xcb::ButtonPressEvent;
pub type XMotionEvent<'a> = &'a xcb::MotionNotifyEvent;

// Mouse move event struct
#[derive(Default)]
pub struct MouseInfo {
    root_x: i16,
    root_y: i16,
    child: u32,
    detail: u8,
    geo: Option<(u32, u32, u32, u32)>,
}

impl MouseInfo {
    pub fn new(
        event: &Event<ffi::xcb_button_press_event_t>,
        geo: Option<Reply<ffi::xcb_get_geometry_reply_t>>,
    ) -> Self {
        Self {
            root_x: event.root_x(),
            root_y: event.root_y(),
            child: event.child(),
            detail: event.detail(),
            geo: if let Some(geo) = geo {
                Some((
                    geo.x() as u32,
                    geo.y() as u32,
                    geo.width() as u32,
                    geo.height() as u32,
                ))
            } else {
                None
            },
        }
    }

    pub fn motion(event: &Event<ffi::xcb_motion_notify_event_t>) -> Self {
        Self {
            root_x: event.root_x(),
            root_y: event.root_y(),
            child: event.child(),
            detail: event.detail(),
            geo: None,
        }
    }
}

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
        let screen = setup.roots().nth(0).unwrap();
        // Create cursor
        // Establish a grab for mouse events
        xcb::randr::select_input(
            &conn,
            screen.root(),
            xcb::randr::NOTIFY_MASK_CRTC_CHANGE as u16,
        );
        for button in [0, 3] {
            xcb::grab_button(
                &conn,
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
                xcb::MOD_MASK_4 as u16,
            );
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
                    self.focus = self.floating.len().saturating_sub(1);
                }
                // On Window destroy
                xcb::DESTROY_NOTIFY => {
                    let destroy_notify: XDestroyEvent = unsafe { xcb::cast_event(&event) };
                    let window = destroy_notify.window();
                    self.floating.retain(|&w| w != window);
                    if self.focus >= self.floating.len() {
                        self.focus = self.floating.len().saturating_sub(1);
                    }
                    if let Some(&target) = self.floating.get(self.focus) {
                        xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, target, 0);
                    }
                }
                // On Window mouse enter
                xcb::ENTER_NOTIFY => {
                    let enter_notify: XEnterEvent = unsafe { xcb::cast_event(&event) };
                    let window = enter_notify.event();
                    xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, window, 0);
                    self.focus = self.floating.iter().position(|w| w == &window).unwrap();
                }
                // On Window mouse leave
                xcb::LEAVE_NOTIFY => {
                    let _: XLeaveEvent = unsafe { xcb::cast_event(&event) };
                }
                // On mouse press
                xcb::BUTTON_PRESS => {
                    let button_press: XButtonPressEvent = unsafe { xcb::cast_event(&event) };
                    let geo = xcb::get_geometry(&self.conn, button_press.child())
                        .get_reply()
                        .ok();
                    self.mouse = Some(MouseInfo::new(button_press, geo));
                }
                // On mouse movement
                xcb::MOTION_NOTIFY => {
                    // Get the start and end of the mouse event
                    let motion_event: XMotionEvent = unsafe { xcb::cast_event(&event) };
                    if let Some(start) = self.mouse.as_ref() {
                        let end = MouseInfo::motion(motion_event);
                        // Calculate deltas
                        let delta_x = end.root_x - start.root_x;
                        let delta_y = end.root_y - start.root_y;
                        if delta_x == 0 && delta_y == 0 {
                            // Exit if only a click
                            continue;
                        } else if start.detail == 1 {
                            // Move window if drag was performed
                            if let Some(geo) = start.geo {
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
                // On mouse release
                xcb::BUTTON_RELEASE => {
                    self.mouse = None;
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
        if let Some(&target) = self.floating.get(self.focus) {
            self.destroy(target);
        }
    }

    fn key_input(&mut self, key: Key) {
        // For handling key inputs
        if let Some(handler) = self.conf.key(key) {
            handler(self);
        }
    }
}
