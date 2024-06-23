use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{ OnceLock, Mutex, MutexGuard };
use std::time::Instant;

use mlua::Lua;

use crate::render::{ self, post_process };
use super::renderable::{ RenderableRenderingManagerObjectSafe, register_rendering_managers };
use super::Presentation;

static LUA_STATE: OnceLock<Mutex<Lua>> = OnceLock::new();

/// The state used for presentation-related stuff.
/// 
/// Only one instance of this can exist at a time, as it borrows a private
/// static variable mutably.
pub struct PresentationState {
    /// The lua state
    lua: MutexGuard<'static, Lua>,

    pub loaded_presentation: Option<Presentation>,

    post_process_pass: post_process::PostProcessPipeline,
    camera: render::camera::Camera,

    rendering_managers: HashMap<TypeId, Box<dyn RenderableRenderingManagerObjectSafe>>,
    cached_render_bundles: HashMap<TypeId, wgpu::RenderBundle>,

    last_time: Instant,
}

impl PresentationState {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, window_size: winit::dpi::PhysicalSize<u32>, config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self> {
        let lua;
        match LUA_STATE.get() {
            Some(m) => {
                match m.try_lock() {
                    Ok(g) => lua = g,
                    Err(std::sync::TryLockError::Poisoned(e)) => anyhow::bail!("A previously created PresentationState poisoned the Lua instance: {e}"),
                    Err(std::sync::TryLockError::WouldBlock) => anyhow::bail!("An instance of PresentationState already exists!")
                }
            },
            None => {
                LUA_STATE.set(Mutex::new(Lua::new()));
                lua = LUA_STATE.get().unwrap().lock().map_err(|e|anyhow::anyhow!("Another thread intervened while this PresentationState was being created: {e}"))?;
            }
        }
        let post_process_pass = post_process::PostProcessPipeline::new(device, window_size, config)?;

        let camera = render::camera::Camera::new(window_size.width as f32, window_size.height as f32, 0.0, 10000.0, device)?;

        let mut rendering_managers = HashMap::new();

        register_rendering_managers(&mut rendering_managers, device, queue, config);

        Ok(Self {
            lua,
            loaded_presentation: Some(Presentation::load_file("test_presentation.toml", device, queue)?),
            post_process_pass,
            camera,
            rendering_managers,
            cached_render_bundles: HashMap::new(),
            last_time: Instant::now(),
        })
    }

    pub fn on_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) -> anyhow::Result<()> {
        self.camera.update_dimensions(new_size.width as f32, new_size.height as f32);
        self.post_process_pass.update_surface_size(device, new_size)?;

        Ok(())
    }

    pub fn update(&mut self) {
        let dt = self.last_time.elapsed();
        self.last_time = Instant::now();

        if let Some(presentation) = self.loaded_presentation.as_mut() {
            presentation.update(&self.lua, dt.as_secs_f64());
        }
    }

    pub fn on_input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    pub fn begin_render(&mut self, queue: &wgpu::Queue) -> anyhow::Result<()> {
        self.camera.update_buffer(queue);
        Ok(())
    }

    pub fn do_render(&mut self, output_view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<wgpu::CommandBuffer> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Presentation Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.post_process_pass.get_intermediate_surface(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        if let Some(presentation) = self.loaded_presentation.as_mut() {
            let args = mlua::Variadic::new();
            presentation.render(&mut self.rendering_managers, args)?;

            for (typeid, manager) in self.rendering_managers.iter_mut() {
                self.cached_render_bundles.insert(*typeid, manager.render_instances(device, queue, &self.camera)?);
            }

            render_pass.execute_bundles(self.cached_render_bundles.values());
        }

        drop(render_pass);
        self.post_process_pass.execute(&mut encoder, output_view);

        Ok(encoder.finish())
    }
}