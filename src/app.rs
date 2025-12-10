use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::routes::{kv_routes, pubsub_routes, system_routes};
use crate::state::kv::KvStore;
use crate::config::AppConfig;

/// Build the complete Axum application:
/// - /kv       (key/value operations)
/// - /pubsub   (subscribe/unsubscribe for events)
/// - /system   (alive + version)
///
/// `store` is cloned as needed.
/// `cfg` is passed to /system/version so the server can expose its version.
pub fn build_app(store: KvStore, cfg: AppConfig) -> Router {
    Router::new()
        // /kv/*
        .nest("/kv", kv_routes::routes(store.clone()))

        // /pubsub/*
        .nest("/pubsub", pubsub_routes::routes())

        // /system/*
        .nest("/system", system_routes::routes(cfg.clone()))

        // Logging middleware
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}