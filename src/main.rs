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

const APPLICATION_VERSION: &'static str = "0.1.0";

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len()!=2 {
        println!("Usage:\n\ta_presentation.exe [PATH_TO_FILE]");
        return;
    }

    let mut application = app::Application::create(OpenGL::V3_2);

    let mut window: PistonWindow = application.init(format!("APresentation - {}",APPLICATION_VERSION), (640,480), true, true, true, args[1].clone());

    let mut fullscreen = false;

    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            application.render(&args);
        }

        if let Some(args) = e.update_args() {
            application.update(&args);
        }
        
        if let Some(args) = e.button_args() {
            fullscreen = application.input(&args);

            if fullscreen {
                window.window.window.set_fullscreen(match window.window.window.fullscreen().is_none() { true => Some(winit::window::Fullscreen::Borderless(None)), false => None });
            }
        }
    }
}