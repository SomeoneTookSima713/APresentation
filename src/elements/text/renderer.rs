use std::ops::Range;

use ab_glyph::GlyphId;

use hashbrown::HashMap;

use wgpu::util::{ BufferInitDescriptor, DeviceExt };

use crate::PUSH_CONSTANT_MANAGER;
use crate::util::renderer_camera;

use super::*;

use assets::{ Font, GLYPH_DATA_BIND_GROUP_LAYOUT, GPUGlyphData };

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2]
}

impl Vertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2
        ]
    };

    pub const VERTS: &[Self] = &[
        Vertex { pos: [-0.5, -0.5] },
        Vertex { pos: [ 0.5, -0.5] },
        Vertex { pos: [-0.5,  0.5] },
        Vertex { pos: [ 0.5,  0.5] },
    ];
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    transform: [[f32;4];4],
    color: [f32; 4],
    glyph_id: u32
}

impl Instance {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Uint32,
        ]
    };

    pub fn new(rect: [f32; 4], rotation: f32, color: [f32; 4], glyph_id: GlyphId) -> Self {
        type Mat = nalgebra::Matrix4::<f32>;
        let translation = Mat::new_translation(&nalgebra::Vector3::new(rect[0], rect[1], 0.0));
        let scale = Mat::new_nonuniform_scaling(&nalgebra::Vector3::new(rect[2], rect[3], 0.0));
        let rot = Mat::from_euler_angles(0.0, 0.0, rotation);
        let transform = rot * scale * translation;

        Self { transform: transform.data.0, color, glyph_id: glyph_id.0 as u32 }
    }
}

pub struct TextRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    push_constant_range: Range<u32>,
    push_constant_data: [u8;8],
    camera_uniform: renderer_camera::CameraUniform,
    vertex_buffer: wgpu::Buffer,
    elements_by_font: HashMap<String, SubmittedElements>,
}

struct SubmittedElements {
    font_data: GPUGlyphData,
    elements: Vec<ProcessedText>,
    instance_buffer: Option<wgpu::Buffer>
}

impl SubmittedElements {
    pub fn new(font_data: GPUGlyphData) -> Self {
        Self {
            font_data,
            elements: Vec::new(),
            instance_buffer: None
        }
    }
}

#[derive(Clone, Copy)]
struct GlyphInfo {
    id: GlyphId,
    rect: [f32; 4],
    color: [f32; 4]
}

#[derive(Clone)]
struct ProcessedText {
    base_pos: [f32; 2],
    glyphs: Vec<GlyphInfo>
}

impl ElementRenderer for TextRenderer {
    type Element = Text;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized {
        let push_constant_range = PUSH_CONSTANT_MANAGER.get_range(8);
        let push_constant_data = unsafe { std::mem::transmute([surface_config.width as f32, surface_config.height as f32]) };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Renderer Pipeline Layout"),
            bind_group_layouts: &[
                GLYPH_DATA_BIND_GROUP_LAYOUT.get().expect("All assets should already be initialized!"),
                renderer_camera::get_cam_bind_group_layout(&device)
            ],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    range: push_constant_range.clone()
                }
            ]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Renderer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::DESC, Instance::DESC]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL
                    })
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None
        });

        let camera_uniform = renderer_camera::CameraUniform::new(0.0, 0.0, surface_config.width as f32, surface_config.height as f32);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Text Renderer Vertex Buffer"),
            contents: bytemuck::cast_slice(Vertex::VERTS),
            usage: wgpu::BufferUsages::VERTEX
        });

        Self {
            device,
            queue,
            pipeline,
            push_constant_range,
            push_constant_data,
            camera_uniform,
            vertex_buffer,
            elements_by_font: HashMap::new()
        }
    }

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        self.push_constant_data = unsafe { std::mem::transmute([surface_config.width as f32, surface_config.height as f32]) };

        self.camera_uniform.update_cam_rect(0.0, 0.0, surface_config.width as f32, surface_config.height as f32);
    }

    fn submit_to_render(
        &mut self,
        element: &Self::Element,
        eval_engine: &rhai::Engine,
        mut eval_scope: rhai::Scope<'static>,
        asset_manager: &AssetManager,
        render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()> {
        use ab_glyph::Font as AbGlyphFont;

        fn eval_failed(prop: &str) -> anyhow::Error {
            anyhow::anyhow!("Evaluation of property '{prop}' failed!")
        }

        let text = element.text.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("text"))?.text;
        let font_family = element.font.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("font"))?;
        let font = asset_manager.get_asset::<Font, _>(&font_family).ok_or(anyhow::anyhow!("Font family not found!"))?;

        let font_info = font.font_data.values().next().unwrap();
        let elems = self.elements_by_font.entry(font_family).or_insert(SubmittedElements::new(font_info.1.clone()));
        let id = font_info.0.glyph_id('e');
        elems.elements.push(ProcessedText { base_pos: [0.0,0.0], glyphs: vec![GlyphInfo { id, rect: [0.0,0.0,100.0,100.0], color: [1.0,1.0,1.0,1.0] }] });
        Ok(())
    }

    fn finish_render(&mut self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX_FRAGMENT, self.push_constant_range.start, &self.push_constant_data);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(1, self.camera_uniform.update_and_get_bind_group(&self.device, &self.queue), &[]);

        for elems in self.elements_by_font.values_mut() {
            let cap = elems.elements.iter().map(|t| t.glyphs.len()).sum();
            let mut instances = Vec::with_capacity(cap);
            for elem in elems.elements.iter() {
                for glyph in elem.glyphs.iter() {
                    instances.push(Instance::new(
                        glyph.rect, // TODO: Add base position of ProcessedText struct
                        0.0,
                        glyph.color,
                        glyph.id
                    ));
                }
            }

            if let Some(ref b) = elems.instance_buffer && b.size() >= (instances.len() * std::mem::size_of::<Instance>()) as u64 {
                self.queue.write_buffer(b, 0, bytemuck::cast_slice(&instances));
            } else {
                elems.instance_buffer = Some(self.device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
                }));
            }

            render_pass.set_bind_group(0, elems.font_data.get_bind_group().unwrap(), &[]);
            render_pass.set_vertex_buffer(1, elems.instance_buffer.as_ref().unwrap().slice(..(instances.len() * std::mem::size_of::<Instance>()) as u64));
            render_pass.draw(0..4, 0..instances.len() as u32);
        }
    }
}