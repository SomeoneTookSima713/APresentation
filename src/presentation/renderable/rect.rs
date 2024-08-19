use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use wgpu::{ RenderPipeline, RenderPipelineDescriptor, PipelineLayoutDescriptor };

use super::{ Renderable, RenderableRenderingManager, RenderableRenderingManagerObjectSafe, ParseableRenderable, BaseProperties, extended_structure };
use crate::presentation::property::{ PropertyStructure, TypedProperty, Property, PropertyCompatible, PropertyValue, common_properties as properties };
use crate::presentation::resource_managers;
use crate::shaders::rect as shader;
use crate::render::texture;
use crate::util::{ hashmap_ext::HashMapExt, math };

#[derive(Debug)]
pub struct Rectangle<'lua> {
    base_properties: BaseProperties,
    size: TypedProperty<'lua, (f64, f64)>,
    corner_rounding: TypedProperty<'lua, properties::ContextualList<'lua, f64, 4>>,
    texture: RectangleTextureType<'lua>,
}

#[derive(Debug)]
pub enum RectangleTextureType<'lua> {
    White,
    Custom(properties::Image<'lua>)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RectangleVertex {
    position: [f32;2],
}

impl RectangleVertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<RectangleVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }
        ]
    };

    pub const VERTICES: &'static [Self] = &[
        Self { position: [-0.5, -0.5] },
        Self { position: [ 0.5, -0.5] },
        Self { position: [ 0.5,  0.5] },
        Self { position: [-0.5,  0.5] },
    ];

    pub const INDICES: &'static [u16] = &[
        0, 1, 2,
        2, 3, 0
    ];
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RectangleInstance {
    pub position: [f32;3],
    pub size: [f32;2],
    pub color: [f32;4],
    pub uv: [f32; 4],
    pub corner_rounding: [f32;4],
}

impl RectangleInstance {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<RectangleInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 13]>() as wgpu::BufferAddress,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            }
        ]
    };
}

pub struct RectangleRenderer {
    pipeline: RenderPipeline,
    instance_textures: HashMap<String, Arc<texture::Texture>>,
    instances: HashMap<String, Vec<RectangleInstance>>,
    instance_buffers: HashMap<String, wgpu::Buffer>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl RenderableRenderingManager for RectangleRenderer {
    type Renderable = Rectangle<'static>;

    fn instantiate(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self>
    where Self: Sized{
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(shader::SOURCE.into())
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rectangle Render Pipeline Layout"),
            bind_group_layouts: &[
                texture::TEXTURE_BIND_GROUP_LAYOUT.get().unwrap(),
                crate::render::camera::CAMERA_BIND_GROUP_LAYOUT.get().unwrap()
            ],
            push_constant_ranges: &[]
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Rectangle Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    RectangleVertex::DESC,
                    RectangleInstance::DESC
                ] },
            fragment: Some(wgpu::FragmentState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // This is a 2D-application, so we don't need any backface-
                // culling, because there are no back or front faces in 2D.
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

        let texture = texture::Texture::from_color(device, queue, [255,255,255,255], (8,8), texture::TextureSamplerSelection::PixelPerfect, None)?;
        resource_managers::TEXTURE_MANAGER.insert("DEFAULT".to_string(), Arc::new(texture))?;

        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(RectangleVertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(RectangleVertex::INDICES),
            usage: wgpu::BufferUsages::INDEX
        });

        Ok(Self { pipeline, instance_textures: HashMap::new(), instances: HashMap::new(), instance_buffers: HashMap::new(), vertex_buffer, index_buffer })
    }

    fn render_instances(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, camera: &crate::render::camera::Camera) -> anyhow::Result<wgpu::RenderBundle> {
        use wgpu::util::DeviceExt;

        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb)],
            depth_stencil: None,
            sample_count: 1,
            multiview: None
        });

        encoder.set_pipeline(&self.pipeline);

        // Two loops are neccessary here to prevent mutable and immutable borrows from occuring at once.

        for (texture_path, instances) in self.instances.iter() {
            self.instance_buffers.insert(texture_path.to_string(), device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX
            }));
        }

        encoder.set_bind_group(1, &camera.get_bind_group(), &[]);

        for texture_path in self.instances.keys() {
            let texture = self.instance_textures.get(texture_path).unwrap();

            encoder.set_bind_group(0, &texture.bind_group(), &[]);
            encoder.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, self.instance_buffers.get(texture_path).unwrap().slice(..));
            encoder.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            encoder.draw_indexed(0..RectangleVertex::INDICES.len() as u32, 0, 0..self.instances.get(texture_path).unwrap().len() as u32);
        }

        for val in self.instances.values_mut() {
            val.clear();
        }

        Ok(encoder.finish(&wgpu::RenderBundleDescriptor { label: None }))
    }

    fn submit_instance<A: mlua::IntoLuaMulti<'static> + Clone>(&mut self, instance: &Self::Renderable, args: A) -> anyhow::Result<()> {
        let pos = instance.base_properties.position.get(args.clone())?;
        let z = instance.base_properties.z_index.get(args.clone())?;
        let size = instance.size.get(args.clone())?;
        let color = instance.base_properties.color.get(args.clone())?;
        let corner_rounding = instance.corner_rounding.get(args.clone())?;

        let image = match &instance.texture {
            RectangleTextureType::White => properties::Image::new_constant(Rc::new("DEFAULT".to_string()), (0.0,0.0,1.0,1.0), texture::TextureSamplerSelection::PixelPerfect),
            RectangleTextureType::Custom(ref p) => p.clone()
        };

        self.instances.create_if_none(image.source().to_string(), Vec::new);

        self.instance_textures.try_create_if_none(image.source().to_string(), || {
            resource_managers::TEXTURE_MANAGER.get(image.source()).ok_or(anyhow::anyhow!("Couldn't find texture for rectangle!"))
        })?;

        self.instances.get_mut(image.source()).unwrap().push(RectangleInstance {
            position: [pos.0 as f32, pos.1 as f32, *z as f32],
            size: [size.0 as f32, size.1 as f32],
            color: (*color).into(),
            uv: (*image.rect().get(args.clone())?).into(),
            corner_rounding: corner_rounding.get_list_copied().map(|f| f as f32),
        });
        Ok(())
    }
}

