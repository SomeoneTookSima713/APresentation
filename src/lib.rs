#![allow(incomplete_features)]
#![feature(min_specialization)]
#![feature(maybe_uninit_array_assume_init, const_maybe_uninit_array_assume_init)]
#![feature(const_float_bits_conv, const_swap, const_mut_refs, const_trait_impl, const_maybe_uninit_write)]
#![feature(generic_arg_infer)]
#![feature(exclusive_wrapper)]
#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![feature(const_refs_to_cell)]
#![feature(allocator_api, alloc_layout_extra, slice_ptr_get)]
#![feature(box_into_inner)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]
#![feature(const_intrinsic_copy)]
#![feature(coerce_unsized)]
#![feature(iterator_try_reduce)]
#![feature(entry_insert)]

use winit::dpi::PhysicalSize;
use winit::event::{ WindowEvent, KeyEvent, ElementState };
use winit::event_loop::{ EventLoop, ActiveEventLoop };
use winit::keyboard::{ Key, NamedKey };
use winit::window::{ Window, WindowId, WindowAttributes };

use std::sync::Arc;

pub mod util;
pub mod render;
pub mod presentation;
pub mod parse;
pub mod shaders;

use render::texture;

use util::fallible_app_handler::{ FallibleAppHandler, FallibleAppHandlerWrap };

pub struct BaseState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    presentation_state: presentation::state::PresentationState,

    #[cfg(debug_assertions)]
    debug_state: util::debug_state::DebugState,
}

impl<'a> BaseState<'a> {
    async fn new(window: Window) -> anyhow::Result<BaseState<'a>> {
        let window_arc = Arc::new(window);

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window_arc.clone())?;

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
            .filter(|f| f.eq(&wgpu::TextureFormat::Rgba8UnormSrgb))
            .next()
            .unwrap();
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
                log::info!("Found best Presentmode: {b:?}");
                *b
            }).unwrap_or(wgpu::PresentMode::Fifo),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 4
        };
        surface.configure(&device, &config);

        texture::init(&device)?;
        render::camera::init(&device)?;

        let win_size = window_arc.inner_size();

        let presentation_state = presentation::state::PresentationState::new(&device, &queue, win_size, &config)?;

        // presentation::resource_managers::TEXTURE_MANAGER.insert(
        //     "test".to_string(),
        //     Arc::new(texture::Texture::from_image(&device, &queue, "test.jpeg", texture::TextureSamplerSelection::Linear, None)?)
        // )?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: win_size,
            window: window_arc,
            presentation_state,
            #[cfg(debug_assertions)]
            debug_state: util::debug_state::DebugState::new(),
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
            self.presentation_state.on_resize(new_size, &self.device).unwrap();
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.presentation_state.on_input(event)
    }

    fn update(&mut self) {
        self.presentation_state.update();

        #[cfg(debug_assertions)]
        self.debug_state.update().unwrap();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.presentation_state.begin_render(&self.queue).unwrap();        
        let presentation_commands = self.presentation_state.do_render(&view, &self.device, &self.queue).unwrap();

        // submit will accept anything that implements IntoIter
        self.queue.submit([presentation_commands]);
        output.present();

        Ok(())
    }
}

#[derive(Default)]
pub struct App<'a> {
    base_state: Option<BaseState<'a>>
}

impl<'a> FallibleAppHandler<()> for App<'a> {
    async fn resumed(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if self.base_state.is_none() {
            let window = init_window(event_loop)?;
            self.base_state = Some(BaseState::new(window).await?);
        }
        Ok(())
    }

    async fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) -> anyhow::Result<()> {
        let base_state = if self.base_state.is_some() {
            self.base_state.as_mut().unwrap()
        } else {
            return Ok(())
        };
        if base_state.input(&event) { return Ok(()); }
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
                    base_state.resize(base_state.window.inner_size());
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
                base_state.window.request_redraw();
            },
            _ => {}
        }
        Ok(())
    }
}

pub async fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    let mut state_wrap = FallibleAppHandlerWrap::from(App::default());

    event_loop.run_app(&mut state_wrap)?;

    Ok(())
}

fn init_window(event_loop: &ActiveEventLoop) -> anyhow::Result<Window> {
    let window = event_loop.create_window(
        WindowAttributes::default()
            .with_inner_size(PhysicalSize::<u32>::from(util::consts::WINDOW_SIZE))
            .with_resizable(true)
            .with_resize_increments(PhysicalSize::<u32>::from(util::consts::WINDOW_RESIZE_INCREMENTS))
            .with_title(util::consts::WINDOW_TITLE)
            .with_window_icon(Some(winit::window::Icon::from_rgba(util::consts::ICON_DATA.as_raw().clone(), util::consts::ICON_DATA.width(), util::consts::ICON_DATA.height())?))
    )?;
    Ok(window)
}