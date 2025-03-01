use crate::presentation::{ asset, element, parser };

use element::{ Element, ElementRenderer };
use element::property::{ Property, PropertyCompatible };
use element::property::base::{ BaseProperties, BasePropertiesProvider };
use parser::data::Value;

pub mod assets;
pub use assets::*;

// Example definition:
// Rect (
//     position: (100, Rhai("h * 0.5")),
//     anchor: TopRight,
//     alignment: MidRight,
//     source: Color(1.0, 0.5, 0.0, 1.0),
//     corner_rounding: Circle(
//         top_left: 1.0,
//         top_right: 0.5,
//         bottom_left: 1.0,
//         bottom_right: 0.5
//     )
// )

pub struct Rect {
    base_properties: BaseProperties,
    source: Property<RectSource>,
    corner_rounding: Property<CornerRounding>
}


#[derive(Clone, Copy)]
pub enum RectSource {
    Color(f64, f64, f64, f64),
    Image()
}

pub enum UncomputedRectSource {
    Color([Property<f64>; 4]),
    Image()
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
                    todo!()
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
            UncomputedRectSource::Image() => todo!()
        })
    }
}

#[derive(Clone, Copy)]
pub enum CornerRounding {
    Squircle {
        top_left: f64,
        top_right: f64,
        bottom_left: f64,
        bottom_right: f64,
    },
    Circle {
        top_left: f64,
        top_right: f64,
        bottom_left: f64,
        bottom_right: f64,
    }
}

pub enum UncomputedCornerRounding {
    Squircle([Property<f64>; 4]),
    Circle([Property<f64>; 4])
}

impl PropertyCompatible for CornerRounding {
    type InnerRepresentation = UncomputedCornerRounding;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val && let Some(Value::Map(map)) = v.map(Box::into_inner) {
            Some(match variant.as_str() {
                "Squircle" => UncomputedCornerRounding::Squircle([
                    Property::from_value(map.get("top_left")?.clone(), engine)?,
                    Property::from_value(map.get("top_right")?.clone(), engine)?,
                    Property::from_value(map.get("bottom_left")?.clone(), engine)?,
                    Property::from_value(map.get("bottom_right")?.clone(), engine)?,
                ]),
                "Circle" => UncomputedCornerRounding::Circle([
                    Property::from_value(map.get("top_left")?.clone(), engine)?,
                    Property::from_value(map.get("top_right")?.clone(), engine)?,
                    Property::from_value(map.get("bottom_left")?.clone(), engine)?,
                    Property::from_value(map.get("bottom_right")?.clone(), engine)?,
                ]),
                _ => None?
            })
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(match base {
            UncomputedCornerRounding::Squircle([tl, tr, bl, br]) => Self::Squircle {
                top_left: tl.evaluate(scope, engine)?,
                top_right: tr.evaluate(scope, engine)?,
                bottom_left: bl.evaluate(scope, engine)?,
                bottom_right: br.evaluate(scope, engine)?
            },
            UncomputedCornerRounding::Circle([tl, tr, bl, br]) => Self::Circle {
                top_left: tl.evaluate(scope, engine)?,
                top_right: tr.evaluate(scope, engine)?,
                bottom_left: bl.evaluate(scope, engine)?,
                bottom_right: br.evaluate(scope, engine)?
            }
        })
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
            source: structure.try_get_property("source", engine)?,
            corner_rounding: structure.try_get_property("corner_rounding", engine)?
        })
    }
}

pub struct RectRenderer {

}

impl ElementRenderer for RectRenderer {
    type Element = Rect;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized {
        todo!()
    }

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        todo!()
    }

    fn render(
            &mut self,
            element: &Self::Element,
            eval_engine: &rhai::Engine,
            eval_scope: rhai::Scope<'static>,
            render_pass: &mut wgpu::RenderPass
    ) {
        todo!()
    }
}