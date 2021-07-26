// Wm.rs - This is where all the magic happens
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
use crate::config::{Config, Handler};
use crate::key::{get_lookup, Key, SymTable, META, META_SHIFT};
use crate::mouse::MouseInfo;
use crate::window::Workspace;
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
    keymap: SymTable,
    workspaces: Vec<Workspace>,
    workspace: usize,
    mouse: Option<MouseInfo>,
}

impl StarMan {
    pub fn new() -> Self {
        // Establish connection with X
        let (conn, _) = Connection::connect(None).expect("Failed to connect to X");
        let setup = conn.get_setup();
        let screen = setup.roots().next().unwrap();
        // Set up workspaces
        let workspaces = vec![
            // New workspace, triggered on [Meta] + [WORKSPACE NUMBER]
            Workspace::new((META, "1")),
            Workspace::new((META, "2")),
            Workspace::new((META, "3")),
            Workspace::new((META, "4")),
            Workspace::new((META, "5")),
            Workspace::new((META, "6")),
            Workspace::new((META, "7")),
            Workspace::new((META, "8")),
            Workspace::new((META, "9")),
            Workspace::new((META, "0")),
        ];
        // Establish grab for workspace trigger events
        let keymap = get_lookup(&conn);
        for trigger in workspaces.iter().map(|w| &w.trigger) {
            StarMan::grab_key(&conn, &screen, trigger, &keymap);
        }
        // Establish a grab for mouse events
        StarMan::grab_button(&conn, &screen, 1, META as u16);
        StarMan::grab_button(&conn, &screen, 1, META_SHIFT as u16);
        // Set root cursor as normal left pointer
        StarMan::set_cursor(&conn, &screen, 68);
        // Establish a grab for notification events
        StarMan::grab_notify_events(&conn, &screen);
        // Write buffer to server
        conn.flush();
        // Instantiate and return
        Self {
            keymap,
            workspaces,
            workspace: 0,
            conf: Config::new(),
            conn,
            mouse: None,
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
        // Add to the workspace
        self.workspace_mut().add(window);
        // Grab the events where the cursor leaves and enters the window
        self.grab_enter_leave(window);
        // Focus on this window
        self.focus_window(window);
    }

    fn destroy_event(&mut self, destroy_notify: XDestroyEvent) {
        // Handle window destroy event
        let window = destroy_notify.window();
        // Remove from workspace
        self.workspace_mut().remove(window);
        // Refocus
        if let Some(target) = self.workspace().get_focus() {
            self.focus_window(target);
        }
    }

    fn enter_event(&mut self, enter_notify: XEnterEvent) {
        // Handle window enter event
        let window = enter_notify.event();
        // Focus window
        unsafe {
            xcb::xproto::change_window_attributes(
                &self.conn,
                window,
                &[(xcb::xproto::CW_OVERRIDE_REDIRECT, true as u32)],
            );

            let display = x11::xlib::XOpenDisplay(std::ptr::null());
            let raise_result = x11::xlib::XRaiseWindow(display, window.into());
            x11::xlib::XCloseDisplay(display);
        }
        self.focus_window(window);
        self.workspace_mut().set_focus(window);
    }

    fn button_press_event(&mut self, button_press: XButtonPressEvent) {
        // Handle mouse button click event
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
            let delta_x = i64::from(end.root_x - start.root_x);
            let delta_y = i64::from(end.root_y - start.root_y);
            if (delta_x == 0 && delta_y == 0) || start.detail != 1 {
                // Exit if only a click, or not using the left mouse button
                return;
            }
            // Move window if drag was performed
            if let Some(geo) = start.geo {
                if resize {
                    let w = i64::from(geo.2) + delta_x;
                    let h = i64::from(geo.3) + delta_y;
                    if w > 0 && h > 0 {
                        self.resize_window(start.child, w, h);
                    }
                } else {
                    let x = geo.0 as i64 + delta_x;
                    let y = geo.1 as i64 + delta_y;
                    self.move_window(start.child, x, y);
                }
            }
        }
    }

