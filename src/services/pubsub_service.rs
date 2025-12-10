use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;

/// Internal subscription stored in memory.
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: u64,
    pub key: String,
    pub callback: String,
}

/// Payload coming from the C# client on /pubsub/subscribe
/// NOTE: **no id here** – server generates it.
#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub key: String,
    pub callback: String,
}

lazy_static! {
    static ref SUBSCRIPTIONS: Mutex<HashMap<u64, Subscription>> =
        Mutex::new(HashMap::new());
    static ref HTTP_CLIENT: Client = Client::new();
}

/// Generate a new numeric id for subscriptions.
fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Create a subscription and return `{ "subscription_id": <id> }`.
pub async fn subscribe(req: SubscribeRequest) -> Value {
    let id = next_id();

    let sub = Subscription {
        id,
        key: req.key,
        callback: req.callback,
    };

    {
        let mut map = SUBSCRIPTIONS.lock().unwrap();
        map.insert(id, sub);
    }

    serde_json::json!({ "subscription_id": id })
}

/// Remove a subscription by id and report if it existed.
pub async fn unsubscribe(id: u64) -> Value {
    let mut map = SUBSCRIPTIONS.lock().unwrap();
    let existed = map.remove(&id).is_some();

    serde_json::json!({
        "subscription_id": id,
        "unsubscribed": existed
    })
}

/// Called by `kv_service::set` when a key is updated.
pub async fn notify_key_update(key: &str, old_value: Option<Value>, new_value: Value) {
    // Take a snapshot of matching subscriptions so we don’t hold the lock
    // while doing HTTP calls.
    let subs: Vec<Subscription> = {
        let map = SUBSCRIPTIONS.lock().unwrap();
        map.values()
            .filter(|s| s.key == key)
            .cloned()
            .collect()
    };

    if subs.is_empty() {
        return;
    }

    let body = serde_json::json!({
        "key": key,
        "event": "update",
        "old_value": old_value.unwrap_or(Value::Null),
        "new_value": new_value,
        "timestamp": Utc::now().to_rfc3339(),
    });

    for sub in subs {
        let callback = sub.callback.clone();
        let body_clone = body.clone();

        tokio::spawn(async move {
            let res = HTTP_CLIENT
                .post(&callback)
                .json(&body_clone)
                .send()
                .await;

            if let Err(e) = res {
                warn!("Error sending webhook to {}: {}", callback, e);
            }
        });
    }
}