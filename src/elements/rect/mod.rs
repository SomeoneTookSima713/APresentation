use std::sync::OnceLock;

use crate::PUSH_CONSTANT_MANAGER;
use crate::presentation::{ asset, element, parser };

use asset::AssetManager;
use element::{ Element, ElementRenderer };
use element::property::{ Property, PropertyCompatible };
use element::property::base::{ BaseProperties, BasePropertiesProvider };
use parser::data::Value;

pub mod assets;
pub use assets::*;

pub struct Rect {
    base_properties: BaseProperties,
    size: Property<(f64, f64)>,
    source: Property<RectSource>,
    corner_rounding: Property<CornerRounding>
}

#[derive(Clone, Copy)]
pub enum SamplerType {
    Linear,
    Nearest
}

impl PropertyCompatible for SamplerType {
    type InnerRepresentation = Self;

    fn from_value(val: Value, _engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::EnumVariant(variant, None) = val {
            match variant.as_str() {
                "Linear" => Some(Self::Linear),
                "Nearest" => Some(Self::Nearest),
                _ => None
            }
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, _engine: &rhai::Engine, _scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(*base)
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("Linear", || Ok(Self::Linear));
        module.set_native_fn("Nearest", || Ok(Self::Nearest));
        module.set_custom_type::<Self>("SamplerType");
        Some(("SamplerType".to_string(), module))
    }
}

#[derive(Clone)]
pub enum RectSource {
    Color(f64, f64, f64, f64),
    Image(String, SamplerType)
}

pub enum UncomputedRectSource {
    Color([Property<f64>; 4]),
    Image(Property<String>, Property<SamplerType>)
}

impl PropertyCompatible for RectSource {
    type InnerRepresentation = UncomputedRectSource;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val && let Some(v) = v.map(Box::into_inner) {
            Some(match variant.as_str() {
                "Color" => {
                    let col = <(f64, f64, f64, f64) as PropertyCompatible>::from_value(v, engine)?;
                    UncomputedRectSource::Color([col.0, col.1, col.2, col.3])
                },
                "Image" => {
                    let p = <(String, SamplerType) as PropertyCompatible>::from_value(v, engine)?;
                    UncomputedRectSource::Image(p.0, p.1)
                },
                _ => None?
            })
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(match base {
            UncomputedRectSource::Color([r, g, b, a]) => Self::Color(
                r.evaluate(scope, engine)?,
                g.evaluate(scope, engine)?,
                b.evaluate(scope, engine)?,
                a.evaluate(scope, engine)?
            ),
            UncomputedRectSource::Image(src, sampler) => Self::Image(src.evaluate(scope, engine)?, sampler.evaluate(scope, engine)?)
        })
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("Color", |r: f64, g: f64, b: f64, a: f64| Ok(Self::Color(r, g, b, a)));
        module.set_native_fn("Image", |src: rhai::ImmutableString, sampler: SamplerType| Ok(Self::Image(src.into_owned(), sampler)));
        Some(("RectSource".to_string(), module))
    }
}

#[derive(Clone, Copy)]
pub struct CornerRounding {
    top_left: f64,
    top_right: f64,
    bottom_left: f64,
    bottom_right: f64,
}

pub struct UncomputedCornerRounding([Property<f64>; 4]);

impl PropertyCompatible for CornerRounding {
    type InnerRepresentation = UncomputedCornerRounding;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::Map(map) = val {
            Some(UncomputedCornerRounding([
                Property::from_value(map.get("top_left")?.clone(), engine)?,
                Property::from_value(map.get("top_right")?.clone(), engine)?,
                Property::from_value(map.get("bottom_left")?.clone(), engine)?,
                Property::from_value(map.get("bottom_right")?.clone(), engine)?,
            ]))
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(Self {
            top_left: base.0[0].evaluate(scope, engine)?,
            top_right: base.0[1].evaluate(scope, engine)?,
            bottom_left: base.0[2].evaluate(scope, engine)?,
            bottom_right: base.0[3].evaluate(scope, engine)?
        })
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("new", |tl: f64, tr: f64, bl: f64, br: f64| Ok(Self { top_left: tl, top_right: tr, bottom_left: bl, bottom_right: br }));
        Some(("CornerRounding".to_string(), module))
    }
}

impl BasePropertiesProvider for Rect {
    fn get_base_properties(&self) -> &BaseProperties { &self.base_properties }
}

impl Element for Rect {
    type Renderer = RectRenderer;

