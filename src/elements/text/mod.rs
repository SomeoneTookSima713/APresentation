use crate::presentation::{ asset, element, parser };

use asset::AssetManager;
use element::{ Element, ElementRenderer };
use element::property::{ Property, PropertyCompatible };
use element::property::base::{ BaseProperties, BasePropertiesProvider };
use parser::data::Value;

pub mod assets;
pub mod renderer;
pub mod text_parser;

pub use renderer::TextRenderer;
pub use text_parser::TextProperty;

pub struct Text {
    base_properties: BaseProperties,
    text: Property<TextProperty>,
    size: Property<Option<(f64, f64)>>
}

impl BasePropertiesProvider for Text {
    fn get_base_properties(&self) -> &BaseProperties {
        &self.base_properties
    }
}

impl Element for Text {
    type Renderer = TextRenderer;

    fn from_structure(structure: parser::data::ParsedStructure, engine: &rhai::Engine) -> anyhow::Result<Self>
    where Self: Sized {
        Ok(Self {
            base_properties: structure.try_get_base_properties(engine)?,
            text: structure.try_get_property("text", engine)?,
            size: structure.try_get_property("size", engine).unwrap_or(Property::Constant(None))
        })
    }
}