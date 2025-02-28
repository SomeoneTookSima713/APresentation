use std::io::Read;

use hashbrown::HashSet;

use crate::presentation::PresentationState;
use crate::presentation::element::RegisteredElements;

pub mod impls;
pub mod data;

pub trait Parser {
    /// A list of file extensions that are automatically recognized to be
    /// parseable by this parser implementation.
    const FILE_EXTENSIONS: &'static [&'static str];

    /// Parses a file and returns a list of [`PresentationState`]s.
    fn parse(file: impl Read, registered_elements: &RegisteredElements, rhai_engine: &rhai::Engine) -> Result<Vec<PresentationState>, ParserError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("input/output error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("error during tokenization stage: {0}")]
    TokenizerError(anyhow::Error),
    #[error("error during creation of Value: {0}")]
    ValueCreationError(anyhow::Error),
    #[error("generic error during parsing: {0}")]
    GenericError(anyhow::Error)
}

struct RegisteredParser {
    file_extensions: &'static [&'static str],
    parse_fn: Box<dyn for<'a> Fn(&'a mut dyn Read, &'a RegisteredElements, &'a rhai::Engine) -> Result<Vec<PresentationState>, ParserError>>
}

impl RegisteredParser {
    fn new<P: Parser>() -> Self {
        Self {
            file_extensions: P::FILE_EXTENSIONS,
            parse_fn: Box::new(|f, r, e| P::parse(f, r, e))
        }
    }
}

impl std::hash::Hash for RegisteredParser {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for s in self.file_extensions.iter() {
            state.write_str(s);
        }
    }
}

impl std::cmp::PartialEq for RegisteredParser {
    fn eq(&self, other: &Self) -> bool {
        self.file_extensions.iter().zip(other.file_extensions.iter()).all(|(a, b)| a.eq(b))
    }
}

impl std::cmp::Eq for RegisteredParser {}

pub struct ParserCollection(HashSet<RegisteredParser>);

impl ParserCollection {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn register_parser<P: Parser>(&mut self) {
        self.0.insert(RegisteredParser::new::<P>());
    }

    pub fn parse<'a>(
        &self,
        file: &mut impl Read,
        filename: &'a str,
        registered_elements: &RegisteredElements,
        rhai_engine: &rhai::Engine
    ) -> Result<Vec<PresentationState>, ParserError> {
        let _span = tracing::info_span!("apresentation::presentation::parser::ParserCollection::parse()");

        match std::path::Path::new(filename).extension() {
            Some(ext)
            if let Some(v) = self.0.iter().find(|table| table.file_extensions.contains(&&*ext.to_string_lossy())) => {
                (v.parse_fn)(file, registered_elements, rhai_engine)
            },
            _ => {
                tracing::warn!(filename = filename, "No suitable parser for supplied extension found! Using first registered parser.");
                if let Some(v) = self.0.iter().next() {
                    (v.parse_fn)(file, registered_elements, rhai_engine)
                } else {
                    tracing::error!("No parsers registered!");
                    panic!("No parsers registered!");
                }
            }
        }
    }
}