use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::util::atomic_vec;
use crate::texture;

use atomic_vec::AtomicVec;

pub struct TextureManager(AtomicVec<Arc<texture::Texture>>);

impl TextureManager {
    pub fn new() -> Self {
        Self(AtomicVec::new(512*1024).unwrap())
    }
}

impl std::ops::Deref for TextureManager {
    type Target = AtomicVec<Arc<texture::Texture>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub static TEXTURE_MANAGER: Lazy<TextureManager> = Lazy::new(TextureManager::new);

pub fn test(device: &wgpu::Device, queue: &wgpu::Queue) {
    TEXTURE_MANAGER.push(Arc::new(texture::Texture::new(device, queue, (16, 16), texture::TextureSamplerSelection::PixelPerfect, None).unwrap())).unwrap();

    log::info!("\n\t{:?}", TEXTURE_MANAGER.0);
}