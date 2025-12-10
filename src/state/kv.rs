use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

/// A single KV entry with a value and creation timestamp.
///
/// `created_at` is the Unix timestamp (seconds since epoch) at which
/// the key was last set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub value: String,
    pub created_at: i64,
}

/// Internal HashMap type.
pub type InnerMap = HashMap<String, Entry>;

/// Shared KV store type used across the app.
pub type KvStore = Arc<RwLock<InnerMap>>;

/// Create a new, empty store.
pub fn new_store() -> KvStore {
    Arc::new(RwLock::new(HashMap::new()))
}