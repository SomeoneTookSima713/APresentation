use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::super::{ Property, PropertyCompatible, PropertyStructure, EvaluatedPropertyValue, PropertyValue, TypedProperty };

use crate::texture;
use crate::util::math::Rect;

fn prop_to_num(num: &EvaluatedPropertyValue) -> Option<f32> {
    match num {
        EvaluatedPropertyValue::Float(f) => Some(*f as f32),
        EvaluatedPropertyValue::Int(i) => Some(*i as f32),
        EvaluatedPropertyValue::UInt(u) => Some(*u as f32),
        _ => None
    }
}

#[derive(Clone, Debug)]
pub struct Image<'lua> {
    source: Rc<String>,
    rect: TypedProperty<'lua, Rect>,
    sampler: texture::TextureSamplerSelection
}

impl<'lua> Image<'lua> {
    pub fn new(source: Rc<String>, rect: TypedProperty<'lua, Rect>, sampler: texture::TextureSamplerSelection) -> Self {
        Self { source, rect, sampler }
    }

    pub fn new_constant<R: Into<Rect>>(source: Rc<String>, rect: R, sampler: texture::TextureSamplerSelection) -> Self {
        Self { source, rect: TypedProperty::new(<R as Into<Rect>>::into(rect).move_into()).unwrap(), sampler }
    }

    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    pub fn rect(&self) -> &TypedProperty<'lua, Rect> {
        &self.rect
    }

    pub fn rect_evaluated<A: Clone + mlua::IntoLuaMulti<'lua>>(&mut self, args: A) -> anyhow::Result<std::cell::Ref<Rect>> {
        self.rect.get(args)
    }

    pub fn sampler(&self) -> texture::TextureSamplerSelection {
        self.sampler
    }
}

impl<'lua> PropertyCompatible<'lua> for Image<'lua> {
    const STRUCTURE: PropertyStructure = PropertyStructure::Dict(&[
        ("source", PropertyStructure::String(None))
    ]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args.clone())? {
            PropertyValue::Dict(d) => {
                let dict = d.borrow();

                let source = if let Some(val) = dict.get("source") {
                    match val.get(args.clone())? {
                        PropertyValue::String(s) => s.clone(),
                        _ => anyhow::bail!("Property 'source' has invalid structure! (Expected String)")
                    }
                } else {
                    anyhow::bail!("Required property 'source' is missing!")
                };

                let rect = if let Some(val) = dict.get("rect") {
                    TypedProperty::new(val.clone())?
                } else {
                    TypedProperty::new(Rect::new_vals(0.0, 0.0, 1.0, 1.0).move_into())?
                };

                let sampler = if let Some(val) = dict.get("sampler") {
                    texture::TextureSamplerSelection::convert_from(val.clone(), args.clone())?
                } else {
                    texture::TextureSamplerSelection::Linear
                };

                Ok(Self { source, rect, sampler })
            },
            _ => anyhow::bail!("Invalid property!")
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(crate::presentation::property::PropertyValue::Dict(Rc::new(RefCell::new(HashMap::from([
            ("source", Property::Constant(crate::presentation::property::PropertyValue::String(self.source.clone()))),
            ("sampler", self.sampler.convert_into()),
            ("rect", self.rect.convert_into())
        ].map(|(k,v)| (k.to_string(), v)))))))
    }

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        Property::Constant(crate::presentation::property::PropertyValue::Dict(Rc::new(RefCell::new(HashMap::from([
            ("source", Property::Constant(crate::presentation::property::PropertyValue::String(self.source.clone()))),
            ("sampler", self.sampler.move_into()),
            ("rect", self.rect.move_into())
        ].map(|(k,v)| (k.to_string(), v)))))))
    }
}