    fn from_structure(structure: crate::presentation::parser::data::ParsedStructure, engine: &rhai::Engine) -> anyhow::Result<Self>
    where Self: Sized {
        Ok(Self {
            base_properties: structure.try_get_base_properties(engine)?,
            size: structure.try_get_property("size", engine)?,
            source: structure.try_get_property("source", engine)?,
            corner_rounding: structure.try_get_property("corner_rounding", engine)?
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    tex_coords: [f32; 2]
}

impl Vertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2
        ]
    };

    pub const RECT_VERTS: &[Self] = &[
        Self::new([-0.5, -0.5], [0.0, 0.0]),
        Self::new([ 0.5, -0.5], [1.0, 0.0]),
        Self::new([-0.5,  0.5], [0.0, 1.0]),
        Self::new([ 0.5,  0.5], [1.0, 1.0]),
    ];

    pub const fn new(pos: [f32; 2], tex_coords: [f32; 2]) -> Self {
        Self { pos, tex_coords }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Instance {
    pos: [f32; 3],
    size: [f32; 2],
    color: [f32; 4],
    texture_ind: u32,
    rounding: [f32; 4]
}

impl Instance {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            2 => Float32x3,
            3 => Float32x2,
            4 => Float32x4,
            5 => Uint32,
            6 => Float32x4,
        ]
    };

