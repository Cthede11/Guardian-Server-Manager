pub async fn suggested_radius_for(_server_id: &str) -> Option<u32> { Some(5000) }

pub fn gpu_available() -> bool {
  #[cfg(target_os = "windows")]
  unsafe {
    libloading::Library::new("nvcuda.dll").is_ok()
  }
  #[cfg(not(target_os = "windows"))]
  { false }
}