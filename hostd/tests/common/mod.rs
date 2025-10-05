use hostd::database::DatabaseManager;
use hostd::minecraft::MinecraftManager;
use hostd::mod_manager::ModManager;
use hostd::websocket::WebSocketManager;
use hostd::api::AppState;
use std::sync::Arc;

pub async fn create_test_app_state() -> AppState {
    let database = DatabaseManager::new(":memory:").await.unwrap();
    let minecraft_manager = MinecraftManager::new();
    let mod_manager = ModManager::new();
    let websocket_manager = WebSocketManager::new();
    
    AppState {
        websocket_manager,
        minecraft_manager,
        database,
        mod_manager,
    }
}

pub fn create_test_server_config() -> hostd::database::ServerConfig {
    use chrono::Utc;
    use uuid::Uuid;
    
    hostd::database::ServerConfig {
        id: Uuid::new_v4().to_string(),
        name: "Test Server".to_string(),
        minecraft_version: "1.20.1".to_string(),
        loader: "vanilla".to_string(),
        loader_version: "1.20.1".to_string(),
        port: 25565,
        rcon_port: 25575,
        query_port: 25566,
        max_players: 20,
        memory: 2048,
        java_args: r#"["-Xmx2G", "-Xms1G"]"#.to_string(),
        server_args: r#"["--nogui"]"#.to_string(),
        auto_start: false,
        auto_restart: true,
        world_name: "world".to_string(),
        difficulty: "normal".to_string(),
        gamemode: "survival".to_string(),
        pvp: true,
        online_mode: true,
        whitelist: false,
        enable_command_block: false,
        view_distance: 10,
        simulation_distance: 10,
        motd: "A Minecraft Server".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
