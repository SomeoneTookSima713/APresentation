use std::sync::OnceLock;

pub static TEXTURE_SAMPLERS: OnceLock<TextureSamplerList> = OnceLock::new();
pub static TEXTURE_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub struct TextureSamplerList {
    pub pixel_perfect: wgpu::Sampler,
    pub linear: wgpu::Sampler,
}

#[derive(Clone, Copy)]
pub enum TextureSamplerSelection {
    PixelPerfect,
    Linear,
}

impl TextureSamplerSelection {
    pub fn get_ref(self) -> anyhow::Result<&'static wgpu::Sampler> {
        match self {
            Self::PixelPerfect => TEXTURE_SAMPLERS.get().map(|r|&r.pixel_perfect).ok_or(anyhow::anyhow!("texture::init() wasn't called!")),
            Self::Linear => TEXTURE_SAMPLERS.get().map(|r|&r.linear).ok_or(anyhow::anyhow!("texture::init() wasn't called!")),
        }
    }
}

impl TextureSamplerList {
    pub fn new(device: &wgpu::Device) -> Self {
        TextureSamplerList {
            pixel_perfect: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Pixel Perfect Image Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            linear: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Linear Image Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
        }
    }
}

pub fn init(device: &wgpu::Device) -> anyhow::Result<()> {
    TEXTURE_SAMPLERS.set(TextureSamplerList::new(&device)).map_err(|_|anyhow::anyhow!("texture::init() was called twice!"))?;
    TEXTURE_BIND_GROUP_LAYOUT.set(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    })).map_err(|_|anyhow::anyhow!("texture::init() was called twice!"))?;

    Ok(())
}

#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    sampler: &'static wgpu::Sampler
}

#[allow(unused)]
impl Texture {
    pub fn raw_texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn sampler(&self) -> &'static wgpu::Sampler {
        self.sampler
    }
}

impl Texture {
    /// Loads an image from disk and creates a texture on the gpu from that.
    /// 
    /// # Panics
    /// Panics if the image couldn't be loaded from disk.
    /// 
    /// This function, just like basically all functions from this module, also
    /// panics if this module's [`init()`]-function() wasn't called.
    pub fn from_image<P: AsRef<std::path::Path>>(device: &wgpu::Device, queue: &wgpu::Queue, path: P, sampler: TextureSamplerSelection, additional_usages: Option<wgpu::TextureUsages>) -> anyhow::Result<Self> {
        use image::GenericImageView;
        use wgpu::TextureUsages;

        let img = image::open(path.as_ref())?;
        let img_data = img.to_rgba8();

        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path.as_ref().to_string_lossy().as_ref()),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | additional_usages.unwrap_or(wgpu::TextureUsages::empty()),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1)
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler_ref = sampler.get_ref()?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(path.as_ref().to_string_lossy().as_ref()),
            layout: TEXTURE_BIND_GROUP_LAYOUT.get().ok_or(anyhow::anyhow!("texture::init() wasn't called!"))?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler_ref)
                }
            ]
        });

        Ok(Texture {
            texture,
            view: texture_view,
            sampler: sampler_ref,
            bind_group
        })
    }

    /// Constructs a texture on the gpu based on raw pixel data in RGBA8-Format
    /// (8bits for red, green, blue and alpha channels).
    /// 
    /// # Panics
    /// Panics (in debug mode) if the size of the supplied data doesn't exactly
    /// match the required data for the creation of the texture.
    /// In release mode, this case doesn't necessarily cause a panic for
    /// performance reasons.
    /// 
    /// This function, just like basically all functions from this module, also
    /// panics if this module's [`init()`]-function() wasn't called.
    pub fn from_data<D: AsRef<[u8]>>(device: &wgpu::Device, queue: &wgpu::Queue, data: D, size: (u32, u32), sampler: TextureSamplerSelection, additional_usages: Option<wgpu::TextureUsages>) -> anyhow::Result<Self> {
        debug_assert!(data.as_ref().len() as u32 == size.0*size.1*4, "Length of data stream doesn't match requested size!");

        let texture_size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | additional_usages.unwrap_or(wgpu::TextureUsages::empty()),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data.as_ref(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.0),
                rows_per_image: Some(size.1)
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler_ref = sampler.get_ref()?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: TEXTURE_BIND_GROUP_LAYOUT.get().ok_or(anyhow::anyhow!("texture::init() wasn't called!"))?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler_ref)
                }
            ]
        });

        Ok(Texture {
            texture,
            view: texture_view,
            sampler: sampler_ref,
            bind_group
        })
    }

    /// Constructs a texture on the gpu based on a RGBA-color.
    /// 
    /// # Panics
    /// This function, just like basically all functions from this module,
    /// panics if the module's [`init()`]-function() wasn't called.
    pub fn from_color(device: &wgpu::Device, queue: &wgpu::Queue, color: [u8;4], size: (u32, u32), sampler: TextureSamplerSelection, additional_usages: Option<wgpu::TextureUsages>) -> anyhow::Result<Self> {
        let data = color.repeat(size.0 as usize * size.1 as usize);
        Self::from_data(device, queue, data, size, sampler, additional_usages)
    }

    /// Creates a new empty texture on the gpu.
    /// 
    /// # Panics
    /// This function, just like basically all functions from this module,
    /// panics if this module's [`init()`]-function() wasn't called.
    pub fn new(device: &wgpu::Device, size: (u32, u32), sampler: TextureSamplerSelection, additional_usages: Option<wgpu::TextureUsages>) -> anyhow::Result<Self> {
        let texture_size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | additional_usages.unwrap_or(wgpu::TextureUsages::empty()),
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler_ref = sampler.get_ref()?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: TEXTURE_BIND_GROUP_LAYOUT.get().ok_or(anyhow::anyhow!("texture::init() wasn't called!"))?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler_ref)
                }
            ]
        });

        Ok(Texture {
            texture,
            view: texture_view,
            sampler: sampler_ref,
            bind_group
        })
    }
}