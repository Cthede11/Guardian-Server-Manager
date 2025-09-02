use wgpu::*;
use anyhow::Result;
use std::borrow::Cow;

/// Density generation kernel for terrain generation
pub struct DensityKernel {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl DensityKernel {
    /// Create a new density kernel
    pub async fn new(device: &Device, bind_group_layout: &BindGroupLayout) -> Result<Self> {
        // Load shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Density Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("density.wgsl"))),
        });
        
        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Density Pipeline"),
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
    
    /// Generate density data for a chunk
    pub async fn generate(
        &self,
        device: &Device,
        queue: &Queue,
        input_buffer: &Buffer,
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
    ) -> Result<Vec<u8>> {
        // Create output buffer
        let output_size = 16 * 16 * 16; // 16x16x16 density values
        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Density Output Buffer"),
            size: output_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Density Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Density Command Encoder"),
        });
        
        // Dispatch compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Density Compute Pass"),
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
