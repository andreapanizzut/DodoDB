use crate::state::Subscription;
use tokio::fs;
use serde_json;

const FILE: &str = "subscriptions.json";

pub async fn save_subscriptions(list: &Vec<Subscription>) {
    let _ = fs::write(FILE, serde_json::to_string_pretty(list).unwrap()).await;
}

pub async fn load_subscriptions() -> Vec<Subscription> {
    match fs::read_to_string(FILE).await {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}