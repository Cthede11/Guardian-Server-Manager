use std::ffi::CString;
use std::os::raw::{c_char, c_int};

/// Chunk job structure for C ABI
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ChunkJob {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub seed: i64,
    pub dimension: *const c_char,
    pub tick_count: i32,
    pub rule_version: *const c_char,
}

/// Chunk result structure for C ABI
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub seed: i64,
    pub content_hash: *const c_char,
    pub density_data: *mut u8,
    pub density_data_size: i32,
    pub mask_data: *mut u8,
    pub mask_data_size: i32,
    pub biome_data: *mut u8,
    pub biome_data_size: i32,
    pub status: i32, // 0 = success, 1 = error, 2 = not ready
}

/// Job handle for tracking async operations
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JobHandle {
    pub result: Option<ChunkResult>,
    pub completed: bool,
}

impl ChunkJob {
    /// Create a new chunk job from C strings
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        dimension: *const c_char,
        tick_count: i32,
        rule_version: *const c_char,
    ) -> Self {
        Self {
            chunk_x,
            chunk_z,
            seed,
            dimension,
            tick_count,
            rule_version,
        }
    }
    
    /// Get dimension as string
    pub fn get_dimension(&self) -> String {
        unsafe {
            if self.dimension.is_null() {
                "overworld".to_string()
            } else {
                std::ffi::CStr::from_ptr(self.dimension)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }
    
    /// Get rule version as string
    pub fn get_rule_version(&self) -> String {
        unsafe {
            if self.rule_version.is_null() {
                "1.0.0".to_string()
            } else {
                std::ffi::CStr::from_ptr(self.rule_version)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }
}

impl ChunkResult {
    /// Create a new chunk result
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        content_hash: String,
        density_data: Vec<u8>,
        mask_data: Vec<u8>,
        biome_data: Vec<u8>,
    ) -> Self {
        let hash_cstring = CString::new(content_hash).unwrap();
        let hash_ptr = hash_cstring.into_raw();
        
        let density_ptr = if density_data.is_empty() {
            std::ptr::null_mut()
        } else {
            density_data.as_ptr() as *mut u8
        };
        
        let mask_ptr = if mask_data.is_empty() {
            std::ptr::null_mut()
        } else {
            mask_data.as_ptr() as *mut u8
        };
        
        let biome_ptr = if biome_data.is_empty() {
            std::ptr::null_mut()
        } else {
            biome_data.as_ptr() as *mut u8
        };
        
        Self {
            chunk_x,
            chunk_z,
            seed,
            content_hash: hash_ptr,
            density_data: density_ptr,
            density_data_size: density_data.len() as i32,
            mask_data: mask_ptr,
            mask_data_size: mask_data.len() as i32,
            biome_data: biome_ptr,
            biome_data_size: biome_data.len() as i32,
            status: 0, // success
        }
    }
    
    /// Get content hash as string
    pub fn get_content_hash(&self) -> String {
        unsafe {
            if self.content_hash.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(self.content_hash)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }
    
    /// Get density data as slice
    pub fn get_density_data(&self) -> &[u8] {
        unsafe {
            if self.density_data.is_null() || self.density_data_size <= 0 {
                &[]
            } else {
                std::slice::from_raw_parts(self.density_data, self.density_data_size as usize)
            }
        }
    }
    
    /// Get mask data as slice
    pub fn get_mask_data(&self) -> &[u8] {
        unsafe {
            if self.mask_data.is_null() || self.mask_data_size <= 0 {
                &[]
            } else {
                std::slice::from_raw_parts(self.mask_data, self.mask_data_size as usize)
            }
        }
    }
    
    /// Get biome data as slice
    pub fn get_biome_data(&self) -> &[u8] {
        unsafe {
            if self.biome_data.is_null() || self.biome_data_size <= 0 {
                &[]
            } else {
                std::slice::from_raw_parts(self.biome_data, self.biome_data_size as usize)
            }
        }
    }
}

impl Drop for ChunkResult {
    fn drop(&mut self) {
        unsafe {
            if !self.content_hash.is_null() {
                let _ = CString::from_raw(self.content_hash as *mut c_char);
            }
        }
    }
}
