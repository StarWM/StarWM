// Utils.rs - Contains useful tools that help make code concise throughout.
use xcb::{EnterNotifyEvent, KeyPressEvent, LeaveNotifyEvent, MapNotifyEvent};

// Helper macro for creating strings
#[macro_export]
macro_rules! st {
    ($value:expr) => {
        $value.to_string()
    };
}

// Helper macro for running commands
#[macro_export]
macro_rules! cmd {
    ($cmd:expr) => {{
        std::thread::spawn(move || {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg($cmd)
                .status();
        });
    }};
}

// Shorthand for an X events
pub type XMapEvent<'a> = &'a MapNotifyEvent;
pub type XKeyEvent<'a> = &'a KeyPressEvent;
pub type XEnterEvent<'a> = &'a EnterNotifyEvent;
pub type XLeaveEvent<'a> = &'a LeaveNotifyEvent;
