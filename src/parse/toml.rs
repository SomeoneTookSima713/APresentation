use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::presentation::property::{ Property, PropertyValue, EvaluatedPropertyValue, PropertyEnvironment };
use super::{ ParseablePresentation, ParseableSlide, ParseableRenderable, PresentationParser, resource };

use toml::Table;

fn value_to_property(val: toml::Value, lua: &'static mlua::Lua, env: &PropertyEnvironment) -> anyhow::Result<Property<'static>> {
    Ok(match val {
        toml::Value::Array(v) => Property::Constant(PropertyValue::List(Rc::new(RefCell::new(v.into_iter().map(|v| value_to_property(v, lua, env)).try_collect::<Vec<_>>()?)))),
        toml::Value::Boolean(b) => Property::Constant(PropertyValue::Bool(b)),
        toml::Value::Datetime(_) => panic!("Dates aren't supported!"),
        toml::Value::Float(f) => Property::Constant(PropertyValue::Float(f)),
        toml::Value::Integer(i) => Property::Constant(PropertyValue::Int(i)),
        toml::Value::String(s) => if s.starts_with("LUA#") {
            Property::from_lua_string(lua, s.replacen("LUA#", "", 1), env.clone())?
        } else {
            Property::Constant(PropertyValue::String(Rc::new(s)))
        },
        toml::Value::Table(t) => Property::Constant(PropertyValue::Dict(Rc::new(RefCell::new(t.into_iter().map(|(k, v)| Ok::<_, anyhow::Error>((k, value_to_property(v, lua, env)?))).try_collect::<HashMap<_,_>>()?)))),
    })
}

fn value_to_evaluated_property(val: toml::Value) -> EvaluatedPropertyValue {
    match val {
        toml::Value::Array(v) => EvaluatedPropertyValue::List(Rc::new(RefCell::new(v.into_iter().map(|v| value_to_evaluated_property(v)).collect::<Vec<_>>()))),
        toml::Value::Boolean(b) => EvaluatedPropertyValue::Bool(b),
        toml::Value::Datetime(_) => panic!("Dates aren't supported!"),
        toml::Value::Float(f) => EvaluatedPropertyValue::Float(f),
        toml::Value::Integer(i) => EvaluatedPropertyValue::Int(i),
        toml::Value::String(s) => EvaluatedPropertyValue::String(Rc::new(s)),
        toml::Value::Table(t) => EvaluatedPropertyValue::Dict(Rc::new(RefCell::new(t.into_iter().map(|(k, v)| (k, value_to_evaluated_property(v))).collect::<HashMap<_,_>>()))),
    }
}

pub struct TomlParser;

impl PresentationParser for TomlParser {
    const FILE_EXTENSION: &'static str = "toml";

    fn parse(file: String, lua: &'static mlua::Lua, env: &PropertyEnvironment) -> anyhow::Result<ParseablePresentation> {
        let file: Table = file.parse()?;

        let slides = file
            .get("slide").ok_or(anyhow::anyhow!("No slides defined in file!"))?
            .as_array().ok_or(anyhow::anyhow!("Slides array not defined as array of slides!"))?;
        
        let parseable_slides = slides.iter().enumerate().map::<anyhow::Result<ParseableSlide>, _>(|(i, slide)| {
                let slide = slide.as_table().ok_or(anyhow::anyhow!("Slide needs to be a table!"))?;
                let elements = slide.get("element").cloned().unwrap_or_else(|| {
                    log::warn!("Slide #{} doesn't have a list of elements!", i+1);
                    toml::Value::Array(Vec::new())
                }).as_array().cloned().ok_or(anyhow::anyhow!("Element list of slide #{} isn't an array!", i+1))?;

                let renderables = elements.into_iter().enumerate().map(|(ie, elem)| {
                    if let toml::Value::Table(map) = elem {
                        Ok(ParseableRenderable::new(Rc::new(RefCell::new(map.into_iter().map(|(k,v)| Ok::<_, anyhow::Error>((k,value_to_property(v, lua, env)?))).try_collect::<HashMap<String, Property<'static>>>()?))))
                    } else {
                        anyhow::bail!("Element #{} of slide #{} isn't a table!", ie+1, i+1)
                    }
                }).try_collect::<Vec<ParseableRenderable<'static>>>()?;

                Ok(ParseableSlide { renderables })
            }).try_collect::<Vec<_>>()?;

        let resources = if let toml::Value::Array(arr) = file.get("resource").cloned().unwrap_or_else(|| {
            log::info!("No resources defined in presentation.");
            toml::Value::Array(Vec::new())
        }) {
            arr.into_iter().enumerate().map(|(i, v)| {
                let eval = if let EvaluatedPropertyValue::Dict(ref hm) = value_to_evaluated_property(v) {
                    hm.clone()
                } else {
                    anyhow::bail!("Resource #{} isn't a table!", i+1)
                };
                let res_type = if let EvaluatedPropertyValue::String(ref s) = eval.borrow().get("type").ok_or(anyhow::anyhow!("'type' property of resource #{} wasn't specified!", i+1))? {
                    s.clone()
                } else {
                    anyhow::bail!("'type' property of resource #{} wasn't of type String!", i+1)
                };
                resource::RESOURCE_TYPES.with(|types| {
                    let borrow = eval.borrow();
                    (types.get(res_type.as_str()).ok_or(anyhow::anyhow!("No resource type '{res_type}' registered!"))?)(&*borrow)
                })
            }).try_collect::<Vec<Box<dyn resource::Resource>>>()?
        } else {
            anyhow::bail!("Resources array not defined as array!")
        };

        Ok(ParseablePresentation {
            slides: parseable_slides,
            resources
        })
    }
}