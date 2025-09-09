pub fn gpu_available() -> bool {
    #[cfg(target_os = "windows")]
    unsafe { libloading::Library::new("nvcuda.dll").is_ok() }
    #[cfg(not(target_os = "windows"))]
    { false }
}

pub async fn suggested_radius_for(_server_id: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // TODO: implement actual logic based on server specs and world size
    Ok(5000)
}