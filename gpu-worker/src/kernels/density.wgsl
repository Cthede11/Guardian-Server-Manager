// Density Generation GPU Shader
// Generates terrain density data using noise functions

struct DensityParams {
    chunk_x: i32,
    chunk_z: i32,
    seed: u32,
    dimension: u32,
}

@group(0) @binding(0)
var<storage, read_write> density_output: array<f32>;

@group(0) @binding(1)
var<uniform> params: DensityParams;

// Improved noise function for terrain generation
fn hash(p: vec2<u32>) -> f32 {
    var p3 = fract(vec3<f32>(f32(p.x), f32(p.y), f32(p.x)) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = vec2<u32>(floor(p));
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(
        mix(hash(i), hash(i + vec2<u32>(1u, 0u)), u.x),
        mix(hash(i + vec2<u32>(0u, 1u)), hash(i + vec2<u32>(1u, 1u)), u.x),
        u.y
    );
}

fn fbm(p: vec2<f32>, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    
    for (var i = 0u; i < octaves; i++) {
        value += amplitude * noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

fn ridged_noise(p: vec2<f32>) -> f32 {
    return 1.0 - abs(noise(p));
}

fn generate_terrain_height(x: f32, z: f32, seed: u32, dimension: u32) -> f32 {
    let world_x = f32(params.chunk_x) * 16.0 + x;
    let world_z = f32(params.chunk_z) * 16.0 + z;
    
    // Add seed offset to world coordinates
    let offset_x = world_x + f32(seed & 0xFFFF) * 0.001;
    let offset_z = world_z + f32((seed >> 16) & 0xFFFF) * 0.001;
    
    if (dimension == 1u) { // Nether
        // Nether terrain - more chaotic and varied
        let height1 = fbm(vec2<f32>(offset_x * 0.01, offset_z * 0.01), 4u) * 32.0;
        let height2 = ridged_noise(vec2<f32>(offset_x * 0.05, offset_z * 0.05)) * 16.0;
        return 32.0 + height1 + height2;
    } else if (dimension == 2u) { // End
        // End terrain - very flat with occasional spikes
        let height = fbm(vec2<f32>(offset_x * 0.02, offset_z * 0.02), 3u) * 8.0;
        let spikes = ridged_noise(vec2<f32>(offset_x * 0.1, offset_z * 0.1)) * 4.0;
        return 64.0 + height + spikes;
    } else { // Overworld
        // Overworld terrain - varied and realistic
        let height1 = fbm(vec2<f32>(offset_x * 0.005, offset_z * 0.005), 6u) * 64.0;
        let height2 = fbm(vec2<f32>(offset_x * 0.02, offset_z * 0.02), 4u) * 16.0;
        let height3 = ridged_noise(vec2<f32>(offset_x * 0.1, offset_z * 0.1)) * 8.0;
        return 64.0 + height1 + height2 + height3;
    }
}

fn generate_cave_noise(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let world_x = f32(params.chunk_x) * 16.0 + x;
    let world_z = f32(params.chunk_z) * 16.0 + z;
    
    // Add seed offset
    let offset_x = world_x + f32(seed & 0xFFFF) * 0.001;
    let offset_y = y + f32((seed >> 8) & 0xFF) * 0.001;
    let offset_z = world_z + f32((seed >> 16) & 0xFFFF) * 0.001;
    
    // 3D noise for caves
    let noise1 = noise(vec2<f32>(offset_x * 0.1, offset_z * 0.1));
    let noise2 = noise(vec2<f32>(offset_x * 0.1 + 100.0, offset_z * 0.1 + 100.0));
    let noise3 = noise(vec2<f32>(offset_y * 0.1, offset_z * 0.1 + 200.0));
    
    return (noise1 + noise2 + noise3) / 3.0;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let z = global_id.y;
    
    if (x >= 16u || z >= 16u) {
        return;
    }
    
    // Generate density for this column
    let terrain_height = generate_terrain_height(f32(x), f32(z), params.seed, params.dimension);
    
    for (var y = 0u; y < 384u; y++) {
        let local_y = f32(y);
        let index = y * 256u + z * 16u + x;
        
        // Base density calculation
        var density = terrain_height - local_y;
        
        // Add cave generation for overworld
        if (params.dimension == 0u) {
            let cave_noise = generate_cave_noise(f32(x), local_y, f32(z), params.seed);
            let cave_factor = 1.0 - smoothstep(0.2, 0.8, abs(cave_noise));
            density *= cave_factor;
        }
        
        // Add some variation
        let variation = noise(vec2<f32>(f32(x) * 0.5, f32(z) * 0.5)) * 0.1;
        density += variation;
        
        density_output[index] = density;
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn clamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    return max(min_val, min(max_val, x));
}