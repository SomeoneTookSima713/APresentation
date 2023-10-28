use std::path::Path;
use std::collections::HashMap;

use opengl_graphics::{ GlGraphics, Texture, TextureSettings };
use graphics::{Context, Graphics, ImageSize};
use fontdue::Metrics;

use crate::util::DefaultingOption;

/// The scale range used when creating a font.
/// 
/// Consists of minimum size, maximum size and steps between these sizes.
pub const FONT_SCALE: (f32, f32, f32) = (30.0, 240.0, 10.0);

const MAX_FONT_COUNT: usize = ((FONT_SCALE.1 - FONT_SCALE.0) / FONT_SCALE.2) as usize + 1;

pub const ITALIC_FAC: f64 = 0.2;

// #[derive(Clone)]
pub struct Font {
    pub bases: Vec<(fontdue::Font, f32)>,
    cached_glyphs: HashMap<(char, u32), (Texture, Metrics)>
}

#[allow(dead_code)]
impl Font {
    pub fn new<P: AsRef<Path>, F: Into<DefaultingOption<isize>>>(path: P, face_index: F) -> Option<Font> {
        let face_index_option: DefaultingOption<isize> = face_index.into();

        let bytes = std::fs::read(path.as_ref()).ok()?;
        let face_ind = face_index_option.consume(0);

        let face_sizes: [f32; (FONT_SCALE.1 - FONT_SCALE.0) as usize] = std::array::from_fn(|i| FONT_SCALE.0 + i as f32);
        let faces: Vec<(fontdue::Font, f32)> = face_sizes.into_iter().step_by(FONT_SCALE.2 as usize).filter_map(|size| {
            fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings { collection_index: face_ind as u32, scale: size }).ok().map(|font| (font, size))
        }).collect();

        match faces.len() {
            0 => None,
            _ => Some(Font { bases: faces, cached_glyphs: HashMap::with_capacity(MAX_FONT_COUNT * 40) })
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, face_index: isize) -> Option<Font> {
        let face_sizes: [f32; (FONT_SCALE.1 - FONT_SCALE.0) as usize] = std::array::from_fn(|i| FONT_SCALE.0 + i as f32);
        let faces: Vec<(fontdue::Font, f32)> = face_sizes.into_iter().step_by(FONT_SCALE.2 as usize).filter_map(|size| {
            fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings { collection_index: face_index as u32, scale: size }).ok().map(|font| (font, size))
        }).collect();

        match faces.len() {
            0 => None,
            _ => Some(Font { bases: faces, cached_glyphs: HashMap::with_capacity(MAX_FONT_COUNT * 40) })
        }
    }

    fn glyphs(&mut self, text: &str, size: f32) -> (Vec<(&Texture, [f64; 2])>, f64) {
        let base_index = self.bases.binary_search_by(|(_, font_size)| {
            font_size.total_cmp(&size)
        }).unwrap_or_else(|i|i);
        let base = &self.bases[base_index.min(self.bases.len()-1)].0;

        let mut x = 10.0;
        let mut y = 0.0;
        let height = base.rasterize('█', size).0.height as f32;
        let mut res = Vec::with_capacity(text.len());

        let mut last_width: usize = 0;
        let mut last_advance_width: f32 = 0.0;
        let mut last_xmin: i32 = 0;

        let size_ind: u32 = size as u32;

        for ch in text.chars() {
            let ind = (ch, size_ind);
            if self.cached_glyphs.get(&ind).is_none() {
                let g = base.rasterize_subpixel(ch, size);
                
                let mut bitmap = Vec::with_capacity(g.1.len()/3+1);
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

                self.cached_glyphs.insert(ind, (texture, g.0));
            }
        }

        for ch in text.chars() {
            let glyph = self.cached_glyphs.get(&(ch, size_ind)).unwrap();
            let metrics = glyph.1;

            res.push((&glyph.0, [(x + metrics.xmin as f32) as f64, (y + height - metrics.height as f32 - metrics.ymin as f32) as f64]));
    
            x += metrics.advance_width;
            y += metrics.advance_height;

            last_width = metrics.width;
            last_advance_width = metrics.advance_width;
            last_xmin = metrics.xmin;
        }
        (res, x as f64 - last_advance_width as f64 + last_xmin as f64 + last_width as f64)
    }

    fn render_text<G, T>(glyphs: &[(&T, [f64; 2])], c: &Context, gl: &mut G, color: [f32;4], italic: bool)
        where G: Graphics<Texture = T>, T: ImageSize
    {
        for (texture, [x, y]) in glyphs {
            use graphics::*;

            let transform;
            if italic { transform = c.transform.shear(-ITALIC_FAC, 0.0).trans(*x, *y) } else { transform = c.transform.trans(*x, *y) }

            Image::new_color(color).draw(
                *texture,
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
        
        let glyphs = self.glyphs(&text_string, size as f32).0;

        Self::render_text(&glyphs, context, opengl_backend, [color.0,color.1,color.2,color.3], italic);
    }

    pub fn size<Str: Into<String>>(&mut self, text: Str, size: f64) -> (f64, f64) {
        let size = size as u32;
        let text_string: String = text.into();
        // text_string.push(' ');
        // self.base.set_pixel_sizes(0, size)?;
        let glyphs = self.glyphs(&text_string, size as f32);
        let size = glyphs.0[glyphs.0.len()-1].1;
        (glyphs.1,size[1])
    }
}