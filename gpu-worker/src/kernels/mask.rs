use wgpu::*;
use anyhow::Result;
use std::borrow::Cow;

/// Mask generation kernel for caves, ores, and other features
pub struct MaskKernel {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl MaskKernel {
    /// Create a new mask kernel
    pub async fn new(device: &Device, bind_group_layout: &BindGroupLayout) -> Result<Self> {
        // Load shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Mask Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("mask.wgsl"))),
        });
        
        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Mask Pipeline"),
            layout: None,
            compute: ProgrammableStageDescriptor {
                module: &shader,
                entry_point: "main",
                compilation_options: PipelineCompilationOptions::default(),
            },
        });
        
        Ok(Self {
            pipeline,
            bind_group_layout: bind_group_layout.clone(),
        })
    }
    
    /// Generate mask data for a chunk
    pub async fn generate(
        &self,
        device: &Device,
        queue: &Queue,
        input_buffer: &Buffer,
        density_data: &[u8],
    ) -> Result<Vec<u8>> {
        // Create density buffer
        let density_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Density Data Buffer"),
            size: density_data.len() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        queue.write_buffer(&density_buffer, 0, density_data);
        
        // Create output buffer
        let output_size = 16 * 16 * 4; // 16x16 mask values (4 bytes each)
        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Mask Output Buffer"),
            size: output_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create bind group layout for mask kernel
        let mask_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Mask Bind Group Layout"),
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
                BindGroupLayoutEntry {
                    binding: 2,
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
        
        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mask Bind Group"),
            layout: &mask_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: density_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Mask Command Encoder"),
        });
        
        // Dispatch compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Mask Compute Pass"),
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1); // 16x16 workgroups
        }
        
        // Submit command
        queue.submit(std::iter::once(encoder.finish()));
        
        // Read back results
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        
        device.poll(Maintain::Wait);
        receiver.receive().await.unwrap()?;
        
        // Get mapped data
        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();
        drop(data);
        output_buffer.unmap();
        
        Ok(result)
    }
}
