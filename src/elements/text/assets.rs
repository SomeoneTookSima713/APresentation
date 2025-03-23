use std::num::NonZeroU64;
use std::sync::{ Arc, OnceLock };

use ab_glyph::{ FontVec, Font as FontImpl, GlyphId };

use hashbrown::HashMap;

use crate::presentation::asset;

use asset::{ AssetType, AssetLoadingParams, AssetLoadError };

pub static GLYPH_DATA_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FontStyle {
    Regular,
    Italic
}

pub struct Font {
    font_files: Vec<Arc<Vec<u8>>>,
    font_data: HashMap<(Option<u16>, FontStyle), (FontVec, GPUGlyphData)>,
    family_name: String
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphCurve {
    pub start: [f32; 2],
    pub control: [f32; 2],
    pub end: [f32; 2]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphLine {
    pub start: [f32; 2],
    pub end: [f32; 2]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphIndex {
    pub curve_start: u32,
    pub curve_len: u32,
    pub line_start: u32,
    pub line_len: u32
}

#[derive(Clone, Debug)]
pub struct GPUGlyphData {
    pub(self) curve_data: Vec<GlyphCurve>,
    pub(self) line_data: Vec<GlyphLine>,
    pub(self) glyph_slice_inds: Vec<[GlyphIndex; 8]>,
    pub(self) glyph_to_ind: HashMap<GlyphId, usize>,
    buffers: Option<(wgpu::Buffer, wgpu::Buffer, wgpu::Buffer)>,
    bind_group: Option<wgpu::BindGroup>
}

impl GPUGlyphData {
    pub(self) fn new(curve_data: Vec<GlyphCurve>, line_data: Vec<GlyphLine>, glyph_slice_inds: Vec<[GlyphIndex; 8]>, glyph_to_ind: HashMap<GlyphId, usize>) -> Self {
        Self { curve_data, line_data, glyph_slice_inds, glyph_to_ind, buffers: None, bind_group: None }
    }

    pub fn update_bind_group(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        use wgpu::util::{ BufferInitDescriptor, DeviceExt };
        
        if let Some((cbuf, lbuf, ibuf)) = &mut self.buffers {
            let new_cbuf_contents = bytemuck::cast_slice(&self.curve_data);
            let new_lbuf_contents = bytemuck::cast_slice(&self.line_data);
            let new_ibuf_contents = bytemuck::cast_slice(&self.glyph_slice_inds);

            if cbuf.size()<new_cbuf_contents.len() as wgpu::BufferAddress {
                *cbuf = device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: new_cbuf_contents,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            } else {
                queue.write_buffer(&*cbuf, 0, new_cbuf_contents);
            }

            if lbuf.size()<new_lbuf_contents.len() as wgpu::BufferAddress {
                *lbuf = device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: new_lbuf_contents,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            } else {
                queue.write_buffer(&*lbuf, 0, new_lbuf_contents);
            }

            if ibuf.size()<new_ibuf_contents.len() as wgpu::BufferAddress {
                *ibuf = device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: new_ibuf_contents,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                });
            } else {
                queue.write_buffer(&*ibuf, 0, new_ibuf_contents);
            }
        } else {
            self.buffers = Some((
                device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.curve_data),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                }),
                device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.line_data),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                }),
                device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.glyph_slice_inds),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                })
            ))
        }

        let bufs = self.buffers.as_ref().unwrap();
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: GLYPH_DATA_BIND_GROUP_LAYOUT.get().expect("Glyph Data Bind Group Layout wasn't initialized yet!"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &bufs.0,
                        offset: 0,
                        size: NonZeroU64::new((self.curve_data.len() * std::mem::size_of::<GlyphCurve>()) as wgpu::BufferAddress)
                    })
                },
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &bufs.1,
                        offset: 0,
                        size: NonZeroU64::new((self.line_data.len() * std::mem::size_of::<GlyphLine>()) as wgpu::BufferAddress)
                    })
                },
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &bufs.2,
                        offset: 0,
                        size: NonZeroU64::new((self.glyph_slice_inds.len() * std::mem::size_of::<GlyphIndex>()) as wgpu::BufferAddress)
                    })
                },
            ]
        }))
    }
    
    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
}

