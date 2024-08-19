use std::borrow::Borrow;

use markdown_it::MarkdownIt;

pub struct Parser;

impl Parser {
    pub fn parse<S: Borrow<str>>(text: S) {
        let mut parser = MarkdownIt::new();

        {
            use markdown_it::plugins::cmark;
            cmark::inline::backticks::add(&mut parser);
            cmark::inline::emphasis::add(&mut parser);
            cmark::inline::entity::add(&mut parser);
            cmark::inline::escape::add(&mut parser);
            cmark::inline::link::add(&mut parser);
            cmark::inline::newline::add(&mut parser);
            cmark::block::blockquote::add(&mut parser);
            cmark::block::code::add(&mut parser);
            cmark::block::fence::add(&mut parser);
            cmark::block::heading::add(&mut parser);
            cmark::block::hr::add(&mut parser);
            cmark::block::lheading::add(&mut parser);
            cmark::block::list::add(&mut parser);
            cmark::block::paragraph::add(&mut parser);
            cmark::block::reference::add(&mut parser);
        }
        markdown_it::plugins::extra::add(&mut parser);


    }
}