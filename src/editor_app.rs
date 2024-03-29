use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use opengl_graphics::{ GlGraphics, OpenGL, Texture, Filter };
use piston::{RenderArgs, UpdateArgs, ButtonArgs, Button, ButtonState, Key};
use piston_window::PistonWindow;

use egui::{ RawInput, FullOutput, Context, TextureId };

#[allow(unused)]
use log::{ debug as log_dbg, info as log_info, warn as log_warn, error as log_err };

use super::util::{ PanickingOption, AssumeThreadSafe };
use super::presentation;

// Gets used for automatic links in comments.
#[allow(unused)]
use crate::presentation::Renderable;
use crate::presentation::renderable::BaseProperties;

pub struct Application {
    pub opengl_version: OpenGL,
    pub opengl_backend: PanickingOption<GlGraphics>,
    pub data: PanickingOption<AppData>,
    /// The window resolution
    resolution: (f64, f64)
}

/// Struct containing all the app's data.
pub struct AppData {
    /// All the data and state needed for rendering the presentation
    pub presentation: presentation::Presentation,
    /// The time since the current slide was switched to.
    /// 
    /// Gets used for calculating the properties of [`Renderable`] objects.
    pub time: f64,
    /// The time of the last frame.
    /// 
    /// Used for calculating the time elapsed between frames.
    pub last_frame: Instant,
    /// Gets used for measuring FPS
    /// 
    /// Only enabled in debug relases or with the 'debug_features' feature-flag.
    #[cfg(any(debug_features))]
    timeint: u32,
    /// Gets used for measuring FPS
    /// 
    /// Only enabled in debug relases or with the 'debug_features' feature-flag.
    #[cfg(any(debug_features))]
    frames: u32,
    /// Captures the state for the left/A, right/D and F11 keys.
    last_press: (bool, bool, bool),
    /// Contains all the data passed from the application to [`egui`].
    egui_input: RawInput,
    /// Contains all the data passed from [`egui`] to the application.
    egui_output: FullOutput,
    /// The [`Context`] object used for [`egui`].
    egui_context: Context,

    egui_textures: HashMap<TextureId, Texture>,

    egui_time: f64,
}
impl AppData {
    pub fn create(filepath: String) -> AppData {
        use crate::parse::{ self, Parser };

        // Read the contents of the presentation file
        let filecontents: String = std::fs::read_to_string(filepath.as_str()).unwrap();

        // Create an instance of a parser (which parser gets instantiated depends on the file extension)
        let mut parser = parse::get_parser(filepath.as_str()).expect("No parser found for file type!");

        let document_fonts = parser.parse_fonts(filecontents.as_str()).unwrap_or_else(|e| { parser.handle_error(e); unreachable!() });
        crate::FONTS.set({
            let mut map = HashMap::new();

            // Adds the default font in case it was included into the binary at compile time.
            #[cfg(default_font)]
            {
                let bytes = include_bytes!("OpenSans.ttf") as &[u8];

                // let face = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).expect("couldn't parse default font's data");

                let base_font = crate::render::font::Font::from_bytes(bytes.to_vec(), 0, "Default (bundled)".to_owned()).expect("couldn't parse default font's data");
                let bold_font = crate::render::font::Font::from_bytes(bytes.to_vec(), 0, "Default (bundled)".to_owned()).expect("couldn't parse default font's data");

                map.insert("Default".to_owned(), Rc::new(RefCell::new(presentation::TextFont { base_font, bold_font })));
            }

            for (name, path) in document_fonts {
                map.insert(name, Rc::new(RefCell::new(presentation::renderable::TextFont::new(path.0, path.1))));
            }

            AssumeThreadSafe(map)
        }).ok().expect("error initializing fonts");

        let document = parser.parse(filecontents.as_str()).unwrap_or_else(|e| { parser.handle_error(e); unreachable!() });

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

        // Adds an 'End of presentation' slide. This can only be done when including the default
        // font though, as the text needs a font to render itself.
        #[cfg(default_font)]
        {
            let bg = presentation::ColoredRect::new(BaseProperties::new("0;0", "w;h", "0;0;0;1", "TOP_LEFT").map_err(|_|()).unwrap());
            let mut last_slide = presentation::Slide::new(Box::new(bg) as Box<dyn presentation::Renderable>);

            let text = presentation::Text::new(
                BaseProperties::new("0;0","w;4%","1;1;1;1","TOP_LEFT").map_err(|_|()).unwrap(),
                vec!["End of presentation"],
                "Default".to_owned(),
                &*crate::FONTS.get().unwrap(),
                HashMap::new(),
                "LEFT").map_err(|_|()).unwrap();
            last_slide.add(text, 0);

            presentation.add_slide(last_slide);
        }

        AppData {
            presentation,
            time: 0.0,
            last_frame: Instant::now(),
            #[cfg(any(debug_features))]
            timeint: 0,
            #[cfg(any(debug_features))]
            frames: 0,
            last_press: (false, false, false),
            egui_input: Default::default(),
            egui_output: Default::default(),
            egui_context: Default::default(),
            egui_textures: HashMap::new(),
            egui_time: 0.0
        }
    }
}

