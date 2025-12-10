use chrono::Utc;
use serde_json::{Map, Value};

use crate::services::pubsub_service;
use crate::state::kv::{Entry, KvStore};

/// Set a key to a JSON value and trigger Pub/Sub notifications.
pub fn set(store: &KvStore, key: String, value: Value) {
    // We keep all locking / map mutation strictly synchronous.
    // Capture:
    //   - old_json: Option<Value> (if there was a previous value)
    //   - new_json: Value (clone for pubsub)
    //   - key_for_pubsub: String (owned key for the async task)
    let (old_json, new_json, key_for_pubsub) = {
        let mut map = store.write().unwrap();

        // Previous entry, if any
        let old_entry = map.get(&key).cloned();

        // Store JSON as string in the KV store
        let entry = Entry {
            value: value.to_string(),
            created_at: Utc::now().timestamp(),
        };
        map.insert(key.clone(), entry);

        // Try to parse the previous value as JSON
        let old_json = old_entry
            .and_then(|e| serde_json::from_str::<Value>(&e.value).ok());

        // Clone for the pubsub notification
        (old_json, value.clone(), key.clone())
    };

    // 2. Spawn async task to send webhooks.
    //
    // `set` is sync (like the rest of this API), so we don't `await` here.
    tokio::spawn(async move {
        let old_val: Value = old_json.unwrap_or(Value::Null);

        // FIX: passiamo Some(old_val) invece di old_val
        pubsub_service::notify_key_update(&key, Some(old_val), new_json)
            .await;
    });
}

/// Retrieve a JSON value from a key.
pub fn get(store: &KvStore, key: &str) -> Option<Value> {
    let map = store.read().unwrap();

    map.get(key)
        .and_then(|entry| serde_json::from_str::<Value>(&entry.value).ok())
}

/// Delete a key.
pub fn delete(store: &KvStore, key: &str) {
    let mut map = store.write().unwrap();
    map.remove(key);
}

/// List all keys.
pub fn list(store: &KvStore) -> Vec<String> {
    let map = store.read().unwrap();
    map.keys().cloned().collect()
}

/// Return all key–value pairs as a JSON object.
pub fn get_all(store: &KvStore) -> Value {
    let map = store.read().unwrap();

    let mut out = Map::new();

    for (k, entry) in map.iter() {
        if let Ok(json_val) = serde_json::from_str::<Value>(&entry.value) {
            out.insert(k.clone(), json_val);
        }
    }

    Value::Object(out)
}

/// Return pretty JSON representation of all data.
pub fn get_all_pretty(store: &KvStore) -> String {
    let value = get_all(store);
    serde_json::to_string_pretty(&value).unwrap()
}

/// Check if a key exists.
pub fn exists(store: &KvStore, key: &str) -> bool {
    let map = store.read().unwrap();
    map.contains_key(key)
}

/// Clear all keys.
pub fn clear(store: &KvStore) {
    let mut map = store.write().unwrap();
    map.clear();
}

/// Return number of stored keys.
pub fn count(store: &KvStore) -> usize {
    let map = store.read().unwrap();
    map.len()
}