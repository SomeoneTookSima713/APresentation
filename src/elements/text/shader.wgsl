const PI = 3.141592653589793238462643383279; // Pi
const PI2 = 1.57079632679; // Pi over 2

struct PushConstant {
    window_size: vec2<f32>
}
var<push_constant> constants: PushConstant;

struct VertexInput {
    @location(0) pos: vec2<f32>
}

struct Instance {
    @location(2) transform_mat_1: vec4<f32>,
    @location(3) transform_mat_2: vec4<f32>,
    @location(4) transform_mat_3: vec4<f32>,
    @location(5) transform_mat_4: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) glyph_ind: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) rel_pos: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> camera_uniform: mat4x4<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: Instance
) -> VertexOutput {
    var out: VertexOutput;

    var transform_mat = mat4x4(
        instance.transform_mat_1,
        instance.transform_mat_2,
        instance.transform_mat_3,
        instance.transform_mat_4
    );

    var vert_pos = (transform_mat * vec4(vertex.pos, 0.0, 1.0)).xy;
    var pos_offset =  (transform_mat * vec4(0.0, 0.0, 0.0, 1.0)).xyz;

    out.clip_position = camera_uniform * vec4<f32>(vert_pos + pos_offset.xy, pos_offset.z, 1.0);

    out.rel_pos = vertex.pos;

    out.color = instance.color;

    return out;
}

struct GlyphCurve {
    start: vec2<f32>,
    control: vec2<f32>,
    end: vec2<f32>
}

struct CurveTransform {
    angle: f32,
    apex: vec2<f32>,
    sin: f32,
    cos: f32,
    bounds: vec4<f32>
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
@group(0) @binding(3)
var<storage, read_write> font_curve_transforms: array<CurveTransform>;

@compute
@workgroup_size(16, 16, 1)
fn compute_curve_angles(@builtin(global_invocation_id) gid: vec3<u32>) {
    var ind = gid.x + gid.y*16u;
    if ind >= arrayLength(&font_curves) { return; }
    var curve = font_curves[ind];

    var end = vec2(curve.end.x - curve.start.x, curve.end.y - curve.start.y);
    var control = vec2(curve.control.x - curve.start.x, curve.control.y - curve.start.y);

    var alpha = -atan(end.y/end.x);
    var beta = atan(
        ((control.x * cos(alpha) - control.y * sin(alpha)) - 0.5 * (end.x * cos(alpha) - end.y * sin(alpha))) /
        (control.x * sin(alpha) + control.y * cos(alpha))
    );
    var theta = -alpha-beta;

    var t_max: f32;
    if abs((theta % PI)-PI2) < 0.00001 {
        t_max = control.x/(2.0*control.x-end.x);
    } else {
        t_max = ( control.x * tan(theta) - control.y ) / (2.0*control.x*tan(theta) - 2.0*control.y - end.x*tan(theta) + end.y);
    }

    var apex = vec2(
        2.0*(1.0-t_max)*t_max*(control.x*cos(-theta) - control.y*sin(-theta)) + t_max*t_max*(end.x*cos(-theta) - end.y*sin(-theta)),
        2.0*(1.0-t_max)*t_max*(control.x*sin(-theta) + control.y*cos(-theta)) + t_max*t_max*(end.x*sin(-theta) + end.y*cos(-theta)),
    );

    font_curve_transforms[ind] = CurveTransform(
        theta,
        apex,
        sin(theta),
        cos(theta),
        vec4(min(min(curve.start, curve.end), curve.control), max(max(curve.start, curve.end), curve.control))
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0);
}