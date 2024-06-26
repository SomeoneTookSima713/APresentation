#![allow(unused)]

use std::borrow::Borrow;
use std::cell::{ RefCell, Ref, RefMut };
use std::collections::HashMap;
use std::rc::Rc;

use once_cell::sync::Lazy;

use crate::presentation::property::{ Property, PropertyValue, PropertyEnvironment };
use crate::presentation::config;

pub mod resource;

pub mod toml;
use toml::TomlParser;

pub static PARSER_IMPLEMENTATIONS: Lazy<HashMap<String, Box<dyn PresentationParserObjectSafe>>> = Lazy::new(|| [
    (TomlParser::FILE_EXTENSION, TomlParser)
].into_iter().map(|(s, v)| (s.to_owned(), Box::new(v) as Box<dyn PresentationParserObjectSafe>)).collect());

#[derive(Clone, Debug)]
pub struct ParseableRenderable<'lua> {
    properties: Rc<RefCell<HashMap<String, Property<'lua>>>>
}

impl<'lua> ParseableRenderable<'lua> {
    pub fn new(properties: Rc<RefCell<HashMap<String, Property<'lua>>>>) -> Self {
        Self { properties }
    }

    pub fn get<'a, S: std::hash::Hash + std::cmp::Eq>(&'a self, index: &S) -> Option<Ref<'a, Property<'lua>>>
    where String: Borrow<S>, S: ?Sized {
        (*self.properties).try_borrow().ok().and_then(|borrow| Ref::filter_map(borrow, |b| b.get(index)).ok())
    }

    pub fn get_mut<'a, S: std::hash::Hash + std::cmp::Eq>(&'a mut self, index: &S) -> Option<RefMut<'a, Property<'lua>>>
    where String: Borrow<S>, S: ?Sized {
        (*self.properties).try_borrow_mut().ok().and_then(|borrow| RefMut::filter_map(borrow, |b| b.get_mut(index)).ok())
    }

    pub fn extend<'a>(&'a mut self, by: &'a [(String, Property<'lua>)]) -> Result<(), ()> {
        match (*self.properties).try_borrow_mut().ok() {
            Some(mut borrow) => borrow.extend(by.iter().cloned()),
            None => return Err(())
        }
        Ok(())
    }

    pub fn into_property_value(self) -> PropertyValue<'lua> {
        PropertyValue::Dict(self.properties)
    }
}

#[derive(Debug)]
pub struct ParseablePresentation {
    pub slides: Vec<ParseableSlide>,
    pub resources: Vec<Box<dyn resource::Resource>>,
    pub config: config::PresentationConfig,
}

#[derive(Debug)]
pub struct ParseableSlide {
    pub renderables: Vec<ParseableRenderable<'static>>
}

pub trait PresentationParser: Send + Sync {
    const FILE_EXTENSION: &'static str;

    fn parse(file: String, lua: &'static mlua::Lua, env: &PropertyEnvironment) -> anyhow::Result<ParseablePresentation>;
}

pub trait PresentationParserObjectSafe: Send + Sync {
    fn get_file_extension(&self) -> &'static str;

    fn parse(&self, file: String, lua: &'static mlua::Lua, env: &PropertyEnvironment) -> anyhow::Result<ParseablePresentation>;
}

impl<T: PresentationParser> PresentationParserObjectSafe for T {
    fn get_file_extension(&self) -> &'static str { Self::FILE_EXTENSION }

    fn parse(&self, file: String, lua: &'static mlua::Lua, env: &PropertyEnvironment) -> anyhow::Result<ParseablePresentation> { <Self as PresentationParser>::parse(file, lua, env) }
}