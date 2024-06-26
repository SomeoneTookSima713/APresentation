use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::presentation::property::{ TypedProperty, PropertyCompatible, PropertyStructure, Property, PropertyValue, Alignment, common_properties };
use crate::parse::ParseableRenderable;
use crate::render::camera::Camera;

pub mod rect;

pub mod object_safe;
pub use object_safe::*;

pub const fn extended_structure<T: Renderable, const B: usize>(new: [(&'static str, PropertyStructure); B]) -> [(&'static str, PropertyStructure); T::PROPERTY_STRUCTURE_SIZE + B]
where [(&'static str, PropertyStructure); T::PROPERTY_STRUCTURE_SIZE + B]: Sized {
    crate::util::extended_slice::<{ T::PROPERTY_STRUCTURE_SIZE }, B, (&'static str, PropertyStructure)>(sized_property_structure::<T>(), new)
}

pub trait Renderable: std::fmt::Debug {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)];

    /// Please don't define this yourself, things **will** break if you do.
    const PROPERTY_STRUCTURE_SIZE: usize = { Self::PROPERTY_STRUCTURE.len() };

    /// This function gets called at the beginning of a new frame. It's
    /// intended purpose is to be used to delete the caches on any
    /// [`TypedProperty`]s used in the renderable.
    fn begin_new_frame(&self) -> anyhow::Result<()>;

    fn render<M>(&self, rendering_manager: &mut M, args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()>
    where
        M: RenderableRenderingManagerObjectSafe + ?Sized;

    fn to_parseable(&self) -> anyhow::Result<ParseableRenderable<'static>>;

    fn from_parseable(parseable: ParseableRenderable<'static>) -> anyhow::Result<Self>
    where Self: Sized;

    fn from_parseable_boxed(parseable: ParseableRenderable<'static>) -> anyhow::Result<Box<dyn RenderableObjectSafe>>
    where Self: Sized + 'static {
        Self::from_parseable(parseable).map(|s| Box::new(s) as Box<dyn RenderableObjectSafe>)
    }
}

pub trait RenderableRenderingManager {
    type Renderable: Renderable;

    fn instantiate(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self>
    where Self: Sized;

    fn submit_instance<A: mlua::IntoLuaMulti<'static> + Clone>(&mut self, instance: &Self::Renderable, args: A) -> anyhow::Result<()>;

    fn render_instances(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) -> anyhow::Result<wgpu::RenderBundle>;
}

pub(self) trait RRMExt {
    fn get_type_id() -> std::any::TypeId;
}

impl<T: RenderableRenderingManager> RRMExt for T
where <Self as RenderableRenderingManager>::Renderable: 'static {
    fn get_type_id() -> std::any::TypeId { std::any::TypeId::of::<<Self as RenderableRenderingManager>::Renderable>() }
}

const fn sized_property_structure<T: Renderable>() -> &'static [(&'static str, PropertyStructure); T::PROPERTY_STRUCTURE_SIZE]
where [(); T::PROPERTY_STRUCTURE_SIZE]: Sized {
    unsafe { &*T::PROPERTY_STRUCTURE.as_ptr().cast() }
}

impl<T: Renderable + 'static> PropertyCompatible<'static> for T {
    const STRUCTURE: PropertyStructure = PropertyStructure::Dict(T::PROPERTY_STRUCTURE);

    fn convert_from<A: mlua::IntoLuaMulti<'static> + Clone>(value: Property<'static>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::Dict(dict) => {
                Self::from_parseable(ParseableRenderable::new(dict.clone()))
            },
            _ => Err(anyhow::anyhow!("Invalid Property!"))
        }
    }

    fn convert_into(&self) -> Property<'static> {
        // TODO: Maybe change this function signature to allow errors?
        Property::Constant(self.to_parseable().unwrap().into_property_value())
    }

    fn move_into(self) -> Property<'static>
    where Self: Sized {
        Property::Constant(self.to_parseable().unwrap().into_property_value())
    }
}

