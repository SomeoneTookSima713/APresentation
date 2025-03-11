use std::iter::{ Enumerate, Peekable };
use std::rc::Rc;
use std::str::Chars;

use hashbrown::HashMap;

use crate::util::structural_eq;
use crate::presentation::parser::data::{ ParsedStructure, Value };
use crate::presentation::element::RegisteredElements;
use crate::presentation::asset::{ AssetManager, AssetLoadingParams };
use crate::presentation::{ ElementID, PresentationState };
use super::{ Parser, ParserError, ParsedPresentationStates };

mod tokenization {
    #[derive(Clone, Copy, Debug)]
    pub enum Token<'a> {
        Punctuation(Punctuation),
        Literal(Literal<'a>),
        Identifier(&'a str),
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Punctuation {
        OpeningParen,
        ClosingParen,
        OpeningBracket,
        ClosingBracket,
        OpeningBrace,
        ClosingBrace,
        Colon,
        Comma
    }

    impl TryFrom<char> for Punctuation {
        type Error = char;
        
        fn try_from(value: char) -> Result<Self, Self::Error> {
            use Punctuation::*;
            Ok(match value {
                '(' => OpeningParen,
                '[' => OpeningBracket,
                '{' => OpeningBrace,
                ')' => ClosingParen,
                ']' => ClosingBracket,
                '}' => ClosingBrace,
                ':' => Colon,
                ',' => Comma,
                c => Err(c)?
            })
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Literal<'a> {
        String(&'a str, bool),
        Number(&'a str),
        Boolean(bool),
        None
    }

    #[derive(Debug, thiserror::Error, Clone, Copy)]
    pub enum TokenizerErrorKind {
        #[error("expected '/' or '*' after '/' to indicate comment, got {0}")]
        InvalidComment(char),
        #[error("unexpected end of file")]
        UnexpectedEOF,
        #[error("unexpected character: {0}")]
        UnexpectedChar(char)
    }

    #[derive(Debug, Clone, Copy)]
    pub struct TokenizerError {
        col: usize,
        line: usize,
        kind: TokenizerErrorKind
    }

    impl TokenizerError {
        pub(super) fn new(col: usize, line: usize, kind: TokenizerErrorKind) -> Self {
            Self { col, line, kind }
        }
    }

    impl std::fmt::Display for TokenizerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Tokenizer encountered an error on line {}, column {}: {}", self.line, self.col, self.kind)
        }
    }
}
use tokenization::*;
pub use tokenization::{ TokenizerError, TokenizerErrorKind };

mod ast {
    #[derive(Clone, Debug)]
    pub enum Command<'a> {
        Slide(String, [f64; 3]),
        NewElem {
            ident: Option<String>,
            elem_type: String,
            value_struct: Vec<super::Token<'a>>
        },
        ModifyElem {
            ident: String,
            value_struct: Vec<super::Token<'a>>
        },
        Discard(String)
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ParserError<'a> {
        line: usize,
        col: usize,
        kind: ParserErrorKind<'a>
    }

    impl<'a> ParserError<'a> {
        pub fn new(location: super::FileLocation, kind: ParserErrorKind<'a>) -> Self {
            Self { line: location.line, col: location.column, kind }
        }
    }

    #[derive(Clone, Copy, Debug, thiserror::Error)]
    pub enum ParserErrorKind<'a> {
        #[error("encountered unexpected token: {0:?}")]
        UnexpectedToken(super::Token<'a>)
    }

    impl<'a> std::fmt::Display for ParserError<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Parser encountered an error on line {}, column {}: {}", self.line, self.col, self.kind)
        }
    }
}
use ast::*;

#[derive(Clone, Copy, Debug)]
struct FileLocation {
    pub line: usize,
    pub column: usize
}

pub struct ApresParser<'a> {
    file: &'a str,
    iter: Peekable<Enumerate<Chars<'a>>>,
    tokens: Vec<Token<'a>>,
    token_locations: Vec<FileLocation>,
    commands: Vec<Command<'a>>
}

