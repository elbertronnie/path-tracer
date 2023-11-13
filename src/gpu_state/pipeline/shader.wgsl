struct VertexInput {
    @location(0) vertex: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Vertex shader

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.vertex, 0.0, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

// Fragment shader

@group(0) @binding(0) var color_buffer: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(color_buffer, screen_sampler, tex_coords);
}
