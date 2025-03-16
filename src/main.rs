#![feature(hasher_prefixfree_extras)]
#![feature(if_let_guard)]
#![feature(box_into_inner)]
#![feature(array_try_from_fn)]
#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![feature(box_patterns)]

use std::sync::{ Arc, OnceLock };

use winit::window::Window;

mod util;
mod config;
mod cli;
mod presentation;
mod elements;

use util::improved_app_handler::{ App, AppHandler };

const CONFIG_PATH: &str = "config.toml";
// It's really hacky, but this is the way I supply my application loop code my
// cli arguments. Don't ask how I got the genious idea of doing it this overly
// complicated way.
static CURR_CONFIG_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();
static CURR_FILE_PATH: OnceLock<String> = OnceLock::new();

pub struct PushConstantManager(std::sync::atomic::AtomicU32);
/// Use the methods on this static's type in your element renderer's init
/// function if you need to use push constants. This static ensures that no two
/// [`ElementRenderer`](presentation::element::ElementRenderer)s overlap
/// eachother in their push constant ranges.
pub static PUSH_CONSTANT_MANAGER: PushConstantManager = PushConstantManager(std::sync::atomic::AtomicU32::new(0));
impl PushConstantManager {
    pub fn get_range(&self, size: u32) -> std::ops::Range<u32> {
        use std::sync::atomic::Ordering;

        let val = self.0.fetch_add(size, Ordering::SeqCst);
        val..val+size
    }
}

struct APresentation {
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    presentation: presentation::Presentation
}

impl AppHandler for APresentation {
    #[tracing::instrument]
    async fn init(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self>
    where Self: Sized
    {
        use winit::window::WindowAttributes;
        use winit::dpi::PhysicalSize;

        let conf_path = CURR_CONFIG_PATH.get_or_init(|| CONFIG_PATH.into());

        let config = match config::AppConfig::load(&conf_path) {
            Ok(c) => c,
            Err(config::AppConfigLoadError::IOError(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!("Config file doesn't exist! Creating new file with default values...");
                let def = config::AppConfig::default();
                def.save(&conf_path)?;
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
                    wgpu::Backend::Metal => -1,
                    _ => 0
                };

                (adapter, score)
            })
            .reduce(|e, acc| if e.1 > acc.1 { e } else { acc })
            .map(|v| v.0)
            .ok_or(anyhow::anyhow!("No suitable GPU adapter found!"))?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features:
                    wgpu::Features::TEXTURE_BINDING_ARRAY |
                    wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING |
                    wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY |
                    wgpu::Features::PUSH_CONSTANTS,
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

        let mut surface_present_mode = surface_caps.present_modes.clone();
        fn pmtv(p: &wgpu::PresentMode) -> u8 {
            match p {
                wgpu::PresentMode::AutoVsync | wgpu::PresentMode::AutoNoVsync => 0,
                wgpu::PresentMode::Fifo => 1,
                wgpu::PresentMode::Immediate => 2,
                wgpu::PresentMode::FifoRelaxed => 3,
                wgpu::PresentMode::Mailbox => 4
            }
        }

        surface_present_mode.sort_by(|a, b| pmtv(b).cmp(&pmtv(a)));

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_present_mode[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 4
        };

        let mut reg_elems = presentation::element::RegisteredElements::new();
        reg_elems.register_element::<elements::rect::Rect>("Rect".to_string());
        reg_elems.register_element_renderer::<elements::rect::RectRenderer>();

        let mut asset_manager = presentation::asset::AssetManager::new();
        asset_manager.register_asset_type::<elements::rect::Image>("image".to_string());
        asset_manager.register_asset_type::<elements::text::assets::Font>("font".to_string());

        let mut parser_collection = presentation::parser::ParserCollection::new();
        parser_collection.register_parser::<presentation::parser::impls::apres::ApresParser>();

        let presentation = presentation::Presentation::new(
            CURR_FILE_PATH.get().expect("Unreachable"),
            reg_elems,
            asset_manager,
            parser_collection,
            device.clone(),
            queue.clone(),
            &surface_config
        )?;

        Ok(APresentation {
            surface,
            adapter,
            device,
            queue,
            surface_config,
            window,
            presentation,
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
                self.presentation.reconfigure(&self.surface_config);
            },
            winit::event::WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key, state, .. }, .. } => {
                if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight) = physical_key
                && let winit::event::ElementState::Pressed = state {
                    self.presentation.next_slide();
                }
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

        self.presentation.render(&view, &mut encoder, &self.surface_config)?;

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
            .with_max_level({
                #[cfg(debug_assertions)]
                {tracing::Level::DEBUG}
                #[cfg(not(debug_assertions))]
                {tracing::Level::INFO}
            })
            .pretty()
            .finish()
    ).expect("Couldn't initialize logger!");

    use clap::Parser;
    let cli = cli::CLI::parse();

    CURR_CONFIG_PATH.set(cli.config.unwrap_or(CONFIG_PATH.into())).expect("Unreachable");
    CURR_FILE_PATH.set(cli.file).expect("Unreachable");

    match cli.command {
        cli::Command::Generate => { std::fs::write(CURR_FILE_PATH.get().unwrap(), include_bytes!("template.apres"))?; Ok(()) },
        cli::Command::Present => {
            let event_loop = winit::event_loop::EventLoop::new()?;

            let mut app: App<APresentation> = App::default();

            event_loop.run_app(&mut app)?;

            app.get_errors()
        }
    }
}
