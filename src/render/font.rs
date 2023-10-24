use std::path::Path;

use opengl_graphics::{ GlGraphics, Texture, TextureSettings };
use graphics::{Context, Graphics, ImageSize};

use crate::util::DefaultingOption;

/// The scale range used when creating a font.
/// 
/// Consists of minimum size, maximum size and steps between these sizes.
pub const FONT_SCALE: (f32, f32, f32) = (30.0, 240.0, 10.0);

const MAX_FONT_COUNT: usize = ((FONT_SCALE.1 - FONT_SCALE.0) / FONT_SCALE.2) as usize + 1;

pub const ITALIC_FAC: f64 = 0.2;

// #[derive(Clone)]
pub struct Font {
    pub bases: heapless::Vec<(fontdue::Font, f32), { MAX_FONT_COUNT }>,
    // cached_glyphs: heapless::FnvIndexMap<(char, u32), (Texture, [f64; 2], (f32, f32)), { 64 * MAX_FONT_COUNT }>
}

#[allow(dead_code)]
impl Font {
    pub fn new<P: AsRef<Path>, F: Into<DefaultingOption<isize>>>(path: P, face_index: F) -> Option<Font> {
        let face_index_option: DefaultingOption<isize> = face_index.into();

        let bytes = std::fs::read(path.as_ref()).ok()?;
        let face_ind = face_index_option.consume(0);

        let face_sizes: [f32; (FONT_SCALE.1 - FONT_SCALE.0) as usize] = std::array::from_fn(|i| FONT_SCALE.0 + i as f32);
        let faces: heapless::Vec<(fontdue::Font, f32), { MAX_FONT_COUNT }> = face_sizes.into_iter().step_by(FONT_SCALE.2 as usize).filter_map(|size| {
            fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings { collection_index: face_ind as u32, scale: size }).ok().map(|font| (font, size))
        }).collect();

        match faces.len() {
            0 => None,
            _ => Some(Font { bases: faces })
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, face_index: isize) -> Option<Font> {
        let face_sizes: [f32; (FONT_SCALE.1 - FONT_SCALE.0) as usize] = std::array::from_fn(|i| FONT_SCALE.0 + i as f32);
        let faces: heapless::Vec<(fontdue::Font, f32), { MAX_FONT_COUNT }> = face_sizes.into_iter().step_by(FONT_SCALE.2 as usize).filter_map(|size| {
            fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings { collection_index: face_index as u32, scale: size }).ok().map(|font| (font, size))
        }).collect();

        match faces.len() {
            0 => None,
            _ => Some(Font { bases: faces })
        }
    }

    fn glyphs(&self, text: &str, size: f32) -> Vec<(Texture, [f64; 2])> {
        let base_index = self.bases.binary_search_by(|(_, font_size)| {
            font_size.total_cmp(&size)
        }).unwrap_or_else(|i|i);
        let base = &self.bases[base_index.min(self.bases.len()-1)].0;

        let mut x = 10.0;
        let mut y = 0.0;
        let height = base.rasterize('█', size).0.height as f32;
        let mut res = Vec::with_capacity(text.len());

        let size_ind: u32 = size as u32;

        for ch in text.chars() {
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
            // self.cached_glyphs.insert((ch, size_ind), (texture, [(x + g.0.xmin as f32) as f64, (y + height - g.0.height as f32 - g.0.ymin as f32) as f64], (g.0.advance_width,g.0.advance_height)));
            // let tex = &self.cached_glyphs.get(&(ch, size_ind)).unwrap().0;
            res.push((texture, [(x + g.0.xmin as f32) as f64, (y + height - g.0.height as f32 - g.0.ymin as f32) as f64]));
    
            x += g.0.advance_width;
            y += g.0.advance_height;
        }
        res
    }

    fn render_text<G, T>(glyphs: &[(T, [f64; 2])], c: &Context, gl: &mut G, color: [f32;4], italic: bool)
        where G: Graphics<Texture = T>, T: ImageSize
    {
        for (texture, [x, y]) in glyphs {
            use graphics::*;

            let transform;
            if italic { transform = c.transform.shear(-ITALIC_FAC, 0.0).trans(*x, *y) } else { transform = c.transform.trans(*x, *y) }

            Image::new_color(color).draw(
                texture,
                &c.draw_state,
                transform,
                gl
            );
        }
    }

    pub fn draw<Str: Into<heapless::String<64>>>(&mut self, text: Str, size: f64, color: (f32,f32,f32,f32), italic: bool, context: &Context, opengl_backend: &mut GlGraphics) {
        let size = size as u32;
        let mut text_string: heapless::String<64> = text.into();
        text_string.push(' ');
        // self.base.set_pixel_sizes(0, size)?;
        
        let glyphs = self.glyphs(&text_string, size as f32);

        Self::render_text(&glyphs, context, opengl_backend, [color.0,color.1,color.2,color.3], italic);
    }

    pub fn size<Str: Into<heapless::String<64>>>(&self, text: Str, size: f64) -> (f64, f64) {
        let size = size as u32;
        let text_string: heapless::String<64> = text.into();
        // text_string.push(' ');
        // self.base.set_pixel_sizes(0, size)?;
        let glyphs = self.glyphs(&text_string, size as f32);
        let size = glyphs[glyphs.len()-1].1;
        (size[0],size[1])
    }
}