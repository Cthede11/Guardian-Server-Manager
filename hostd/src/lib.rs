// Core modules
pub mod core;
pub mod database;
pub mod api;
pub mod websocket;
pub mod backup;
pub mod compatibility;
pub mod compatibility_engine;
pub mod pregeneration;
pub mod hot_import;
pub mod lighting;
pub mod mod_management;

// Legacy modules (to be phased out)
pub mod routes;
pub mod boot;
pub mod metrics_collector;
pub mod websocket_manager;
pub mod console_streamer;
pub mod world_manager;
pub mod mod_manager;
pub mod backup_manager;
pub mod security;
pub mod minecraft;
pub mod rcon;