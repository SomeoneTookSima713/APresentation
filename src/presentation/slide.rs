#![allow(dead_code)]

use std::collections::HashMap;

use opengl_graphics::GlGraphics;
use graphics::Context;
use indexmap::IndexMap;

use crate::util::DefaultingOption;
use crate::presentation::renderable;

use renderable::{ Renderable, BaseProperties };

use once_cell::sync::Lazy;
/// An instance of the default background for slides:
/// A white rectangle
pub const DEFAULT_BACKGROUND_RENDERABLE: Lazy<renderable::ColoredRect> = Lazy::new(|| {
    use crate::presentation::util::PropertyError;

    let err_begin = "Error instantiating default slide background:";
    let err_end = "This should not happen! Please report this on the issue tracker!";

    let properties = match BaseProperties::new("0;0", "w;h", "1;1;1;1", "TOP_LEFT") {
        Ok(p) => p,
        Err(e) => match e {
            PropertyError::BadAlignment => panic!("{err_begin} Invalid alignment! {err_end}"),
            PropertyError::MismatchedExprCount => panic!("{err_begin} Invalid expression count! {err_end}"),
            PropertyError::SyntaxError(_, prop, spec) => panic!("{err_begin} Error in field {prop}: {} {err_end}",spec.unwrap_or("No furhter information given.".to_owned())),
            PropertyError::LuaError(e) => panic!("{err_begin} Lua error: {e} {err_end}"),
            PropertyError::MultiError(e) => panic!("{err_begin} Multiple errors occured (probably while parsing an expression): \n{:#?}\n {err_end}",e.iter().map(|e|{
                let (a,b,c) = e.syntax_error("Unknown", "Unknown", "No Description");
                format!("Renderable: {a} | Property: {b} | Error description: {c}")
            }).collect::<Vec<_>>()),
        }
    };
    renderable::ColoredRect::new(properties)
});

/// Contains all the objects (including a background object) used for rendering a slide.
pub struct Slide {
    objects: IndexMap<u8, Vec<Box<dyn Renderable>>>,
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
            objects: IndexMap::new(),
            background: bg.consume(Box::new(DEFAULT_BACKGROUND_RENDERABLE.clone()))
        }
    }

    /// Creates a new slide from a hashmap containing layers of objects (sorted by z-index) and a background object.
    pub fn with_objects_ordered(objects: HashMap<u8, Vec<Box<dyn Renderable>>>, background: Box<dyn Renderable>) -> Slide {
        let mut slide = Slide {
            // Convert from HashMap to IndexMap
            //   The contained object also get sorted by z-index.
            objects: objects.into_iter().collect::<IndexMap<u8, Vec<Box<dyn Renderable>>>>(),
            background: background.into()
        };

        slide.objects.sort_by(|a,_,b,_| a.cmp(b));

        slide
    }

    /// Creates new slide from a vec containing objects and a background object.
    pub fn with_objects_unordered<B>(vec: Vec<Box<dyn Renderable>>, background: B) -> Slide
    where B: Into< Box<dyn Renderable> > {
        let mut objects = IndexMap::new();
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

            self.objects.sort_by(|a,_,b,_| a.cmp(b));
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

            self.objects.sort_by(|a,_,b,_| a.cmp(b));
        }
    }

    /// Renders the slide.
    pub fn render(&self, time: f64, context: Context, opengl: &mut GlGraphics) {
        // Render the background
        self.background.render(time, context, opengl);

        // Render all objects of the slide
        //   The order of objects when iterating needs to be based on the z-index, which is also
        //   used as an index to the `Vec`s. This order gets established through an IndexMap that
        //   has it's items sorted by z-index (it is sorted upon creationg and gets re-sorted when
        //   inserting an object with a new z-index).
        for (_, vec) in self.objects.iter() {
            for renderable in vec.iter() {
                renderable.render(time, context, opengl);
            }
        }
    }
}