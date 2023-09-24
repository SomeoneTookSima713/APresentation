#![feature(const_trait_impl)]

extern crate proc_macro;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate freetype as ft;

use std::env;
use opengl_graphics::{ OpenGL };
use piston::input::*;
use piston_window::PistonWindow;

mod app;
mod util;
mod render;
mod parse;

mod presentation;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len()!=2 {
        println!("Usage:\n\ta_presentation.exe [PATH_TO_FILE]");
        return;
    }

    let mut application = app::Application::create(OpenGL::V3_2);

    let mut window: PistonWindow = application.init("Test", (640,480), true, true, true, args[1].clone());

    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            application.render(&args);
        }

        if let Some(args) = e.update_args() {
            application.update(&args);
        }
        
        if let Some(args) = e.button_args() {
            application.input(&args);
        }
    }
}