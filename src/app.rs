use std::collections::HashMap;
use std::cell::{ RefCell };
use std::sync::OnceLock;

use opengl_graphics::{ GlGraphics, OpenGL };
use piston::{RenderArgs, UpdateArgs, ButtonArgs, Button, ButtonState, Key};
use piston_window::PistonWindow;

use super::util::{ PanickingOption, AssumeThreadSafe };
use super::presentation;

pub struct Application {
    pub opengl_version: OpenGL,
    pub opengl_backend: PanickingOption<GlGraphics>,
    pub data: PanickingOption<AppData>
}

pub static FONTS: OnceLock<AssumeThreadSafe<HashMap<String, RefCell<presentation::TextFont>>>> = OnceLock::new();

/// Struct containing all the app's data.
pub struct AppData {
    presentation: presentation::Presentation,
    time: f64,
    timeint: u32,
    frames: u32,
    last_press: (bool, bool, bool)
    // font: super::render::font::Font
}
impl AppData {
    pub fn create(app: &Application, filepath: String) -> AppData {
        use crate::parse::{ self, Parser };

        // Read the contents of the presentation file
        let filecontents: String = std::fs::read_to_string(filepath.as_str()).unwrap();

        let mut parser = parse::get_parser(filepath.as_str()).expect("No parser found for file type!");

        let document_fonts = parser.parse_fonts(filecontents.as_str()).unwrap();
        FONTS.set({
            let mut map = HashMap::new();

            #[cfg(default_font)]
            {
                let bytes = include_bytes!("OpenSans.ttf") as &[u8];

                let face = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).expect("couldn't parse default font's data");

                let font = crate::render::font::Font { base: face };

                map.insert("Default".to_owned(), RefCell::new(presentation::TextFont { base_font: font.clone(), bold_font: font.clone() }));
            }

            for (name, path) in document_fonts {
                map.insert(name, RefCell::new(presentation::renderable::TextFont::new(app, path.0, path.1)));
            }

            AssumeThreadSafe(map)
        }).ok().expect("error initializing fonts");

        let document = parser.parse(filecontents.as_str()).unwrap();

        let mut presentation = presentation::Presentation::new();

        for slide_data in document {
            let mut slide = presentation::Slide::new(slide_data.background);
            for (z, content) in slide_data.content {
                for renderable in content {
                    slide.add_boxed(renderable, z);
                }
            }
            presentation.add_slide(slide);
        }

        #[cfg(default_font)]
        {
            let mut last_slide = presentation::Slide::new(Box::new(presentation::ColoredRect::new("0;0", "w;h", "0;0;0;1", "TOP_LEFT")) as Box<dyn presentation::Renderable>);
            last_slide.add(presentation::Text::new("0;0", vec!["End of presentation"], "w", "4%", "TOP_LEFT", "1;1;1;1", "Default".to_owned(), &*FONTS.get().unwrap()), 0);

            presentation.add_slide(last_slide);
        }

        // println!("{:?}", document);

        AppData {
            presentation,
            time: 0.0,
            timeint: 0,
            frames: 0,
            last_press: (false, false, false)
            // font: super::render::font::Font::new(app, "assets/Rubik-Regular.ttf", 0).unwrap()
        }
    }
}

impl Application {
    pub fn create(opengl_version: OpenGL) -> Self {
        Application { opengl_version, opengl_backend: PanickingOption::None, data: PanickingOption::None }
    }
    pub fn init<Str: Into<String>>(&mut self, title: Str, resolution: (u32, u32), vsync: bool, resizable: bool, decoration: bool, filepath: String) -> PistonWindow {
        // Initialize the logging backend
        pretty_env_logger::try_init_timed_custom_env("LOG").unwrap();

        let window = piston::window::WindowSettings::new(title.into(), [resolution.0,resolution.1])
            .graphics_api(self.opengl_version)
            .exit_on_esc(true)
            .vsync(vsync)
            .resizable(resizable)
            .decorated(decoration)
            .build()
            .unwrap();
        self.opengl_backend = PanickingOption::Some(GlGraphics::new(self.opengl_version));

        self.data = PanickingOption::Some(AppData::create(&self, filepath));

        window
    }

    pub fn render(&mut self, args: &RenderArgs) {

        // const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.data.frames += 1;

        self.opengl_backend.draw(args.viewport(), |c, gl| {
            // graphics::clear(GREEN, gl);
            let time = self.data.time;

            self.data.presentation.render(time, c, gl);
            // println!("{time}");
            // self.data.font.draw("Hello World!", 48, (0.0,0.0,0.0,1.0), true, &c.trans(4.0, 52.0), gl).unwrap();
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // println!("FPS: {}",1.0/args.dt);
        self.data.time += args.dt;
        if self.data.time>= self.data.timeint as f64 + 1.0 {
            self.data.timeint += 1;
            // println!("FPS: {} / {}", self.data.frames, 1.0/args.dt);
            self.data.frames = 0;
        }
    }

    pub fn input(&mut self, args: &ButtonArgs) -> bool {
        match (args.button, args.state, self.data.last_press) {
            (Button::Keyboard(Key::A | Key::Left), ButtonState::Press, (false, _, _)) => {
                self.data.presentation.previous_slide();
                self.data.last_press.0 = true;
            },
            (Button::Keyboard(Key::A | Key::Left), ButtonState::Release, (true, _, _)) => {
                self.data.last_press.0 = false;
            },

            (Button::Keyboard(Key::D | Key::Right), ButtonState::Press, (_, false, _)) => {
                self.data.presentation.next_slide();
                self.data.last_press.1 = true;
            },
            (Button::Keyboard(Key::D | Key::Right), ButtonState::Release, (_, true, _)) => {
                self.data.last_press.1 = false;
            },
            (Button::Keyboard(Key::F11), ButtonState::Press, (_, _, false)) => {
                self.data.last_press.2 = true;
                return true
            },
            (Button::Keyboard(Key::F11), ButtonState::Release, (_, _, true)) => {
                self.data.last_press.2 = false;
            },
            _ => {}
        }

        false
    }
}