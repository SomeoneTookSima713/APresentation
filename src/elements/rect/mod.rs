use std::sync::OnceLock;

use crate::PUSH_CONSTANT_MANAGER;
use crate::presentation::{ asset, element, parser };

use asset::AssetManager;
use element::{ Element, ElementRenderer };
use element::property::{ Property, PropertyCompatible };
use element::property::base::{ BaseProperties, BasePropertiesProvider };
use parser::data::Value;

pub mod assets;
pub mod renderer;
pub use assets::*;

pub use renderer::RectRenderer;

pub struct Rect {
    base_properties: BaseProperties,
    rotation: Property<f64>,
    size: Property<(f64, f64)>,
    source: Property<RectSource>,
    corner_rounding: Property<CornerRounding>
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SamplerType {
    Linear,
    Nearest
}

impl PropertyCompatible for SamplerType {
    type InnerRepresentation = Self;

    fn from_value(val: Value, _engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::EnumVariant(variant, None) = val {
            match variant.as_str() {
                "Linear" => Some(Self::Linear),
                "Nearest" => Some(Self::Nearest),
                _ => None
            }
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, _engine: &rhai::Engine, _scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(*base)
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("Linear", || Ok(Self::Linear));
        module.set_native_fn("Nearest", || Ok(Self::Nearest));
        module.set_custom_type::<Self>("SamplerType");
        Some(("SamplerType".to_string(), module))
    }
}

#[derive(Clone)]
pub enum RectSource {
    Color(f64, f64, f64, f64),
    Image(String, SamplerType)
}

pub enum UncomputedRectSource {
    Color([Property<f64>; 4]),
    Image(Property<String>, Property<SamplerType>)
}

impl PropertyCompatible for RectSource {
    type InnerRepresentation = UncomputedRectSource;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val && let Some(v) = v.map(Box::into_inner) {
            Some(match variant.as_str() {
                "Color" => {
                    let col = <(f64, f64, f64, f64) as PropertyCompatible>::from_value(v, engine)?;
                    UncomputedRectSource::Color([col.0, col.1, col.2, col.3])
                },
                "Image" => {
                    let p = <(String, SamplerType) as PropertyCompatible>::from_value(v, engine)?;
                    UncomputedRectSource::Image(p.0, p.1)
                },
                _ => None?
            })
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(match base {
            UncomputedRectSource::Color([r, g, b, a]) => Self::Color(
                r.evaluate(scope, engine)?,
                g.evaluate(scope, engine)?,
                b.evaluate(scope, engine)?,
                a.evaluate(scope, engine)?
            ),
            UncomputedRectSource::Image(src, sampler) => Self::Image(src.evaluate(scope, engine)?, sampler.evaluate(scope, engine)?)
        })
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("Color", |r: f64, g: f64, b: f64, a: f64| Ok(Self::Color(r, g, b, a)));
        module.set_native_fn("Image", |src: rhai::ImmutableString, sampler: SamplerType| Ok(Self::Image(src.into_owned(), sampler)));
        Some(("RectSource".to_string(), module))
    }
}

#[derive(Clone, Copy, Default)]
pub struct CornerRounding {
    top_left: f64,
    top_right: f64,
    bottom_left: f64,
    bottom_right: f64,
}

pub struct UncomputedCornerRounding([Property<f64>; 4]);

impl PropertyCompatible for CornerRounding {
    type InnerRepresentation = UncomputedCornerRounding;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::Map(map) = val {
            Some(UncomputedCornerRounding([
                Property::from_value(map.get("top_left")?.clone(), engine)?,
                Property::from_value(map.get("top_right")?.clone(), engine)?,
                Property::from_value(map.get("bottom_left")?.clone(), engine)?,
                Property::from_value(map.get("bottom_right")?.clone(), engine)?,
            ]))
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(Self {
            top_left: base.0[0].evaluate(scope, engine)?,
            top_right: base.0[1].evaluate(scope, engine)?,
            bottom_left: base.0[2].evaluate(scope, engine)?,
            bottom_right: base.0[3].evaluate(scope, engine)?
        })
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("new", |tl: f64, tr: f64, bl: f64, br: f64| Ok(Self { top_left: tl, top_right: tr, bottom_left: bl, bottom_right: br }));
        Some(("CornerRounding".to_string(), module))
    }
}

impl BasePropertiesProvider for Rect {
    fn get_base_properties(&self) -> &BaseProperties { &self.base_properties }
}

impl Element for Rect {
    type Renderer = RectRenderer;

    fn from_structure(structure: crate::presentation::parser::data::ParsedStructure, engine: &rhai::Engine) -> anyhow::Result<Self>
    where Self: Sized {
        Ok(Self {
            base_properties: structure.try_get_base_properties(engine)?,
            rotation: structure.try_get_property("rotation", engine).unwrap_or(Property::Constant(0.0)),
            size: structure.try_get_property("size", engine)?,
            source: structure.try_get_property("source", engine)?,
            corner_rounding: structure.try_get_property("corner_rounding", engine).unwrap_or(Property::Constant(CornerRounding::default()))
        })
    }
}