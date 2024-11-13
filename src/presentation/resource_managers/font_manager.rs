use std::sync::{ Arc, RwLock };
use std::borrow::Borrow;
use std::hash::Hash;

use once_cell::sync::Lazy;

use crate::render::font;
use crate::util::{ArcHashmap, get_font_source_from_query};

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FontSelector {
    pub family: String,
    pub bold: Boldness,
    pub italic: bool
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u16)]
pub enum Boldness {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FontStyle {
    Serif,
    SansSerif,
    Code
}

pub struct FontManager {
    /// A list of all fonts currently loaded.
    /// 
    /// While system fonts only get indexed (they don't actually get loaded, their style info just
    /// gets queried), user fonts automatically get loaded the moment they're registered.
    loaded_fonts: RwLock<ArcHashmap<FontSelector, font::Font>>,

    fallback_fonts: ArcHashmap<FontStyle, font::Font>,

    system_fonts: fontdb::Database
}

#[allow(unused)]
impl FontManager {
    pub fn new() -> Self {
        use fontdb::{ Family, Query };

        let mut system_fonts = fontdb::Database::new();

        system_fonts.load_system_fonts();

        let mut fallback_fonts = ArcHashmap::new();

        let errfunc = || {
            log::error!("Your system font's do not contain enough fonts of different styles to run this application!");
            panic!("Insufficient System fonts");
        };

        let (data, index) = get_font_source_from_query(&system_fonts, &Query { families: &[ Family::Serif ], weight: fontdb::Weight::NORMAL, stretch: fontdb::Stretch::Normal, style: fontdb::Style::Normal }).unwrap_or_else(errfunc);
        fallback_fonts.insert(FontStyle::Serif, Arc::new(font::Font::from_data(data, index).expect("One of your system fonts contains corrupted data!")));

        let (data, index) = get_font_source_from_query(&system_fonts, &Query { families: &[ Family::SansSerif ], weight: fontdb::Weight::NORMAL, stretch: fontdb::Stretch::Normal, style: fontdb::Style::Normal }).unwrap_or_else(errfunc);
        fallback_fonts.insert(FontStyle::SansSerif, Arc::new(font::Font::from_data(data, index).expect("One of your system fonts contains corrupted data!")));

        let (data, index) = get_font_source_from_query(&system_fonts, &Query { families: &[ Family::Monospace ], weight: fontdb::Weight::NORMAL, stretch: fontdb::Stretch::Normal, style: fontdb::Style::Normal }).unwrap_or_else(errfunc);
        fallback_fonts.insert(FontStyle::Code, Arc::new(font::Font::from_data(data, index).expect("One of your system fonts contains corrupted data!")));

        Self {
            loaded_fonts: RwLock::new(ArcHashmap::new()),
            fallback_fonts,
            system_fonts
        }
    }

    /// Gets a font with the specified selector, or loads a system font if no loaded font matching the selector is found.
    /// 
    /// If no matching font is currently loaded and no system font matches the given selector, `None` is returned.
    pub fn get(&self, key: &FontSelector) -> Option<Arc<font::Font>>
    {
        use fontdb::{ Family, Query, Source, Stretch, Style, Weight };

        if !self.loaded_fonts.read().unwrap().contains_key(key) {
            if let Some(id) = self.system_fonts.query(&fontdb::Query {
                families: &[ Family::Name(key.family.as_str()) ],
                weight: Weight(key.bold as u16),
                stretch: Stretch::Normal,
                style: if key.italic { Style::Italic } else { Style::Normal }
            }) {
                let mut lock = self.loaded_fonts.write().unwrap();
                let mut entry = lock.entry(key.clone());

                Some(entry.insert(Arc::new(match self.system_fonts.face_source(id).unwrap() {
                    (Source::Binary(data), index) => font::Font::from_data(Vec::from(data.as_ref().as_ref()), index).ok()?,
                    (Source::File(path), index) => font::Font::from_data(std::fs::read(path).ok()?, index).ok()?,
                    (Source::SharedFile(_, data), index) => font::Font::from_data(data.as_ref().as_ref().iter().copied().collect::<Vec<_>>(), index).ok()?
                })).get().clone())
            } else {
                None
            }
        } else {
            self.loaded_fonts.read().unwrap().get(key).cloned()
        }
    }

    pub fn get_fallback(&self, style: FontStyle) -> Arc<font::Font> {
        self.fallback_fonts.get(&style).unwrap().clone()
    }

    pub fn insert(&self, key: FontSelector, font: font::Font) -> anyhow::Result<Option<Arc<font::Font>>> {
        Ok(self.loaded_fonts.write().map_err(|_|anyhow::anyhow!("loaded_fonts has a poisoned lock!"))?.insert(key, Arc::new(font)))
    }

    pub fn load_font<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let file_data = std::fs::read(path.as_ref())?;

        let font_count;
        if let Some(c) = ttf_parser::fonts_in_collection(&file_data) {
            font_count = c;
        } else {
            // POSSIBLE ERROR: I'm not sure, but it could be that regular .ttf files don't count as font collections (collections are normally .tfc files I think)
            anyhow::bail!("File at path \"{:?}\" isn't a font!", path.as_ref())
        }

        for i in 0..font_count {
            let font_data = ttf_parser::Face::parse(&file_data, i)?;

            let family = {
                let mut f = None;
                for name in font_data.names() {
                    // TODO: Also take the language into account
                    if name.name_id == ttf_parser::name_id::FAMILY {
                        if let Some(name_str) = name.to_string() {
                            f = Some(name_str);
                            break
                        }
                    }
                }
                f
            }.expect("Font doesn't have a family name!");

            let style = FontSelector {
                family,
                bold: unsafe { std::mem::transmute((font_data.weight().to_number() as f64/100.0).round().clamp(1.0, 9.0) as u16*100) },
                italic: font_data.is_italic() || font_data.is_oblique()
            };

            if let Ok(mut lock) = self.loaded_fonts.write() {
                if lock.get(&style).is_some() {
                    log::warn!("Font face loaded with style {:#?} already exists and is being loaded again! Replacing old version...", style);
                }
                lock.insert(style, Arc::new(font::Font::from_data(file_data.clone(), i)?));
            } else {
                anyhow::bail!("loaded_fonts has a poisoned lock!")
            }
        }

        Ok(())
    }

    pub fn exists(&self, key: &FontSelector) -> bool {
        self.loaded_fonts.read().unwrap().contains_key(key)
    }
}