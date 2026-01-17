//! Compiled WASM module cache for Javy compiler
//!
//! Provides LRU-style caching for compiled WASM modules.

use std::collections::HashMap;

/// Compiled WASM module cache
#[derive(Debug)]
pub(super) struct ModuleCache {
    pub(super) modules: HashMap<String, Vec<u8>>,
    pub(super) access_order: Vec<String>,
    pub(super) max_size: usize,
}

impl ModuleCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            modules: HashMap::new(),
            access_order: Vec::new(),
            max_size,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&Vec<u8>> {
        // Move to end of access order (most recently used)
        if let Some(pos) = self.access_order.iter().position(|k| k == key) {
            let key = self.access_order.remove(pos);
            self.access_order.push(key);
        }
        self.modules.get(key)
    }

    pub fn insert(&mut self, key: String, module: Vec<u8>) {
        // Remove oldest if at capacity
        if self.modules.len() >= self.max_size && !self.modules.contains_key(&key) {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.modules.remove(&oldest);
                self.access_order.remove(0);
            }
        }

        self.modules.insert(key.clone(), module);

        // Update access order
        if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key);
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn clear(&mut self) {
        self.modules.clear();
        self.access_order.clear();
    }
}
