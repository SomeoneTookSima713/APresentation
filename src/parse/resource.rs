use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::presentation::property::EvaluatedPropertyValue;

type FnResource = dyn Fn(&HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Box<dyn Resource>>;

thread_local! {
    pub static RESOURCE_TYPES: Lazy<HashMap<String, Box<FnResource>>> = Lazy::new(|| [
        (Image::get_type(), Box::new(Image::create_boxed))
    ].into_iter().map(|(k,v)| (k.to_string(),v as Box<FnResource>)).collect::<HashMap<String, Box<FnResource>>>());
}

pub trait Resource {

    fn load(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()>;

    fn create(value: &HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Self>
    where Self: Sized;

    fn create_boxed(value: &HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Box<dyn Resource>>
    where Self: Sized + 'static {
        Self::create(value).map(|s| Box::new(s) as Box<dyn Resource>)
    }

    fn get_name(&self) -> &str;

    fn get_type() -> &'static str
    where Self: Sized;
}

pub trait ResourceExt {
    fn get_type_unsized(&self) -> &'static str;
}

impl<T: Resource> ResourceExt for T {
    fn get_type_unsized(&self) -> &'static str { T::get_type() }
}

pub struct Image {
    filepath: Rc<String>,
    name: Rc<String>,
    sampler: crate::render::texture::TextureSamplerSelection
}

impl Resource for Image {
    fn load(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        use crate::presentation::resource_managers::TEXTURE_MANAGER;
        use crate::render::texture;

        if self.name.as_str() == "DEFAULT" {
            anyhow::bail!("DEFAULT is a reserved resource name and cannot be used!")
        }

        let texture = texture::Texture::from_image(device, queue, self.filepath.as_str(), self.sampler, None)?;

        TEXTURE_MANAGER.insert(self.name.as_str().to_string(), Arc::new(texture))?;

        Ok(())
    }

    fn create(value: &HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Self> {
        let name = if let EvaluatedPropertyValue::String(s) = value.get("name").ok_or(anyhow::anyhow!("Name of Image unspecified!"))? {
            s.clone()
        } else {
            anyhow::bail!("'name' property needs to be of type String!")
        };
        let filepath = if let EvaluatedPropertyValue::String(s) = value.get("file").ok_or(anyhow::anyhow!("File path of Image unspecified! (Property name: 'file')"))? {
            s.clone()
        } else {
            anyhow::bail!("'file' property needs to be of type String!")
        };
        let sampler = if let EvaluatedPropertyValue::String(s) = value.get("sampler").cloned().unwrap_or(EvaluatedPropertyValue::String(std::rc::Rc::new("Linear".to_string()))) {
            use crate::render::texture::TextureSamplerSelection;
            match s.to_lowercase().as_str() {
                "linear" => TextureSamplerSelection::Linear,
                "pixelperfect" | "pixel_perfect" => TextureSamplerSelection::PixelPerfect,
                _ => anyhow::bail!("Specified texture sampler doesn't exist!")
            }
        } else {
            anyhow::bail!("'sampler' property needs to be of type String!")
        };

        Ok(Self { filepath, name, sampler })
    }

    fn get_name(&self) -> &str { self.name.as_str() }

    fn get_type() -> &'static str { "Image" }
}