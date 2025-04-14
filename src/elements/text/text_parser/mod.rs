use super::{ PropertyCompatible, Value };

#[derive(Clone, Debug)]
pub struct TextProperty {
    pub text: String
}

pub struct UncompTextProp {
    text: String
}

impl PropertyCompatible for TextProperty {
    type InnerRepresentation = UncompTextProp;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        match val {
            Value::String(s) | Value::Option(Some(box Value::String(s))) => {
                Some(UncompTextProp { text: s })
            },
            _ => None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(TextProperty { text: base.text.clone() })
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        Some((
            "TextProperty".to_string(),
            rhai::Module::new()
        ))
    }
}

#[derive(Debug, thiserror::Error)]
enum MarkdownParseError {

}

fn parse_markdown(string: &str) -> Result<UncompTextProp, MarkdownParseError> {
    use markdown_it::MarkdownIt;
    use markdown_it::plugins::{ cmark, extra };

    macro_rules! add_plugins {
        ($p:ident;$($($path:tt)::+),*) => {
            $(add_plugins!($p ;; $($path)::+));*
        };
        ($p:ident ;; $($path:tt)::+) => {
            $($path)::+ ::add(&mut $p);
        };
    }

    let mut parser = MarkdownIt::new();
    add_plugins!(
        parser;
        cmark::inline::newline,
        cmark::inline::escape,
        cmark::inline::emphasis,
        cmark::inline::backticks,
        cmark::block::heading,
        cmark::block::blockquote,
        cmark::block::fence,
        cmark::block::lheading,
        cmark::block::list,
        cmark::block::paragraph,
        extra::typographer,
        extra::syntect,
        extra::strikethrough
    );

    let root_node = parser.parse(string);

    todo!()
}