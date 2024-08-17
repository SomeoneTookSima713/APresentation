use std::sync::Arc;

use crate::presentation::resource_managers::FONT_MANAGER;
use crate::util::{ FontDBSourceExt, OwnedParsedFace };

#[derive(Clone)]
pub struct Font {
    render_font: ab_glyph::FontArc,
    layout_font: harfbuzz_rs::Shared<harfbuzz_rs::Font<'static>>,
    parsed_info: Arc<OwnedParsedFace>
}

impl Font {
    pub fn from_id(id: fontdb::ID) -> anyhow::Result<Self> {
        use ab_glyph::Font;

        let face = FONT_MANAGER.get_database().face(id).ok_or(anyhow::anyhow!("No font with specified ID exists!"))?;
        let source = face.source.get_data().ok_or(anyhow::anyhow!("Font source couldn't be loaded!"))?;

        let parsed_info = Arc::new(OwnedParsedFace::parse(source.clone(), face.index)?);

        let render_font = ab_glyph::FontArc::new(ab_glyph::FontVec::try_from_vec_and_index(source, face.index)?);
        
        let layout_font = harfbuzz_rs::Font::new(harfbuzz_rs::Face::new(render_font.font_data().to_vec(), face.index)).to_shared();

        Ok(Self { render_font, layout_font, parsed_info })
    }

    
}