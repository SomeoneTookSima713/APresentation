use std::any::TypeId;

use downcast_rs::{ Downcast, impl_downcast };

use super::*;

pub trait RenderableObjectSafe: Downcast {
    /// This function gets called at the beginning of a new frame. It's
    /// intended purpose is to be used to delete the caches on any
    /// [`TypedProperty`]s used in the renderable.
    fn begin_new_frame(&self) -> anyhow::Result<()>;

    fn render(&self, rendering_manager: &mut dyn RenderableRenderingManagerObjectSafe, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()>;

    fn to_parseable(&self) -> anyhow::Result<ParseableRenderable<'static>>;

    fn get_type(&self) -> TypeId;
}
impl_downcast!(RenderableObjectSafe);

pub trait RenderableRenderingManagerObjectSafe {
    fn submit_instance(&mut self, instance: &dyn RenderableObjectSafe, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()>;

    fn render_instances(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) -> anyhow::Result<wgpu::RenderBundle>;
}

impl<T: Renderable + 'static> RenderableObjectSafe for T {
    fn begin_new_frame(&self) -> anyhow::Result<()> { <Self as Renderable>::begin_new_frame(self) }

    fn render(&self, rendering_manager: &mut dyn RenderableRenderingManagerObjectSafe, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()> {
        <Self as Renderable>::render(self, rendering_manager, args)
    }

    fn to_parseable(&self) -> anyhow::Result<ParseableRenderable<'static>> {
        <Self as Renderable>::to_parseable(self)
    }

    fn get_type(&self) -> TypeId { TypeId::of::<T>() }
}

impl<T: RenderableRenderingManager> RenderableRenderingManagerObjectSafe for T
where <T as RenderableRenderingManager>::Renderable: 'static {
    fn submit_instance(&mut self, instance: &dyn RenderableObjectSafe, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()> {
        if instance.get_type() == Self::get_type_id() {
            <Self as RenderableRenderingManager>::submit_instance(self, instance.downcast_ref().ok_or(anyhow::anyhow!("Downcast failed!"))?, args)
        } else {
            anyhow::bail!("Wrong type of Renderable!")
        }
    }

    fn render_instances(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) -> anyhow::Result<wgpu::RenderBundle> {
        <Self as RenderableRenderingManager>::render_instances(self, device, queue, camera)
    }
}