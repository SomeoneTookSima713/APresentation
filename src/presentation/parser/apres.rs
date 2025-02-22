use super::{ Parser, ParserError };

pub struct ApresParser;

impl Parser for ApresParser {
    const FILE_EXTENSIONS: &[&str] = &[ "apres" ];

    fn parse(mut file: impl std::io::Read) -> Result<Vec<crate::presentation::PresentationState>, ParserError> {
        let mut contents_str = String::new();
        file.read_to_string(&mut contents_str)?;

        

        todo!()
    }
}