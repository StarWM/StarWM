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

use key::{KeyCode, META, META_SHIFT};
use wm::StarMan;

const ROFI: &str = "rofi -show run";
const MAIM: &str = "maim --delay=0.1 > ~/pic/capture.png";

fn main() {
    // Initialise and run StarWM
    let mut starman = StarMan::new();
    // Exit on [Meta] + [Shift] + [Q]
    starman.bind((META_SHIFT, KeyCode::Char('q')), || std::process::exit(0));
    // Start application launcher on [Meta] + [T]
    starman.bind((META, KeyCode::Char('t')), || cmd!(ROFI));
    // Screenshot on [Meta] + [S]
    starman.bind((META, KeyCode::Char('s')), || cmd!(MAIM));
    // Run the window manager
    starman.run();
}
