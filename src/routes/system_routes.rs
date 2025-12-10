use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde_json::json;

use crate::config::AppConfig;

pub fn routes(config: AppConfig) -> Router {
    Router::new()
        .route("/alive", get(is_alive))
        .route("/version", get(version))
        .with_state(config)
}

/// GET /system/alive
async fn is_alive() -> &'static str {
    "OK"
}

/// GET /system/version
async fn version(State(config): State<AppConfig>) -> Json<serde_json::Value> {
    Json(json!({
        "version": config.server_version
    }))
}