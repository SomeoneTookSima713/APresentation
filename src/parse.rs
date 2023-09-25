use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

use serde::Deserialize;
use serde::de::Visitor;

use std::marker::PhantomData;
pub struct HashMapVisitor<K, V>(PhantomData<(K,V)>);

impl<'de, K: Deserialize<'de> + Hash + Eq, V: Deserialize<'de>> Visitor<'de> for HashMapVisitor<K, V> {
    type Value = HashMap<K, V>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map of key-value pairs")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut hashmap = HashMap::new();
        match map.size_hint() {
            Some(size) => {
                for _ in 0..size {
                    let (key, value) = map.next_entry()?.unwrap();
                    hashmap.insert(key, value);
                }
            },
            None => {
                while let Some((key, value)) = map.next_entry()? {
                    hashmap.insert(key, value);
                }
            }
        }
        Ok(hashmap)
    }
}

/// Any value contained in a JSON-document.
/// 
/// Also acts as a [`Visitor`] for itself; just use [`JSONValue::Null`] whenever you need one.
#[derive(Clone, PartialEq, Debug)]
pub enum JSONValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>)
}
impl TryInto<bool> for JSONValue {
    type Error = JSONValue;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            JSONValue::Bool(s) => Ok(s),
            _ => Err(self)
        }
    }
}
impl TryInto<f64> for JSONValue {
    type Error = JSONValue;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            JSONValue::Number(s) => Ok(s),
            _ => Err(self)
        }
    }
}
impl TryInto<String> for JSONValue {
    type Error = JSONValue;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            JSONValue::String(s) => Ok(s),
            _ => Err(self)
        }
    }
}
impl TryInto<Vec<JSONValue>> for JSONValue {
    type Error = JSONValue;

    fn try_into(self) -> Result<Vec<JSONValue>, Self::Error> {
        match self {
            JSONValue::Array(s) => Ok(s),
            _ => Err(self)
        }
    }
}
impl TryInto<HashMap<String,JSONValue>> for JSONValue {
    type Error = JSONValue;

    fn try_into(self) -> Result<HashMap<String,JSONValue>, Self::Error> {
        match self {
            JSONValue::Object(s) => Ok(s),
            _ => Err(self)
        }
    }
}

