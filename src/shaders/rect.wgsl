// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) corner_rounding: vec4<f32>,
    @location(3) centered_position: vec2<f32>,
    @location(4) size: vec2<f32>,
    @location(5) tex_start: vec2<f32>
};

struct Instance {
    @location(1) position: vec3<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) uv: vec4<f32>,
    @location(5) corner_rounding: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: Instance,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var index = i32(vertex_index);
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(
        model.position.x*instance.size.x + instance.position.x,
        model.position.y*instance.size.y + instance.position.y,
        1.0 + instance.position.z, 1.0);
    // var tex_coords: vec2<f32> = vec2(
    //     instance.uv.x + instance.uv.z*f32((1-(index+1)%2+index>>1)%2),
    //     instance.uv.y + instance.uv.w*f32(1-(index+1)>>1)
    // );
    var tex_coords: vec2<f32>;
    switch vertex_index {
        case 0u: {
            tex_coords = vec2(instance.uv.x              , instance.uv.y+instance.uv.w);
        }
        case 1u: {
            tex_coords = vec2(instance.uv.x+instance.uv.z, instance.uv.y+instance.uv.w);
        }
        case 2u: {
            tex_coords = vec2(instance.uv.x+instance.uv.z, instance.uv.y);
        }
        case 3u: {
            tex_coords = vec2(instance.uv.x              , instance.uv.y);
        }
        default: {
            tex_coords = vec2(0.0,0.0);
        }
    }
    out.color = instance.color;
    out.tex_coords = tex_coords;
    out.corner_rounding = instance.corner_rounding;
    out.centered_position = model.position * instance.size;
    out.size = instance.size;
    out.tex_start = instance.uv.xy;
    return out;
}

@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var samp: sampler;

const GAUSSIAN_STUFF = array(1.0, 0.5, 0.25);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ind: u32 = u32(round(in.tex_coords.x - in.tex_start.x))+u32(2f*round(in.tex_coords.y - in.tex_start.y));

    if in.corner_rounding[ind] < 0.0001 {
        return in.color * textureSample(tex, samp, in.tex_coords);
    }
    var p: f32 = 2.0/(min(in.size.x, in.size.y)*pow(in.corner_rounding[ind], 2.0));
    var coverage: f32 = 0.0;
    var div: f32 = 0.0;
    for (var x = 0; x<=0; x+=1) {
        for (var y = 0; y<=0; y+=1) {
            var mult = pow(2.0,-sqrt(f32(x*x + y*y)));
            div += mult;
            coverage += floor(pow(abs(2.0 * (in.centered_position.x+f32(x)) / in.size.x), in.size.x*p) + pow(abs(2.0 * (in.centered_position.y+f32(y)) / in.size.y), in.size.y*p)) * mult;
        }
    }
    coverage /= div;
    // return vec4(coverage, coverage, coverage, coverage);
    return in.color * textureSample(tex, samp, in.tex_coords) * vec4(1.0, 1.0, 1.0, max(1.0-coverage, 0.0));
}