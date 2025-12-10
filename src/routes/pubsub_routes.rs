use axum::{
    http::StatusCode,
    routing::post,
    Json,
    Router,
};

use serde_json::Value;

use crate::services::pubsub_service::{subscribe, unsubscribe, SubscribeRequest};

pub fn routes() -> Router {
    Router::new()
        .route("/subscribe", post(handle_subscribe))
        .route("/unsubscribe", post(handle_unsubscribe))
}

async fn handle_subscribe(
    Json(req): Json<SubscribeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // If deserialization succeeds, we always answer 200
    // so the C# client’s EnsureSuccessStatusCode() is happy.
    let resp = subscribe(req).await;
    Ok(Json(resp))
}

async fn handle_unsubscribe(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let id = payload
        .get("subscription_id")
        .and_then(|v| v.as_u64())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let resp = unsubscribe(id).await;
    Ok(Json(resp))
}