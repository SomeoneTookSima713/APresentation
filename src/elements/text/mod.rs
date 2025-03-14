use crate::presentation::{ asset, element, parser };

use asset::AssetManager;
use element::{ Element, ElementRenderer };
use element::property::{ Property, PropertyCompatible };
use element::property::base::{ BaseProperties, BasePropertiesProvider };
use parser::data::Value;

pub mod renderer;

pub use renderer::TextRenderer;

pub struct Text {
    base_properties: BaseProperties
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
            base_properties: structure.try_get_base_properties(engine)?
        })
    }
}