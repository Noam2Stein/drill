const PIXELS_PER_UNIT: f32 = 16.0;
const ASPECT: f32 = 16.0 / 9.0;
const MAX_ORTHO_SIZE: f32 = 18.0;

struct Vertex {
    @location(0) vertex_pos: vec2f,
    @location(1) quad_center: vec2f,
    @location(2) quad_sprite_center: vec2f,
    @location(3) quad_sprite_extents: vec2f,
    @location(4) quad_layer: f32,
}

struct Fragment {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
}

struct Uniform {
    cam_center: vec2f,
}

@group(0) @binding(0) var<uniform> u: Uniform;
@group(0) @binding(1) var sprites: texture_2d<f32>;
@group(0) @binding(2) var sprites_sampler: sampler;

@vertex
fn vs_main(input: Vertex) -> Fragment {
    var output: Fragment;

    let quad_extents = input.quad_sprite_extents
        * vec2f(textureDimensions(sprites))
        / PIXELS_PER_UNIT;

    let quad_center = round(input.quad_center * PIXELS_PER_UNIT) / PIXELS_PER_UNIT;
    let world_pos = quad_center + input.vertex_pos * quad_extents;
    let screen_pos = (world_pos - u.cam_center) / MAX_ORTHO_SIZE / vec2f(ASPECT, 1.0);

    output.pos = vec4f(screen_pos, input.quad_layer / 1000.0, 1.0);
    output.uv = input.quad_sprite_center + input.vertex_pos * input.quad_sprite_extents;
    
    return output;
}

@fragment
fn fs_main(input: Fragment) -> @location(0) vec4f {
    return textureSample(sprites, sprites_sampler, input.uv);
}
