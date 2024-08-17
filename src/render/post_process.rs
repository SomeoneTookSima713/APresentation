use wgpu::{ RenderPipeline, Buffer, BufferUsages, Device, util::DeviceExt, TextureUsages, TextureView, CommandEncoder };
use winit::dpi::PhysicalSize;

use crate::texture::{ Texture, TextureSamplerSelection, TEXTURE_BIND_GROUP_LAYOUT };
use crate::shaders::post_process as shader;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostProcessVertex {
    position: [f32; 2],
    tex_coords: [f32; 2]
}

impl PostProcessVertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<PostProcessVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            }
        ]
    };

    pub(self) const fn new(position: [f32; 2], tex_coords: [f32; 2]) -> Self {
        Self { position, tex_coords }
    }
}

const fn vert(pos: [f32; 2], tex: [f32; 2]) -> PostProcessVertex {
    PostProcessVertex::new(pos, tex)
}

pub const SCREEN_QUAD_VERTICES: [PostProcessVertex; 4] = [
    vert([-1.0,  1.0], [0.0, 0.0]),
    vert([ 1.0,  1.0], [1.0, 0.0]),
    vert([ 1.0, -1.0], [1.0, 1.0]),
    vert([-1.0, -1.0], [0.0, 1.0]),
];

pub const SCREEN_QUAD_INDICES: [u16; 6] = [
    0,1,2,
    2,3,0
];

pub struct PostProcessPipeline {
    pipeline: RenderPipeline,
    vertex_buf: Buffer,
    index_buf: Buffer,
    intermediate_texture: Texture
}

impl PostProcessPipeline {
    pub fn new(device: &Device, window_size: PhysicalSize<u32>, config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self> {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post-Processing Pipeline"),
            bind_group_layouts: &[
                TEXTURE_BIND_GROUP_LAYOUT.get().ok_or(anyhow::anyhow!("texture::init() wasn't called!"))?
            ],
            push_constant_ranges: &[]
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post-Process shader"),
            source: wgpu::ShaderSource::Wgsl(shader::SOURCE.into())
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post-Process Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    PostProcessVertex::DESC
                ] },
            fragment: Some(wgpu::FragmentState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // We don't need backface-culling in a post-process pass.
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
            cache: None
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&SCREEN_QUAD_VERTICES),
            usage: BufferUsages::VERTEX
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&SCREEN_QUAD_INDICES),
            usage: BufferUsages::INDEX
        });

        let intermediate_texture = Texture::new(device, (window_size.width * 2, window_size.height * 2), TextureSamplerSelection::Linear, Some(TextureUsages::RENDER_ATTACHMENT))?;

        Ok(Self {
            pipeline,
            vertex_buf,
            index_buf,
            intermediate_texture
        })
    }

    pub fn update_surface_size(&mut self, device: &Device, new_size: PhysicalSize<u32>) -> anyhow::Result<()> {
        self.intermediate_texture = Texture::new(device, (new_size.width * 2, new_size.height * 2), TextureSamplerSelection::Linear, Some(TextureUsages::RENDER_ATTACHMENT))?;
        Ok(())
    }

    pub fn get_intermediate_surface(&self) -> &TextureView {
        self.intermediate_texture.view()
    }

    pub fn execute(&mut self, encoder: &mut CommandEncoder, final_surface: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post-Process Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: final_surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.intermediate_texture.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..SCREEN_QUAD_INDICES.len() as u32, 0, 0..1);
    }
}