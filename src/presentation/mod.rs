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

    pub fn next_slide(&mut self) {
        self.current_slide = (self.current_slide + 1) % self.slides.len();
    }

    pub fn previous_slide(&mut self) {
        let mut new = self.current_slide as isize - 1;
        if new<0 { new = self.slides.len() as isize-1 }
        self.current_slide = new as usize;
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