struct Vertex {
    @location(0)
    center: vec2f,
    @location(1)
    sprite_center: vec2f,
    @location(2)
    sprite_extents: vec2f,
    @location(3)
    layer: f32,
    @location(4)
    vertex_pos: vec2f,
}

struct Fragment {
    @builtin(position)
    pos: vec4f,
    @location(0)
    uv: vec2f,
}

struct Camera {
    center: vec2f,
    extents: vec2f,
}

@group(0) @binding(0)
var(uniform) cam: Camera;
@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var sampler: sampler;

@vertex
fn vs_main(input: Vertex) -> Fragment {
    var output: Fragment;

    let extents = input.sprite_extents * vec2f(textureDimensions(texture)) / 16.0;
    let world_pos = input.center + input.vertex_pos * extents;
    let cam_pos = (world_pos - cam.center) / cam.extents;

    output.pos = vec4f(cam_pos, input.layer / 1_000.0, 1.0);
    output.uv = input.sprite_center + input.vertex_pos * input.sprite_extents;
    
    return output;
}