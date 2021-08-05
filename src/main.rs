/*
    StarWM is an attempt at a window manager.
    It's written in Rust for stability and speed.
    It was written to be manually edited, if need be.
    The code is commented throughout,
    feel free to modify it if you dislike any part of StarWM.
*/

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

mod config;
#[macro_use]
mod utils;
mod key;
mod mouse;
mod window;
mod wm;

use key::{META, META_SHIFT, NONE};
use wm::StarMan;

// List of commands to run within the WM
const ROFI: &str = "rofi -show run";
const ALACRITTY: &str = "alacritty";
const MAIM: &str = "maim -suB --delay=0.1 | xclip -selection clipboard -t image/png";

fn main() {
    // Initialise and run StarWM
    let mut starman = StarMan::new();

    // Exit on [Meta] + [Shift] + [BackSpace]
    starman.bind((META_SHIFT, "BackSpace"), |_| std::process::exit(0));
    // Close window on [Meta] + [Q]
    starman.bind((META, "q"), StarMan::destroy_focus);
    // Move window to workspace on [Meta] + [Shift] + [WORKSPACE]
    starman.bind((META_SHIFT, "1"), |s| s.move_window_to_workspace(0));
    starman.bind((META_SHIFT, "2"), |s| s.move_window_to_workspace(1));
    starman.bind((META_SHIFT, "3"), |s| s.move_window_to_workspace(2));
    starman.bind((META_SHIFT, "4"), |s| s.move_window_to_workspace(3));
    starman.bind((META_SHIFT, "5"), |s| s.move_window_to_workspace(4));
    starman.bind((META_SHIFT, "6"), |s| s.move_window_to_workspace(5));
    starman.bind((META_SHIFT, "7"), |s| s.move_window_to_workspace(6));
    starman.bind((META_SHIFT, "8"), |s| s.move_window_to_workspace(7));
    starman.bind((META_SHIFT, "9"), |s| s.move_window_to_workspace(8));
    starman.bind((META_SHIFT, "0"), |s| s.move_window_to_workspace(9));
    // Toggle monocle mode on [Meta] + [M]
    starman.bind((META, "m"), |s| {
        if s.workspace().get_monocle().is_none() {
            s.monocle_focus();
        } else {
            s.monocle_clear();
        }
    });

    // Start application launcher on [Meta] + [Space]
    starman.bind((META, "space"), |_| cmd!(ROFI));
    // Start terminal on [Meta] + [Return]
    starman.bind((META, "Return"), |_| cmd!(ALACRITTY));
    // Screenshot on [Meta] + [S]
    starman.bind((META, "s"), |_| cmd!(MAIM));
    // Open rofi on search key
    starman.bind((NONE, "XF86Search"), |_| cmd!(ROFI));

    // Run the window manager
    starman.run();
}
