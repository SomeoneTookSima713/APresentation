use std::borrow::Borrow;

use hashbrown::HashMap;

use markdown_it::MarkdownIt;

pub struct Parser;

pub enum InlineNode {
    Bold(Vec<InlineNode>),
    Italic(Vec<InlineNode>),
    Strikethrough(Vec<InlineNode>),
    Code(String),
    Link(String, String),
    Text(String),
    SoftLineBreak,
}

impl InlineNode {
    pub fn from_node(node: &markdown_it::Node) -> Self {
        use markdown_it::plugins::cmark::inline;
        use markdown_it::plugins::extra::strikethrough;
        use markdown_it::parser::inline::Text as TextNode;
        if node.is::<TextNode>() {
            InlineNode::Text(node.cast::<TextNode>().unwrap().content.clone())
        } else if node.is::<inline::emphasis::Strong>() {
            InlineNode::Bold(node.children.iter().map(|n| Self::from_node(n)).collect())
        } else if node.is::<inline::emphasis::Em>() {
            InlineNode::Italic(node.children.iter().map(|n| Self::from_node(n)).collect())
        } else if node.is::<strikethrough::Strikethrough>() {
            InlineNode::Strikethrough(node.children.iter().map(|n| Self::from_node(n)).collect())
        } else if node.is::<inline::backticks::CodeInline>() {
            InlineNode::Code(node.collect_text())
        } else if node.is::<inline::link::Link>() {
            InlineNode::Link(node.collect_text(), node.cast::<inline::link::Link>().unwrap().url.clone())
        } else if node.is::<inline::newline::Softbreak>() {
            InlineNode::SoftLineBreak
        } else {
            panic!("Unrecognized inline-level node: {node:#?}");
        }
    }
}

pub enum BlockNode {
    Title(u8, Vec<InlineNode>),
    Paragraph(Vec<InlineNode>),
    Code(String),
    BlockQuote(Vec<BlockNode>),
    HorizontalRule,
    UnorderedList(Vec<Vec<InlineNode>>),
    OrderedList(Vec<Vec<InlineNode>>),
    // Image(String), // TODO
}

impl Parser {
    fn parse_node(node: &markdown_it::Node) -> (Vec<BlockNode>, Vec<(String, String)>) {
        use markdown_it::plugins::cmark::block::heading::ATXHeading;
        use markdown_it::plugins::cmark::block::lheading::SetextHeader;
        use markdown_it::plugins::cmark::block::code::CodeBlock;
        use markdown_it::plugins::cmark::block::list::{ BulletList, OrderedList };
        use markdown_it::plugins::cmark::block::reference::Definition as ReferenceDefinition;
        use markdown_it::plugins::cmark::block::hr::ThematicBreak;
        use markdown_it::plugins::cmark::block::paragraph::Paragraph;
        use markdown_it::plugins::cmark::block::blockquote::Blockquote;

        let mut res = Vec::with_capacity(2);
        let mut references = Vec::new();

        if node.is::<ATXHeading>() {
            let mut children = Vec::new();
            for inline_node in node.children.iter() {
                children.push(InlineNode::from_node(inline_node));
            }
            res.push(BlockNode::Title(node.cast::<ATXHeading>().unwrap().level, children));
        } else if node.is::<SetextHeader>() {
            let mut children = Vec::new();
            for inline_node in node.children.iter() {
                children.push(InlineNode::from_node(inline_node));
            }
            res.push(BlockNode::Title(node.cast::<SetextHeader>().unwrap().level, children));
            res.push(BlockNode::HorizontalRule);
        } else if node.is::<CodeBlock>() {
            res.push(BlockNode::Code(node.cast::<CodeBlock>().unwrap().content.clone()));
        } else if node.is::<BulletList>() {
            let mut items = Vec::new();
            for item in &node.children {
                items.push(item.children.iter().map(|n| InlineNode::from_node(n)).collect());
            }
            res.push(BlockNode::UnorderedList(items));
        } else if node.is::<OrderedList>() {
            let mut items = Vec::new();
            for item in &node.children {
                items.push(item.children.iter().map(|n| InlineNode::from_node(n)).collect());
            }
            res.push(BlockNode::OrderedList(items));
        } else if node.is::<ReferenceDefinition>() {
            let def = node.cast::<ReferenceDefinition>().unwrap();
            references.push((def.label.clone(), def.destination.clone()));
        } else if node.is::<ThematicBreak>() {
            res.push(BlockNode::HorizontalRule);
        } else if node.is::<Paragraph>() {
            res.push(BlockNode::Paragraph(node.children.iter().map(|n| InlineNode::from_node(n)).collect()));
        } else if node.is::<Blockquote>() {
            res.push(BlockNode::BlockQuote(node.children.iter().flat_map(|n| {
                let nodes_and_refs = Self::parse_node(n);
                for (label, dest) in nodes_and_refs.1 {
                    references.push((label, dest));
                }
                nodes_and_refs.0
            }).collect()));
        } else {
            panic!("Unrecognized block-level node! {node:#?}");
        }

        (res, references)
    }

    pub fn parse<S: Borrow<str>>(text: S) -> (Vec<BlockNode>, HashMap<String, String>) {
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
        markdown_it_tasklist::add(&mut parser);

        // markdown_it::plugins::extra::add(&mut parser); // TODO; Instead, a subset of extra plugins is implemented
        markdown_it::plugins::extra::strikethrough::add(&mut parser);
        markdown_it::plugins::extra::smartquotes::add(&mut parser);
        markdown_it::plugins::extra::typographer::add(&mut parser);
        // markdown_it_tasklist::add(&mut parser); // TODO

        let mut res = Vec::new();
        let mut references = HashMap::new();

        for node in parser.parse(text.borrow()).children.iter() {
            let nodes_and_refs = Self::parse_node(node);
            res.extend(nodes_and_refs.0);
            for (label, dest) in nodes_and_refs.1 {
                references.insert(label, dest);
            }
        }

        (res, references)
    }
}