impl Application {
    /// Creates the application's data.
    /// 
    /// Needs to be initialized seperately using the `init()` function.
    pub fn create(opengl_version: OpenGL) -> Self {
        Application { opengl_version, opengl_backend: PanickingOption::None, data: PanickingOption::None, resolution: (-1.0,-1.0) }
    }
    /// Initializes all the data and state of the application.
    pub fn init<Str: Into<String>>(&mut self, title: Str, resolution: (u32, u32), vsync: bool, resizable: bool, decoration: bool, filepath: String) -> PistonWindow {
        // Initialize the logging backend
        pretty_env_logger::try_init_timed_custom_env("LOG").unwrap();

        // Create the window
        let window = piston::window::WindowSettings::new(title.into(), [resolution.0,resolution.1])
            .graphics_api(self.opengl_version)
            .exit_on_esc(true)
            .vsync(vsync)
            .resizable(resizable)
            .decorated(decoration)
            .samples(0)
            .srgb(true)
            .build()
            .unwrap();
        // Create the OpenGL context
        self.opengl_backend = PanickingOption::Some(GlGraphics::new(self.opengl_version));

        // Create the application's data
        self.data = PanickingOption::Some(AppData::create(filepath));

        self.resolution = (resolution.0 as f64, resolution.1 as f64);

        window
    }

