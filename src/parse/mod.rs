use std::fmt::Debug;
use std::path::Path;
use std::collections::HashMap;

use crate::presentation::Renderable;

pub mod json;

pub trait Parser {
    type Error: serde::de::Error;

    fn parse<'a>(&mut self, contents: &'a str) -> Result<Vec<SlideData>, Self::Error>;

    fn parse_fonts<'a>(&mut self, contents: &'a str) -> Result<HashMap<String, (String, String)>, Self::Error>;

    fn handle_error(&self, err: Self::Error);
}

#[derive(Debug)]
pub struct SlideData {
    pub background: Box<dyn Renderable>,
    pub content: HashMap<u8, Vec<Box<dyn Renderable>>>
}

pub use json::JSONParser;

/// Automatically chooses a parser based on the supplied filename and returns it.
/// 
/// Returns [`None`] if no suitable parser was found.
pub fn get_parser<P: AsRef<Path>>(file: P) -> Option<impl Parser> {
    match file.as_ref().extension()?.to_string_lossy().as_ref() {
        "hjson" | "json" | "json5" => Some(JSONParser),
        _ => None
    }
}