// Core application modules
pub mod app_state;
pub mod config;
pub mod guardian_config;
pub mod crash_watchdog;
pub mod scheduler;
pub mod resource_monitor;
pub mod test_harness;
pub mod server_manager;
pub mod process_manager;
pub mod file_manager;
pub mod security;
pub mod monitoring;
pub mod websocket;
pub mod auth;
pub mod middleware;
pub mod error_handler;
pub mod retry;
pub mod validation;
pub mod logging;
pub mod performance;
pub mod caching;

pub use app_state::AppState;
pub use config::Config;
pub use error_handler::{AppError, Result};