impl AssetType for Font {
    fn global_init(loading_params: AssetLoadingParams) {
        GLYPH_DATA_BIND_GROUP_LAYOUT.set(loading_params.gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph Data Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: true, min_binding_size: None },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: true, min_binding_size: None },
                    count: None
                }
            ]
        })).unwrap_or_else(|_| tracing::error!("Initializer functions of assets should only be called once!"));
    }

    fn load_asset(data: toml::Table, loading_params: AssetLoadingParams) -> Result<(String, Self), AssetLoadError>
    where Self: Sized {
        let family_name;
        if let Some(toml::Value::String(s)) = data.get("family") {
            family_name = s.clone();
        } else {
            return Err(AssetLoadError::CreationError(anyhow::anyhow!("Property 'family' doesn't exist or isn't a string value!")));
        }

        let mut font_files = Vec::new();

        let mut font_data = HashMap::new();
        if let Some(toml::Value::Array(a)) = data.get("paths") {
            for v in a.iter() {
                if let toml::Value::String(s) = v {
                    let font_file = Arc::new(std::fs::read(s)?);
                    font_files.push(font_file.clone());

                    for i in 0..ttf_parser::fonts_in_collection(&font_file).unwrap_or(1) {
                        let font_info = ttf_parser::Face::parse(&font_file, i).map_err(|e| AssetLoadError::CreationError(anyhow::anyhow!("Font parsing error: {e}")))?;
                        let style = if font_info.is_italic() { FontStyle::Italic } else { FontStyle::Regular };
                        let weight = if font_info.is_variable() { None } else { Some(font_info.weight().to_number()) };

                        let f = FontVec::try_from_vec_and_index(font_file.as_ref().clone(), i).map_err(|e| AssetLoadError::CreationError(anyhow::anyhow!("Font parsing error: {e}")))?;

                        let mut curve_data = Vec::new();
                        let mut line_data = Vec::new();
                        let mut glyph_slice_inds = Vec::new();
                        let mut glyph_to_ind = HashMap::new();
                        
                        for (glyph, char) in f.codepoint_ids() {
                            let curve_start = curve_data.len();
                            let line_start = line_data.len();

                            let mut curves: [Vec<GlyphCurve>; 8] = std::array::from_fn(|_| Vec::new());
                            let mut lines: [Vec<GlyphLine>; 8] = std::array::from_fn(|_| Vec::new());

                            if let Some(outline) = f.outline(glyph) {
                                let bounds = outline.bounds;
                                for curve in outline.curves {
                                    match curve {
                                        ab_glyph::OutlineCurve::Line(start, end) => {
                                            let line_bounds = (
                                                ((start.y - bounds.min.y) / (bounds.max.y - bounds.min.y)).min((end.y - bounds.min.y) / (bounds.max.y - bounds.min.y)),
                                                ((start.y - bounds.min.y) / (bounds.max.y - bounds.min.y)).max((end.y - bounds.min.y) / (bounds.max.y - bounds.min.y)),
                                            );

                                            for i in 0..8 {
                                                if !(line_bounds.0>(i as f32+1.0)/8.0 || line_bounds.1<(i as f32)/8.0) {
                                                    lines[i].push(GlyphLine {
                                                        start: [
                                                            (start.x - bounds.min.x) / (bounds.max.x - bounds.min.x),
                                                            (start.y - bounds.min.y) / (bounds.max.y - bounds.min.y)
                                                        ],
                                                        end: [
                                                            (end.x - bounds.min.x) / (bounds.max.x - bounds.min.x),
                                                            (end.y - bounds.min.y) / (bounds.max.y - bounds.min.y)
                                                        ]
                                                    });
                                                }
                                            }
                                        },
                                        ab_glyph::OutlineCurve::Quad(start, control, end) => {
                                            let curve_bounds = (
                                                ((start.y - bounds.min.y) / (bounds.max.y - bounds.min.y))
                                                .min((end.y - bounds.min.y) / (bounds.max.y - bounds.min.y))
                                                .min((control.y - bounds.min.y) / (bounds.max.y - bounds.min.y)),
                                                ((start.y - bounds.min.y) / (bounds.max.y - bounds.min.y))
                                                .max((end.y - bounds.min.y) / (bounds.max.y - bounds.min.y))
                                                .max((control.y - bounds.min.y) / (bounds.max.y - bounds.min.y))
                                            );

                                            for i in 0..8 {
                                                if !(curve_bounds.0>(i as f32+1.0)/8.0 || curve_bounds.1<(i as f32)/8.0) {
                                                    curves[i].push(GlyphCurve {
                                                        start: [
                                                            (start.x - bounds.min.x) / (bounds.max.x - bounds.min.x),
                                                            (start.y - bounds.min.y) / (bounds.max.y - bounds.min.y)
                                                        ],
                                                        control: [
                                                            (control.x - bounds.min.x) / (bounds.max.x - bounds.min.x),
                                                            (control.y - bounds.min.y) / (bounds.max.y - bounds.min.y)
                                                        ],
                                                        end: [
                                                            (end.x - bounds.min.x) / (bounds.max.x - bounds.min.x),
                                                            (end.y - bounds.min.y) / (bounds.max.y - bounds.min.y)
                                                        ]
                                                    });
                                                }
                                            }
                                        },
                                        ab_glyph::OutlineCurve::Cubic(_, _, _, _) => {
                                            panic!("Cubic curves aren't supported!")
                                        }
                                    }
                                }
                            }

                            for i in 0..8 {
                                curve_data.extend_from_slice(&curves[i]);
                                line_data.extend_from_slice(&lines[i]);
                            }
                            glyph_to_ind.insert(glyph, glyph_slice_inds.len());
                            let mut l = (0,0);
                            glyph_slice_inds.push(std::array::from_fn(|i| {
                                let r = GlyphIndex {
                                    curve_start: (curve_start+l.0) as u32,
                                    curve_len: (curves[i].len()) as u32,
                                    line_start: (line_start+l.1) as u32,
                                    line_len: (lines[i].len()) as u32
                                };
                                l.0 += curves[i].len();
                                l.1 += lines[i].len();
                                r
                            }));
                        }

                        font_data.insert((weight, style), (f, GPUGlyphData::new(curve_data, line_data, glyph_slice_inds, glyph_to_ind)));
                    }
                } else {
                    Err(AssetLoadError::CreationError(anyhow::anyhow!("Items in 'paths' array must be strings!")))?;
                }
            }
        } else {
            Err(AssetLoadError::CreationError(anyhow::anyhow!("Property 'paths' doesn't exist or isn't an array value!")))?;
        }

        let f = Font {
            font_files,
            family_name: family_name.clone(),
            font_data
        };

        Ok((family_name.clone(), f))
    }
}