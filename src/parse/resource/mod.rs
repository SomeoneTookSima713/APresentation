use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::presentation::property::EvaluatedPropertyValue;

pub mod image;
pub mod font;

pub use image::Image;
pub use font::Font;

use crate::util::macros::resource::resources;

type FnResource = dyn Fn(&HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Box<dyn Resource>>;

thread_local! {
    pub static RESOURCE_TYPES: Lazy<HashMap<String, Box<FnResource>>> = resources!(
        Image,
        Font
    );
}

pub trait Resource: std::fmt::Debug {

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