    pub const fn new(
        pos: [f32; 3],
        size: [f32; 2],
        color: [f32; 4],
        texture_ind: u32,
        rounding: [f32; 4]
    ) -> Self {
        Self { pos, size, color, texture_ind, rounding }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstant {
    window_res: [u32; 2]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    matrix: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn set(&mut self, new: nalgebra::Matrix4<f32>) {
        self.matrix = new.data.0;
    }
}

/// Concept for the render model:
/// 
/// We have one big bind group containing all the textures needed for the
/// current slide, as well as a dummy white texture for rectangles not using an
/// image. We are then able to draw all [`Rect`] elements in one draw call by
/// instancing them.
pub struct RectRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    push_constant: PushConstant,
    push_constant_start: u32,
    /// A white dummy texture for rectangles not using an image.
    dummy_texture: (wgpu::Texture, wgpu::TextureView),
    /// The texture samplers used in the bind group: one using linear filtering
    /// and one using nearest-neighbor filtering.
    texture_samplers: (wgpu::Sampler, wgpu::Sampler),
    image_bind_group: wgpu::BindGroup,
    current_images: Vec<String>,
    needed_images: Vec<(String, wgpu::TextureView)>,
    /// Indicates wether or not a reconstruction of the image bind group is
    /// necessary before dispatching the next render call.
    ibg_reconstruct_necessary: bool,
    current_rect_instances: Vec<Instance>
}

static CAMERA_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

static IMAGE_ARR_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();
const IMAGE_ARR_MAX_ITEMS: std::num::NonZero<u32> = std::num::NonZero::<u32>::new(128).unwrap();

const INSTANCE_BUFFER_BASE_SIZE: wgpu::BufferAddress = 16; // wgpu::BufferAddress currently coerces to u64 (wgpu version 24.0.1)

const DUMMY_TEXTURE_DIMENSIONS: (u32, u32) = (8,8);
const DUMMY_TEXTURE_DATA: &[u8] = &[255u8; 4*(DUMMY_TEXTURE_DIMENSIONS.0 as usize)*(DUMMY_TEXTURE_DIMENSIONS.1 as usize)];

impl ElementRenderer for RectRenderer {
    type Element = Rect;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized {
        let _span = tracing::info_span!("elements::rect::RectRenderer::init()");

        use wgpu::util::{ BufferInitDescriptor, DeviceExt };

        let mut set_succeeded = CAMERA_BIND_GROUP_LAYOUT.set(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rect Caera Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None
                }
            ]
        })).is_err();
        set_succeeded = set_succeeded || IMAGE_ARR_BIND_GROUP_LAYOUT.set(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rect Image Array Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: Some(IMAGE_ARR_MAX_ITEMS)
                },
                wgpu::BindGroupLayoutEntry { // Linear sampler
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
                wgpu::BindGroupLayoutEntry { // Nearest-neighbor sampler
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
            ]
        })).is_err();
        if set_succeeded {
            // This theoretically isn't problematic (unless you actually render using a different
            // adapter), so we only print an error and continue.
            tracing::error!("Multiple RectRenderer instances were initialised!");
        }

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let push_constant_range = PUSH_CONSTANT_MANAGER.get_range(8);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                IMAGE_ARR_BIND_GROUP_LAYOUT.get().expect("Unreachable"),
                CAMERA_BIND_GROUP_LAYOUT.get().expect("Unreachable")
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: push_constant_range.clone()
            }]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
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

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rect Renderer Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform {
                matrix: nalgebra::Orthographic3::new(
                    0.0,
                    surface_config.width as f32,
                    surface_config.height as f32,
                    0.0,
                    1000.0,
                    -1.0,
                ).as_matrix().data.0
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rect Renderer Camera Uniform Bind Group"),
            layout: &CAMERA_BIND_GROUP_LAYOUT.get().expect("Unreachable"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(camera_buffer.as_entire_buffer_binding())
                }
            ]
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rect Renderer Vertex Buffer"),
            contents: bytemuck::cast_slice(Vertex::RECT_VERTS),
            usage: wgpu::BufferUsages::VERTEX
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Renderer Instance Buffer"),
            size: std::mem::size_of::<Instance>() as wgpu::BufferAddress * INSTANCE_BUFFER_BASE_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let push_constant = PushConstant { window_res: [surface_config.width, surface_config.height] };

        let size = wgpu::Extent3d { width: DUMMY_TEXTURE_DIMENSIONS.0, height: DUMMY_TEXTURE_DIMENSIONS.1, depth_or_array_layers: 1 };
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]
        });
        let dummy_texture_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &dummy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            bytemuck::cast_slice(DUMMY_TEXTURE_DATA),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height)
            },
            size
        );

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            .. Default::default()
        });
        let nearest_neighbor_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            .. Default::default()
        });

        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rect Renderer Image Array Bind Group"),
            layout: IMAGE_ARR_BIND_GROUP_LAYOUT.get().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&[&dummy_texture_view])
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&nearest_neighbor_sampler)
                }
            ]
        });

        Self {
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            push_constant,
            push_constant_start: push_constant_range.start,
            dummy_texture: (dummy_texture, dummy_texture_view),
            texture_samplers: (linear_sampler, nearest_neighbor_sampler),
            image_bind_group,
            ibg_reconstruct_necessary: false,
            current_images: Vec::new(),
            needed_images: Vec::new(),
            current_rect_instances: Vec::new()
        }
    }

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        self.push_constant.window_res = [surface_config.width, surface_config.height];

        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[CameraUniform {
            matrix: nalgebra::Orthographic3::new(
                0.0,
                surface_config.width as f32,
                surface_config.height as f32,
                0.0,
                1000.0,
                -1.0,
            ).as_matrix().data.0
        }]));
    }

    fn submit_to_render(
            &mut self,
            element: &Rect,
            eval_engine: &rhai::Engine,
            mut eval_scope: rhai::Scope<'static>,
            asset_manager: &AssetManager,
            _render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()> {
        fn eval_failed(prop: &str) -> anyhow::Error {
            anyhow::anyhow!("Evaluation of property '{prop}' failed!")
        }

        let pos = element.base_properties.position.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("position"))?;
        let z = element.base_properties.z_index.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("z_index"))?;
        let size = element.size.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("size"))?;
        let source = element.source.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("source"))?;
        let rounding = {
            let CornerRounding { top_left, top_right, bottom_left, bottom_right } = element.corner_rounding.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("source"))?;
            [top_left as f32, top_right as f32, bottom_left as f32, bottom_right as f32]
        };

        let anchor = element.base_properties.anchor.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("anchor"))?;
        let align = element.base_properties.alignment.evaluate(&mut eval_scope, eval_engine).ok_or(eval_failed("alignment"))?;

        let final_pos = (
            self.push_constant.window_res[0] as f64 * anchor.into_fracts().0 + pos.0 - size.0 * (align.into_fracts().0 - 0.5),
            self.push_constant.window_res[1] as f64 * anchor.into_fracts().1 + pos.1 - size.1 * (align.into_fracts().1 - 0.5),
        );

        match source {
            RectSource::Color(r, g, b, a) => {
                self.current_rect_instances.push(Instance::new(
                    [final_pos.0 as f32, final_pos.1 as f32, z as f32],
                    [size.0 as f32, size.1 as f32],
                    [r as f32, g as f32, b as f32, a as f32],
                    0,
                    rounding,
                ));
            },
            RectSource::Image(id, sampler) => {
                let view = asset_manager.get_asset::<Image, _>(&id)
                    .ok_or(anyhow::anyhow!("Couldn't load image with ID '{id}'!"))?
                    .view.clone();
                let tex_ind;
                if let Some(i) = self.current_images.iter().enumerate().find(|i| i.1 == &id).map(|v| v.0) {
                    tex_ind = i+1;
                } else {
                    self.ibg_reconstruct_necessary = true;
                    tex_ind = self.needed_images.len() + 1;
                }
                self.needed_images.push((id, view));

                self.current_rect_instances.push(Instance::new(
                    [final_pos.0 as f32, final_pos.1 as f32, z as f32],
                    [size.0 as f32, size.1 as f32],
                    [1.0,1.0,1.0,1.0],
                    tex_ind as u32,
                    rounding,
                ));
            }
        }

        Ok(())
    }

    fn finish_render(&mut self, render_pass: &mut wgpu::RenderPass) {
        use wgpu::util::{ BufferInitDescriptor, DeviceExt };

        if self.ibg_reconstruct_necessary {
            let mut views = Vec::with_capacity(self.needed_images.len() + 1);
            views.push(&self.dummy_texture.1);
            for img in self.needed_images.iter() {
                println!("{}", img.0);
                views.push(&img.1);
            }
            self.image_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Rect Renderer Image Array Bind Group"),
                layout: IMAGE_ARR_BIND_GROUP_LAYOUT.get().unwrap(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&views)
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.texture_samplers.0)
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.texture_samplers.1)
                    }
                ]
            });

            self.current_images = self.needed_images.iter().map(|(s, _)| s.clone()).collect();
        }

        let instance_data_bytes = bytemuck::cast_slice(&self.current_rect_instances);

        if self.instance_buffer.size() < instance_data_bytes.len() as wgpu::BufferAddress {
            self.instance_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Rect Renderer Instance Buffer"),
                contents: instance_data_bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            });
        }else {
            self.queue.write_buffer(&self.instance_buffer, 0, instance_data_bytes);
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.image_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..instance_data_bytes.len() as wgpu::BufferAddress));
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, self.push_constant_start, bytemuck::cast_slice(&[self.push_constant]));
        render_pass.draw(0..Vertex::RECT_VERTS.len() as u32, 0..self.current_rect_instances.len() as u32);

        self.current_rect_instances.clear();
        self.needed_images.clear();
        self.ibg_reconstruct_necessary = false;
    }
}