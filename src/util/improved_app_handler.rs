use tracing::Level;

use winit::application::ApplicationHandler;
use winit::event::{ DeviceEvent, DeviceId, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[allow(unused_variables)]
pub trait AppHandler {
    async fn init(event_loop: &ActiveEventLoop) -> anyhow::Result<Self>
    where Self: Sized;

    async fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, window_event: WindowEvent) -> anyhow::Result<()>;

    async fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, device_event: DeviceEvent) -> anyhow::Result<()> { Ok(()) }

    async fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> { Ok(()) }

    async fn exiting(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> { Ok(()) }
}

pub struct App<A: AppHandler> {
    inner: Option<A>,
    errors: Vec<anyhow::Error>
}

impl<A: AppHandler> Default for App<A> {
    fn default() -> Self {
        Self { inner: None, errors: Vec::new() }
    }
}

impl<A: AppHandler> App<A> {
    pub fn get_errors(&self) -> Result<(), anyhow::Error> {
        match self.errors.iter().map(|e| anyhow::anyhow!("{}", e)).reduce(|e, acc| e.context(acc)) {
            Some(e) => Err(e),
            None => Ok(())
        }
    }
}

impl<A: AppHandler> ApplicationHandler for App<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let span = tracing::span!(Level::INFO, "<App as winit::ApplicationHandler>::resumed");
        let _enter = span.enter();

        if self.inner.is_some() { return; }
        match pollster::block_on(A::init(event_loop)) {
            Ok(inner) => self.inner = Some(inner),
            Err(e) => {
                tracing::error!("{e}");
                self.errors.push(e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
    ) {
        let span = tracing::span!(Level::INFO, "<App as winit::ApplicationHandler>::window_event");
        let _enter = span.enter();

        if let Some(inner) = self.inner.as_mut() {
            if let Err(e) = pollster::block_on(inner.window_event(event_loop, window_id, event)) {
                tracing::error!("{e}");
                self.errors.push(e);
                event_loop.exit();
            }
        }
    }

    fn device_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            device_id: DeviceId,
            event: DeviceEvent,
    ) {
        let span = tracing::span!(Level::INFO, "<App as winit::ApplicationHandler>::device_event");
        let _enter = span.enter();

        if let Some(inner) = self.inner.as_mut() {
            if let Err(e) = pollster::block_on(inner.device_event(event_loop, device_id, event)) {
                tracing::error!("{e}");
                self.errors.push(e);
                event_loop.exit();
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let span = tracing::span!(Level::INFO, "<App as winit::ApplicationHandler>::about_to_wait");
        let _enter = span.enter();

        if let Some(inner) = self.inner.as_mut() {
            if let Err(e) = pollster::block_on(inner.about_to_wait(event_loop)) {
                tracing::error!("{e}");
                self.errors.push(e);
                event_loop.exit();
            }
        }
    }
    
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let span = tracing::span!(Level::INFO, "<App as winit::ApplicationHandler>::exiting");
        let _enter = span.enter();

        if let Some(inner) = self.inner.as_mut() {
            if let Err(e) = pollster::block_on(inner.exiting(event_loop)) {
                tracing::error!("{e}");
                self.errors.push(e);
                event_loop.exit();
            }
        }
    }
}