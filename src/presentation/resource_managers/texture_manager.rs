use std::collections::HashMap;
use std::sync::{ Arc, RwLock };
use std::borrow::Borrow;
use std::hash::Hash;

use crate::texture;

pub struct TextureManager(RwLock<Vec<Arc<texture::Texture>>>, RwLock<HashMap<String, usize>>);

#[allow(unused)]
impl TextureManager {
    pub fn new() -> Self {
        Self(RwLock::new(Vec::with_capacity(128)), RwLock::new(HashMap::new()))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<Arc<texture::Texture>>
    where Q: ?Sized + Eq + Hash, String: Borrow<Q> {
        let Ok(hm) = self.1.read() else {
            log::warn!("Couldn't access TextureManager's String-ID HashMap! (this indicates a poisoned lock -> Something panicked!)");
            return None;
        };
        hm.get(key).and_then(|k| self.0.read().inspect_err(|_| log::warn!("Couldn't access TextureManager's Texture List! (this indicates a poisoned lock -> Something panicked!)")).ok().and_then(|v| v.get(*k).cloned()))
    }

    pub fn insert(&self, key: String, texture: Arc<texture::Texture>) -> anyhow::Result<()> {
        match self.1.read().map(|l| l.get(&key).is_some()) {
            Ok(true) => log::warn!("Trying to insert texture with key \"{key}\" into TextureManager even though a texture already exists at that key! Replacing old texture."),
            Ok(false) =>  {},
            Err(_) => {
                log::warn!("Couldn't access TextureManager's String-ID HashMap! (this indicates a poisoned lock -> Something panicked!)");
                anyhow::bail!("TextureManager's String-ID HashMap has a poisoned lock!")
            }
        }
        let ind = self.0.write().inspect_err(|_| log::warn!("Couldn't access TextureManager's Texture List! (this indicates a poisoned lock -> Something panicked!)")).map_err(|_| anyhow::anyhow!("TextureManager's Texture List has a poisoned lock!")).map(|mut v| {
            v.push(texture);
            v.len()-1
        })?;
        let Ok(mut hm) = self.1.write() else {
            anyhow::bail!("TextureManager's String-ID HashMap has a poisoned lock!");
        };
        hm.insert(key, ind);
        Ok(())
    }

    pub fn exists<Q>(&self, key: &Q) -> bool
    where Q: ?Sized + Eq + Hash, String: Borrow<Q> {
        let Ok(hm) = self.1.read() else {
            log::warn!("Couldn't access TextureManager's String-ID HashMap! (this indicates a poisoned lock -> Something panicked!)");
            return false;
        };
        hm.get(key).is_some()
    }
}