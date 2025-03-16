use pulldown_cmark::{ Parser, Options };

use super::{ PropertyCompatible, Value };

#[derive(Clone, Debug)]
pub struct TextProperty {

}

pub struct UncompTextProp {

}

impl PropertyCompatible for TextProperty {
    type InnerRepresentation = UncompTextProp;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        match val {
            Value::String(s) | Value::Option(Some(box Value::String(s))) => {
                

                todo!()
            },
            _ => None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        todo!()
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        todo!()
    }
}