// Key.rs - Handles key reading and processing
use std::collections::HashMap;
use std::ffi::CStr;
use xcb::get_keyboard_mapping;
pub use xcb::{
    ModMask, MOD_MASK_1 as ALT, MOD_MASK_4 as META, MOD_MASK_CONTROL as CONTROL,
    MOD_MASK_SHIFT as SHIFT, NONE,
};

// Common combinations
pub const META_SHIFT: ModMask = META | SHIFT;
/*
pub const CONTROL_SHIFT: ModMask = CONTROL | SHIFT;
pub const CONTROL_ALT_SHIFT: ModMask = CONTROL | ALT | SHIFT;
pub const CONTROL_ALT: ModMask = CONTROL | ALT;
pub const META_ALT: ModMask = META | ALT;
pub const META_ALT_SHIFT: ModMask = META | ALT | SHIFT;
*/

// Representation of a key, with modifiers
#[derive(PartialEq, Eq, Hash)]
pub struct Key {
    pub code: String,
    pub mods: ModMask,
}

impl Key {
    pub fn new(mods: ModMask, code: &str) -> Self {
        // Create a new key, from X key input data
        Self {
            code: st!(code),
            mods,
        }
    }

    pub fn xcode(&self, table: &HashMap<u8, Vec<String>>) -> Vec<u8> {
        // This gives out the X code for the key
        let mut result = vec![];
        for (k, v) in table {
            if v.contains(&self.code) {
                result.push(*k);
            }
        }
        result
    }
}

// Helpful into trait for short arguments
impl Into<Key> for (ModMask, String) {
    fn into(self) -> Key {
        Key::new(self.0, &self.1)
    }
}

// Helpful into trait for short arguments
impl Into<Key> for (ModMask, &str) {
    fn into(self) -> Key {
        Key::new(self.0, self.1)
    }
}

pub fn get_lookup(conn: &xcb::Connection) -> HashMap<u8, Vec<String>> {
    // Retrieve the lookup table for keypresses
    let setup = conn.get_setup();
    // Work out range of keycodes
    let start = setup.min_keycode();
    let width = setup.max_keycode() - start + 1;
    // Get the keyboard mapping
    let keyboard_mapping = get_keyboard_mapping(&conn, start, width)
        .get_reply()
        .unwrap();
    // Retrieve the key symbols and how many there are per keycode
    let keysyms = keyboard_mapping.keysyms();
    let keysyms_per_keycode = keyboard_mapping.keysyms_per_keycode() as usize;
    let ptr_value = unsafe { &*(keyboard_mapping.ptr) };
    // Work out how many keycodes there are in total
    let keycode_count = ptr_value.length as usize / keysyms_per_keycode as usize;
    // Prepare final table
    let mut result = HashMap::new();
    for keycode in 0..keycode_count {
        // Prepare list of symbols
        let mut syms = vec![];
        for keysym in 0..keysyms_per_keycode {
            // Retrieve each symbol
            let sym = keysyms[keysym + keycode * keysyms_per_keycode];
            if sym == 0 {
                continue;
            }
            let string_ptr = unsafe { x11::xlib::XKeysymToString(sym as u64) };
            syms.push(if string_ptr.is_null() {
                st!("None")
            } else {
                unsafe { CStr::from_ptr(string_ptr) }
                    .to_str()
                    .unwrap()
                    .to_owned()
            });
        }
        // Insert into result table
        result.insert(start + keycode as u8, syms);
    }
    result
}
