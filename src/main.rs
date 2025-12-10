/*****************************************************************************************
 *
 *  DodoDB – Lightweight Key–Value Microservice in Rust
 *  ----------------------------------------------------
 *
 *  VERSION: KV + Persistence with Retention (snapshot load/save)
 *
 *****************************************************************************************/

mod app;
mod config;
mod errors;
mod persistence;
mod routes;
mod services;
mod state;

use std::path::PathBuf;

use tokio::net::TcpListener;
use tokio::task;
use axum::serve;

use tracing_subscriber::FmtSubscriber;
use tracing::level_filters::LevelFilter;

use crate::config::AppConfig;
use crate::state::kv::{new_store, KvStore};
use crate::persistence::{load_snapshot, autosave_loop, save_snapshot, cleanup_loop};
use crate::services::pubsub_service;
use crate::routes::system_routes;

#[tokio::main]
async fn main() {
    //
    // ────────────────────────────────────────────────────────
    //  Locate config.json (EXE folder or project root)
    // ────────────────────────────────────────────────────────
    //
    let exe_path = std::env::current_exe().expect("Cannot get executable path");
    let exe_dir = exe_path.parent().expect("Cannot get executable directory");

    let mut config_path: PathBuf = exe_dir.join("config.json");

    if !config_path.exists() {
        let fallback = exe_dir.join("..").join("config.json");
        if fallback.exists() {
            config_path = fallback;
        } else {
            panic!(
                "config.json not found in:\n  {}\n  {}\nCopy config.json to one of these paths.",
                exe_dir.join("config.json").display(),
                fallback.display()
            );
        }
    }

    tracing::info!("Loading config.json from {}", config_path.display());

    //
    // ────────────────────────────────────────────────────────
    //  Load configuration
    // ────────────────────────────────────────────────────────
    //
    let cfg = AppConfig::load_from_file(config_path.to_str().unwrap());
    let cfg_for_routes = cfg.clone(); // so system_routes can access version

    //
    // ────────────────────────────────────────────────────────
    //  Configure logging
    // ────────────────────────────────────────────────────────
    //
    let level = match cfg.log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info"  => LevelFilter::INFO,
        "warn"  => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    //
    // ────────────────────────────────────────────────────────
    //  Initialize Pub/Sub
    // ────────────────────────────────────────────────────────
    //
    //pubsub_service::init().await;

    tracing::info!("Starting DodoDB…");
    tracing::info!("Loaded configuration: {:?}", cfg);

    //
    // ────────────────────────────────────────────────────────
    //  Create KV store and load snapshot
    // ────────────────────────────────────────────────────────
    //
    let store = new_store();
    load_snapshot(&cfg.snapshot_path, &store, cfg.retention_seconds).await;

    //
    // ────────────────────────────────────────────────────────
    //  Start autosave loop
    // ────────────────────────────────────────────────────────
    //
    {
        let store_clone = store.clone();
        let path = cfg.snapshot_path.clone();
        let interval = cfg.snapshot_interval;

        task::spawn(async move {
            autosave_loop(path, store_clone, interval).await;
        });
    }

    //
    // ────────────────────────────────────────────────────────
    //  Start cleanup loop (optional)
    // ────────────────────────────────────────────────────────
    //
    if let (Some(retention), Some(clean_interval)) =
        (cfg.retention_seconds, cfg.cleanup_interval)
    {
        let store_clone = store.clone();
        tracing::info!(
            "Starting cleanup loop: retention={}s, interval={}s",
            retention,
            clean_interval
        );

        task::spawn(async move {
            cleanup_loop(store_clone, retention, clean_interval).await;
        });
    }

    //
    // ────────────────────────────────────────────────────────
    //  Build Axum app (KV + PubSub + System routes)
    // ────────────────────────────────────────────────────────
    //
    let app = app::build_app(store.clone(), cfg.clone());

    //
    // ────────────────────────────────────────────────────────
    //  Bind server and start listening
    // ────────────────────────────────────────────────────────
    //
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Listening on http://{}", addr);

    serve(listener, app)
        .with_graceful_shutdown(shutdown(store.clone(), cfg.snapshot_path.clone()))
        .await
        .expect("Server error");
}

//
// ─────────────────────────────────────────────────────────────
//  Graceful shutdown handler
// ─────────────────────────────────────────────────────────────
//
async fn shutdown(store: KvStore, path: String) {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    tracing::warn!("CTRL+C received — saving snapshot…");
    save_snapshot(&path, &store).await;
    tracing::info!("Snapshot saved. Goodbye.");
}