impl Renderable for Rectangle<'static> {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)] = &extended_structure::<BaseProperties, _>([
        ("size", <(f64, f64) as PropertyCompatible<'static>>::STRUCTURE),
        ("corner_rounding", PropertyStructure::Or(&[
            <[f64;4] as PropertyCompatible<'static>>::STRUCTURE,
            f64::STRUCTURE
        ])),
    ]);

    fn from_parseable(parseable: ParseableRenderable<'static>) -> anyhow::Result<Self>
    where Self: Sized {
        let size = TypedProperty::new(parseable.get("size").ok_or(anyhow::anyhow!("Property 'size' missing!"))?.clone()).map_err(|e| anyhow::anyhow!("Error parsing property 'size': {e}"))?;
        let corner_rounding = TypedProperty::new(parseable.get("corner_rounding").ok_or(anyhow::anyhow!("Property 'corner_rounding' missing!"))?.clone()).map_err(|e| anyhow::anyhow!("Error parsing property 'corner_rounding': {e}"))?;
        let texture = match parseable.get("texture") {
            Some(r) => RectangleTextureType::Custom(
                match &*r {
                    &Property::Constant(PropertyValue::String(ref s)) => {
                        properties::Image::new_constant(s.clone(), [0.0,0.0,1.0,1.0], texture::TextureSamplerSelection::Linear)
                    },
                    &Property::Constant(PropertyValue::Dict(ref d)) => {
                        let dict = d.borrow();
                        let source = match dict.get("source").ok_or(anyhow::anyhow!("Required Field 'source' of Property 'texture' is missing!"))? {
                            Property::Constant(PropertyValue::String(s)) => s.clone(),
                            Property::Constant(_) => anyhow::bail!("Field 'source' of Property 'texture' has wrong type! (Expected String)"),
                            Property::Eval(_) => anyhow::bail!("Field 'source' of Property 'texture' cannot be an expression!")
                        };
                        let rect = if let Some(r) = dict.get("rect") {
                            TypedProperty::new(r.clone())
                        } else {
                            TypedProperty::new(math::Rect::new_vals(0.0, 0.0, 1.0, 1.0).move_into())
                        }?;
                        let sampler = if let Some(s) = dict.get("sampler") {
                            match s {
                                Property::Constant(PropertyValue::String(ref str)) => {
                                    match str.replace("_", "").to_lowercase().as_str() {
                                        "linear" => texture::TextureSamplerSelection::Linear,
                                        "pixelperfect"|"nearestneighbor"|"nearestneighbour" => texture::TextureSamplerSelection::PixelPerfect,
                                        _ => anyhow::bail!("Field 'sampler' of Property 'texture' has invalid value! (Expected \"linear\" or any of \"pixel_perfect\", \"nearest_neighbor\", \"nearest_neighbour\" but got \"{str}\")")
                                    }
                                },
                                Property::Constant(_) => anyhow::bail!("Field 'sampler' of Property 'texture' has wrong type! (Expected String)"),
                                Property::Eval(_) => anyhow::bail!("Field 'sampler' of Property 'texture' cannot be an expression!")
                            }
                        } else {
                            texture::TextureSamplerSelection::Linear
                        };
                        properties::Image::new(source, rect, sampler)
                    }
                    &Property::Constant(_) => {
                        anyhow::bail!("Property 'texture' has invalid type! (Expected String or {{ source: String, rect?: [Number; 4], sampler?: \"linear\" | \"pixel_perfect\" }})")
                    },
                    &Property::Eval(_) => {
                        anyhow::bail!("Property 'texture' cannot be an expression!");
                    },
                }
            ),
            None => RectangleTextureType::White
        };
        Ok(Self {
            base_properties: BaseProperties::from_parseable(parseable)?,
            size,
            corner_rounding,
            texture
        })
    }

    fn to_parseable(&self) -> anyhow::Result<ParseableRenderable<'static>> {
        let mut properties = self.base_properties.to_parseable()?;
        properties.extend(&[
            ("size".to_string(), self.size.clone().move_into()),
            ("corner_rounding".to_string(), self.corner_rounding.clone().move_into()),
        ]).map_err(|_| anyhow::anyhow!("Impossible error: Couldn't immutably borrow unborrowed RefCell!"))?;
        Ok(properties)
    }

    fn begin_new_frame(&self) -> anyhow::Result<()> {
        self.base_properties.begin_new_frame()?;
        if !self.corner_rounding.delete_cache() {
            anyhow::bail!("Couldn't delete corner_rounding's cache in Rectangle!");
        }
        if !self.size.delete_cache() {
            anyhow::bail!("Couldn't delete size's cache in Rectangle!");
        }
        Ok(())
    }

    fn render<M>(&self, rendering_manager: &mut M, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()>
    where
        M: RenderableRenderingManagerObjectSafe + ?Sized
    {
        // \operatorname{abs}\left(\frac{x}{w}\right)^{p}+\operatorname{abs}\left(\frac{y}{h}\right)^{p}\le1

        rendering_manager.submit_instance(self as &dyn super::RenderableObjectSafe, args)?;
        Ok(())
    }
}