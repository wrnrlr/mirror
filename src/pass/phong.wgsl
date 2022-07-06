// Vertex shader

struct Globals {
    view_proj: mat4x4<f32>
}

@group(0) @binding(0) var<uniform> globals: Globals;

struct Locals {
    color: vec4<f32>
}

@group(1) @binding(0) var<uniform> locals: Locals;

struct Vertex {
    @location(0) position: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>
}

@vertex fn vs_main(model: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return locals.color;
}