impl<'de> Visitor<'de> for JSONValue {
    type Value = Self;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "any value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v as f64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Number(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::String(v.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::String(v.to_string()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::String(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Array(v.iter().map(|r| JSONValue::Number(*r as f64)).collect()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_borrowed_bytes(v.as_slice())
    }
    
    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>, {
        deserializer.deserialize_any(JSONValue::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(JSONValue::Object(HashMap::new()))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>, {
        deserializer.deserialize_any(JSONValue::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut vec: Vec<JSONValue> = Vec::new();
        while let Some(elem) = seq.next_element::<JSONValue>()? {
            vec.push(elem)
        }
        Ok(JSONValue::Array(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut hashmap = HashMap::new();
        while let Some((key, value)) = map.next_entry::<String, JSONValue>()? {
            hashmap.insert(key, value);
        }
        Ok(JSONValue::Object(hashmap))
    }
}
impl<'de> Deserialize<'de> for JSONValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_any(JSONValue::Null)
    }
}

use crate::presentation::renderable::*;

#[derive(Debug)]
pub struct Document {
    pub fonts: HashMap<String, (String, String)>,
    pub slides: Vec<SlideData>
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let err = |a| Err(serde::de::Error::custom(a));
        let json_parsed: JSONValue = deserializer.deserialize_map(JSONValue::Null)?;
        match json_parsed {
            JSONValue::Object(map) => {
                let fonts;
                match map.get("fonts").ok_or(serde::de::Error::custom("required field \"fonts\" is missing"))? {
                    JSONValue::Object(fonts_json) => {
                        fonts = fonts_json.iter()
                            .map(|(key, value)| (key.clone(), match value {
                                JSONValue::Array(arr)=>(
                                    <JSONValue as TryInto<String>>::try_into(arr.get(0).ok_or::<deser_hjson::Error>(serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap().clone()).map_err::<deser_hjson::Error, _>(|_|serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap(),
                                    <JSONValue as TryInto<String>>::try_into(arr.get(1).ok_or::<deser_hjson::Error>(serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap().clone()).map_err::<deser_hjson::Error, _>(|_|serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap()
                                ),
                                _=>("".to_owned(),"".to_owned()) }))
                            .filter(|(key,(val1,val2))| if val1.len()>0 && val2.len()>0 {true} else {println!("WARN: Font {} has invalid paths",key);false}).collect();
                    },
                    _ => return err("field \"fonts\" needs to be a dictionary")
                };

                let mut slides = Vec::new();
                match map.get("slides").ok_or(serde::de::Error::custom("required field \"slides\" is missing"))? {
                    JSONValue::Array(vec) => {
                        for obj in vec.iter() {
                            slides.push(match obj { JSONValue::Object(hashmap)=>SlideData::from_json_data(hashmap)?, _=>return err("contents of \"slides\" array need to be objects") });
                        }
                    },
                    _ => return err("field \"slides\" needs to be an array")
                }

                Ok(Document { fonts, slides })
            },
            _ => err("base object isn't a map")
        }
    }
}

#[derive(Debug)]
pub struct DocumentFonts(pub HashMap<String, (String, String)>);
impl<'de> Deserialize<'de> for DocumentFonts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let err = |a| Err(serde::de::Error::custom(a));
        let json_parsed: JSONValue = deserializer.deserialize_map(JSONValue::Null)?;
        match json_parsed {
            JSONValue::Object(map) => {
                let fonts;
                match map.get("fonts").ok_or(serde::de::Error::custom("required field \"fonts\" is missing"))? {
                    JSONValue::Object(fonts_json) => {
                        fonts = fonts_json.iter()
                            .map(|(key, value)| (key.clone(), match value {
                                JSONValue::Array(arr)=>(
                                    <JSONValue as TryInto<String>>::try_into(arr.get(0).ok_or::<deser_hjson::Error>(serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap().clone()).map_err::<deser_hjson::Error, _>(|_|serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap(),
                                    <JSONValue as TryInto<String>>::try_into(arr.get(1).ok_or::<deser_hjson::Error>(serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap().clone()).map_err::<deser_hjson::Error, _>(|_|serde::de::Error::custom("entry in field \"fonts\" needs to be an array of two strings")).unwrap()
                                ),
                                _=>("".to_owned(),"".to_owned()) }))
                            .filter(|(key,(val1,val2))| if val1.len()>0 && val2.len()>0 {true} else {println!("WARN: Font {} has invalid paths",key);false}).collect();
                    },
                    _ => return err("field \"fonts\" needs to be a dictionary")
                }

                Ok(DocumentFonts(fonts))
            },
            _ => err("base object isn't a map")
        }
    }
}

#[derive(Debug)]
pub struct SlideData {
    pub background: Box<dyn Renderable>,
    pub content: HashMap<u8, Vec<Box<dyn Renderable>>>
}
impl SlideData {
    pub fn from_json_data<E: serde::de::Error>(data: &HashMap<String, JSONValue>) -> Result<SlideData, E> {
        use once_cell::sync::Lazy;
        const RENDERABLE_FUNCS: Lazy<Vec<Box<dyn Fn(&HashMap<String, JSONValue>) -> Result<Box<dyn Renderable>, String>>>> = Lazy::new(|| {
            vec![
                Box::new(|d: &HashMap<String, JSONValue>| ColoredRect::from_json(d).map(|o| Box::new(o) as Box<dyn Renderable>).map_err(|e: deser_hjson::Error|format!("{e}"))) as Box<dyn Fn(&HashMap<String, JSONValue>) -> Result<Box<dyn Renderable>, String>>,
                Box::new(|d: &HashMap<String, JSONValue>| RoundedRect::from_json(d).map(|o| Box::new(o) as Box<dyn Renderable>).map_err(|e: deser_hjson::Error|format!("{e}"))) as Box<dyn Fn(&HashMap<String, JSONValue>) -> Result<Box<dyn Renderable>, String>>,
                Box::new(|d: &HashMap<String, JSONValue>| Text::from_json(d).map(|o| Box::new(o) as Box<dyn Renderable>).map_err(|e: deser_hjson::Error|format!("{e}"))) as Box<dyn Fn(&HashMap<String, JSONValue>) -> Result<Box<dyn Renderable>, String>>,
                Box::new(|d: &HashMap<String, JSONValue>| Image::from_json(d).map(|o| Box::new(o) as Box<dyn Renderable>).map_err(|e: deser_hjson::Error|format!("{e}"))) as Box<dyn Fn(&HashMap<String, JSONValue>) -> Result<Box<dyn Renderable>, String>>
            ]
        });
        let err_bg_invalid = ||serde::de::Error::custom("field \"background\" is invalid");

        let background: Box<dyn Renderable>;
        match data.get("background").ok_or(serde::de::Error::custom("required field \"background\" is missing in slide"))? {
            JSONValue::Array(vec) => {
                let r: f64 = vec.get(0).ok_or((err_bg_invalid)())?.clone().try_into().map_err(|_|(err_bg_invalid)())?;
                let g: f64 = vec.get(1).ok_or((err_bg_invalid)())?.clone().try_into().map_err(|_|(err_bg_invalid)())?;
                let b: f64 = vec.get(2).ok_or((err_bg_invalid)())?.clone().try_into().map_err(|_|(err_bg_invalid)())?;

                background = Box::new( ColoredRect::new("0;0", "100%;100%", format!("{r};{g};{b};1"), "TOP_LEFT") ) as Box<dyn Renderable>;
            },
            JSONValue::Object(hashmap) => {
                let mut result: Result<Box<dyn Renderable>, String> = Err("".to_owned());

                for func in RENDERABLE_FUNCS.iter() {
                    result = result.or((func)(hashmap));
                }

                match result {
                    Ok(b) => background = b,
                    Err(_) => return Err((err_bg_invalid)())
                }
            },
            _ => return Err((err_bg_invalid)())
        }

        let mut content: HashMap<u8, Vec<Box<dyn Renderable>>> = HashMap::new();

        /*
            let mut result: Result<Box<dyn Renderable>, String> = Err("".to_owned());

            for func in RENDERABLE_FUNCS.iter() {
                result = result.or((func)(data));
            }
        */

        match data.get("content").ok_or(serde::de::Error::custom("required field \"content\" is missing in slide"))? {
            JSONValue::Array(vec) => {
                let z_index_default = JSONValue::Number(0.0);
                for renderable_json in vec.iter() {
                    let m = renderable_json.clone().try_into().map_err(|_|serde::de::Error::custom("array \"content\" has invalid contents"))?;
                    let mut result: Result<Box<dyn Renderable>, String> = Err("".to_owned());

                    let mut errors: Vec<String> = Vec::new();

                    for func in RENDERABLE_FUNCS.iter() {
                        result = match result {
                            Ok(r) => Ok(r),
                            Err(e) => {
                                errors.push(e);
                                (func)(&m)
                            }
                        };
                    }

                    let values_result: Result<&JSONValue, E> = get_value_alternates(&m, vec!["z_index","z-index","z"]);
                    let z_index: f64 = values_result.unwrap_or(&z_index_default)
                        .clone().try_into().map_err(|_|serde::de::Error::custom("invalid z-index"))?;
                    
                    match content.get_mut(&(z_index as u8)) {
                        Some(list) => {
                            list.push(result.map_err(|_|serde::de::Error::custom(format!("invalid contents of renderable object ({:?})",errors)))?);
                        },
                        None => {
                            content.insert(z_index as u8, vec![result.map_err(|_|serde::de::Error::custom(format!("invalid contents of renderable object ({:?})",errors)))?]);
                        }
                    }
                }
            },
            _ => return Err((err_bg_invalid)())
        }

        Ok(SlideData { background, content })
    }
}

/// Helper function for getting a value of a [`HashMap`], allowing it to be stored in multiple alternative keys.
/// 
/// Returns a [`Result<&V, serde::de::Error>`] primarily for usage in implementations of the [`Deserialize`]-trait.
fn get_value_alternates<K, V, Q, E>(map: &HashMap<K, V>, keys: Vec<Q>) -> Result<&V, E>
where
    K: Hash + Eq + std::fmt::Display,
    Q: Into<K> + std::fmt::Debug + Clone,
    E: serde::de::Error {
    let mut val: Option<&V> = None;
    for key in keys.iter() {
        val = val.or(map.get(&key.clone().into()))
    }
    val.ok_or(serde::de::Error::custom(format!("required parameter unspecified; possible keys: {:?}",keys)))
}

trait FromJson {
    fn from_json<E: serde::de::Error>(dict: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized;
}

impl<'a> FromJson for ColoredRect<'a> {
    fn from_json<E: serde::de::Error>(hashmap: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized {
        match hashmap.get("type").ok_or(serde::de::Error::custom("no type given"))?.eq(&JSONValue::String("ColoredRect".to_owned())) {
            true => {},
            false => return Err(serde::de::Error::custom("wrong type"))
        }

        let pos: String;
        match get_value_alternates(hashmap, vec!["pos", "position"])?.clone().try_into() {
            Ok(p) => pos = p,
            Err(_) => return Err(serde::de::Error::custom("position needs to be a string"))
        }
        let size: String;
        match get_value_alternates(hashmap, vec!["size"])?.clone().try_into() {
            Ok(v) => size = v,
            Err(_) => return Err(serde::de::Error::custom("size needs to be a string"))
        }
        let col: String;
        match get_value_alternates(hashmap, vec!["col", "color"])?.clone().try_into() {
            Ok(v) => col = v,
            Err(_) => return Err(serde::de::Error::custom("color needs to be a string"))
        }
        let alignment: String;
        match get_value_alternates(hashmap, vec!["align", "alignment"])?.clone().try_into() {
            Ok(v) => alignment = v,
            Err(_) => return Err(serde::de::Error::custom("alignment needs to be a string"))
        }
        Ok(
            ColoredRect::new(
                pos,
                size,
                col,
                alignment)
        )
    }
}

impl<'a> FromJson for RoundedRect<'a> {
    fn from_json<E: serde::de::Error>(hashmap: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized {
        match hashmap.get("type").ok_or(serde::de::Error::custom("no type given"))?.eq(&JSONValue::String("RoundedRect".to_owned())) {
            true => {},
            false => return Err(serde::de::Error::custom("wrong type"))
        }

        let pos: String;
        match get_value_alternates(hashmap, vec!["pos", "position"])?.clone().try_into() {
            Ok(p) => pos = p,
            Err(_) => return Err(serde::de::Error::custom("position needs to be a string"))
        }
        let size: String;
        match get_value_alternates(hashmap, vec!["size"])?.clone().try_into() {
            Ok(v) => size = v,
            Err(_) => return Err(serde::de::Error::custom("size needs to be a string"))
        }
        let col: String;
        match get_value_alternates(hashmap, vec!["col", "color"])?.clone().try_into() {
            Ok(v) => col = v,
            Err(_) => return Err(serde::de::Error::custom("color needs to be a string"))
        }
        let corner_rounding: String;
        match get_value_alternates(hashmap, vec!["corners", "corner_rounding", "rounding", "radius", "corner_radius"])?.clone().try_into() {
            Ok(v) => corner_rounding = v,
            Err(_) => return Err(serde::de::Error::custom("corner radius needs to be a string"))
        }
        let alignment: String;
        match get_value_alternates(hashmap, vec!["align", "alignment"])?.clone().try_into() {
            Ok(v) => alignment = v,
            Err(_) => return Err(serde::de::Error::custom("alignment needs to be a string"))
        }
        Ok(
            RoundedRect::new(
                pos,
                size,
                col,
                corner_rounding,
                alignment)
        )
    }
}

impl<'a> FromJson for Text<'a> {
    fn from_json<E: serde::de::Error>(hashmap: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized {
        match hashmap.get("type").ok_or(serde::de::Error::custom("no type given"))?.eq(&JSONValue::String("Text".to_owned())) {
            true => {},
            false => return Err(serde::de::Error::custom("wrong type"))
        }

        let pos: String;
        match get_value_alternates(hashmap, vec!["pos", "position"])?.clone().try_into() {
            Ok(p) => pos = p,
            Err(_) => return Err(serde::de::Error::custom("position needs to be a string"))
        }
        let width: String;
        match get_value_alternates(hashmap, vec!["width", "wrapping_width"])?.clone().try_into() {
            Ok(p) => width = p,
            Err(_) => return Err(serde::de::Error::custom("wrapping width needs to be a string"))
        }
        let size: String;
        match get_value_alternates(hashmap, vec!["size", "height", "text_size", "text_height"])?.clone().try_into() {
            Ok(v) => size = v,
            Err(_) => return Err(serde::de::Error::custom("text height needs to be a string"))
        }
        let col: String;
        match get_value_alternates(hashmap, vec!["col", "color"])?.clone().try_into() {
            Ok(v) => col = v,
            Err(_) => return Err(serde::de::Error::custom("color needs to be a string"))
        }
        let font: String;
        match get_value_alternates(hashmap, vec!["font", "base_font"])?.clone().try_into() {
            Ok(v) => font = v,
            Err(_) => return Err(serde::de::Error::custom("font radius needs to be a string"))
        }
        let alignment: String;
        match get_value_alternates(hashmap, vec!["align", "alignment"])?.clone().try_into() {
            Ok(v) => alignment = v,
            Err(_) => return Err(serde::de::Error::custom("alignment needs to be a string"))
        }
        let mut texts: Vec<String> = Vec::new();
        match <JSONValue as TryInto<Vec<JSONValue>>>::try_into(get_value_alternates(&hashmap, vec!["text"])?.clone()) {
            Ok(v) => {
                for val in v {
                    match val.try_into() {
                        Ok(s) => texts.push(s),
                        Err(_) => return Err(serde::de::Error::custom("text needs to be an array of strings"))
                    }
                }
            },
            Err(_) => return Err(serde::de::Error::custom("text needs to be an array of strings"))
        }
        Ok(
            Text::new(
                pos,
                texts,
                width,
                size,
                alignment,
                col,
                font,
                &*crate::app::FONTS.get().ok_or(serde::de::Error::custom("error getting font-list"))?)
        )
    }
}

impl<'a> FromJson for Image<'a> {
    fn from_json<E: serde::de::Error>(hashmap: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized {
        match hashmap.get("type").ok_or(serde::de::Error::custom("no type given"))?.eq(&JSONValue::String("Image".to_owned())) {
            true => {},
            false => return Err(serde::de::Error::custom("wrong type"))
        }

        let pos: String;
        match get_value_alternates(hashmap, vec!["pos", "position"])?.clone().try_into() {
            Ok(p) => pos = p,
            Err(_) => return Err(serde::de::Error::custom("position needs to be a string"))
        }
        let size: String;
        match get_value_alternates(hashmap, vec!["size"])?.clone().try_into() {
            Ok(v) => size = v,
            Err(_) => return Err(serde::de::Error::custom("size needs to be a string"))
        }
        let path: String;
        match get_value_alternates(hashmap, vec!["path", "file", "file_path"])?.clone().try_into() {
            Ok(v) => path = v,
            Err(_) => return Err(serde::de::Error::custom("file path needs to be a string"))
        }
        let alignment: String;
        match get_value_alternates(hashmap, vec!["align", "alignment"])?.clone().try_into() {
            Ok(v) => alignment = v,
            Err(_) => return Err(serde::de::Error::custom("alignment needs to be a string"))
        }
        Ok(
            Image::new(
                PathBuf::try_from(path).map_err(|_| serde::de::Error::custom("invalid file path specified"))?,
                pos,
                size,
                alignment)
        )
    }
}