    /// Renders the application
    pub fn render(&mut self, args: &RenderArgs) {
        // Increase the 'frames' counter if debugging
        //   Gets used for measuring FPS.
        #[cfg(any(debug_features))]
        {
            self.data.frames += 1;
        }

        // Calculate how much time has passed since rendering the last frame, then set
        // self.data.last_frame to the current point in time for the next frame.
        let now = Instant::now();
        let dt = self.data.last_frame.elapsed().as_secs_f64();
        self.data.time += dt;
        self.data.last_frame = now;

        // Draw the presentation
        self.opengl_backend.draw(args.viewport(), |c, gl| {
            use graphics::{ Transformed, Graphics };
            use opengl_graphics::{ TextureSettings, Wrap };

            // We need to set a local variable here to copy the value, because we already mutably
            // borrowed 'self' in the call above and would immutably borrow it by directly passing
            // the value into the function call below, which we aren't allowed to do.
            let time = self.data.time;

            let ctx = c.trans(self.resolution.0*0.25, self.resolution.1*0.25).scale(0.5,0.5);

            self.data.presentation.render(time, ctx, gl);

            let gui = &self.data.egui_output;

            let tris = self.data.egui_context.tessellate(gui.shapes.clone());
            let mut to_add = HashMap::new();
            let mut to_remove = Vec::new();

            {
                let textures = &self.data.egui_output.textures_delta;

                for (id, data) in &textures.set {
                    let texture = match &data.image {
                        egui::ImageData::Color(data) => {
                            use image::RgbaImage;
                            let img = RgbaImage::from_vec(data.size[0] as u32, data.size[1] as u32, data.pixels.iter().map(|c|c.to_srgba_unmultiplied().to_vec()).flatten().collect()).unwrap();
                            Texture::from_image(&img, &TextureSettings::new().wrap_u(Wrap::ClampToEdge).wrap_v(Wrap::ClampToEdge).generate_mipmap(false).convert_gamma(true))
                        },
                        egui::ImageData::Font(data) => {
                            use image::RgbaImage;
                            let img = RgbaImage::from_vec(data.size[0] as u32, data.size[1] as u32, data.srgba_pixels(None).map(|c|c.to_srgba_unmultiplied().to_vec()).flatten().collect()).unwrap();
                            Texture::from_image(&img, &TextureSettings::new().wrap_u(Wrap::ClampToEdge).wrap_v(Wrap::ClampToEdge).generate_mipmap(false).convert_gamma(true))
                        }
                    };

                    to_add.insert(id.clone(), texture);
                }

                for id in &textures.free {
                    to_remove.push(id.clone());
                }
            }

            for id in to_remove {
                drop(self.data.egui_textures.remove(&id));
            }
            self.data.egui_textures.extend(to_add);

            for tri in tris {
                match tri.primitive {
                    egui::epaint::Primitive::Mesh(m) => {
                        let vertices = &m.vertices;
                        let indices = &m.indices;
                        gl.tri_list_uv_c(&c.draw_state, self.data.egui_textures.get(&m.texture_id).unwrap(), |c| {
                            let verts = indices.iter().map(|i|{
                                let p = &vertices[*i as usize].pos;
                                [p.x,p.y]
                            }).collect::<Vec<[f32;2]>>();
                            let uv = indices.iter().map(|i|{
                                let p = &vertices[*i as usize].uv;
                                [p.x,p.y]
                            }).collect::<Vec<[f32;2]>>();
                            let cols = indices.iter().map(|i|{
                                let p = &vertices[*i as usize].color;
                                p.to_srgba_unmultiplied().map(|v|v as f32/255.0)
                            }).collect::<Vec<[f32;4]>>();
                            (c)(verts.as_slice(),uv.as_slice(),cols.as_slice())
                        });
                    },
                    egui::epaint::Primitive::Callback(_) => {
                        unimplemented!()
                    }
                }
            }
        });
    }

    /// Updates the application.
    /// 
    /// Currently only used for measuring FPS if debugging is enabled.
    pub fn update(&mut self, args: &UpdateArgs) {
        // self.data.time += args.dt;
        #[cfg(any(debug_features))]
        if self.data.time>= self.data.timeint as f64 + 1.0 {
            self.data.timeint += 1;
            // The amount that update() gets called every second is fixed at 120 times per second.
            log_dbg!("FPS: {} / {}", self.data.frames, 120.0);
            self.data.frames = 0;
        }

        self.data.egui_time += args.dt;

        let mut input = std::mem::replace(&mut self.data.egui_input, Default::default());
        input.time = Some(self.data.egui_time);

        let out = self.data.egui_context.run(input, |c| self.ui(c));
        self.data.egui_output = out;
    }

    pub fn resize(&mut self, new_res: (u32, u32)) {
        self.resolution = (new_res.0 as f64, new_res.1 as f64);
    }

    /// Checks for input and updates the applications state accordingly.
    pub fn input(&mut self, args: &ButtonArgs) -> bool {
        use egui::Event;

        todo!();

        match (args.button, args.state, self.data.last_press) {
            (Button::Keyboard(Key::A | Key::Left), ButtonState::Press, (false, _, _)) => {
                self.data.presentation.previous_slide();
                self.data.time = 0.0;
                self.data.last_press.0 = true;
            },
            (Button::Keyboard(Key::A | Key::Left), ButtonState::Release, (true, _, _)) => {
                self.data.last_press.0 = false;
            },

            (Button::Keyboard(Key::D | Key::Right), ButtonState::Press, (_, false, _)) => {
                self.data.presentation.next_slide();
                self.data.time = 0.0;
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

    pub fn ui(&self, ctx: &Context) {

    }
}