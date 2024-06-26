use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::presentation::property::EvaluatedPropertyValue;

pub mod image;
// pub mod font;

pub use image::Image;

type FnResource = dyn Fn(&HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Box<dyn Resource>>;

thread_local! {
    pub static RESOURCE_TYPES: Lazy<HashMap<String, Box<FnResource>>> = Lazy::new(|| [
        (Image::get_type(), Box::new(Image::create_boxed))
    ].into_iter().map(|(k,v)| (k.to_string(),v as Box<FnResource>)).collect::<HashMap<String, Box<FnResource>>>());
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