// Window.rs - Handles window arrangement and management
use crate::key::Key;

pub const BLACKLIST: [&str; 11] = [
    "_NET_WM_WINDOW_TYPE_MENU",
    "_NET_WM_WINDOW_TYPE_POPUP_MENU",
    "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU",
    "_NET_WM_WINDOW_TYPE_TOOLTIP",
    "_NET_WM_WINDOW_TYPE_UTILITY",
    "_NET_WM_WINDOW_TYPE_NOTIFICATION",
    "_NET_WM_WINDOW_TYPE_TOOLBAR",
    "_NET_WM_WINDOW_TYPE_SPLASH",
    "_NET_WM_WINDOW_TYPE_DIALOG",
    "_NET_WM_WINDOW_TYPE_DOCK",
    "_NET_WM_WINDOW_TYPE_DND",
];

// Workspace struct that holds information about a specific workspace
pub struct Workspace {
    pub trigger: Key,
    floating: Vec<u32>,
    focus: usize,
}

impl Workspace {
    pub fn new<K: Into<Key>>(trigger: K) -> Self {
        // Create a new workspace
        Self {
            trigger: trigger.into(),
            floating: vec![],
            focus: 0,
        }
    }

    pub fn add(&mut self, window: u32) {
        // Add window to this workspace
        self.floating.push(window);
        self.focus = self.floating.len().saturating_sub(1);
    }

    pub fn remove(&mut self, window: u32) {
        // Remove a window from this workspace
        self.floating.retain(|&w| w != window);
        // Fix focus if need be
        if self.focus >= self.floating.len() {
            self.focus = self.floating.len().saturating_sub(1);
        }
    }

    pub fn get_focus(&self) -> Option<u32> {
        // Get the currently focussed window
        Some(*self.floating.get(self.focus)?)
    }

    pub fn set_focus(&mut self, window: u32) {
        // Set the currently focussed window
        self.focus = self.floating.iter().position(|w| w == &window).unwrap();
    }

    pub fn show(&self, conn: &xcb::Connection) {
        // Show all windows within this workspace
        for window in &self.floating {
            xcb::map_window(conn, *window);
        }
    }

    pub fn hide(&self, conn: &xcb::Connection) {
        // Hide all windows within this workspace
        for window in &self.floating {
            xcb::unmap_window(conn, *window);
        }
    }

    pub fn contains(&self, window: u32) -> bool {
        self.floating.contains(&window)
    }
}