impl<'a> ApresParser<'a> {
    fn new(file: &'a str) -> Self {
        Self {
            file,
            iter: file.chars().enumerate().peekable(),
            tokens: Vec::new(),
            token_locations: Vec::new(),
            commands: Vec::new()
        }
    }

    fn push_token(&mut self, token: (Token<'a>, FileLocation)) {
        self.tokens.push(token.0);
        self.token_locations.push(token.1);
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.iter.peek().copied()
    }

    fn erroring_peek(&mut self, col: usize, line: usize) -> Result<(usize, char), TokenizerError> {
        self.peek().ok_or(TokenizerError::new(col, line, TokenizerErrorKind::UnexpectedEOF))
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.iter.next()
    }

    fn advance(&mut self) {
        self.next();
    }

    fn parse_tokens_to_color(tokens: &[Token<'_>]) -> Option<[f64; 3]> {
        match tokens[0..7] {
            [
                Token::Punctuation(Punctuation::OpeningParen),
                Token::Literal(Literal::Number(n1)),
                Token::Punctuation(Punctuation::Comma),
                Token::Literal(Literal::Number(n2)),
                Token::Punctuation(Punctuation::Comma),
                Token::Literal(Literal::Number(n3)),
                Token::Punctuation(Punctuation::ClosingParen)
            ] => Some([ n1.parse().ok()?, n2.parse().ok()?, n3.parse().ok()? ]),
            _ => None
        }
    }

    fn parse_to_tokens(&mut self) -> Result<(), TokenizerError> {
        const STRING_DELIMITERS: &[char] = &['"', '\''];
        const IDENT_ADDITIONAL_CHARS: &[char] = &['_', '-'];

        let mut col = 0;
        let mut line = 1;

        // Please ignore the convolutedness of the rest of this function...

        let mut in_singleline_comment = false;
        let mut in_multiline_comment = false;

        let mut token_start: usize = 0;
        let mut token_location: FileLocation = FileLocation { line: 0, column: 0 };

        let mut last_char: char = '\0';

        let mut in_string = false;
        let mut is_multiline_string = false;
        let mut string_quote_type: char = '"';

        let mut in_digit = false;

        let mut in_ident = false;

        while let Some((i, c)) = self.next() {
            if c == '\n' {
                col = 1;
                line += 1;
            } else {
                col += 1;
            }

            if in_singleline_comment {
                in_singleline_comment = c != '\n';
                last_char = c;
                continue;
            } else if in_multiline_comment {
                if c == '*' && self.erroring_peek(col, line)?.1 == '/' {
                    in_multiline_comment = false;
                    self.advance();
                }
                last_char = c;
                continue;
            } else if in_string {
                // tracing::debug!("In String! {c}");
                if is_multiline_string {
                    if c == string_quote_type && self.erroring_peek(col, line)?.1 == string_quote_type {
                        self.advance();
                        if self.erroring_peek(col, line)?.1 == string_quote_type {
                            self.push_token((Token::Literal(Literal::String(&self.file[token_start+3..i], true)), token_location));
                            in_string = false;
                        }
                    }
                } else if c == string_quote_type && last_char != '\\' {
                    self.push_token((Token::Literal(Literal::String(&self.file[token_start+1..i], false)), token_location));
                    in_string = false;
                } else if c == '\n' {
                    Err(TokenizerError::new(col, line, TokenizerErrorKind::UnexpectedEOF))?;
                }
                last_char = c;
                continue;
            } else if in_digit {
                if !(c.is_ascii_digit() || c == '.' || c == '-') {
                    self.push_token((Token::Literal(Literal::Number(&self.file[token_start..i])), token_location));
                    in_digit = false;
                } else {
                    last_char = c;
                    continue;
                }
            } else if in_ident {
                if !(c.is_alphanumeric() || IDENT_ADDITIONAL_CHARS.contains(&c)) {
                    let str_slice = &self.file[token_start..i];
                    if str_slice.eq("true") || str_slice.eq("false") {
                        self.push_token((Token::Literal(Literal::Boolean(str_slice.eq("true"))), token_location));
                    } else if str_slice.eq("None") {
                        self.push_token((Token::Literal(Literal::None), token_location));
                    } else {
                        self.push_token((Token::Identifier(&self.file[token_start..i]), token_location));
                    }
                    in_ident = false;
                } else {
                    last_char = c;
                    continue;
                }
            }

            if c.is_whitespace() {
                last_char = c;
                continue;
            }

            match c {
                // Comment handling
                '/' => {
                    match self.erroring_peek(col, line)?.1 {
                        '/' => in_singleline_comment = true,
                        '*' => in_multiline_comment = true,
                        c => Err(TokenizerError::new(col, line, TokenizerErrorKind::InvalidComment(c)))?
                    }
                },
                c if STRING_DELIMITERS.contains(&c) => {
                    string_quote_type = c;
                    is_multiline_string = false;
                    if self.erroring_peek(col, line)?.1 == string_quote_type {
                        self.advance();
                        if self.erroring_peek(col, line)?.1 == string_quote_type {
                            is_multiline_string = true;
                            self.advance();
                        }
                    }
                    in_string = true;
                    token_start = i;
                    token_location = FileLocation { line, column: col };
                },
                c if c.is_ascii_digit() || c == '.' || c == '-' => {
                    in_digit = true;
                    token_start = i;
                    token_location = FileLocation { line, column: col };
                },
                c if c.is_alphanumeric() || IDENT_ADDITIONAL_CHARS.contains(&c) => {
                    in_ident = true;
                    token_start = i;
                    token_location = FileLocation { line, column: col };
                },
                c if let Ok(p) = Punctuation::try_from(c) => {
                    self.push_token((Token::Punctuation(p), FileLocation { line, column: col }));
                },
                c => {
                    tracing::debug!(
                        ?in_string,
                        ?is_multiline_string,
                        ?string_quote_type,
                        ?in_digit,
                        ?in_ident
                    );
                    Err(TokenizerError::new(col, line, TokenizerErrorKind::UnexpectedChar(c)))?
                }
            }
            last_char = c;
        }

        if in_string {
            Err(TokenizerError::new(col, line, TokenizerErrorKind::UnexpectedEOF))?;
        } else if in_digit {
            self.push_token((Token::Literal(Literal::Number(&self.file[token_start..])), token_location));
        } else if in_ident {
            self.push_token((Token::Identifier(&self.file[token_start..]), token_location));
        }

        Ok(())
    }

    fn parse_to_ast(&mut self) -> Result<(), ast::ParserError<'a>> {
        #[derive(Clone, Copy, Debug)]
        enum ExpectedSemantic {
            CommandIdent,
            StringLiteral,
            StringLiteralOrIdent,
            Ident,
            Paren(u8)
        }

        let mut expected = ExpectedSemantic::CommandIdent;

        let mut current_command: Option<String> = None;
        let mut command_literal: Option<String> = None;
        let mut command_element_type: Option<String> = None;
        let mut command_element_tokens_start: Option<usize> = None;

        for (i, token) in self.tokens.iter().enumerate() {
            match (expected, token) {
                (ExpectedSemantic::CommandIdent, Token::Identifier("slide")) => {
                    current_command = Some(String::from("slide"));
                    expected = ExpectedSemantic::StringLiteral;
                },
                (ExpectedSemantic::CommandIdent, Token::Identifier("discard")) => {
                    current_command = Some(String::from("discard"));
                    expected = ExpectedSemantic::StringLiteral;
                },
                (ExpectedSemantic::CommandIdent, Token::Identifier("newelem")) => {
                    current_command = Some(String::from("newelem"));
                    expected = ExpectedSemantic::StringLiteralOrIdent;
                },
                (ExpectedSemantic::CommandIdent, Token::Identifier("modifyelem")) => {
                    current_command = Some(String::from("modifyelem"));
                    expected = ExpectedSemantic::StringLiteral;
                },
                (ExpectedSemantic::StringLiteral, Token::Literal(Literal::String(s, _multiline))) => {
                    match current_command.as_ref().map(|s| s.as_str()).unwrap() {
                        "modifyelem" | "slide" => {
                            command_literal = Some(s.to_string());
                            command_element_tokens_start = Some(i);
                            expected = ExpectedSemantic::Paren(0);
                        },
                        "discard" => {
                            self.commands.push(Command::Discard(s.to_string()));
                            current_command = None;
                            command_literal = None;
                            command_element_type = None;
                            command_element_tokens_start = None;
                            expected = ExpectedSemantic::CommandIdent;
                        },
                        _ => unreachable!()
                    }
                },
                (ExpectedSemantic::StringLiteralOrIdent, Token::Literal(Literal::String(s, _multiline))) => {
                    command_literal = Some(s.to_string());
                    expected = ExpectedSemantic::Ident;
                },
                (ExpectedSemantic::StringLiteralOrIdent | ExpectedSemantic::Ident, Token::Identifier(s)) => {
                    command_element_type = Some(s.to_string());
                    command_element_tokens_start = Some(i);
                    expected = ExpectedSemantic::Paren(0);
                }
                (ExpectedSemantic::Paren(1), Token::Punctuation(Punctuation::ClosingParen)) => {
                    self.commands.push(match current_command.as_ref().unwrap().as_str() {
                        "newelem" => Command::NewElem {
                            ident: command_literal,
                            elem_type: command_element_type.take().unwrap(),
                            value_struct: self.tokens[command_element_tokens_start.take().unwrap()..=i].iter().cloned().collect()
                        },
                        "modifyelem" => Command::ModifyElem {
                            ident: command_literal.take().unwrap(),
                            value_struct: self.tokens[command_element_tokens_start.take().unwrap()..=i].iter().cloned().collect()
                        },
                        "slide" => Command::Slide(
                            command_literal.take().unwrap(),
                            Self::parse_tokens_to_color(&self.tokens[command_element_tokens_start.take().unwrap()+1..=i])
                                .ok_or(ast::ParserError::new(self.token_locations[i], ParserErrorKind::UnexpectedToken(*token)))?
                        ),
                        _ => unreachable!()
                    });
                    current_command = None;
                    command_literal = None;
                    command_element_type = None;
                    command_element_tokens_start = None;
                    expected = ExpectedSemantic::CommandIdent;
                },
                (ExpectedSemantic::Paren(n), t) => {
                    if let Token::Punctuation(Punctuation::OpeningParen) = t {
                        expected = ExpectedSemantic::Paren(n+1);
                    } else if let Token::Punctuation(Punctuation::ClosingParen) = t {
                        expected = ExpectedSemantic::Paren(n-1);
                    }
                },
                (_, t) => Err(ast::ParserError::new(self.token_locations[i], ParserErrorKind::UnexpectedToken(*t)))?
            }
        }

        Ok(())
    }

    fn find_token_paren_matching<P: Fn(&'a Token<'a>) -> bool>(tokens: &'a [Token<'a>], predicate: P) -> Option<usize> {
        let mut paren = 0;
        for (i, t) in tokens.iter().enumerate() {
            match t {
                _ if (predicate)(t) && paren == 0 => return Some(i),
                Token::Punctuation(Punctuation::OpeningBrace | Punctuation::OpeningBracket | Punctuation::OpeningParen) => paren += 1,
                Token::Punctuation(Punctuation::ClosingBrace | Punctuation::ClosingBracket | Punctuation::ClosingParen) => paren -= 1,
                _ => {}
            }
        }
        None
    }

