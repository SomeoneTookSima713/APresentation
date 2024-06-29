use crate::presentation::resource_managers::FONT_MANAGER;
use crate::util::FontDBSourceExt;

pub struct Font<'f> {
    font: ab_glyph::FontRef<'f>
}

impl<'f> Font<'f> {
    pub fn from_id(id: fontdb::ID) -> anyhow::Result<Self> {
        let face = FONT_MANAGER.get_database().face(id).ok_or(anyhow::anyhow!("No font with specified ID exists!"))?;
        let font = face.source.with_data(|data| ab_glyph::FontRef::try_from_slice_and_index(data, face.index)).ok_or(anyhow::anyhow!("Couldn't open font file!"))??;

        Ok(Self { font })
    }
}