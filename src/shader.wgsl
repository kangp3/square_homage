// Vertex shader
//copy pasted from learn_wgpu Buffers and Indices
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> window_dims: vec4<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let w = window_dims[0];
    let h = window_dims[1];
    let is_wider = w > h;
    var proj_matrix = mat3x3<f32>(
        1.0, 0.0, 0.0,
        0.0, w/h, 0.0,
        0.0, 0.0, 1.0,
    );
    if is_wider {
        proj_matrix = mat3x3<f32>(
            h/w, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        );
    }
    let clip_position = proj_matrix * model.position;

    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(clip_position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
