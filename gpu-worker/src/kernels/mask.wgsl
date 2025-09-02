// Mask generation kernel for Minecraft chunk generation
// This kernel generates masks for caves, ores, and other features

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

struct MaskOutput {
    cave_mask: u32,
    ore_mask: u32,
    water_mask: u32,
    structure_mask: u32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<MaskOutput>;

@group(0) @binding(1)
var<uniform> input: ChunkInput;

@group(0) @binding(2)
var<storage, read> density_data: array<DensityOutput>;

// Simple hash function for deterministic noise
fn hash(seed: u32) -> f32 {
    var x = seed;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = (x >> 16) ^ x;
    return f32(x) / 4294967296.0;
}

// 3D noise function for cave generation
fn noise3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let x0 = floor(x);
    let y0 = floor(y);
    let z0 = floor(z);
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let z1 = z0 + 1.0;
    
    let fx = x - x0;
    let fy = y - y0;
    let fz = z - z0;
    
    let n000 = hash(u32(x0) + u32(y0) * 374761393 + u32(z0) * 668265263 + seed);
    let n001 = hash(u32(x0) + u32(y0) * 374761393 + u32(z1) * 668265263 + seed);
    let n010 = hash(u32(x0) + u32(y1) * 374761393 + u32(z0) * 668265263 + seed);
    let n011 = hash(u32(x0) + u32(y1) * 374761393 + u32(z1) * 668265263 + seed);
    let n100 = hash(u32(x1) + u32(y0) * 374761393 + u32(z0) * 668265263 + seed);
    let n101 = hash(u32(x1) + u32(y0) * 374761393 + u32(z1) * 668265263 + seed);
    let n110 = hash(u32(x1) + u32(y1) * 374761393 + u32(z0) * 668265263 + seed);
    let n111 = hash(u32(x1) + u32(y1) * 374761393 + u32(z1) * 668265263 + seed);
    
    let nx00 = mix(n000, n100, fx);
    let nx01 = mix(n001, n101, fx);
    let nx10 = mix(n010, n110, fx);
    let nx11 = mix(n011, n111, fx);
    
    let nxy0 = mix(nx00, nx10, fy);
    let nxy1 = mix(nx01, nx11, fy);
    
    return mix(nxy0, nxy1, fz);
}

// Multi-octave 3D noise
fn noise3d_octaves(x: f32, y: f32, z: f32, seed: u32, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;
    
    for (var i = 0u; i < octaves; i++) {
        value += noise3d(x * frequency, y * frequency, z * frequency, seed + i) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value / max_value;
}

// Cave generation
fn generate_cave_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let cave_noise = noise3d_octaves(x * 0.05, y * 0.05, z * 0.05, u32(seed) + 5000, 3u);
    let cave_threshold = 0.3;
    
    if (cave_noise > cave_threshold) {
        return 1u; // Cave
    } else {
        return 0u; // Solid
    }
}

// Ore generation
fn generate_ore_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let ore_noise = noise3d_octaves(x * 0.1, y * 0.1, z * 0.1, u32(seed) + 6000, 2u);
    let ore_threshold = 0.8;
    
    if (ore_noise > ore_threshold) {
        // Determine ore type based on depth and noise
        if (y < 16.0) {
            return 1u; // Coal
        } else if (y < 32.0) {
            return 2u; // Iron
        } else if (y < 48.0) {
            return 3u; // Gold
        } else {
            return 4u; // Diamond
        }
    } else {
        return 0u; // No ore
    }
}

// Water generation
fn generate_water_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let water_noise = noise3d_octaves(x * 0.02, y * 0.02, z * 0.02, u32(seed) + 7000, 2u);
    let water_threshold = 0.4;
    
    if (water_noise > water_threshold && y < 64.0) {
        return 1u; // Water
    } else {
        return 0u; // Air/Solid
    }
}

// Structure generation
fn generate_structure_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let structure_noise = noise3d_octaves(x * 0.01, y * 0.01, z * 0.01, u32(seed) + 8000, 2u);
    let structure_threshold = 0.9;
    
    if (structure_noise > structure_threshold) {
        // Determine structure type based on position and noise
        if (y > 60.0) {
            return 1u; // Surface structure
        } else if (y < 20.0) {
            return 2u; // Underground structure
        } else {
            return 3u; // Mid-level structure
        }
    } else {
        return 0u; // No structure
    }
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
    
    // Get density data for this position
    let density_index = z * 16u + x;
    let density = density_data[density_index];
    
    // Generate masks for each Y level (simplified to single level for now)
    let y = density.density; // Use terrain height as reference
    
    let cave_mask = generate_cave_mask(world_x, y, world_z, input.seed);
    let ore_mask = generate_ore_mask(world_x, y, world_z, input.seed);
    let water_mask = generate_water_mask(world_x, y, world_z, input.seed);
    let structure_mask = generate_structure_mask(world_x, y, world_z, input.seed);
    
    // Store result
    let index = z * 16u + x;
    output[index] = MaskOutput(
        cave_mask,
        ore_mask,
        water_mask,
        structure_mask
    );
}
