// Chunk Generator GPU Shader
// Generates Minecraft chunk data using GPU compute shaders

struct ChunkParams {
    chunk_x: i32,
    chunk_z: i32,
    seed: u32,
    dimension: u32,
}

struct ChunkData {
    density_data: array<f32, 98304>, // 16x16x384 density values
    mask_data: array<u32, 98304>,    // 16x16x384 mask values
    biome_data: array<u32, 256>,     // 16x16 biome values
    content_hash: u32,
}

@group(0) @binding(0)
var<storage, read_write> output: ChunkData;

@group(0) @binding(1)
var<uniform> params: ChunkParams;

// Noise functions for terrain generation
fn noise2d(x: f32, z: f32, seed: u32) -> f32 {
    let x_int = u32(x * 1000.0) + (seed & 0xFFFF);
    let z_int = u32(z * 1000.0) + ((seed >> 16) & 0xFFFF);
    
    // Simple hash-based noise
    var hash = x_int * 374761393 + z_int * 668265263 + 1274126177;
    hash = hash ^ (hash >> 13);
    hash = hash * 1274126177;
    hash = hash ^ (hash >> 16);
    
    return f32(hash) / 4294967295.0 * 2.0 - 1.0;
}

fn noise3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let x_int = u32(x * 1000.0) + (seed & 0xFFFF);
    let y_int = u32(y * 1000.0) + ((seed >> 8) & 0xFF);
    let z_int = u32(z * 1000.0) + ((seed >> 16) & 0xFFFF);
    
    var hash = x_int * 374761393 + y_int * 668265263 + z_int * 1274126177 + 1274126177;
    hash = hash ^ (hash >> 13);
    hash = hash * 1274126177;
    hash = hash ^ (hash >> 16);
    
    return f32(hash) / 4294967295.0 * 2.0 - 1.0;
}

fn fractal_noise(x: f32, z: f32, seed: u32, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;
    
    for (var i = 0u; i < octaves; i++) {
        value += noise2d(x * frequency, z * frequency, seed + u32(i)) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value / max_value;
}

fn generate_density(x: f32, y: f32, z: f32, seed: u32, dimension: u32) -> f32 {
    let world_x = f32(params.chunk_x) * 16.0 + x;
    let world_z = f32(params.chunk_z) * 16.0 + z;
    
    if (dimension == 1u) { // Nether
        // Nether terrain generation
        let height = 32.0 + fractal_noise(world_x * 0.1, world_z * 0.1, seed, 4u) * 16.0;
        return height - y;
    } else if (dimension == 2u) { // End
        // End terrain generation
        let height = 64.0 + fractal_noise(world_x * 0.05, world_z * 0.05, seed, 2u) * 8.0;
        return height - y;
    } else { // Overworld
        // Overworld terrain generation
        let height = 64.0 + fractal_noise(world_x * 0.01, world_z * 0.01, seed, 6u) * 32.0;
        let cave_noise = noise3d(world_x * 0.1, y * 0.1, world_z * 0.1, seed + 1000u);
        let cave_factor = 1.0 - smoothstep(0.3, 0.7, abs(cave_noise));
        
        return (height - y) * cave_factor;
    }
}

fn generate_biome(x: f32, z: f32, seed: u32) -> u32 {
    let world_x = f32(params.chunk_x) * 16.0 + x;
    let world_z = f32(params.chunk_z) * 16.0 + z;
    
    let temperature = fractal_noise(world_x * 0.01, world_z * 0.01, seed, 4u);
    let humidity = fractal_noise(world_x * 0.01, world_z * 0.01, seed + 1000u, 4u);
    
    // Simple biome determination
    if (temperature > 0.5) {
        if (humidity > 0.5) {
            return 1u; // Forest
        } else {
            return 2u; // Desert
        }
    } else {
        if (humidity > 0.5) {
            return 3u; // Taiga
        } else {
            return 4u; // Plains
        }
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn clamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    return max(min_val, min(max_val, x));
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let z = global_id.y;
    
    if (x >= 16u || z >= 16u) {
        return;
    }
    
    // Generate density data for this column
    for (var y = 0u; y < 384u; y++) {
        let local_x = f32(x);
        let local_y = f32(y);
        let local_z = f32(z);
        
        let density = generate_density(local_x, local_y, local_z, params.seed, params.dimension);
        let biome = generate_biome(local_x, local_z, params.seed);
        
        let index = y * 256u + z * 16u + x;
        output.density_data[index] = density;
        output.mask_data[index] = select(0u, 1u, density > 0.0);
        
        if (y == 0u) {
            let biome_index = z * 16u + x;
            output.biome_data[biome_index] = biome;
        }
    }
    
    // Generate content hash
    if (x == 0u && z == 0u) {
        var hash = u32(0);
        for (var i = 0u; i < 256u; i++) {
            hash = hash * 31u + output.biome_data[i];
        }
        output.content_hash = hash;
    }
}
