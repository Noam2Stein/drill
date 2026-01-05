struct Vertex {
    @location(0) vertex_pos: vec2f,
}

struct Fragment {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
}

struct Uniform {
    dst_extents: vec2f,
}

@group(0) @binding(0) var<uniform> u: Uniform;
@group(0) @binding(1) var render_texture: texture_2d<f32>;
@group(0) @binding(2) var render_texture_sampler: sampler;

@vertex
fn vs_main(input: Vertex) -> Fragment {
    var output: Fragment;

    output.pos = vec4f(input.vertex_pos * u.dst_extents, 0.0, 1.0);
    output.uv = input.vertex_pos * vec2f(0.5, -0.5) + 0.5;
    
    return output;
}

@fragment
fn fs_main(input: Fragment) -> @location(0) vec4f {
    return textureSample(render_texture, render_texture_sampler, input.uv);
}
