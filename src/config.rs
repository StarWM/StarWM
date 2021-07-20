// Config.rs - Handles configuration of the editor
use crate::key::Key;
use crate::StarMan;
use std::collections::HashMap;

pub type Handler = fn(&mut StarMan) -> ();

pub struct Config {
    pub key_bindings: HashMap<Key, Handler>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            key_bindings: HashMap::new(),
        }
    }

    pub fn bind_handler(&mut self, key: Key, handler: Handler) {
        self.key_bindings.insert(key, handler);
    }

    pub fn key(&self, key: Key) -> Option<&Handler> {
        self.key_bindings.get(&key)
    }
}
