pub async fn suggested_radius_for(_server_id: &str) -> Option<u32> {
    // TODO: compute from max_players policy; default for now
    Some(5000)
}

pub fn gpu_available() -> bool {
    // TODO: real CUDA probe; default true on Windows as a hint
    #[cfg(target_os = "windows")]
    { true }
    #[cfg(not(target_os = "windows"))]
    { false }
}