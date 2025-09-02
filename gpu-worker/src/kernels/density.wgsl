// Density generation kernel for Minecraft chunk generation
// This kernel generates density values for terrain generation

struct ChunkInput {
    chunk_x: i32,
    chunk_z: i32,
    seed: i64,
    dimension_hash: u32,
}

struct DensityOutput {
    density: f32,
    biome: u32,
    temperature: f32,
    humidity: f32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<DensityOutput>;

@group(0) @binding(1)
var<uniform> input: ChunkInput;

// Simple hash function for deterministic noise
fn hash(seed: u32) -> f32 {
    var x = seed;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = (x >> 16) ^ x;
    return f32(x) / 4294967296.0;
}

// 2D noise function
fn noise2d(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = floor(x);
    let z0 = floor(z);
    let x1 = x0 + 1.0;
    let z1 = z0 + 1.0;
    
    let fx = x - x0;
    let fz = z - z0;
    
    let n00 = hash(u32(x0) + u32(z0) * 374761393 + seed);
    let n01 = hash(u32(x0) + u32(z1) * 374761393 + seed);
    let n10 = hash(u32(x1) + u32(z0) * 374761393 + seed);
    let n11 = hash(u32(x1) + u32(z1) * 374761393 + seed);
    
    let nx0 = mix(n00, n10, fx);
    let nx1 = mix(n01, n11, fx);
    
    return mix(nx0, nx1, fz);
}

// Multi-octave noise
fn noise2d_octaves(x: f32, z: f32, seed: u32, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;
    
    for (var i = 0u; i < octaves; i++) {
        value += noise2d(x * frequency, z * frequency, seed + i) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value / max_value;
}

// Terrain height calculation
fn calculate_terrain_height(x: f32, z: f32, seed: i64) -> f32 {
    let base_height = 64.0;
    let height_variation = 32.0;
    
    // Main terrain noise
    let terrain_noise = noise2d_octaves(x * 0.01, z * 0.01, u32(seed), 4u);
    
    // Mountain noise
    let mountain_noise = noise2d_octaves(x * 0.005, z * 0.005, u32(seed) + 1000, 3u);
    let mountain_factor = smoothstep(0.3, 0.7, mountain_noise);
    
    // Valley noise
    let valley_noise = noise2d_octaves(x * 0.02, z * 0.02, u32(seed) + 2000, 2u);
    let valley_factor = smoothstep(0.4, 0.6, valley_noise);
    
    let height = base_height + 
                 terrain_noise * height_variation + 
                 mountain_factor * 64.0 - 
                 valley_factor * 32.0;
    
    return height;
}

// Biome calculation
fn calculate_biome(x: f32, z: f32, seed: i64) -> u32 {
    let temperature = noise2d_octaves(x * 0.008, z * 0.008, u32(seed) + 3000, 2u);
    let humidity = noise2d_octaves(x * 0.012, z * 0.012, u32(seed) + 4000, 2u);
    
    // Simple biome classification
    if (temperature > 0.6 && humidity > 0.4) {
        return 1u; // Forest
    } else if (temperature > 0.4 && humidity < 0.3) {
        return 2u; // Plains
    } else if (temperature < 0.3) {
        return 3u; // Tundra
    } else if (humidity > 0.7) {
        return 4u; // Swamp
    } else {
        return 0u; // Default
    }
}

// Smoothstep function
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Clamp function
fn clamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    return max(min(x, max_val), min_val);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let z = global_id.y;
    
    if (x >= 16u || z >= 16u) {
        return;
    }
    
    let world_x = f32(input.chunk_x * 16 + i32(x));
    let world_z = f32(input.chunk_z * 16 + i32(z));
    
    // Calculate terrain height
    let height = calculate_terrain_height(world_x, world_z, input.seed);
    
    // Calculate biome
    let biome = calculate_biome(world_x, world_z, input.seed);
    
    // Calculate temperature and humidity
    let temperature = noise2d_octaves(world_x * 0.008, world_z * 0.008, u32(input.seed) + 3000, 2u);
    let humidity = noise2d_octaves(world_x * 0.012, world_z * 0.012, u32(input.seed) + 4000, 2u);
    
    // Store result
    let index = z * 16u + x;
    output[index] = DensityOutput(
        height,
        biome,
        temperature,
        humidity
    );
}
