use ab_glyph::FontVec;

pub struct Font {
    pub regular: FontVec,
    pub bold: FontVec,
    pub italic: FontVec,
    pub bold_italic: FontVec
}

impl Font {
    pub fn new(regular: FontVec, bold: FontVec, italic: FontVec, bold_italic: FontVec) -> Self {
        Self { regular, bold, italic, bold_italic }
    }
}