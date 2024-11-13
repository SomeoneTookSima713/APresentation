use std::sync::Arc;

use crate::presentation::resource_managers::FONT_MANAGER;
use crate::util::OwnedParsedFace;

#[derive(Clone)]
pub struct Font {
    render_font: ab_glyph::FontArc,
    layout_font: harfbuzz_rs::Shared<harfbuzz_rs::Font<'static>>,
    parsed_info: Arc<OwnedParsedFace>
}

impl Font {
    pub fn from_data(data: Vec<u8>, index: u32) -> anyhow::Result<Self> {
        use ab_glyph::Font;

        let parsed_info = Arc::new(OwnedParsedFace::parse(data.clone(), index)?);

        let render_font = ab_glyph::FontArc::new(ab_glyph::FontVec::try_from_vec_and_index(data.clone(), index)?);
        
        let layout_font = harfbuzz_rs::Font::new(harfbuzz_rs::Face::new(data, index)).to_shared();

        Ok(Self { render_font, layout_font, parsed_info })
    }
}