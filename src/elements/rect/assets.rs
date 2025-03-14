use std::path::Path;
use std::sync::OnceLock;

use image::{ ImageReader, RgbaImage };

use crate::presentation::asset;

use asset::{ AssetType, AssetLoadingParams, AssetLoadError };

pub(super) static IMAGE_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub struct Image {
    pub data: RgbaImage,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView
}

impl AssetType for Image {
    fn global_init(loading_params: AssetLoadingParams) {
        IMAGE_BIND_GROUP_LAYOUT.set(loading_params.gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
            ]
        })).expect("Shouldn't happen!");
    }

    fn load_asset(data: toml::Table, loading_params: AssetLoadingParams) -> Result<(String, Self), AssetLoadError>
    where Self: Sized {
        fn field_missing_err(field: &'static str) -> AssetLoadError {
            AssetLoadError::CreationError(anyhow::anyhow!("Required field '{field}' missing!"))
        }
        fn field_invalid_val_err(field: &'static str, expected: &'static str, v: &toml::Value) -> AssetLoadError {
            AssetLoadError::CreationError(anyhow::anyhow!("Field '{field}' expected to be {expected}, got {v:?}!"))
        }

        let path_val = data.get("path").ok_or(field_missing_err("path"))?;
        let path_string;
        let path = if let toml::Value::String(s) = path_val {
            path_string = s.clone();
            Path::new(s).canonicalize()?
        } else {
            return Err(field_invalid_val_err("path", "string", path_val));
        };

        let name_val = data.get("name").cloned().unwrap_or(toml::Value::String(path_string));
        let name = if let toml::Value::String(s) = name_val {
            s.clone()
        } else {
            return Err(field_invalid_val_err("name", "string", &name_val));
        };

        let reader = ImageReader::open(path)?;

        let data_dyn = reader.decode().map_err(|e| AssetLoadError::CreationError(anyhow::anyhow!(e)))?;
        let data = data_dyn.to_rgba8();

        let size = wgpu::Extent3d { width: data_dyn.width(), height: data_dyn.height(), depth_or_array_layers: 1 };
        let texture = loading_params.gpu_device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]
        });
        loading_params.gpu_queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            bytemuck::cast_slice(data.as_ref()),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * data_dyn.width()),
                rows_per_image: None
            },
            size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });
        
        Ok((name, Image {
            data,
            texture,
            view
        }))
    }
}