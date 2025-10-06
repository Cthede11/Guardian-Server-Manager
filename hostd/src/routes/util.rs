pub fn gpu_available() -> bool {
    #[cfg(target_os = "windows")]
    unsafe { libloading::Library::new("nvcuda.dll").is_ok() }
    #[cfg(not(target_os = "windows"))]
    { false }
}

pub async fn suggested_radius_for(server_id: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // AI-EXPLAIN: Calculate suggested pregen radius based on server specs
    // For now, return a reasonable default based on typical server needs
    // In the future, this could query server configuration for world type, mods, etc.
    let base_radius = 5000;
    
    // Could be enhanced to check server configuration for:
    // - World type (overworld vs nether vs end)
    // - Modded vs vanilla (modded worlds often need larger radius)
    // - Server performance specs
    // - Historical player activity patterns
    
    Ok(base_radius)
}