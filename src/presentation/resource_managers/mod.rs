use once_cell::sync::Lazy;

mod texture_manager;
mod font_manager;

use texture_manager::TextureManager;
use font_manager::FontManager;

pub static TEXTURE_MANAGER: Lazy<TextureManager> = Lazy::new(TextureManager::new);
pub static FONT_MANAGER: Lazy<FontManager> = Lazy::new(FontManager::new);
