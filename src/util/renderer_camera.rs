use std::sync::OnceLock;

pub static CAMERA_UNIFORM_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();
pub fn construct_cam_bind_group_layout(dev: &wgpu::Device) -> impl Fn() -> wgpu::BindGroupLayout {
    || dev.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            }
        ]
    })
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniformData {
    pub matrix: [[f32; 4]; 4]
}

#[derive(Clone)]
pub struct CameraUniform {
    data: CameraUniformData,
    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>
}

impl CameraUniform {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            data: CameraUniformData {
                matrix: nalgebra::Orthographic3::new(
                    x,
                    x+w,
                    y+h,
                    y,
                    1000.0,
                    -1.0,
                ).as_matrix().data.0
            },
            buffer: None,
            bind_group: None
        }
    }

    pub fn update_cam_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.data = CameraUniformData {
            matrix: nalgebra::Orthographic3::new(
                x,
                x+w,
                y+h,
                y,
                1000.0,
                -1.0,
            ).as_matrix().data.0
        }
    }

    pub fn update_bind_group<D,  Q>(&mut self, device: D, queue: Q)
    where D: std::borrow::Borrow<wgpu::Device>, Q: std::borrow::Borrow<wgpu::Queue> {
        use wgpu::util::{ BufferInitDescriptor, DeviceExt };
        
        let device: &wgpu::Device = device.borrow();
        let queue: &wgpu::Queue = queue.borrow();

        if let Some(b) = self.buffer.as_ref() {
            queue.write_buffer(b, 0, bytemuck::cast_slice(&[self.data]));
        } else {
            self.buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[self.data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }));
        }

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: CAMERA_UNIFORM_BIND_GROUP_LAYOUT.get_or_init(construct_cam_bind_group_layout(device)),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(self.buffer.as_ref().unwrap().as_entire_buffer_binding())
                }
            ]
        }));
    }

    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }

    pub fn update_and_get_bind_group<'a, 'b, D, Q>(&'a mut self, device: D, queue: Q) -> &'b wgpu::BindGroup
    where
        D: std::borrow::Borrow<wgpu::Device>,
        Q: std::borrow::Borrow<wgpu::Queue>,
        'a: 'b
    {
        self.update_bind_group(device, queue);
        self.get_bind_group().expect("Unreachable!")
    }
}

pub fn get_cam_bind_group_layout(dev: &wgpu::Device) -> &wgpu::BindGroupLayout {
    CAMERA_UNIFORM_BIND_GROUP_LAYOUT.get_or_init(construct_cam_bind_group_layout(dev))
}