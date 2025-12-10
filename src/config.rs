use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// HTTP port to listen on.
    pub port: u16,

    /// Log level for tracing (e.g. "info", "debug").
    pub log_level: String,

    /// Path to the snapshot JSON file.
    pub snapshot_path: String,

    /// Interval (seconds) between automatic snapshot saves.
    pub snapshot_interval: u64,

    pub server_version: String,

    

    /// Global retention window (seconds).
    ///
    /// If set, keys older than this will be removed:
    /// - On startup when loading the snapshot
    /// - Periodically by a cleanup loop (see `cleanup_interval`)
    ///
    /// If `None`, keys never expire automatically.
    pub retention_seconds: Option<u64>,

    /// How often (seconds) to run the cleanup loop.
    ///
    /// If `None`, no cleanup loop is started and expiration only
    /// happens on snapshot load.
    pub cleanup_interval: Option<u64>,
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Self {
        let file = fs::read_to_string(Path::new(path))
            .expect("Failed to read config.json");

        serde_json::from_str::<AppConfig>(&file)
            .expect("Invalid config.json")
    }
}

