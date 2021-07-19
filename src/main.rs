/*
    StarWM is an attempt at a window manager.
    It's written in Rust for stability and speed.
    It was written to be manually edited, if need be.
    The code is commented throughout,
    feel free to modify it if you dislike any part of StarWM.
*/

#[macro_use]
mod utils;
mod key;
mod wm;

use wm::StarMan;

fn main() {
    // Initialise and run StarWM
    let mut starman = StarMan::new();
    starman.run();
}
