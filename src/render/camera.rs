use std::sync::OnceLock;

use glam::{ Mat4 };

pub static CAMERA_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub fn init(device: &wgpu::Device) -> anyhow::Result<()> {
    CAMERA_BIND_GROUP_LAYOUT.set(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }
        ]
    })).map_err(|_|anyhow::anyhow!("camera::init() was called twice!"))?;
    Ok(())
}

pub struct Camera {
    pub width: f32,
    pub height: f32,
    znear: f32,
    zfar: f32,
    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_lh(-self.width / 2.0, self.width / 2.0, -self.height / 2.0, self.height / 2.0, self.znear, self.zfar)
    }

    pub fn new(width: f32, height: f32, near_clip: f32, far_clip: f32, device: &wgpu::Device) -> anyhow::Result<Self> {
        use wgpu::util::DeviceExt;

        let uniform = CameraUniform { proj_mat: Mat4::IDENTITY.to_cols_array_2d() };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer Descriptor"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: CAMERA_BIND_GROUP_LAYOUT.get().ok_or(anyhow::anyhow!("camera::init() wasn't called!"))?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ]
        });

        Ok(Self { width, height, znear: near_clip, zfar: far_clip, uniform, buffer, bind_group })
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue) {
        let mat = self.build_view_projection_matrix();
        self.uniform.proj_mat = mat.to_cols_array_2d();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn update_dimensions(&mut self, new_width: f32, new_height: f32) {
        self.width = new_width;
        self.height = new_height;
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub(self) proj_mat: [[f32; 4]; 4]
}