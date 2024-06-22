use winit::application::ApplicationHandler;
use winit::event::{ DeviceEvent, DeviceId, WindowEvent };
use winit::event_loop::ActiveEventLoop;

use log::error;

use std::ops::{ Deref, DerefMut };

#[allow(unused)]
pub trait FallibleAppHandler<E> {
    async fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> { Ok(()) }

    async fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) -> anyhow::Result<()> { Ok(()) }

    async fn exiting(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> { Ok(()) }

    async fn memory_warning(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> { Ok(()) }

    async fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) -> anyhow::Result<()> { Ok(()) }

    async fn resumed(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()>;

    async fn suspended(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> { Ok(()) }

    async fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) -> anyhow::Result<()> { Ok(()) }

    async fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) -> anyhow::Result<()>;
}

pub struct FallibleAppHandlerWrap<E: 'static, T: FallibleAppHandler<E>>(T, std::marker::PhantomData<E>);

impl<E: 'static, T: FallibleAppHandler<E>> From<T> for FallibleAppHandlerWrap<E, T> {
    fn from(value: T) -> Self {
        FallibleAppHandlerWrap(value, std::marker::PhantomData)
    }
}

impl<E: 'static, T: FallibleAppHandler<E>> Deref for FallibleAppHandlerWrap<E, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: 'static, T: FallibleAppHandler<E>> DerefMut for FallibleAppHandlerWrap<E, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E: 'static, T: FallibleAppHandler<E>> FallibleAppHandler<E> for FallibleAppHandlerWrap<E, T> {
    async fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        self.0.about_to_wait(event_loop).await
    }

    async fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) -> anyhow::Result<()> {
        self.0.device_event(event_loop, device_id, event).await
    }

    async fn exiting(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        self.0.exiting(event_loop).await
    }

    async fn memory_warning(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        self.0.memory_warning(event_loop).await
    }
    
    async fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) -> anyhow::Result<()> {
        self.0.new_events(event_loop, cause).await
    }

    async fn resumed(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        self.0.resumed(event_loop).await
    }

    async fn suspended(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        self.0.suspended(event_loop).await
    }

    async fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) -> anyhow::Result<()> {
        self.0.user_event(event_loop, event).await
    }

    async fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) -> anyhow::Result<()> {
        self.0.window_event(event_loop, window_id, event).await
    }
}

impl<E: 'static, T: FallibleAppHandler<E>> ApplicationHandler<E> for FallibleAppHandlerWrap<E, T> {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::about_to_wait(self, event_loop)) {
            error!("Error in app handler's about_to_wait()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::device_event(self, event_loop, device_id, event)) {
            error!("Error in app handler's device_event()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::exiting(self, event_loop)) {
            let msg = format!("Error in app handler's exiting()-function: {e} Panicking to prevent infinite error-loop!");
            error!("{msg}");
            panic!("{msg}");
        }
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::memory_warning(self, event_loop)) {
            error!("Error in app handler's memory_warning()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::new_events(self, event_loop, cause)) {
            error!("Error in app handler's new_events()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::resumed(self, event_loop)) {
            error!("Error in app handler's resumed()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }
    
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::suspended(self, event_loop)) {
            error!("Error in app handler's suspended()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::user_event(self, event_loop, event)) {
            error!("Error in app handler's user_event()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
        if let Err(e) = pollster::block_on(<Self as FallibleAppHandler<E>>::window_event(self, event_loop, window_id, event)) {
            error!("Error in app handler's window_event()-function: {e} Exiting event loop...");
            event_loop.exit();
        }
    }
}