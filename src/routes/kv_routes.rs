use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put, delete, post},
    Json, Router,
};
use serde_json::Value;

use crate::state::kv::KvStore;
use crate::services::{kv_service, pubsub_service};

/// Build all KV routes under /kv
pub fn routes(store: KvStore) -> Router {
    Router::new()
        .route(
            "/:key",
            get(get_key)
                .put(put_key)
                .delete(delete_key),
        )
        .route("/", get(list_keys))
        .route("/all", get(get_all))
        .route("/all/pretty", get(get_all_pretty))
        .route("/:key/exists", get(key_exists))
        .route("/clear", post(clear_all))
        .route("/count", get(count_keys))
        .with_state(store)
}

//
// ─────────────────────────────────────────────────────────────
// PUT /kv/{key}
// Set or update value for a key
// ─────────────────────────────────────────────────────────────
//
/*async fn put_key(
    Path(key): Path<String>,
    State(store): State<KvStore>,
    Json(new_value): Json<Value>,
) -> StatusCode
{
    // Get old value (if exists)
    let old_value = kv_service::get(&store, &key);

    // Set new value (applies created_at timestamp in kv_service)
    kv_service::set(&store, key.clone(), new_value.clone());

    // Notify Pub/Sub listeners
    pubsub_service::notify_value_changed(&key, old_value.as_ref(), &new_value).await;

    StatusCode::OK
}*/

// TEMPORARY TEST HANDLER
async fn put_key(
    Path(key): Path<String>,
    State(store): State<KvStore>,
    Json(new_value): Json<Value>,
) -> StatusCode
{
    // Only touch the KV store, no Pub/Sub.
    kv_service::set(&store, key, new_value);
    StatusCode::OK
}

//
// ─────────────────────────────────────────────────────────────
// GET /kv/{key}
// Return JSON value or 404
// ─────────────────────────────────────────────────────────────
//
async fn get_key(
    Path(key): Path<String>,
    State(store): State<KvStore>,
) -> Result<Json<Value>, StatusCode>
{
    match kv_service::get(&store, &key) {
        Some(value) => Ok(Json(value)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

//
// ─────────────────────────────────────────────────────────────
// DELETE /kv/{key}
// Remove a key if it exists
// ─────────────────────────────────────────────────────────────
//
async fn delete_key(
    Path(key): Path<String>,
    State(store): State<KvStore>,
) -> StatusCode
{
    kv_service::delete(&store, &key);
    StatusCode::OK
}

//
// ─────────────────────────────────────────────────────────────
// GET /kv
// List keys only
// ─────────────────────────────────────────────────────────────
//
async fn list_keys(
    State(store): State<KvStore>,
) -> Json<Vec<String>>
{
    Json(kv_service::list(&store))
}

//
// ─────────────────────────────────────────────────────────────
// GET /kv/all
// Return full JSON object of all key/value pairs
// ─────────────────────────────────────────────────────────────
//
async fn get_all(
    State(store): State<KvStore>,
) -> Json<Value>
{
    Json(kv_service::get_all(&store))
}

//
// ─────────────────────────────────────────────────────────────
// GET /kv/all/pretty
// Return pretty-printed JSON as text
// ─────────────────────────────────────────────────────────────
//
async fn get_all_pretty(
    State(store): State<KvStore>,
) -> String
{
    kv_service::get_all_pretty(&store)
}

//
// ─────────────────────────────────────────────────────────────
// GET /kv/{key}/exists
// Return true/false
// ─────────────────────────────────────────────────────────────
//
async fn key_exists(
    Path(key): Path<String>,
    State(store): State<KvStore>,
) -> Json<bool>
{
    Json(kv_service::exists(&store, &key))
}

//
// ─────────────────────────────────────────────────────────────
// POST /kv/clear
// Clear the entire store (destructive)
// ─────────────────────────────────────────────────────────────
//
async fn clear_all(
    State(store): State<KvStore>,
) -> StatusCode
{
    kv_service::clear(&store);
    StatusCode::OK
}

//
// ─────────────────────────────────────────────────────────────
// GET /kv/count
// Return the number of keys stored
// ─────────────────────────────────────────────────────────────
//
async fn count_keys(
    State(store): State<KvStore>,
) -> Json<usize>
{
    Json(kv_service::count(&store))
}