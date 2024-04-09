use winit::dpi::PhysicalSize;
use winit::event::{ Event, WindowEvent, KeyEvent, ElementState };
use winit::event_loop::EventLoop;
use winit::keyboard::{ Key, NamedKey };
use winit::window::{ Window, WindowBuilder };

mod util;

pub struct BaseState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
}

impl<'a> BaseState<'a> {
    async fn new(window: &'a Window) -> anyhow::Result<BaseState<'a>> {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(debug_assertions))]
            backends: wgpu::Backends::all(),
            // If we're compiling in debug mode, I don't want DirectX12 as a
            // backend because it spams the logs with unfixable soft-errors on
            // my machine for some reason.
            #[cfg(debug_assertions)]
            backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.ok_or(anyhow::anyhow!("No suitable GPU adapter found!"))?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: util::consts::WINDOW_SIZE.0,
            height: util::consts::WINDOW_SIZE.1,
            present_mode: surface_caps.present_modes.iter().reduce(|acc,e| {
                use wgpu::PresentMode;
                fn rate(p: PresentMode) -> u8 {
                    match p {
                        PresentMode::Immediate => 0,
                        PresentMode::AutoNoVsync => 1,
                        PresentMode::Fifo => 2,
                        PresentMode::AutoVsync => 3,
                        PresentMode::FifoRelaxed => 4,
                        PresentMode::Mailbox => 5,
                    }
                }
                if rate(*acc)>rate(*e) { acc }
                else { e }
            }).map(|b|{
                #[cfg(debug_assertions)]
                log::debug!("Found best Presentmode: {b:?}");
                *b
            }).unwrap_or(wgpu::PresentMode::Fifo),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 4
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: window.inner_size(),
            window,
        })
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        // The bool indicates wether or not the upplied event shouldn't be
        // processed further by the event loop. If `true`, the event won't be
        // processed any further. If `false`, it will be processed further.
        false
    }

    fn update(&mut self) {
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
        }
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() -> anyhow::Result<()> {
    let (event_loop, window) = init_window()?;

    let mut base_state = BaseState::new(&window).await?;

    event_loop.run(|event, event_loop| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => if !base_state.input(&event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput { event: KeyEvent { logical_key: Key::Named(NamedKey::Escape), state: ElementState::Pressed, .. }, .. } => {
                        event_loop.exit();
                    },
                    WindowEvent::Resized(new_size) => {
                        base_state.resize(new_size);
                    },
                    WindowEvent::ScaleFactorChanged { scale_factor, mut inner_size_writer } => {
                        let new_inner_size = {
                            let s = base_state.window().inner_size();
                            PhysicalSize { width: (s.width as f64 * scale_factor) as u32, height: (s.height as f64 * scale_factor) as u32 }
                        };
                        if let Err(e) = inner_size_writer.request_inner_size(new_inner_size) {
                            log::error!("Error resizing window on scale factor change: {e}");
                            base_state.resize(window.inner_size());
                        } else {
                            base_state.resize(new_inner_size);
                        }
                    },
                    WindowEvent::RedrawRequested => {
                        base_state.update();
                        match base_state.render() {
                            Ok(_) => {},
                            Err(wgpu::SurfaceError::Lost) => base_state.resize(base_state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("System or GPU is out of memory! Exiting application...");
                                event_loop.exit()
                            },
                            Err(e) => {
                                log::error!("Error with GPU Surface: {e}")
                            }
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    })?;

    Ok(())
}

fn init_window() -> anyhow::Result<(EventLoop<()>, Window)> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::<u32>::from(util::consts::WINDOW_SIZE))
        .with_resizable(true)
        .with_resize_increments(PhysicalSize::<u32>::from(util::consts::WINDOW_RESIZE_INCREMENTS))
        .with_title(util::consts::WINDOW_TITLE)
        .with_window_icon(Some(winit::window::Icon::from_rgba(Vec::from(*util::consts::ICON_DATA), util::consts::ICON_WIDTH as u32, util::consts::ICON_HEIGHT as u32)?))
        .build(&event_loop)?;
    Ok((event_loop, window))
}