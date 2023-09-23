#![feature(const_trait_impl)]

extern crate proc_macro;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate freetype as ft;

// use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ OpenGL };
use piston::input::*;
// use piston::event_loop::{ Events, EventSettings };
use piston_window::PistonWindow;

mod app;
mod util;
mod render;
mod parse;

mod presentation;

fn main() {
    let mut application = app::Application::create(OpenGL::V3_2);

    let mut window: PistonWindow = application.init("Test", (640,480), true, true, true);

    // let mut events = Events::new(EventSettings::new());
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            application.render(&args);
        }

        if let Some(args) = e.update_args() {
            application.update(&args);
        }
    }

    // TODO!!!: Find *the memory leak*
    panic!("Memory leak!!!")
}