    fn key_event(&mut self, key_press: XKeyEvent) {
        // Handle key press events
        let code = st!(self.keymap[&key_press.detail()][0]);
        let modifiers = key_press.state();
        // Create key
        let key = Key::new(modifiers.into(), &code);
        // Check if user defined handler
        if let Some(handler) = self.conf.key(&key) {
            handler(self);
            return;
        }
        // Check for workspace trigger
        if let Some(idx) = self.workspaces.iter().position(|w| w.trigger == key) {
            // Exit if already focussed
            if idx == self.workspace {
                return;
            }
            // Hide previous workspace windows
            self.workspace().hide(&self.conn);
            // Update index
            self.workspace = idx;
            // Show new workspace windows
            self.workspace().show(&self.conn);
        }
    }

    pub fn bind<K: Into<Key>>(&mut self, key: K, handler: Handler) {
        // Bind a key to a handler
        let key = key.into();
        let setup = self.conn.get_setup();
        let screen = setup.roots().next().unwrap();
        // Establish a grab on this shortcut
        StarMan::grab_key(&self.conn, &screen, &key, &self.keymap);
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
        if let Some(target) = self.workspace().get_focus() {
            self.destroy(target);
        }
    }

    fn move_window(&self, window: u32, x: i64, y: i64) {
        // Move a window to a specific X and Y coordinate
        xcb::configure_window(
            &self.conn,
            window,
            &[
                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
            ],
        );
    }

    fn resize_window(&self, window: u32, w: i64, h: i64) {
        // Resize a window to a specific W and H size
        xcb::configure_window(
            &self.conn,
            window,
            &[
                (xcb::CONFIG_WINDOW_WIDTH as u16, w as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, h as u32),
            ],
        );
    }

    fn set_cursor(conn: &xcb::Connection, screen: &xcb::Screen, k: u16) {
        // Set the cursor on the screen
        let f = conn.generate_id();
        xcb::open_font(conn, f, "cursor");
        let c = conn.generate_id();
        xcb::create_glyph_cursor(conn, c, f, f, k, k + 1, 0, 0, 0, 0xffff, 0xffff, 0xffff);
        xcb::change_window_attributes(conn, screen.root(), &[(xcb::CW_CURSOR, c)]);
    }

    fn grab_button(conn: &xcb::Connection, screen: &xcb::Screen, button: u8, mods: u16) {
        // Tell X to grab all mouse events with specific modifiers and buttons
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

    fn grab_key(conn: &xcb::Connection, screen: &xcb::Screen, key: &Key, keymap: &SymTable) {
        // Tell X to grab all key events from a specific key
        for code in key.xcode(keymap) {
            xcb::grab_key(
                conn,
                false,
                screen.root(),
                key.mods as u16,
                code,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
    }

    fn grab_notify_events(conn: &xcb::Connection, screen: &xcb::Screen) {
        // Tell X to grab all notify events on a screen
        StarMan::grab(
            conn,
            screen.root(),
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY as u32,
        );
    }

    fn grab_enter_leave(&self, window: u32) {
        // Tell X to grab all enter and level events on screen
        StarMan::grab(
            &self.conn,
            window,
            xcb::EVENT_MASK_ENTER_WINDOW | xcb::EVENT_MASK_LEAVE_WINDOW,
        );
    }

    fn focus_window(&self, window: u32) {
        // Tell X to set focus on a specific window
        xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, window, 0);
    }

    fn grab(conn: &xcb::Connection, window: u32, events: u32) {
        // Generic helper function to set up an event grab on a window
        xcb::change_window_attributes(conn, window, &[(xcb::CW_EVENT_MASK, events)]);
    }

    fn workspace(&self) -> &Workspace {
        // Get the current workspace (immutable operations)
        &self.workspaces[self.workspace]
    }

    fn workspace_mut(&mut self) -> &mut Workspace {
        // Get the current workspace (mutable operations)
        &mut self.workspaces[self.workspace]
    }
}
