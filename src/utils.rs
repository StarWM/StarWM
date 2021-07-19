// Utils.rs - Contains useful tools that help make code concise throughout.

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
