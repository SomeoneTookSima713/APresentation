use std::sync::Arc;

use winit::window::Window;

mod util;
mod config;
mod presentation;

use util::improved_app_handler::{ App, AppHandler };

const CONFIG_PATH: &str = "config.toml";

struct APresentation {
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
}

impl AppHandler for APresentation {
    #[tracing::instrument]
    async fn init(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self>
    where Self: Sized
    {
        use winit::window::WindowAttributes;
        use winit::dpi::PhysicalSize;

        let config = match config::AppConfig::load(CONFIG_PATH) {
            Ok(c) => c,
            Err(config::AppConfigLoadError::IOError(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!("Config file doesn't exist! Creating new file with default values...");
                let def = config::AppConfig::default();
                def.save(CONFIG_PATH)?;
                def
            },
            Err(e) => anyhow::bail!(e)
        };

        let window_size = PhysicalSize::new(config.window_size.0, config.window_size.1);

        let window = Arc::new(event_loop.create_window(
            WindowAttributes::default()
                .with_inner_size(window_size)
                .with_resizable(true)
                .with_title("APresentation")
        )?);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            
            #[cfg(debug_assertions)]
            flags: wgpu::InstanceFlags::advanced_debugging(),

            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance.enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| {
                adapter.is_surface_supported(&surface)
            })
            .map(|adapter| {
                let mut score: i8 = 0;

                score += match adapter.get_info().device_type {
                    wgpu::DeviceType::IntegratedGpu => 1,
                    wgpu::DeviceType::Cpu => -1,
                    _ => 0
                };

                score += match adapter.get_info().backend {
                    wgpu::Backend::Vulkan => 1,
                    wgpu::Backend::Metal => 2,
                    _ => 0
                };

                (adapter, score)
            })
            .reduce(|e, acc| if e.1 > acc.1 { e } else { acc })
            .map(|v| v.0)
            .ok_or(anyhow::anyhow!("No suitable GPU adapter found!"))?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                memory_hints: Default::default()
            },
            None
        ).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 4
        };

        Ok(APresentation {
            surface,
            adapter,
            device,
            queue,
            surface_config,
            window
        })
    }

    async fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        window_event: winit::event::WindowEvent
    ) -> anyhow::Result<()>
    {
        match window_event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(new_size) if new_size.width > 0 && new_size.height > 0 => {
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;
                self.surface.configure(&self.device, &self.surface_config);
            },
            winit::event::WindowEvent::RedrawRequested => self.render()?,
            _ => {}
        }
        Ok(())
    }

    async fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<()> {
        self.window.request_redraw();
        Ok(())
    }
}

impl APresentation {
    fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            });
        }

        self.queue.submit([encoder.finish()]);

        self.window.pre_present_notify();

        output.present();

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_level(true)
            .with_thread_names(true)
            .with_target(true)
            .pretty()
            .finish()
    ).expect("Couldn't initialize logger!");

    let event_loop = winit::event_loop::EventLoop::new()?;

    let mut app: App<APresentation> = App::default();

    event_loop.run_app(&mut app)?;

    app.get_errors()
}
