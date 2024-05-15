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

struct MyUniform {
    window_dims: vec2<f32>,
    elapsed: f32,
}
@group(0) @binding(0) var<uniform> my_uniform: MyUniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let w = my_uniform.window_dims[0];
    let h = my_uniform.window_dims[1];

    let proj_matrix = world_to_clip_mat(w, h);
    let clip_position = proj_matrix * model.position;

    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(clip_position, 1.0);
    return out;
}

@vertex
fn vs_main_bloop(
    model: VertexInput,
) -> VertexOutput {
    let w = my_uniform.window_dims[0];
    let h = my_uniform.window_dims[1];
    let scale_offset = 0.05*sin(my_uniform.elapsed/500.);

    var world_pos = model.position;
    var proj_matrix = world_to_clip_mat(w, h);
    if (w > h) {
        proj_matrix[0][0] = proj_matrix[0][0] + scale_offset;
    } else {
        proj_matrix[1][1] = proj_matrix[1][1] + scale_offset;
    }
    let clip_position = proj_matrix * world_pos;

    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(clip_position, 1.0);
    return out;
}

fn world_to_clip_mat(win_w: f32, win_h: f32) -> mat3x3<f32> {
    let is_wider = win_w > win_h;

    if is_wider {
        return mat3x3<f32>(
            win_h/win_w, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        );
    }
    return mat3x3<f32>(
        1.0, 0.0, 0.0,
        0.0, win_w/win_h, 0.0,
        0.0, 0.0, 1.0,
    );
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color_offset = vec3<f32>(
        0.1*sin(my_uniform.elapsed/200.),
        0.1*cos(my_uniform.elapsed/300.),
        0.1*sin(my_uniform.elapsed/420.),
    );
    let offset_color_raw = in.color + color_offset;
    let low = vec3<f32>(0., 0., 0.);
    let high = vec3<f32>(1., 1., 1.);
    return vec4<f32>(smoothstep(low, high, offset_color_raw), 1.0);
}
