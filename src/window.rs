// Window.rs - Handles window arrangement and management
use crate::key::Key;

pub const BLACKLIST: [&str; 14] = [
    "_NET_WM_WINDOW_TYPE_DESKTOP",
    "_NET_WM_WINDOW_TYPE_COMBO",
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
    "WM_ZOOM_HINTS",
];

// Workspace struct that holds information about a specific workspace
pub struct Workspace {
    pub trigger: Key,
    floating: Vec<u32>,
    monocle: Option<u32>,
    pub previous_geometry: Option<(i64, i64, u32, u32)>,
    focus: usize,
}

impl Workspace {
    pub fn new<K: Into<Key>>(trigger: K) -> Self {
        // Create a new workspace
        Self {
            trigger: trigger.into(),
            floating: vec![],
            monocle: None,
            previous_geometry: None,
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
        // Get the currently focused window
        Some(*self.floating.get(self.focus)?)
    }

    pub fn set_focus(&mut self, window: u32) {
        // Set the currently focused window
        self.focus = self.find(window).unwrap();
    }

    pub fn set_monocle(&mut self) -> Option<u32> {
        // Set focused to monocle window
        let focus = self.get_focus()?;
        self.floating.retain(|&w| w != focus);
        self.monocle = Some(focus);
        return self.monocle;
    }

    pub fn get_monocle(&self) -> Option<u32> {
        // Get the current monocle
        self.monocle
    }

    pub fn clear_monocle(&mut self) -> Option<u32> {
        // Clear the monocle
        let monocle = self.monocle?;
        self.floating.insert(self.focus, monocle);
        self.monocle = None;
        Some(monocle)
    }

    pub fn show(&self, conn: &xcb::Connection) {
        // Show all windows within this workspace
        for window in &self.floating {
            xcb::map_window(conn, *window);
        }
        // Show monocled window if need be
        if let Some(monocle) = self.monocle {
            xcb::map_window(conn, monocle);
        }
    }

    pub fn hide(&self, conn: &xcb::Connection) {
        // Hide all windows within this workspace
        for window in &self.floating {
            xcb::unmap_window(conn, *window);
        }
        // Hide monocled window if need be
        if let Some(monocle) = self.monocle {
            xcb::unmap_window(conn, monocle);
        }
    }

    pub fn contains(&self, window: u32) -> bool {
        // Check if this workspace contains a window
        self.floating.contains(&window) || self.monocle == Some(window)
    }

    pub fn find(&self, window: u32) -> Option<usize> {
        // Find this window, returns None if not found, or if in monocle mode
        self.floating.iter().position(|w| w == &window)
    }
}
