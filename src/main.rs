/*
    StarWM is an attempt at a window manager.
    It's written in Rust for stability and speed.
    It was written to be manually edited, if need be.
    The code is commented throughout,
    feel free to modify it if you dislike any part of StarWM.
*/

mod config;
#[macro_use]
mod utils;
mod key;
mod wm;

use key::{META, META_SHIFT, NONE};
use wm::StarMan;

const ROFI: &str = "rofi -show run";
const ALACRITTY: &str = "alacritty";
const MAIM: &str = "maim -suB --delay=0.1 | xclip -selection clipboard -t image/png";

fn main() {
    // Initialise and run StarWM
    let mut starman = StarMan::new();
    // Exit on [Meta] + [Shift] + [BackSpace]
    starman.bind((META_SHIFT, "BackSpace"), |_| std::process::exit(0));
    // Start application launcher on [Meta] + [Space]
    starman.bind((META, "space"), |_| cmd!(ROFI));
    // Start terminal on [Meta] + [Return]
    starman.bind((META, "Return"), |_| cmd!(ALACRITTY));
    // Screenshot on [Meta] + [S]
    starman.bind((META, "s"), |_| cmd!(MAIM));
    // Close window on [Meta] + [Q]
    starman.bind((META, "q"), |wm| wm.destroy_focus());
    // Open rofi on search key
    starman.bind((NONE, "XF86Search"), |_| cmd!(ROFI));
    // Run the window manager
    starman.run();
}
