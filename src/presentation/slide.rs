use std::collections::HashMap;
use std::marker::PhantomData;

use opengl_graphics::GlGraphics;
use graphics::Context;
use once_cell::sync::Lazy;

use crate::util::DefaultingOption;
use super::util::SimplestHasher;
use super::renderable;

fn DEFAULT_BACKGROUND_RENDERABLE<'a>() -> renderable::ColoredRect<'a> {
    renderable::ColoredRect::new("0;0", "w;h", "1;1;1;1", "TOP_LEFT")
}

pub struct Slide {
    objects: HashMap<u8, Vec<Box<dyn renderable::Renderable>>, SimplestHasher>,
    background: Box<dyn renderable::Renderable>
}

impl Slide {
    pub fn new<B: Into<DefaultingOption<Box<dyn renderable::Renderable>>>>(background: B) -> Slide {
        Slide { objects: HashMap::with_hasher(SimplestHasher::default()), background: <B as Into<DefaultingOption<Box<dyn renderable::Renderable>>>>::into(background).consume(Box::new(DEFAULT_BACKGROUND_RENDERABLE())) }
    }

    pub fn with_objects_ordered<B: Into<Box<dyn renderable::Renderable>>>(objects: HashMap<u8, Vec<B>>, background: B) -> Slide {
        Slide { objects: objects.into_iter().map(|(key, value)| (key, value.into_iter().map(|value| <B as Into<Box<dyn renderable::Renderable>>>::into(value)).collect())).collect(), background: background.into() }
    }

    pub fn with_objects_unordered<B: Into<Box<dyn renderable::Renderable>>>(vec: Vec<B>, background: B) -> Slide {
        let mut objects = HashMap::with_hasher(SimplestHasher::default());
        objects.insert(0, vec.into_iter().map(|value| <B as Into<Box<dyn renderable::Renderable>>>::into(value)).collect());
        Slide { objects, background: background.into() }
    }

    pub fn add<B: renderable::Renderable + 'static, Z: Into<DefaultingOption<u8>>>(&mut self, obj: B, z_index: Z) {
        let z = <Z as Into<DefaultingOption<u8>>>::into(z_index).consume(0);
        if self.objects.contains_key(&z) {
            self.objects.get_mut(&z).unwrap().push(Box::new(obj) as Box<dyn renderable::Renderable>)
        } else {
            self.objects.insert(z, vec![Box::new(obj) as Box<dyn renderable::Renderable>]);
        }
    }

    pub fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        self.background.render(time, context, opengl);
        for (_, vec) in self.objects.iter() {
            vec.iter().for_each(|renderable| renderable.render(time, context, opengl));
        }
    }
}