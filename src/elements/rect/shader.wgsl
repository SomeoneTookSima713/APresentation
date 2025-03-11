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
    @location(5) texture_ind_and_rounding_type: u32,
    @location(6) rounding: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_ind: u32,
    @location(2) rounding_type: u32,
    @location(3) rounding: vec4<f32>,
    @location(4) tex_coords: vec2<f32>
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

    out.clip_position = camera_uniform * vec4<f32>(vert_pos.x + pos_offset.x, vert_pos.y + pos_offset.y, instance.pos.z, 1.0);

    out.color = instance.color;
    out.texture_ind = instance.texture_ind_and_rounding_type & 0x7FFFFFFFu;
    out.rounding_type = (instance.texture_ind_and_rounding_type & 0x80000000u) >> 31u;
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: Corner rounding implementation

    return in.color * textureSample(texture_array[in.texture_ind], linear_sampler, in.tex_coords);
}