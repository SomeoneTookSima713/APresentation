use std::path::Path;

use opengl_graphics::{ GlGraphics, Texture, TextureSettings };
use graphics::{Context, Graphics, ImageSize};

use crate::util::{ DefaultingOption };

pub const FONT_SCALE: f32 = 40.0;

#[derive(Clone)]
pub struct Font {
    pub base: fontdue::Font
}

#[allow(dead_code)]
impl Font {
    pub fn new<P: AsRef<Path>, F: Into<DefaultingOption<isize>>>(path: P, face_index: F) -> Option<Font> {
        let face_index_option: DefaultingOption<isize> = face_index.into();

        let bytes = std::fs::read(path.as_ref()).ok()?;

        let face = fontdue::Font::from_bytes(bytes, fontdue::FontSettings { collection_index: face_index_option.consume(0) as u32, scale: FONT_SCALE });

        match face {
            Ok(v) => Some(Font { base: v }),
            Err(_) => None
        }
    }

    fn glyphs(&self, text: &str, size: f32) -> Vec<(Texture, [f64; 2])> {
        let mut x = 10.0;
        let mut y = 0.0;
        let height = self.base.rasterize('█', size).0.height as f32;
        let mut res = vec![];
        for ch in text.chars() {
            let g = self.base.rasterize_subpixel(ch, size);

            let metrics_smaller = self.base.metrics(ch, size.floor());
            let metrics_bigger = self.base.metrics(ch, size.ceil());
            let xmin = metrics_smaller.xmin as f32 + (metrics_bigger.xmin - metrics_smaller.xmin) as f32*size.fract();
            let ymin = metrics_smaller.ymin as f32 + (metrics_bigger.ymin - metrics_smaller.ymin) as f32*size.fract();
            
            let mut bitmap = Vec::new();
            for col in g.1.chunks_exact(3) {
                let (r,g,b) = (col[0],col[1],col[2]);
                bitmap.push(((r as f64 + g as f64 + b as f64)/3.0) as u8);
            }
            let texture = Texture::from_memory_alpha(
                bitmap.as_slice(),
                g.0.width as u32,
                g.0.height as u32,
                &TextureSettings::new()
            ).unwrap();
            res.push((texture, [(x + xmin) as f64, (y + height - g.0.height as f32 - ymin) as f64]));
    
            x += g.0.advance_width;
            y += g.0.advance_height;
        }
        res
    }

    fn render_text<G, T>(glyphs: &[(T, [f64; 2])], c: &Context, gl: &mut G, color: [f32;4], italic: bool, size: u32)
        where G: Graphics<Texture = T>, T: ImageSize
    {
        for &(ref texture, [x, y]) in glyphs {
            use graphics::*;

            let transform;
            if italic { transform = c.transform.shear(size as f64 / -140.0, 0.0).trans(x, y) } else { transform = c.transform.trans(x, y) }

            Image::new_color(color).draw(
                texture,
                &c.draw_state,
                transform,
                gl
            );
        }
    }

    pub fn draw<Str: Into<String>>(&mut self, text: Str, size: f64, color: (f32,f32,f32,f32), italic: bool, context: &Context, opengl_backend: &mut GlGraphics) {
        let size = size as u32;
        let mut text_string: String = text.into();
        text_string.push(' ');
        // self.base.set_pixel_sizes(0, size)?;
        
        let glyphs = self.glyphs(&text_string, size as f32);

        Self::render_text(&glyphs, context, opengl_backend, [color.0,color.1,color.2,color.3], italic, size);
    }

    pub fn size<Str: Into<String>>(&self, text: Str, size: f64) -> (f64, f64) {
        let size = size as u32;
        let text_string: String = text.into();
        // text_string.push(' ');
        // self.base.set_pixel_sizes(0, size)?;
        let glyphs = self.glyphs(&text_string, size as f32);
        let size = glyphs[glyphs.len()-1].1;
        (size[0],size[1])
    }
}