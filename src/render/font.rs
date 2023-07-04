use std::path::Path;

use opengl_graphics::{ GlGraphics, Texture, TextureSettings };
use graphics::{Context, Graphics, ImageSize};

use crate::app::Application;
use crate::util::{ DefaultingOption };

#[derive(Clone)]
pub struct Font {
    base: freetype::Face
}

impl Font {
    pub fn new<P: AsRef<Path>, F: Into<DefaultingOption<isize>>>(app: &Application, path: P, face_index: F) -> Option<Font> {
        let face_index_option: DefaultingOption<isize> = face_index.into();
        let face = app.freetype_instance.new_face(path.as_ref(), face_index_option.consume(0));

        match face {
            Ok(v) => Some(Font { base: v }),
            Err(_) => None
        }
    }

    fn glyphs(&self, text: &str) -> Vec<(Texture, [f64; 2])> {
        let mut x = 10;
        let mut y = 0;
        let mut res = vec![];
        for ch in text.chars() {
            self.base.load_char(ch as usize, ft::face::LoadFlag::RENDER).unwrap();
            let g = self.base.glyph();
    
            let bitmap = g.bitmap();
            let texture = Texture::from_memory_alpha(
                bitmap.buffer(),
                bitmap.width() as u32,
                bitmap.rows() as u32,
                &TextureSettings::new()
            ).unwrap();
            res.push((texture, [(x + g.bitmap_left()) as f64, (y - g.bitmap_top()) as f64]));
    
            x += (g.advance().x >> 6) as i32;
            y += (g.advance().y >> 6) as i32;
        }
        res
    }
    fn render_text<G, T>(glyphs: &[(T, [f64; 2])], c: &Context, gl: &mut G, color: [f32;4], italic: bool, size: u32)
        where G: Graphics<Texture = T>, T: ImageSize
    {
        for &(ref texture, [x, y]) in glyphs {
            use graphics::*;

            let transform;
            if italic { transform = c.transform.shear(size as f64 / -220.0, 0.0).trans(x, y) } else { transform = c.transform.trans(x, y) }

            Image::new_color(color).draw(
                texture,
                &c.draw_state,
                transform,
                gl
            );
        }
    }

    pub fn draw<Str: Into<String>>(&mut self, text: Str, size: u32, color: (f32,f32,f32,f32), italic: bool, context: &Context, opengl_backend: &mut GlGraphics) -> Result<(), freetype::error::Error> {
        let mut text_string: String = text.into();
        text_string.push(' ');
        self.base.set_pixel_sizes(0, size)?;
        
        let glyphs = self.glyphs(&text_string);

        Self::render_text(&glyphs, context, opengl_backend, [color.0,color.1,color.2,color.3], italic, size);

        Ok(())
    }

    pub fn size<Str: Into<String>>(&self, text: Str, size: u32) -> Result<(f64, f64), freetype::error::Error> {
        let mut text_string: String = text.into();
        text_string.push(' ');
        self.base.set_pixel_sizes(0, size)?;
        let glyphs = self.glyphs(&text_string);
        let size = glyphs[glyphs.len()-1].1;
        Ok((size[0],size[1]))
    }
}