mod density;
mod mask;

use wgpu::*;
use anyhow::Result;
use crate::ffi::ChunkResult;

pub use density::DensityKernel;
pub use mask::MaskKernel;

/// Chunk generator that coordinates GPU kernels for world generation
pub struct ChunkGenerator {
    density_kernel: DensityKernel,
    mask_kernel: MaskKernel,
    bind_group_layout: BindGroupLayout,
}

impl ChunkGenerator {
    /// Create a new chunk generator
    pub async fn new(device: &Device) -> Result<Self> {
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Chunk Generator Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Initialize kernels
        let density_kernel = DensityKernel::new(device, &bind_group_layout).await?;
        let mask_kernel = MaskKernel::new(device, &bind_group_layout).await?;
        
        Ok(Self {
            density_kernel,
            mask_kernel,
            bind_group_layout,
        })
    }
    
    /// Generate a chunk using GPU kernels
    pub async fn generate_chunk(
        &self,
        device: &Device,
        queue: &Queue,
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        dimension: &str,
    ) -> Result<ChunkResult> {
        // Create input buffer with chunk parameters
        let input_data = ChunkInput {
            chunk_x,
            chunk_z,
            seed,
            dimension_hash: self.hash_dimension(dimension),
        };
        
        let input_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Chunk Input Buffer"),
            size: std::mem::size_of::<ChunkInput>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        queue.write_buffer(&input_buffer, 0, bytemuck::bytes_of(&input_data));
        
        // Generate density data
        let density_data = self.density_kernel.generate(
            device,
            queue,
            &input_buffer,
            chunk_x,
            chunk_z,
            seed,
        ).await?;
        
        // Generate mask data
        let mask_data = self.mask_kernel.generate(
            device,
            queue,
            &input_buffer,
            &density_data,
        ).await?;
        
        // Generate biome data (placeholder)
        let biome_data = self.generate_biome_data(chunk_x, chunk_z, seed, dimension);
        
        // Create content hash
        let content_hash = self.create_content_hash(chunk_x, chunk_z, seed, &density_data, &mask_data);
        
        Ok(ChunkResult::new(
            chunk_x,
            chunk_z,
            seed,
            content_hash,
            density_data,
            mask_data,
            biome_data,
        ))
    }
    
    /// Generate biome data (placeholder implementation)
    fn generate_biome_data(&self, chunk_x: i32, chunk_z: i32, seed: i64, dimension: &str) -> Vec<u8> {
        // This would generate actual biome data based on the chunk position and seed
        // For now, return placeholder data
        vec![0u8; 16 * 16] // 16x16 biome grid
    }
    
    /// Create content hash for validation
    fn create_content_hash(&self, chunk_x: i32, chunk_z: i32, seed: i64, density_data: &[u8], mask_data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        chunk_x.hash(&mut hasher);
        chunk_z.hash(&mut hasher);
        seed.hash(&mut hasher);
        density_data.hash(&mut hasher);
        mask_data.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Hash dimension string to integer
    fn hash_dimension(&self, dimension: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        dimension.hash(&mut hasher);
        hasher.finish() as u32
    }
}

/// Input data for chunk generation
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkInput {
    chunk_x: i32,
    chunk_z: i32,
    seed: i64,
    dimension_hash: u32,
}
