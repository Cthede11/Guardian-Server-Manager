// Mask Generation GPU Shader
// Generates terrain mask data based on density values

struct MaskParams {
    chunk_x: i32,
    chunk_z: i32,
    seed: u32,
    dimension: u32,
}

@group(0) @binding(0)
var<storage, read_write> mask_output: array<u32>;

@group(0) @binding(1)
var<storage, read> density_input: array<f32>;

@group(0) @binding(2)
var<uniform> params: MaskParams;

// Generate mask values based on density
fn generate_mask_value(density: f32, y: f32, dimension: u32) -> u32 {
    // Basic solid/air determination
    if (density > 0.0) {
        // Solid block
        if (dimension == 1u) { // Nether
            // Nether blocks
            if (y < 32.0) {
                return 1u; // Netherrack
            } else if (y < 40.0) {
                return 2u; // Soul sand
            } else {
                return 3u; // Nether brick
            }
        } else if (dimension == 2u) { // End
            // End blocks
            if (y < 64.0) {
                return 4u; // End stone
            } else {
                return 5u; // Air
            }
        } else { // Overworld
            // Overworld blocks
            if (y < 5.0) {
                return 6u; // Bedrock
            } else if (y < 16.0) {
                return 7u; // Stone
            } else if (y < 32.0) {
                return 8u; // Dirt
            } else if (y < 64.0) {
                return 9u; // Grass
            } else if (y < 80.0) {
                return 10u; // Sand
            } else {
                return 11u; // Air
            }
        }
    } else {
        // Air block
        return 0u;
    }
}

// Apply smoothing to mask values
fn smooth_mask(x: u32, y: u32, z: u32, dimension: u32) -> u32 {
    let index = y * 256u + z * 16u + x;
    let density = density_input[index];
    
    // Check neighboring blocks for smoothing
    var solid_neighbors = 0u;
    var total_neighbors = 0u;
    
    // Check 6 neighbors (up, down, north, south, east, west)
    for (var i = 0u; i < 6u; i++) {
        var neighbor_x = i32(x);
        var neighbor_y = i32(y);
        var neighbor_z = i32(z);
        
        if (i == 0u) { neighbor_y += 1; }      // up
        else if (i == 1u) { neighbor_y -= 1; } // down
        else if (i == 2u) { neighbor_z += 1; } // north
        else if (i == 3u) { neighbor_z -= 1; } // south
        else if (i == 4u) { neighbor_x += 1; } // east
        else if (i == 5u) { neighbor_x -= 1; } // west
        
        // Check bounds
        if (neighbor_x >= 0 && neighbor_x < 16 && 
            neighbor_y >= 0 && neighbor_y < 384 && 
            neighbor_z >= 0 && neighbor_z < 16) {
            
            let neighbor_index = u32(neighbor_y) * 256u + u32(neighbor_z) * 16u + u32(neighbor_x);
            if (density_input[neighbor_index] > 0.0) {
                solid_neighbors++;
            }
            total_neighbors++;
        }
    }
    
    // Apply smoothing based on neighbor count
    if (total_neighbors > 0u) {
        let solid_ratio = f32(solid_neighbors) / f32(total_neighbors);
        
        if (solid_ratio > 0.5) {
            return generate_mask_value(density, f32(y), dimension);
        } else if (solid_ratio < 0.3) {
            return 0u; // Air
        }
    }
    
    return generate_mask_value(density, f32(y), dimension);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let z = global_id.y;
    
    if (x >= 16u || z >= 16u) {
        return;
    }
    
    // Generate mask for this column
    for (var y = 0u; y < 384u; y++) {
        let index = y * 256u + z * 16u + x;
        let density = density_input[index];
        
        // Generate base mask value
        let base_mask = generate_mask_value(density, f32(y), params.dimension);
        
        // Apply smoothing
        let smoothed_mask = smooth_mask(x, y, z, params.dimension);
        
        // Final mask value
        mask_output[index] = smoothed_mask;
    }
}