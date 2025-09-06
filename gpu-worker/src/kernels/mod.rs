mod density;
mod mask;

use wgpu::*;
use wgpu::util::DeviceExt;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};

pub use density::DensityKernel;
pub use mask::MaskKernel;

/// Chunk generation parameters
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ChunkParams {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub seed: u32,
    pub dimension: u32, // 0 = overworld, 1 = nether, 2 = end
}

/// Chunk generation result
#[repr(C)]
#[derive(Clone, Debug)]
pub struct ChunkData {
    pub density_data: [f32; 16 * 16 * 384], // 16x16x384 density values
    pub mask_data: [u32; 16 * 16 * 384],    // 16x16x384 mask values
    pub biome_data: [u32; 16 * 16],         // 16x16 biome values
    pub content_hash: u32,
}

/// Chunk generator that coordinates GPU kernels for world generation
pub struct ChunkGenerator {
    density_kernel: DensityKernel,
    mask_kernel: MaskKernel,
    bind_group_layout: BindGroupLayout,
    compute_pipeline: ComputePipeline,
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
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Chunk Generator Pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Chunk Generator Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Chunk Generator Shader"),
                source: ShaderSource::Wgsl(include_str!("chunk_generator.wgsl").into()),
            }),
            entry_point: "main",
        });

        // Initialize kernels
        let density_kernel = DensityKernel::new(device).await?;
        let mask_kernel = MaskKernel::new(device).await?;

        Ok(Self {
            density_kernel,
            mask_kernel,
            bind_group_layout,
            compute_pipeline,
        })
    }

    /// Generate a chunk using GPU kernels
    pub async fn generate_chunk(
        &self,
        device: &Device,
        queue: &Queue,
        chunk_x: i32,
        chunk_z: i32,
        seed: u32,
        dimension: &str,
    ) -> Result<ChunkData> {
        // Convert dimension string to u32
        let dimension_id = match dimension {
            "overworld" => 0,
            "nether" => 1,
            "end" => 2,
            _ => 0,
        };

        // Create chunk parameters
        let params = ChunkParams {
            chunk_x,
            chunk_z,
            seed,
            dimension: dimension_id,
        };

        // Create buffers
        let params_data = unsafe { std::slice::from_raw_parts(&params as *const ChunkParams as *const u8, std::mem::size_of::<ChunkParams>()) };
        let params_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Chunk Params Buffer"),
            contents: params_data,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Chunk Output Buffer"),
            size: std::mem::size_of::<ChunkData>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Generator Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // Create command encoder
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Chunk Generation Encoder"),
        });

        // Dispatch compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Chunk Generation Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(16, 16, 1); // 16x16 workgroups for 16x16 chunks
        }

        // Submit command buffer
        queue.submit(std::iter::once(encoder.finish()));

        // Read back results
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        device.poll(Maintain::Wait);
        receiver.receive().await.unwrap()?;

        // Get the data
        let data = buffer_slice.get_mapped_range();
        let chunk_data = unsafe { 
            let ptr = data.as_ptr() as *const ChunkData;
            (*ptr).clone()
        };
        drop(data);
        output_buffer.unmap();

        Ok(chunk_data)
    }
}