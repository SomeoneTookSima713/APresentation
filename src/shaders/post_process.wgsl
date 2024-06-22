// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position.x, model.position.y, 1.0, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var res = vec2<f32>(textureDimensions(tex));

    var col = (
        textureSample(tex, samp, in.tex_coords + vec2(-0.5 / res.x, -0.5 / res.y))+
        textureSample(tex, samp, in.tex_coords + vec2( 0.5 / res.x, -0.5 / res.y))+
        textureSample(tex, samp, in.tex_coords + vec2(-0.5 / res.x,  0.5 / res.y))+
        textureSample(tex, samp, in.tex_coords + vec2( 0.5 / res.x,  0.5 / res.y))
    ) * vec4(0.25,0.25,0.25,0.25);
    return col;
}