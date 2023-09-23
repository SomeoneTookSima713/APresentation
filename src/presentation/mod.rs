use std::collections::HashMap;

use opengl_graphics::GlGraphics;
use graphics::Context;

pub mod slide;
pub mod renderable;
pub mod util;

pub use slide::Slide;
pub use renderable::*;

pub struct Presentation {
    slides: Vec<slide::Slide>,
    current_slide: usize
}

impl Presentation {
    pub fn new() -> Presentation {
        Presentation { slides: Vec::new(), current_slide: 0 }
    }

    pub fn add_slide(&mut self, slide: slide::Slide) {
        self.slides.push(slide);
    }
    
    pub fn render(&mut self, time: f64, context: Context, opengl: &mut GlGraphics) {
        match self.slides.get(self.current_slide) {
            Some(slide) => {
                slide.render(time, context, opengl);
            },
            None => {
                let cslide = self.current_slide;
                println!("WARN: Slide #{cslide} doesn't exist!")
            }
        }
    }
}