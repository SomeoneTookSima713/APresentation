use opengl_graphics::{ GlGraphics, Texture, TextureSettings, OpenGL };
use piston::{RenderArgs, UpdateArgs};

use super::util::{ PanickingOption };
use super::presentation;

pub struct Application {
    pub opengl_version: OpenGL,
    pub freetype_instance: ft::Library,
    pub opengl_backend: PanickingOption<GlGraphics>,
    pub data: PanickingOption<AppData>
}

pub struct AppData {
    presentation: presentation::Presentation,
    time: f64,
    timeint: u32,
    frames: u32,
    // font: super::render::font::Font
}
impl AppData {
    pub fn create(app: &Application) -> AppData {
        let mut presentation = presentation::Presentation::new();

        let mut slide = presentation::Slide::new(None);
        // slide.add(presentation::ColoredRect::new("0.04*h;0.04*h", "0.1*h;0.1*h", "0.8;0.8;0.07;1.0"), 0);
        slide.add(presentation::RoundedRect::new("50%;50%", "0.1*h;0.1*h", "0.8;0.8;0.07;1.0", "60+sin(t)*20", "MID_CENTERED"), 0);

        presentation.add_slide(slide);

        AppData {
            presentation,
            time: 0.0,
            timeint: 0,
            frames: 0,
            // font: super::render::font::Font::new(app, "assets/Rubik-Regular.ttf", 0).unwrap()
        }
    }
}

impl Application {
    pub fn create(opengl_version: OpenGL) -> Self {
        let freetype_instance = freetype::Library::init().unwrap();

        Application { opengl_version, freetype_instance, opengl_backend: PanickingOption::None, data: PanickingOption::None }
    }
    pub fn init<Str: Into<String>>(&mut self, title: Str, resolution: (u32, u32), vsync: bool, resizable: bool, decoration: bool) -> glutin_window::GlutinWindow {
        let window = piston::window::WindowSettings::new(title.into(), [resolution.0,resolution.1])
            .graphics_api(self.opengl_version)
            .exit_on_esc(true)
            .vsync(vsync)
            .resizable(resizable)
            .decorated(decoration)
            .build()
            .unwrap();
        self.opengl_backend = PanickingOption::Some(GlGraphics::new(self.opengl_version));

        self.data = PanickingOption::Some(AppData::create(&self));

        window
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::Transformed;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.data.frames += 1;

        self.opengl_backend.draw(args.viewport(), |c, gl| {
            graphics::clear(GREEN, gl);
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
            println!("FPS: {}", self.data.frames);
            self.data.frames = 0;
        }
    }
}