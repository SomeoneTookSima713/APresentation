use super::*;

pub struct TextRenderer {}

impl ElementRenderer for TextRenderer {
    type Element = Text;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized {
        todo!()
    }

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        todo!()
    }

    fn submit_to_render(
        &mut self,
        element: &Self::Element,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        asset_manager: &AssetManager,
        render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn finish_render(&mut self, render_pass: &mut wgpu::RenderPass) {
        todo!()
    }
}