    #[tracing::instrument]
    fn parse_map_value_from_tokens(tokens: &'a [Token<'a>], token_amount: usize) -> Result<HashMap<String, Value>, ParserError> {
        let mut map = HashMap::new();
        let mut token_ind = 1;

        while token_ind < token_amount-1 && let Some(token) = tokens.get(token_ind) {
            let mut traversed_tokens = 0;

            // Makes code more readable by long code snippets
            macro_rules! gt { () => {{ tracing::debug!("Getting token at index {} + {}", token_ind, traversed_tokens); tokens.get(token_ind + traversed_tokens) }}; }
            macro_rules! err { ($($msg:tt)*) => { {tracing::error!($($msg)*); return Err(ParserError::ValueCreationError(anyhow::anyhow!($($msg)*))) }} }

            let ident;
            if let Token::Identifier(id) = token {
                ident = id;
            } else {
                err!("Expected identifier in map, got {token:?}!");
            }

            traversed_tokens += 1;
            if !structural_eq!(gt!(); Some(Token::Punctuation(Punctuation::Colon))) {
                err!("Expected colon after identifier, got {:?}!", gt!());
            }
            traversed_tokens += 1;
            if gt!().is_none() { err!("Expected value, got nothing!") }
            let replaced = match Self::find_token_paren_matching(
                &tokens[token_ind+traversed_tokens..],
                |t| structural_eq!(t; Token::Punctuation(Punctuation::Comma))
            ) {
                Some(i) => {
                    let r = map.insert(ident.to_string(), Self::tokens_to_value(&tokens[token_ind+traversed_tokens..token_ind+traversed_tokens+i])?);
                    traversed_tokens += i+1;
                    r
                },
                None => {
                    let r = map.insert(ident.to_string(), Self::tokens_to_value(&tokens[token_ind+traversed_tokens..token_amount-1])?);
                    token_ind = token_amount;
                    r
                }
            };
            if replaced.is_some() {
                tracing::warn!("Index in map defined twice: {ident}");
            }

            token_ind += traversed_tokens;
        }
        Ok(map)
    }

    fn process_string(string: &str, multiline: bool) -> String {
        // Converts any escape sequences inside the string from the
        // source file with the actual escaped characters.
        let string_processed = string
            .replace("\r", "")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\'", "'")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
        // If a multiline string starts or ends with a line break, we remove it.
        let start_ind = if multiline && string.starts_with("\n") { 1 } else { 0 };
        let end_ind = if multiline && string.ends_with("\n") { string_processed.len()-1 } else { string_processed.len() };

        let str_starting_tabs = string_processed[start_ind..].find(|c| c != ' ').unwrap_or(0);
        let tab_str = ['\n'].into_iter().chain(std::iter::repeat_n(' ', str_starting_tabs)).collect::<String>();

        // Gets the slice out of the string without the newlines that
        // may exist at the start and end and removes spaces at the
        // start of every line based on the amount of the first line.
        string_processed[start_ind+str_starting_tabs..end_ind]
            .replace(&tab_str, "\n")
    }

    #[tracing::instrument]
    fn tokens_to_value(tokens: &'a [Token<'a>]) -> Result<Value, ParserError> {
        let token_amount = tokens.len();
        if token_amount == 0 { Err(ParserError::ValueCreationError(anyhow::anyhow!("No tokens for value deserialization supplied!")))? }

        if token_amount == 4 && let [
            Token::Identifier("Rhai"),
            Token::Punctuation(Punctuation::OpeningParen),
            Token::Literal(Literal::String(s, multiline)),
            Token::Punctuation(Punctuation::ClosingParen),
        ] = &tokens[0..token_amount] {
            return Ok(Value::RhaiCode(Self::process_string(s, *multiline)))
        }

        match &tokens[0] {
            Token::Literal(Literal::Boolean(b)) => Ok(Value::Bool(*b)),
            Token::Literal(Literal::String(s, multiline)) => {
                Ok(Value::String(Self::process_string(s, *multiline)))
            },
            Token::Literal(Literal::Number(numstr)) => {
                match (numstr.parse::<i64>(), numstr.parse::<f64>()) {
                    (Ok(i), _) => Ok(Value::Int(i)),
                    (Err(_), Ok(f)) => Ok(Value::Float(f)),
                    _ => Err(ParserError::ValueCreationError(anyhow::anyhow!("Couldn't convert '{numstr}' into integer or float value!")))
                }
            },
            Token::Literal(Literal::None) => Ok(Value::Option(None)),
            Token::Identifier(ident) => { // Enum
                let enum_value = if token_amount>1 {
                    Some(Box::new(Self::tokens_to_value(&tokens[1..])?))
                } else {
                    None
                };
                
                Ok(Value::EnumVariant(ident.to_string(), enum_value))
            },
            // Map
            Token::Punctuation(Punctuation::OpeningBrace) => Ok(Value::Map(Self::parse_map_value_from_tokens(&tokens[..], token_amount)?)),
            Token::Punctuation(Punctuation::OpeningParen | Punctuation::OpeningBracket) => { // List/Tuple (they're the same as a Value)
                let mut list = Vec::new();
                let mut token_ind = 1;

                while token_ind < token_amount-1 && let Some(_) = tokens.get(token_ind) {
                    match Self::find_token_paren_matching(
                        &tokens[token_ind..],
                        |t| structural_eq!(t; Token::Punctuation(Punctuation::Comma))
                    ) {
                        Some(i) => {
                            list.push(Self::tokens_to_value(&tokens[token_ind..token_ind+i])?);
                            token_ind += i+1;
                        },
                        None => {
                            list.push(Self::tokens_to_value(&tokens[token_ind..token_amount-1])?);
                            token_ind = token_amount;
                        }
                    };
                }

                Ok(Value::List(list))
            }
            t => Err(ParserError::ValueCreationError(anyhow::anyhow!("Token {t:?} cannot be the start of a value!")))
        }
    }
}

