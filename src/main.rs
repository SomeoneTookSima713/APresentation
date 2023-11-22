#![feature(const_trait_impl)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(auto_traits)]
#![feature(negative_impls)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::env;
use std::fmt::Debug;
use opengl_graphics::OpenGL;
use piston::input::*;
use piston_window::{ PistonWindow, Events, EventSettings };

mod viewer_app;
mod editor_app;
mod util;
mod render;
mod parse;

mod presentation;

/// The version of the application
const APPLICATION_VERSION: &'static str = include_str!("version");

use std::sync::OnceLock;
use std::collections::HashMap;
use std::cell::RefCell;
use util::AssumeThreadSafe;
pub static FONTS: OnceLock<AssumeThreadSafe<HashMap<String, RefCell<presentation::TextFont>>>> = OnceLock::new();

fn run_viewer(args: Vec<String>) -> Result<(), Box<dyn Debug>> {
    let mut application = viewer_app::Application::create(OpenGL::V3_2);

    let mut window: PistonWindow = application.init(format!("APresentation Viewer - {}",APPLICATION_VERSION), (1280,720), false, true, true, args[2].clone());

    let mut fullscreen;

    let mut events = Events::new({
        let mut settings = EventSettings::new();
        settings.lazy = false;
        settings.bench_mode = false;
        settings.max_fps = std::u64::MAX;
        settings
    });

    while let Some(e) = events.next(&mut window) {
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

    Ok(())
}

fn run_editor(args: Vec<String>) -> Result<(), Box<dyn Debug>> {
    let mut application = editor_app::Application::create(OpenGL::V3_2);

    let mut window: PistonWindow = application.init(format!("APresentation Editor - {}",APPLICATION_VERSION), (1280,720), false, true, true, args[2].clone());

    let mut fullscreen;

    let mut events = Events::new({
        let mut settings = EventSettings::new();
        settings.lazy = false;
        settings.bench_mode = false;
        settings.max_fps = std::u64::MAX;
        settings
    });

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            application.render(&args);
        }

        if let Some(args) = e.update_args() {
            application.update(&args);
        }

        if let Some(args) = e.resize_args() {
            application.resize((args.draw_size[0],args.draw_size[1]))
        }
        
        if let Some(args) = e.button_args() {
            fullscreen = application.input(&args);

            if fullscreen {
                window.window.window.set_fullscreen(match window.window.window.fullscreen().is_none() { true => Some(winit::window::Fullscreen::Borderless(None)), false => None });
            }
        }
    }

    Ok(())
}

fn usage() -> Result<(), Box<dyn Debug>> {
    println!("Usage:\n\ta_presentation.exe view [PATH_TO_FILE]\t\t- Opens a file for viewing\n\ta_presentation.exe generate [PATH_TO_FILE]\t- Generates a template for easier creation of presentations");
    Ok(())
}

fn main() -> Result<(), Box<dyn Debug>> {

    let args = env::args().collect::<Vec<String>>();

    if args.len()!=3 {
        return usage();
    }

    match args[1].clone().as_str() {
        "view" => run_viewer(args),
        "generate" => std::fs::write(&args[2], include_str!("template.hjson")).map_err(|e|Box::new(e) as Box<dyn Debug>),
        "edit" => run_editor(args),
        _ => usage()
    }
}