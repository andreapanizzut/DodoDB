use std::{fs, io::Write};

use chrono::Utc;
use serde_json::{Map, Value};
use tokio::time::{sleep, Duration};

use crate::state::kv::{Entry, KvStore};

/// Load snapshot from disk into memory.
///
/// `retention_seconds`:
/// - If `Some`, entries older than `now - retention_seconds` are dropped.
/// - If `None`, everything in the snapshot is loaded.
pub async fn load_snapshot(
    path: &str,
    store: &KvStore,
    retention_seconds: Option<u64>,
) {
    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => {
            tracing::info!("No snapshot found at startup (path = {})", path);
            return;
        }
    };

    let json: Value = match serde_json::from_str(&data) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("Failed to parse snapshot JSON: {e}");
            return;
        }
    };

    let obj = match json.as_object() {
        Some(m) => m,
        None => {
            tracing::warn!("Snapshot is not a JSON object, ignoring");
            return;
        }
    };

    let now = Utc::now().timestamp();
    let max_age = retention_seconds.map(|s| s as i64);

    let mut kv = store.write().unwrap();
    kv.clear();

    for (k, v) in obj {
        // New format: { "value": "...", "created_at": 123456789 }
        if let Some(entry_obj) = v.as_object() {
            let value = entry_obj
                .get("value")
                .and_then(|vv| vv.as_str())
                .unwrap_or("")
                .to_string();

            let created_at = entry_obj
                .get("created_at")
                .and_then(|vv| vv.as_i64())
                .unwrap_or(now);

            if let Some(max_age_sec) = max_age {
                if now - created_at > max_age_sec {
                    // Too old, skip
                    continue;
                }
            }

            kv.insert(
                k.clone(),
                Entry {
                    value,
                    created_at,
                },
            );
        }
        // Old format: "value-as-string" (no metadata)
        else if let Some(s) = v.as_str() {
            let created_at = now;

            if let Some(max_age_sec) = max_age {
                if now - created_at > max_age_sec {
                    continue;
                }
            }

            kv.insert(
                k.clone(),
                Entry {
                    value: s.to_string(),
                    created_at,
                },
            );
        }
    }

    tracing::info!("Loaded snapshot: {} entries", kv.len());
}

/// Save the current KV state to `path`.
///
/// Only non-expired keys are written. Expiration is handled entirely by the
/// cleanup loop and by `load_snapshot`, so here we simply serialize current
/// entries.
pub async fn save_snapshot(path: &str, store: &KvStore) {
    let kv = store.read().unwrap();

    let mut obj = Map::new();
    for (k, entry) in kv.iter() {
        obj.insert(
            k.clone(),
            serde_json::json!({
                "value": entry.value,
                "created_at": entry.created_at,
            }),
        );
    }

    drop(kv); // release lock before I/O

    let json = match serde_json::to_string_pretty(&Value::Object(obj)) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("Failed to serialize snapshot JSON: {e}");
            return;
        }
    };

    match fs::File::create(path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(json.as_bytes()) {
                tracing::warn!("Failed to write snapshot file: {e}");
            } else {
                tracing::info!("Snapshot saved");
            }
        }
        Err(e) => tracing::warn!("Failed to create snapshot file: {e}"),
    }
}

/// Background task that periodically saves the snapshot.
pub async fn autosave_loop(path: String, store: KvStore, every_sec: u64) {
    loop {
        sleep(Duration::from_secs(every_sec)).await;
        save_snapshot(&path, &store).await;
    }
}

/// Background task that periodically removes expired keys
/// based on `retention_seconds`.
///
/// If `retention_seconds` is `0`, everything is immediately expired.
pub async fn cleanup_loop(store: KvStore, retention_seconds: u64, every_sec: u64) {
    if retention_seconds == 0 {
        tracing::warn!(
            "cleanup_loop started with retention_seconds = 0; all keys will be removed"
        );
    }

    loop {
        sleep(Duration::from_secs(every_sec)).await;
        purge_expired(&store, retention_seconds);
    }
}

/// Deletes entries from the store that are older than `retention_seconds`.
fn purge_expired(store: &KvStore, retention_seconds: u64) {
    let now = Utc::now().timestamp();
    let max_age = retention_seconds as i64;

    let mut map = store.write().unwrap();
    let before = map.len();

    map.retain(|_k, entry| now - entry.created_at <= max_age);

    let after = map.len();
    let removed = before.saturating_sub(after);

    if removed > 0 {
        tracing::info!(
            "Cleanup: removed {} expired keys ({} remaining)",
            removed,
            after
        );
    }
}