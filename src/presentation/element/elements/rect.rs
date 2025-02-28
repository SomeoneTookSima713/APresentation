use super::{ Element, ElementRenderer };
use super::property::{ Property, PropertyCompatible, alignment::Alignment };
use crate::presentation::parser::data::Value;

// Rect (
//     position: (100, "h * 0.5"),
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
    position: Property<(f64, f64)>,
    anchor: Property<Alignment>,
    alignment: Property<Alignment>,
    source: Property<RectSource>,
    corner_rounding: Property<CornerRounding>
}


#[derive(Clone, Copy)]
pub enum RectSource {
    Color(f64, f64, f64, f64),
    Image()
}

impl PropertyCompatible for RectSource {
    fn from_value(val: Value) -> Option<Self>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val && let Some(v) = v.map(Box::into_inner) {
            Some(match variant.as_str() {
                "Color" => {
                    let col = <(f64, f64, f64, f64) as PropertyCompatible>::from_value(v)?;
                    Self::Color(col.0, col.1, col.2, col.3)
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
}

#[derive(Clone, Copy)]
pub enum CornerRounding {
    Squircle(f64),
    Circle(f64)
}

impl PropertyCompatible for CornerRounding {
    fn from_value(val: Value) -> Option<Self>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val && let Some(v) = v.map(Box::into_inner) {
            Some(match variant.as_str() {
                "Squircle" => Self::Squircle(<(f64,) as PropertyCompatible>::from_value(v)?.0),
                "Circle" => Self::Circle(<(f64,) as PropertyCompatible>::from_value(v)?.0),
                _ => None?
            })
        } else {
            None
        }
    }
}

impl Element for Rect {
    type Renderer = RectRenderer;

    fn from_structure(structure: crate::presentation::parser::data::ParsedStructure, engine: &rhai::Engine) -> Option<Self>
    where Self: Sized {
        Some(Self {
            position: structure.try_get_property("position", engine)?,
            anchor: structure.try_get_property("anchor", engine)?,
            alignment: structure.try_get_property("alignment", engine)?,
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