// Window.rs - Handles window arrangement and management
use crate::key::Key;

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
}
