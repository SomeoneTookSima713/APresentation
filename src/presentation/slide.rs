#![allow(dead_code)]

use std::collections::HashMap;

use opengl_graphics::GlGraphics;
use graphics::Context;

use crate::util::DefaultingOption;
use crate::presentation::util::SimplestHasher;
use crate::presentation::renderable;

use renderable::Renderable;

/// Constructs an instance of the default background for slides:
/// A white rectangle
#[allow(non_snake_case)]
fn DEFAULT_BACKGROUND_RENDERABLE<'a>() -> renderable::ColoredRect<'a> {
    renderable::ColoredRect::new("0;0", "w;h", "1;1;1;1", "TOP_LEFT")
}

/// Contains all the objects (including a background object) used for rendering a slide.
pub struct Slide {
    objects: HashMap<u8, Vec<Box<dyn Renderable>>, SimplestHasher>,
    background: Box<dyn Renderable>
}

impl Slide {
    /// Creates a new slide from an optional background object.
    /// 
    /// Either pass in a boxed [`Renderable`] or [`None`].
    pub fn new<B>(background: B) -> Slide
    where B: Into< DefaultingOption<Box<dyn Renderable>> >{
        let bg: DefaultingOption<Box<dyn Renderable>> = background.into();
        Slide {
            objects: HashMap::with_hasher(SimplestHasher::default()),
            background: bg.consume(Box::new(DEFAULT_BACKGROUND_RENDERABLE()))
        }
    }

    /// Creates a new slide from a hashmap containing layers of objects (sorted by z-index) and a background object.
    pub fn with_objects_ordered(objects: HashMap<u8, Vec<Box<dyn Renderable>>>, background: Box<dyn Renderable>) -> Slide {
        Slide {
            // Convert from HashMap<_, _, RandomState> to Hashmap<_, _, SimplestHasher>
            //   Requires a complete reconstruction of the map, as we need to recreate all hashes
            //   for the indexes when changing how the hash is computed.
            objects: objects.into_iter().collect(),
            background: background.into()
        }
    }

    /// Creates new slide from a vec containing objects and a background object.
    pub fn with_objects_unordered<B>(vec: Vec<Box<dyn Renderable>>, background: B) -> Slide
    where B: Into< Box<dyn Renderable> > {
        let mut objects = HashMap::with_hasher(SimplestHasher::default());
        objects.insert(0, vec);
        Slide { objects, background: background.into() }
    }

    /// Adds an object to the slide.
    pub fn add<B, Z>(&mut self, obj: B, z_index: Z)
    where
        B: Renderable + 'static,
        Z: Into< DefaultingOption<u8> > {
        // The user can pass anything to the function that can be converted into this type
        //   That could be a `u8`, this type directly or an `Option<u8>`.
        let z: DefaultingOption<u8> = z_index.into();
        if self.objects.contains_key(z.get(&0)) {
            // We can safely unwrap here as we know that the indexed entry exists because of the
            // if-statement
            self.objects.get_mut(z.get(&0)).unwrap().push(Box::new(obj) as Box<dyn Renderable>);
        } else {
            self.objects.insert(z.consume(0), vec![Box::new(obj) as Box<dyn Renderable>]);
        }
    }

    /// Adds a box of an object to the slide.
    /// 
    /// Useful when dealing with trait objects.
    pub fn add_boxed<Z>(&mut self, obj: Box<dyn renderable::Renderable>, z_index: Z)
    where Z: Into< DefaultingOption<u8> >{
        // See the `add()` function for some comments; this function is basically the same, just
        // without the conversion from implicit type to boxed trait

        let z: DefaultingOption<u8> = z_index.into();
        if self.objects.contains_key(z.get(&0)) {
            self.objects.get_mut(z.get(&0)).unwrap().push(obj)
        } else {
            self.objects.insert(z.consume(0), vec![obj]);
        }
    }

    /// Renders the slide.
    pub fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        // Render the background
        self.background.render(time, context, opengl);

        // Render all objects of the slide
        //   The order of objects when iterating needs to be based on the z-index, which is also
        //   used as an index to the `Vec`s. This order *should* be established through the
        //   `SimplestHasher` struct, I'll have to test that though.
        for (_, vec) in self.objects.iter() {
            vec.iter().for_each(|renderable| renderable.render(time, context, opengl));
        }
    }
}