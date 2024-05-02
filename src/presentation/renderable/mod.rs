use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::presentation::property::{ TypedProperty, PropertyCompatible, PropertyStructure, Property, PropertyValue, Alignment };
use crate::parse::ParseableRenderable;

pub mod rect;

pub trait Renderable<'lua> {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)];

    /// Please don't define this yourself, things **will** break if you do.
    const PROPERTY_STRUCTURE_SIZE: usize = { Self::PROPERTY_STRUCTURE.len() };

    fn render(&'lua self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()>;

    fn to_parseable(&'lua self) -> anyhow::Result<ParseableRenderable<'lua>>;

    fn from_parseable(parseable: ParseableRenderable<'lua>) -> anyhow::Result<Self>
    where Self: Sized;
}

const fn sized_property_structure<'lua, T: Renderable<'lua>>() -> &'static [(&'static str, PropertyStructure); T::PROPERTY_STRUCTURE_SIZE]
where [(); T::PROPERTY_STRUCTURE_SIZE]: Sized {
    unsafe { &*T::PROPERTY_STRUCTURE.as_ptr().cast() }
}

impl<'lua, T: Renderable<'lua>> PropertyCompatible<'lua> for T {
    const STRUCTURE: PropertyStructure = PropertyStructure::Dict(T::PROPERTY_STRUCTURE);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::Dict(dict) => {
                Self::from_parseable(ParseableRenderable::new(dict.clone()))
            },
            _ => Err(anyhow::anyhow!("Invalid Property!"))
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        // TODO: Maybe change this function signature to allow errors?
        Property::Constant(self.to_parseable().unwrap().into_property_value())
    }
}

pub struct BaseProperties<'lua> {
    position: TypedProperty<'lua, (f64, f64)>,
    color: TypedProperty<'lua, [f64; 4]>,
    alignment: TypedProperty<'lua, Alignment>,
}

impl<'lua> Renderable<'lua> for BaseProperties<'lua> {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)] = &[
        ("position", <(f64, f64) as PropertyCompatible<'lua>>::STRUCTURE),
        ("color", <[f64; 4] as PropertyCompatible<'lua>>::STRUCTURE),
        ("alignment", <Alignment as PropertyCompatible<'lua>>::STRUCTURE)
    ];

    fn from_parseable(parseable: ParseableRenderable<'lua>) -> anyhow::Result<Self>
    where Self: Sized {

        let get = |index: &'static str| -> anyhow::Result<Property<'lua>> {
            parseable.get(index).map(|r|r.clone()).ok_or(anyhow::anyhow!("Invalid Property!"))
        };

        Ok(Self {
            position: TypedProperty::new((get)("position")?)?,
            color: TypedProperty::new((get)("color")?)?,
            alignment: TypedProperty::new((get)("alignment")?)?
        })
    }

    fn to_parseable(&'lua self) -> anyhow::Result<ParseableRenderable<'lua>> {
        Ok(ParseableRenderable::new(Rc::new(RefCell::new(HashMap::from([
            ("position".to_string(), self.position.convert_into()),
            ("color".to_string(), self.color.convert_into()),
            ("alignment".to_string(), self.alignment.convert_into())
        ])))))
    }

    fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        anyhow::bail!("Can't render a set of properties!");
    }
}

pub const fn extended_structure<'lua, T: Renderable<'lua>, const B: usize>(new: [(&'static str, PropertyStructure); B]) -> [(&'static str, PropertyStructure); T::PROPERTY_STRUCTURE_SIZE + B]
where [(&'static str, PropertyStructure); T::PROPERTY_STRUCTURE_SIZE + B]: Sized {
    crate::util::extended_slice::<{ T::PROPERTY_STRUCTURE_SIZE }, B, (&'static str, PropertyStructure)>(sized_property_structure::<T>(), new)
}