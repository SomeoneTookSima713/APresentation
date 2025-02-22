//! Elements are the objects that get rendered on screen.

use std::any::TypeId;

use rhai::Engine;

pub mod property;

/// The main trait of this module.
/// 
/// An [`Element`] describes an object that gets rendered onto the screen.
pub trait Element {
    type Renderer: ElementRenderer;
}

/// *For internal use only!*
/// 
/// An dyn-safe variant of the [`Element`] trait that is automatically
/// implemented by any type implementing `Element`.
pub trait ElemObjS: downcast_rs::Downcast {
    fn renderer_tid(&self) -> TypeId;
}
downcast_rs::impl_downcast!(ElemObjS);

impl<T: Element + 'static> ElemObjS for T {
    fn renderer_tid(&self) -> TypeId {
        TypeId::of::<T::Renderer>()
    }
}

pub trait ElementRenderer {
    type Element: Element<Renderer = Self> + 'static;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized;

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {}

    fn render(
        &mut self,
        element: &Self::Element,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        render_pass: &mut wgpu::RenderPass
    );

    /// *For internal use only!*
    fn render_dyn(
        &mut self,
        element: &dyn ElemObjS,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        render_pass: &mut wgpu::RenderPass
    ) {
        if let Some(elem) = element.downcast_ref() {
            self.render(elem, eval_engine, eval_scope, render_pass);
        } else {
            panic!("Downcast in presentation::element::ElementRenderer::render_dyn() failed!")
        }
    }
}