impl Parser for ApresParser<'static> {
    const FILE_EXTENSIONS: &[&str] = &[ "apres" ];

    fn parse(
        mut file: impl std::io::Read,
        registered_elements: &RegisteredElements,
        engine: &rhai::Engine,
        asset_manager: &mut AssetManager,
        asset_loading_params: AssetLoadingParams,
    ) -> Result<Vec<PresentationState>, ParserError> {
        let mut string_full = String::new();
        file.read_to_string(&mut string_full)?;
        let (asset_string, apres_string) = match string_full.split_once("\n---\n").or_else(|| string_full.split_once("\r\n---\r\n")) {
            Some(v) => v,
            None => ("", string_full.as_str())
        };

        let asset_table: toml::Table = toml::from_str(asset_string)?;

        for (k, v) in asset_table.iter() {
            let type_id;
            if let Some(tid) = asset_manager.get_asset_type_from_name(k) {
                type_id = tid;
            } else {
                return Err(ParserError::AssetError(crate::presentation::asset::AssetLoadError::ParsingError(anyhow::anyhow!("Couldn't find asset type '{k}'!"))));
            }

            if let toml::Value::Array(vec) = v {
                for val in vec.iter() {
                    if let toml::Value::Table(t) = val {
                        asset_manager.load_asset_dyn(type_id, t.clone(), asset_loading_params.clone())?;
                    } else {
                        tracing::warn!("Invalid asset list formatting, value of asset type array element isn't a table!");
                    }
                }
            } else {
                tracing::warn!("Invalid asset list formatting, value of asset type isn't an array!");
            }
        }

        let mut parser = ApresParser::new(apres_string);

        parser.parse_to_tokens().map_err(|e| ParserError::TokenizerError(anyhow::anyhow!(e)))?;
        parser.parse_to_ast().map_err(|e| ParserError::GenericError(anyhow::anyhow!("{}", e)))?;
        
        let mut states = Vec::new();
        let mut state_element_structures = HashMap::new();
        let mut state_element_types = HashMap::new();
        let mut current_state = None;
        let mut current_state_name = String::new();

        for commmand in parser.commands.iter() {
            match commmand {
                Command::Slide(name, bgcol) => {
                    let s = current_state.replace(PresentationState {
                        background_color: *bgcol,
                        removed_elements: Vec::new(),
                        new_elements: HashMap::new()
                    });
                    current_state_name = name.to_string();
                    if let Some(state) = s {
                        states.push(state);
                    }
                },
                Command::NewElem { ident, elem_type, value_struct } => {
                    if current_state.is_none() { Err(ParserError::GenericError(anyhow::anyhow!("Cannot create element before creating a slide!")))? }

                    let id = match ident {
                        Some(s) => ElementID::Custom(format!("{}::{}", current_state_name, s)),
                        None => ElementID::Generated(rand::random())
                    };

                    let parsed_struct = ParsedStructure::new(ApresParser::parse_map_value_from_tokens(&value_struct[1..], value_struct.len())?);

                    tracing::debug!("Parsed struct: {parsed_struct:?}");

                    if ident.is_some() {
                        let ind = format!("{current_state_name}::{}", ident.as_ref().unwrap());
                        state_element_structures.insert(ind.clone(), parsed_struct.clone());
                        state_element_types.insert(ind, elem_type.clone());
                    }

                    if let Some(e) = registered_elements.element_types.get(elem_type) {
                        let boxed_elem;
                        match e.construct(parsed_struct, engine) {
                            Ok(e) => boxed_elem = e,
                            Err(e) => return Err(ParserError::GenericError(anyhow::anyhow!("Element creation failed: {e}")))
                        }
                        current_state.as_mut().unwrap().new_elements.insert(id, Rc::from(boxed_elem));
                    } else {
                        return Err(ParserError::GenericError(anyhow::anyhow!("No element of type '{elem_type}' found!")));
                    }
                },
                Command::ModifyElem { ident, value_struct } => {
                    let id = ElementID::Custom(ident.to_string());

                    let parsed_struct_overlay = ParsedStructure::new(ApresParser::parse_map_value_from_tokens(&value_struct[..], value_struct.len())?);

                    let mut parsed_struct;
                    if let Some(s) = state_element_structures.get(ident) {
                        parsed_struct = s.clone();
                    } else {
                        return Err(ParserError::GenericError(anyhow::anyhow!("Cannot modify nonexistent element {ident}!")));
                    }
                    for (k, v) in parsed_struct_overlay.into_inner().into_iter() {
                        parsed_struct.inner_mut().insert(k, v);
                    }

                    // We can unwrap here since we already know `state_element_structures` has that
                    // key, and we only populate both HashMaps at once.
                    let elem_type = state_element_types.get(ident).unwrap();
                    if let Some(e) = registered_elements.element_types.get(elem_type) {
                        let boxed_elem;
                        match e.construct(parsed_struct, engine) {
                            Ok(e) => boxed_elem = e,
                            Err(e) => return Err(ParserError::GenericError(anyhow::anyhow!("Element creation failed: {e}")))
                        }
                        current_state.as_mut().unwrap().removed_elements.push(ElementID::Custom(ident.clone()));
                        current_state.as_mut().unwrap().new_elements.insert(id, Rc::from(boxed_elem));
                    } else {
                        return Err(ParserError::GenericError(anyhow::anyhow!("No element of type '{elem_type}' found!")));
                    }
                },
                Command::Discard(ident) => {
                    current_state.as_mut().unwrap().removed_elements.push(ElementID::Custom(ident.clone()));
                }
            }
        }

        let s = current_state.take();
        if let Some(state) = s {
            states.push(state);
        }

        Ok(states)
    }
}