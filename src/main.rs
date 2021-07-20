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
const MAIM: &str = "maim --delay=0.1 > ~/pic/capture.png";

fn main() {
    // Initialise and run StarWM
    let mut starman = StarMan::new();
    // Exit on [Meta] + [Shift] + [BackSpace]
    starman.bind((META_SHIFT, "BackSpace"), || std::process::exit(0));
    // Start application launcher on [Meta] + [Space]
    starman.bind((META, "space"), || cmd!(ROFI));
    // Start terminal on [Meta] + [Return]
    starman.bind((META, "Return"), || cmd!(ALACRITTY));
    // Screenshot on [Meta] + [S]
    starman.bind((META, "s"), || cmd!(MAIM));
    // ..
    starman.bind((META_SHIFT, "3"), || cmd!("kitty"));
    starman.bind((NONE, "XF86Search"), || cmd!("kitty"));
    // Run the window manager
    starman.run();
}
