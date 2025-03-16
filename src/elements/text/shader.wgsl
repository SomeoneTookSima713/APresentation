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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
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

    out.clip_position = camera_uniform * vec4<f32>(vert_pos + pos_offset, instance.pos.z, 1.0);

    out.color = instance.color;

    out.tex_coords = vertex.tex_coords;

    return out;
}

struct GlyphCurve {
    start: vec2<f32>,
    control: vec2<f32>,
    end: vec2<f32>
}

struct GlyphLine {
    start: vec2<f32>,
    end: vec2<f32>
}

struct GlyphIndex {
    curve_start: u32,
    curve_len: u32,
    line_start: u32,
    line_len: u32
}

@group(0) @binding(0)
var<storage, read> font_curves: array<GlyphCurve>;
@group(0) @binding(1)
var<storage, read> font_lines: array<GlyphLine>;
@group(0) @binding(2)
var<storage, read> glyph_slice_inds: array<array<GlyphIndex, 8>>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0);
}