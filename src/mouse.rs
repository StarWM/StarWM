// Mouse.rs - Handling mouse events
use xcb::{ffi, Event, Reply};

// Mouse move event struct
#[derive(Default)]
#[allow(clippy::module_name_repetitions)]
pub struct MouseInfo {
    pub root_x: i16,
    pub root_y: i16,
    pub child: u32,
    pub detail: u8,
    pub geo: Option<(i64, i64, u32, u32)>,
}

impl MouseInfo {
    pub fn new(
        event: &Event<ffi::xcb_button_press_event_t>,
        geo: Option<Reply<ffi::xcb_get_geometry_reply_t>>,
    ) -> Self {
        // Take in a mouse press event, and convert into a friendly struct
        Self {
            root_x: event.root_x(),
            root_y: event.root_y(),
            child: event.child(),
            detail: event.detail(),
            geo: geo.map(|geo| {
                (
                    i64::from(geo.x()),
                    i64::from(geo.y()),
                    u32::from(geo.width()),
                    u32::from(geo.height()),
                )
            }),
        }
    }

    pub fn motion(event: &Event<ffi::xcb_motion_notify_event_t>) -> Self {
        // Take in a mouse movement event, and convert to a friendly struct
        Self {
            root_x: event.root_x(),
            root_y: event.root_y(),
            child: event.child(),
            detail: event.detail(),
            geo: None,
        }
    }
}
