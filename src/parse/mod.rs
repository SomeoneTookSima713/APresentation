use std::fmt::Debug;

pub mod json;

pub trait Parser {
    type FullResult;
    type FontResult;
    type Error: Debug;

    fn parse<'a>(&mut self, contents: &'a str) -> Result<Self::FullResult, Self::Error>;

    fn parse_fonts<'a>(&mut self, contents: &'a str) -> Result<Self::FontResult, Self::Error>;
}

pub use json::JSONParser;