use serde::{Serialize, Deserialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct BlueGreen {
    pub active: String,
    pub candidate_healthy: bool,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ServerSummary {
    pub id: String,
    pub name: String,
    pub status: String, // "running" | "stopped" | "degraded" | "starting" | "stopping"
    pub tps: f64,
    pub tickP95: f64,
    pub heapMb: u32,
    pub playersOnline: u32,
    pub gpuQueueMs: f64,
    pub lastSnapshotAt: Option<String>,
    pub blueGreen: Option<BlueGreen>,
    pub version: Option<String>,
    pub maxPlayers: Option<u32>,
    pub memory: Option<u32>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ConsoleLines {
    pub lines: Vec<ConsoleLine>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ConsoleLine {
    pub ts: String,
    pub level: String, // "INFO" | "WARN" | "ERROR" | "DEBUG"
    pub msg: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Metrics {
    pub tps: f64,
    pub tickP95: f64,
    pub heapMb: u32,
    pub gpuQueueMs: f64,
    pub playersOnline: u32,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub online: bool,
    pub last_seen: Option<String>,
    pub playtime: Option<u32>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct FreezeTicket {
    pub id: String,
    pub actor_id: String,
    pub location: Location,
    pub duration: u32,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Location {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub dimension: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub created_at: String,
    pub scope: String, // "global" | "dimension" | "claim" | "chunk"
    pub status: String, // "creating" | "ready" | "failed"
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub description: String,
    pub code: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct PregenJob {
    pub id: String,
    pub region: Region,
    pub dimension: String,
    pub priority: String, // "low" | "normal" | "high"
    pub status: String, // "queued" | "running" | "completed" | "failed"
    pub progress: f32,
    pub eta: Option<String>,
    pub gpu_assist: bool,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Region {
    pub x: i32,
    pub z: i32,
    pub radius: u32,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ServerHealth {
    pub rcon: bool,
    pub query: bool,
    pub crash_tickets: u32,
    pub freeze_tickets: u32,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ServerSettings {
    pub general: GeneralSettings,
    pub jvm: JVMSettings,
    pub gpu: GPUSettings,
    pub ha: HASettings,
    pub paths: PathSettings,
    pub composer: ComposerSettings,
    pub tokens: TokenSettings,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct GeneralSettings {
    pub name: String,
    pub description: String,
    pub version: String,
    pub loader: String,
    pub max_players: u32,
    pub motd: String,
    pub difficulty: String,
    pub gamemode: String,
    pub pvp: bool,
    pub online_mode: bool,
    pub whitelist: bool,
    pub enable_command_block: bool,
    pub view_distance: u32,
    pub simulation_distance: u32,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct JVMSettings {
    pub memory: u32,
    pub flags: Vec<String>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct GPUSettings {
    pub enabled: bool,
    pub queue_size: u32,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct HASettings {
    pub enabled: bool,
    pub blue_green: bool,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct PathSettings {
    pub world: String,
    pub mods: String,
    pub config: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ComposerSettings {
    pub profile: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct TokenSettings {
    pub rcon: String,
    pub query: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub conflicts: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Conflict {
    pub id: String,
    pub mods: Vec<String>,
    pub severity: String, // "warning" | "error"
    pub description: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scheduled_at: String,
    pub status: String, // "scheduled" | "running" | "completed" | "failed"
    pub actions: Vec<String>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Shard {
    pub id: String,
    pub name: String,
    pub status: String, // "healthy" | "degraded" | "offline"
    pub dimensions: Vec<String>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ShardingTopology {
    pub shards: Vec<Shard>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ShardAssignment {
    pub id: String,
    pub shard_id: String,
    pub server_id: String,
    pub dimensions: Vec<String>,
    pub player_count: u32,
    pub status: String, // "active" | "inactive" | "error"
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct CrashSignature {
    pub id: String,
    pub pattern: String,
    pub severity: String, // "low" | "medium" | "high" | "critical"
    pub description: String,
    pub occurrences: u32,
    pub last_seen: String,
}

// API Response wrapper
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct GpuStatus {
    pub available: bool,
    pub worker_id: Option<String>,
    pub queue_size: usize,
    pub last_activity: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}
