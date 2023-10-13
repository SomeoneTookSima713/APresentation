use opengl_graphics::GlGraphics;
use graphics::Context;

pub mod slide;
pub mod renderable;
pub mod util;

pub use slide::Slide;
pub use renderable::*;

#[allow(unused)]
use log::{ debug as log_dbg, info as log_info, warn as log_warn, error as log_err };

/// Contains all data and state related to rendering the presentation.
pub struct Presentation {
    slides: Vec<slide::Slide>,
    current_slide: usize
}

impl Presentation {
    /// Creates a new Presentation.
    pub fn new() -> Presentation {
        Presentation { slides: Vec::new(), current_slide: 0 }
    }

    /// Adds a new slide.
    pub fn add_slide(&mut self, slide: slide::Slide) {
        self.slides.push(slide);
    }

    /// Changes to the next slide or wraps around to the first one if you're already on the last
    /// slide.
    pub fn next_slide(&mut self) {
        self.current_slide = (self.current_slide + 1) % self.slides.len();
    }

    /// Changes to the previous slide or wraps around to the last one if you're already on the
    /// first slide.
    pub fn previous_slide(&mut self) {
        let mut new = self.current_slide as isize - 1;
        if new<0 { new = self.slides.len() as isize-1 }
        self.current_slide = new as usize;
    }
    
    /// Renders this presentation.
    pub fn render(&mut self, time: f64, context: Context, opengl: &mut GlGraphics) {
        match self.slides.get(self.current_slide) {
            Some(slide) => {
                slide.render(time, context, opengl);
            },
            None => {
                log_err!("Slide #{} doesn't exist! Switching to slide at position 0...",self.current_slide);
                self.current_slide = 0;
            }
        }
    }
}