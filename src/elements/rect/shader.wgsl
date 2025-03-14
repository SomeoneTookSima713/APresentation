struct PushConstant {
    window_size: vec2<f32>
}
var<push_constant> constants: PushConstant;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>
}

struct Instance {
    @location(2) pos: vec3<f32>,
    @location(3) size: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) texture_ind_and_sampler_type: u32,
    @location(6) rounding: vec4<f32>,
    @location(7) rot_mat_1: vec4<f32>,
    @location(8) rot_mat_2: vec4<f32>,
    @location(9) rot_mat_3: vec4<f32>,
    @location(10) rot_mat_4: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_ind: u32,
    @location(2) rounding: vec4<f32>,
    @location(3) tex_coords: vec2<f32>,
    @location(4) sampler_type: u32,
    @location(5) vert_pos_local: vec2<f32>,
    @location(6) size_local: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> camera_uniform: mat4x4<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: Instance
) -> VertexOutput {
    var out: VertexOutput;

    var rot_mat = mat4x4(instance.rot_mat_1, instance.rot_mat_2, instance.rot_mat_3, instance.rot_mat_4);

    var vert_pos = vertex.pos.xy * instance.size;
    var pos_offset =  instance.pos.xy;

    out.vert_pos_local = vert_pos*2.0;
    out.size_local = abs(instance.size);

    out.clip_position = camera_uniform * vec4<f32>((rot_mat * vec4(vert_pos, 0.0, 1.0)).xy + pos_offset, instance.pos.z, 1.0);

    out.color = instance.color;
    out.texture_ind = instance.texture_ind_and_sampler_type & 0x7FFFFFFFu;
    out.sampler_type = instance.texture_ind_and_sampler_type >> 31u;
    out.rounding = instance.rounding;

    out.tex_coords = vertex.tex_coords;

    return out;
}

@group(0) @binding(0)
var texture_array: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var linear_sampler: sampler;
@group(0) @binding(2)
var nearest_sampler: sampler;

fn sample_rounding(pos: vec2<f32>, size: vec2<f32>, r: f32) -> bool {
    var t = min(size.x, size.y)*r;
    var vx = max((abs(pos.x)-size.x)/t+1.0,0.0);
    var vy = max((abs(pos.y)-size.y)/t+1.0,0.0);
    return vx*vx + vy*vy <= 1.0;
}

const AA_SAMPLES_PER_DIM: u32 = 4u;
const AA_WIDTH: f32 = 2.0;
const AA_SAMPLE_OFFSET: f32 = -0.5*AA_WIDTH;
const AA_SAMPLE_MOVE: f32 = AA_WIDTH/f32(AA_SAMPLES_PER_DIM);
const AA_ALPHA_DIV: f32 = 1.0/f32(AA_SAMPLES_PER_DIM*AA_SAMPLES_PER_DIM);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var rind = clamp(u32(ceil(max(in.vert_pos_local.x,0.0))), 0u, 1u) + 2u*clamp(u32(ceil(max(in.vert_pos_local.y,0.0))), 0u, 1u);

    var alpha = 0.0;
    if in.rounding[rind] == 0.0 {
        alpha = 1.0;
    } else {
        var vert_pos_perc = in.vert_pos_local / in.size_local;
        if abs(vert_pos_perc.x) + abs(vert_pos_perc.y) < 1.0 {
            alpha = 1.0;
        } else {
            for (var i = 0u; i<AA_SAMPLES_PER_DIM*AA_SAMPLES_PER_DIM; i++) {
                var offset = vec2(
                    f32(i%AA_SAMPLES_PER_DIM)*AA_SAMPLE_MOVE+AA_SAMPLE_OFFSET,
                    f32(i/AA_SAMPLES_PER_DIM)*AA_SAMPLE_MOVE+AA_SAMPLE_OFFSET
                );
                alpha += f32(sample_rounding(in.vert_pos_local + offset, in.size_local, in.rounding[rind]));
            }
            alpha *= AA_ALPHA_DIV;
        }
    }
    
    if in.sampler_type >= 1u {
        return vec4(1.0, 1.0, 1.0, alpha) * in.color * textureSample(texture_array[in.texture_ind], linear_sampler, in.tex_coords);
    } else {
        return vec4(1.0, 1.0, 1.0, alpha) * in.color * textureSample(texture_array[in.texture_ind], nearest_sampler, in.tex_coords);
    }
}