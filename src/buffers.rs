//! Graphics Buffer stuff I guess.

/// A singular vertex.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32;3],
    color: [f32;4],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x2,
            }
        ]
    };
}

/// Temporary
pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5,0.0], color: [1.0, 0.0, 0.0, 1.0], tex_coords: [0.0,0.0] },
    Vertex { position: [ 0.5, 0.5,0.0], color: [0.25,0.75,0.0, 1.0], tex_coords: [1.0,0.0] },
    Vertex { position: [ 0.5,-0.5,0.0], color: [0.0,0.25,0.75, 1.0], tex_coords: [1.0,1.0] },
    Vertex { position: [-0.5,-0.5,0.0], color: [0.0, 0.0,0.25,0.75], tex_coords: [0.0,1.0] },
];

/// Temporary
pub const INDICES: &[u16] = &[
    0,1,2,
    2,3,0
];