#[derive(Debug)]
pub struct BaseProperties {
    position: TypedProperty<'static, (f64, f64)>,
    color: TypedProperty<'static, common_properties::Color>,
    alignment: TypedProperty<'static, Alignment>,
    z_index: TypedProperty<'static, u16>
}

impl BaseProperties {
    pub fn new(position: TypedProperty<'static, (f64, f64)>, color: TypedProperty<'static, common_properties::Color>, alignment: TypedProperty<'static, Alignment>, z_index: TypedProperty<'static, u16>) -> Self {
        Self { position, color, alignment, z_index }
    }
}

impl Renderable for BaseProperties {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)] = &[
        ("position", <(f64, f64) as PropertyCompatible<'static>>::STRUCTURE),
        ("color", common_properties::Color::STRUCTURE),
        ("alignment", <Alignment as PropertyCompatible<'static>>::STRUCTURE),
        ("z_index", u16::STRUCTURE),
    ];

    fn from_parseable(parseable: ParseableRenderable<'static>) -> anyhow::Result<Self>
    where Self: Sized {

        let get = |index: &'static str| -> anyhow::Result<Property<'static>> {
            parseable.get(index).map(|r|r.clone()).ok_or(anyhow::anyhow!("Invalid Renderable! (missing property \"{index}\")"))
        };

        Ok(Self {
            position: TypedProperty::new((get)("position")?).map_err(|e|anyhow::anyhow!("Error parsing property 'position': {e}"))?,
            color: TypedProperty::new((get)("color")?).map_err(|e|anyhow::anyhow!("Error parsing property 'color': {e}"))?,
            alignment: TypedProperty::new((get)("alignment")?).map_err(|e|anyhow::anyhow!("Error parsing property 'alignment': {e}"))?,
            z_index: TypedProperty::new((get)("z_index")?).map_err(|e|anyhow::anyhow!("Error parsing property 'z_index': {e}"))?,
        })
    }

    fn to_parseable(&self) -> anyhow::Result<ParseableRenderable<'static>> {
        Ok(ParseableRenderable::new(Rc::new(RefCell::new(HashMap::from([
            ("position".to_string(), self.position.clone().move_into()),
            ("color".to_string(), self.color.clone().move_into()),
            ("alignment".to_string(), self.alignment.clone().move_into()),
            ("z_index".to_string(), self.alignment.clone().move_into()),
        ])))))
    }

    fn begin_new_frame(&self) -> anyhow::Result<()> {
        if !self.alignment.delete_cache() {
            anyhow::bail!("Couldn't delete alignment's TypedProperty cache in BaseProperties!");
        }
        if !self.color.delete_cache() {
            anyhow::bail!("Couldn't delete color's TypedProperty cache in BaseProperties!");
        }
        if !self.position.delete_cache() {
            anyhow::bail!("Couldn't delete position's TypedProperty cache in BaseProperties!");
        }
        Ok(())
    }

    fn render<M>(&self, _rendering_manager: &mut M, _args: mlua::Variadic<mlua::Value<'static>>) -> anyhow::Result<()>
    where
        M: RenderableRenderingManagerObjectSafe + ?Sized
    {
        anyhow::bail!("Can't render a set of properties!");
    }
}

pub fn register_rendering_managers(hm: &mut HashMap<std::any::TypeId, Box<dyn RenderableRenderingManagerObjectSafe>>, device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) {
    use std::any::TypeId;
    
    hm.insert(TypeId::of::<rect::Rectangle>(), Box::new(rect::RectangleRenderer::instantiate(device, queue, surface_config).unwrap()));
}

use once_cell::sync::Lazy;
pub static RENDERABLES: Lazy<HashMap<String, fn(ParseableRenderable<'static>) -> anyhow::Result<Box<dyn RenderableObjectSafe>>>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    register_renderables(&mut hm);
    hm
});

pub fn register_renderables(hm: &mut HashMap<String, fn(ParseableRenderable<'static>) -> anyhow::Result<Box<dyn RenderableObjectSafe>>>) {
    hm.insert("Rect".to_string(), rect::Rectangle::from_parseable_boxed);
}