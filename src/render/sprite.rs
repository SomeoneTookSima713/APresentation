use opengl_graphics::{GlGraphics, Texture, TextureSettings, Filter, Wrap};
use graphics::{Image, DrawState, Context};
use graphics::rectangle;
use std::path::Path;

lazy_static::lazy_static! {
    pub static ref DEFAULT_TEXTURE_SETTINGS: TextureSettings = TextureSettings::new()
        .convert_gamma(true)
        .compress(false)
        .wrap_u(Wrap::ClampToEdge)
        .wrap_v(Wrap::ClampToEdge);
}

pub struct Sprite {
    base_image: Image,
    base_texture: Texture,
    draw_state: DrawState
}

impl Sprite {
    pub fn new<P: AsRef<Path>, R: Into<[f64;4]>>(file_path: P, rect: R) -> Self {
        let base_image = Image::new().rect(rect);
        let base_texture = Texture::from_path(file_path, &DEFAULT_TEXTURE_SETTINGS).unwrap();

        Sprite { base_image, base_texture, draw_state: DrawState::default() }
    }

    pub fn with_texture_settings<P: AsRef<Path>, R: Into<[f64;4]>>(file_path: P, rect: R, texture_settings: &TextureSettings) -> Self {
        let base_image = Image::new().rect(rect);
        let base_texture = Texture::from_path(file_path, texture_settings).unwrap();

        Sprite { base_image, base_texture, draw_state: DrawState::default() }
    }

    pub fn draw(&self, context: Context, opengl: &mut GlGraphics) {
        self.base_image.draw(&self.base_texture, &self.draw_state, context.transform, opengl)
    }
}