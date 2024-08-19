use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{
    Renderable,
    RenderableRenderingManager,
    RenderableRenderingManagerObjectSafe,
    ParseableRenderable,
    Camera,
};
use crate::presentation::property::{
    Property,
    PropertyStructure,
    PropertyCompatible,
    EvaluatedPropertyValue,
    TypedProperty,
};
use crate::render::font::Font;

mod markdown;

bitflags::bitflags! {
    struct SyntheticStyle: u8 {
        const BOLD = 0b00000001;
        const ITALIC = 0b00000010;

        const _ = 0b00000011;
    }
}

impl Default for SyntheticStyle {
    fn default() -> Self { Self::empty() }
}

struct TextPart {
    text: String,
    font: Font,
    synthetic_style: SyntheticStyle
}

impl<'lua> PropertyCompatible<'lua> for TextPart {
    const STRUCTURE: PropertyStructure = PropertyStructure::Dict(&[
        ("text", PropertyStructure::String(None)),
        ("font", PropertyStructure::String(None)),
        // ("synthetic_style", PropertyStructure::List(&[PropertyStructure::String(Some("Bold|bold|Italic|italic"))])) // Optional, so commented out
    ]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: crate::presentation::property::Property<'lua>, args: A) -> anyhow::Result<Self>
    where Self: Sized {
        match value.get_recursively(args)? {
            EvaluatedPropertyValue::Dict(d) => {
                let dict = d.borrow();

                let text = if let Some(val) = dict.get("text") {
                    match val {
                        EvaluatedPropertyValue::String(s) => (**s).clone(),
                        _ => anyhow::bail!("Property 'text' has invalid structure! (Expected String)")
                    }
                } else {
                    anyhow::bail!("Required Property 'text' is missing!")
                };

                let fontref = if let Some(val) = dict.get("font") {
                    match val {
                        EvaluatedPropertyValue::String(s) => (**s).clone(),
                        _ => anyhow::bail!("Property 'font' has invalid structure! (Expected String)")
                    }
                } else {
                    anyhow::bail!("Required Property 'font' is missing!")
                };

                

                let synthetic_style = if let Some(val) = dict.get("synthetic_style") {
                    match val {
                        EvaluatedPropertyValue::List(l) => {
                            // Tries to convert all elements in the list to flags 
                            l.borrow().iter().map(|val| match val {
                                EvaluatedPropertyValue::String(s) => Ok(s.clone()),
                                _ => Err(anyhow::anyhow!("Item in property 'synthetic_style' invalid! (Expected String)"))
                            }).enumerate().map(|(i, res)| if let Ok(string) = res {
                                match string.to_lowercase().as_str() {
                                    "bold" => Ok(SyntheticStyle::BOLD),
                                    "italic" => Ok(SyntheticStyle::ITALIC),
                                    s => Err(anyhow::anyhow!("Item #{} in property 'synthetic_style' invalid! (Expected \"Bold\" or \"Italic\", got \"{s}\")", i+1))
                                }
                            } else { res.map(|_| unreachable!()) }).try_reduce(|acc, e| Ok::<_,anyhow::Error>(acc.and_then(|acc_flag| e.map(|extra_flag| acc_flag | extra_flag))))?.unwrap_or(Ok(Default::default()))?
                        },
                        _ => anyhow::bail!("Property 'synthetic_style' has invalid structure! (Expected List of Strings)")
                    }
                } else {
                    SyntheticStyle::empty()
                };

                todo!()
            },
            _ => anyhow::bail!("Invalid Property!")
        }
    }

    fn convert_into(&'lua self) -> crate::presentation::property::Property<'lua> {
        todo!()
    }

    fn move_into(self) -> crate::presentation::property::Property<'lua>
    where Self: Sized {
        todo!()
    }
}

#[derive(Debug)]
pub struct Text {
}

impl Text {
    fn parse(markdown: &str) {
    }
}

impl Renderable for Text {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)] = &[];

    fn from_parseable(parseable: ParseableRenderable<'static>) -> anyhow::Result<Self>
    where Self: Sized {
        Ok(Self {  })
    }

    fn to_parseable(&self) -> anyhow::Result<ParseableRenderable<'static>> {
        Ok(ParseableRenderable::new(Rc::new(RefCell::new(HashMap::new()))))
    }

    fn begin_new_frame(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn render<M>(&self, rendering_manager: &mut M, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()>
    where
        M: RenderableRenderingManagerObjectSafe + ?Sized
    {
        Ok(())
    }
}

pub struct TextRenderingManager;

impl RenderableRenderingManager for TextRenderingManager {
    type Renderable = Text;

    fn instantiate(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self>
    where Self: Sized {
        Ok(Self)
    }

    fn render_instances(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) -> anyhow::Result<wgpu::RenderBundle> {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb)],
            depth_stencil: None,
            sample_count: 1,
            multiview: None
        });

        Ok(encoder.finish(&wgpu::RenderBundleDescriptor { label: None }))
    }

    fn submit_instance<A: mlua::IntoLuaMulti<'static> + Clone>(&mut self, instance: &Self::Renderable, args: A) -> anyhow::Result<()> {
        Ok(())
    }
}