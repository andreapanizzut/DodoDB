use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// A single pub-sub subscription
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub id: u64,
    pub key: String,
    pub callback: String,
}

/// In-memory shared application state
#[derive(Clone)]
pub struct AppState {
    pub subscriptions: Arc<RwLock<Vec<Subscription>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        }
    }
}