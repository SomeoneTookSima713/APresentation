#![feature(const_trait_impl)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(auto_traits)]
#![feature(negative_impls)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::sync::{ OnceLock, RwLock };
use opengl_graphics::OpenGL;
use piston::input::*;
use piston_window::{ PistonWindow, Events, EventSettings };
use mlua::{ Lua, StdLib, LuaOptions };

mod viewer_app;
mod editor_app;
mod util;
mod render;
mod parse;

mod presentation;

use util::AssumeThreadSafe;

/// The version of the application
const APPLICATION_VERSION: &'static str = include_str!("version");

// * SAFETY *
// These statics may only be used on one and only one thread. If any one of
// them is used on multiple threads concurrently, things will go VERY bad.
pub static LUA_INSTANCE: OnceLock<AssumeThreadSafe<Lua>> = OnceLock::new();
pub static FONTS: OnceLock<AssumeThreadSafe<HashMap<String, RefCell<presentation::TextFont>>>> = OnceLock::new();

fn run_viewer(args: Vec<String>) -> anyhow::Result<()> {
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

fn run_editor(args: Vec<String>) -> anyhow::Result<()> {
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

fn usage() {
    println!("Usage:\n\ta_presentation.exe view [PATH_TO_FILE]\t\t- Opens a file for viewing\n\ta_presentation.exe generate [PATH_TO_FILE]\t- Generates a template for easier creation of presentations");
}

fn main() -> anyhow::Result<()> {

    let args = env::args().collect::<Vec<String>>();

    if args.len()!=3 {
        usage();
        return Ok(())
    }

    LUA_INSTANCE.set(AssumeThreadSafe(Lua::new_with(StdLib::TABLE | StdLib::STRING | StdLib::MATH, LuaOptions::new()).unwrap())).map_err(|_|anyhow::anyhow!("Setting the LUA_INSTANCE static failed!"))?;

    let lua = LUA_INSTANCE.get().unwrap();

    match args[1].clone().as_str() {
        "view" => run_viewer(args)?,
        "generate" => std::fs::write(&args[2], include_str!("template.hjson"))?,
        "edit" => run_editor(args)?,
        _ => usage()
    }
    Ok(())
}