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

    var vert_pos = vertex.pos.xy * instance.size;
    var pos_offset =  instance.pos.xy;

    out.vert_pos_local = vert_pos*2.0;
    out.size_local = instance.size;

    out.clip_position = camera_uniform * vec4<f32>(vert_pos + pos_offset, instance.pos.z, 1.0);

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

fn sample_rounding(pos: vec2<f32>, size: vec2<f32>, rounding: vec4<f32>) -> bool {
    var r: f32;
    if pos.x<0.0 && pos.y<0.0 {
        r = rounding.x;
    } else if pos.x>0.0 && pos.y<0.0 {
        r = rounding.y;
    } else if pos.x<0.0 && pos.y>0.0 {
        r = rounding.z;
    } else if pos.x>0.0 && pos.y>0.0 {
        r = rounding.w;
    }
    var t = min(size.x, size.y)*r;
    if r == 0.0 {
        return true;
    } else {
        var vx = max((abs(pos.x)-size.x)/t+1.0,0.0);
        var vy = max((abs(pos.y)-size.y)/t+1.0,0.0);
        return vx*vx + vy*vy <= 1.0;
    }
}

const AA_SAMPLES_PER_DIM: u32 = 4u;
const AA_WIDTH: f32 = 2.0;
const AA_SAMPLE_OFFSET: f32 = -0.5*AA_WIDTH;
const AA_SAMPLE_MOVE: f32 = AA_WIDTH/f32(AA_SAMPLES_PER_DIM);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var half_pixel = vec2(0.5);

    var alpha = 0.0;
    for (var i = 0u; i<AA_SAMPLES_PER_DIM*AA_SAMPLES_PER_DIM; i++) {
        var offset = vec2(
            f32(i%AA_SAMPLES_PER_DIM)*AA_SAMPLE_MOVE+AA_SAMPLE_OFFSET,
            f32(i/AA_SAMPLES_PER_DIM)*AA_SAMPLE_MOVE+AA_SAMPLE_OFFSET
        );
        alpha += f32(sample_rounding(in.vert_pos_local + offset, in.size_local, in.rounding));
    }
    alpha /= f32(AA_SAMPLES_PER_DIM*AA_SAMPLES_PER_DIM);

    if in.sampler_type >= 1u {
        return vec4(1.0, 1.0, 1.0, alpha) * in.color * textureSample(texture_array[in.texture_ind], linear_sampler, in.tex_coords);
    } else {
        return vec4(1.0, 1.0, 1.0, alpha) * in.color * textureSample(texture_array[in.texture_ind], nearest_sampler, in.tex_coords);
    }
}