use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::fmt::Debug;

use serde::Deserialize;
use serde::de::Visitor;

#[allow(unused)]
use log::{ debug as log_dbg, info as log_info, warn as log_warn, error as log_err };

use super::{ Parser, SlideData };

pub struct JSONParser;
impl Parser for JSONParser {

    fn parse<'a>(&mut self, contents: &'a str) -> Result<Vec<SlideData>, Box<dyn Debug>> {
        let document: Document = deser_hjson::from_str(contents).map_err(|e| Box::new(e) as Box<dyn Debug>)?;

        Ok(document.0)
    }

    fn parse_fonts<'a>(&mut self, contents: &'a str) -> Result<HashMap<String, (String, String)>, Box<dyn Debug>> {
        let fonts: DocumentFonts = deser_hjson::from_str(contents).map_err(|e| Box::new(e) as Box<dyn Debug>)?;

        Ok(fonts.0)
    }
}

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

/// Helper struct with functions for parsing the JSON-document
#[derive(Debug)]
pub struct Document(pub Vec<SlideData>);
impl Document {
    /// Parses the document to get a [`Vec`] of [`SlideData`]s
    pub fn slides_from_json<E: serde::de::Error>(data: &HashMap<String, JSONValue>) -> Result<SlideData, E> {
        // Helper function for creating a general error message for the background being invalid.
        let err_bg_invalid = ||serde::de::Error::custom("field \"background\" is invalid");

        // Alias for creating any serde error message.
        let err = serde::de::Error::custom;

        // Parse the background object
        let background: Box<dyn Renderable>;
        match data.get("background").ok_or(serde::de::Error::custom("required field \"background\" is missing in slide"))? {
            // Simplest case: Just an array of RGB-values
            JSONValue::Array(vec) => {
                // Get the RGB-values from the array
                //   Errors when the array is to short or when the conversion from `JSONValue` to f64
                //   fails.
                let r: f64 = vec.get(0).ok_or((err_bg_invalid)())?.clone().try_into().map_err(|_|(err_bg_invalid)())?;
                let g: f64 = vec.get(1).ok_or((err_bg_invalid)())?.clone().try_into().map_err(|_|(err_bg_invalid)())?;
                let b: f64 = vec.get(2).ok_or((err_bg_invalid)())?.clone().try_into().map_err(|_|(err_bg_invalid)())?;

                // Use the RGB-values to create a colored rectangle filling the whole screen
                background = Box::new( ColoredRect::new("0;0", "100%;100%", format!("{r};{g};{b};1"), "TOP_LEFT") ) as Box<dyn Renderable>;
            },
            // More complex case: Any renderable object
            JSONValue::Object(hashmap) => {

                // Tries to construct a Renderable object based on the specified type.
                //   Errors if the specified type doesn't exist, the field is invalid or the
                //   constructor function failed.
                let renderable_type: String = hashmap.get("type").ok_or(err("required field \"type\" missing"))?.clone()
                    .try_into().map_err(|_|err("field \"type\" needs to be a string"))?;
                let result = (RENDERABLE_FUNCS.get(&renderable_type).ok_or(err("field \"type\" is invalid"))?)(hashmap);

                // The error when the constructor function failed occurs here.
                match result {
                    Ok(b) => background = b,
                    Err(_) => return Err((err_bg_invalid)())
                }
            },
            // Last case: Any invalid JSONValue (e.g. a number or string)
            _ => return Err((err_bg_invalid)())
        }

        // Parse all objects defined in the slide
        let mut content: HashMap<u8, Vec<Box<dyn Renderable>>> = HashMap::new();
        match data.get("content").ok_or(serde::de::Error::custom("required field \"content\" is missing in slide"))? {
            JSONValue::Array(vec) => {
                // The default for the z-index of an object
                let z_index_default = JSONValue::Number(0.0);

                for (i, renderable_json) in vec.iter().enumerate() {
                    let map: HashMap<String, JSONValue> = renderable_json.clone().try_into().map_err(|_|serde::de::Error::custom("field \"content\" must be an array of objects"))?;

                    // Tries to construct a Renderable object based on the specified type.
                    //   Errors if the specified type doesn't exist, the field is invalid or the
                    //   constructor function failed.
                    let renderable_type: String = map.get("type").ok_or(err("required field \"type\" missing"))?.clone()
                        .try_into().map_err(|_|err("field \"type\" needs to be a string"))?;
                    let result = (RENDERABLE_FUNCS.get(&renderable_type).ok_or(err("field \"type\" is invalid"))?)(&map);
                    let object = result.map_err(|e|err(format!("invalid contents of renderable object #{i} ({e})").leak()))?;

                    // Note: The error message just says 'expected an integer' because the number
                    //       gets casted to an integer. You can supply a float in theory though.
                    let z_index_result: Result<&JSONValue, E> = get_value_alternates(&map, vec!["z_index","z-index","z"]);
                    let z_index: f64 = z_index_result.unwrap_or(&z_index_default)
                        .clone().try_into().map_err(|_|serde::de::Error::custom("invalid z-index (expected an integer)"))?;
                    
                    // Check in the map if a vec for the specified z-index already exists or not
                    match content.get_mut(&(z_index as u8)) {
                        // If it exists, just push the object to this list
                        Some(list) => {
                            list.push(object);
                        },
                        // If it doesn't exist, create one and then push the object to the list
                        None => {
                            content.insert(z_index as u8, vec![object]);
                        }
                    }
                }
            },
            // Return an error if the 'content'-field isn't actually an array of objects
            _ => return Err((err_bg_invalid)())
        }

        Ok(SlideData { background, content })
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        // Helper function for error creation; keeps the code lines shorter and more readable
        let err = serde::de::Error::custom;

        // Parse the document to a map
        //   (JSONValue also acts as a Visitor from the 'serde'-crate for itself)
        let map: HashMap<String, JSONValue> = deserializer.deserialize_map(JSONValue::Null)?.try_into().map_err(|_|err("base object isn't a map"))?;


        let slides = {
            // Gets the 'slides'-field and checks if it's actually an array
            let slide_array: Vec<JSONValue> = map.get("slides").ok_or(err("required field \"slides\" is missing"))?.clone()
                    .try_into().map_err(|_|err("field \"slides\" must be an array"))?;
            
            // Parses the slides contained in the 'slides'-array
            //   Errors if any item in the array isn't an object or any object couldn't get parsed
            //   into a slide.
            slide_array.into_iter().map(|json_val| {
                let map: HashMap<String, JSONValue> = json_val.try_into().map_err(|_|err("contents of \"slides\" array need to be objects"))?;
                Document::slides_from_json(&map)
            }).collect::< Result<Vec<SlideData>, D::Error> >()?
        };

        Ok(Document(slides))
    }
}

#[derive(Debug)]
pub struct DocumentFonts(pub HashMap<String, (String, String)>);
impl<'de> Deserialize<'de> for DocumentFonts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        // Alias for more compact and more readable code
        let err = serde::de::Error::custom;

        // Get the base object of the document and error if it isn't a map
        let document: HashMap<String, JSONValue> = deserializer.deserialize_map(JSONValue::Null)?.try_into().map_err(|_|err("base object isn't a map"))?;

        // Get the 'fonts'-field from the document
        //   Errors if the 'fonts'-field isn't a dictionary containing tuples of two string paths.
        let fonts = {
            // Check if the 'fonts'-field is a dictionary
            let font_dict: HashMap<String, JSONValue> = document.get("fonts").ok_or(err("required field \"fonts\" is missing"))?.clone()
                .try_into().map_err(|_|err("field \"fonts\" needs to be a dictionary of tuples of two file paths"))?;
            // The fonts will be stored here
            let mut font_list: HashMap<String, (String, String)> = HashMap::new();

            // Iterate over all values in the dict, then check if they're tuples of two strings
            for (key, value) in font_dict.into_iter() {
                let mut array: Vec<JSONValue> = value.try_into().map_err(|_|err("entry in dict \"fonts\" must be a tuple of two strings"))?;
                if array.len()!=2 { return Err(err("entry in dict \"fonts\" must be a tuple of two strings")) }
                let paths: (String, String) = (
                    array.remove(0).try_into().map_err(|_|err("entry in dict \"fonts\" needs to be a tuple of two strings"))?,
                    array.remove(0).try_into().map_err(|_|err("entry in dict \"fonts\" needs to be a tuple of two strings"))?,
                );
                font_list.insert(key, paths);
            }

            font_list
        };

        Ok(DocumentFonts(fonts))
    }
}

use once_cell::sync::Lazy;
type FnRenderableParse = Box<dyn Fn(&HashMap<String, JSONValue>) -> Result<Box<dyn Renderable>, String>>;
/// A [`HashMap`] of functions for parsing each type of [`Renderable`].
/// 
/// The index defines the name of the type.
const RENDERABLE_FUNCS: Lazy<HashMap<String, FnRenderableParse>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("ColoredRect".to_owned(), ColoredRect::renderable_func::<deser_hjson::Error>());
    map.insert("RoundedRect".to_owned(), RoundedRect::renderable_func::<deser_hjson::Error>());
    map.insert("Text".to_owned(), Text::renderable_func::<deser_hjson::Error>());
    map.insert("Image".to_owned(), Image::renderable_func::<deser_hjson::Error>());
    map
});

/// Helper function for getting a value of a [`HashMap`], allowing it to be stored in multiple alternative keys.
/// 
/// Returns a [`Result<&V, serde::de::Error>`], primarily for usage in implementations of the [`Deserialize`] trait.
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

/// Trait for parsing JSON data into a struct.
/// 
/// Also contains some helper functions related to [`Renderable`]s that can be parsed from JSON.
trait FromJson {
    /// Parses JSON-data and into itself
    fn from_json<E: serde::de::Error>(dict: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized;

    /// Returns a closure that constructs a Renderable object
    fn renderable_func<E: serde::de::Error>() -> FnRenderableParse
    where Self: Sized + Renderable + 'static {
        let func = |dict: &'_ HashMap<String, JSONValue>| {
            match Self::from_json::<E>(dict) {
                Ok(s) => Ok(Box::new(s) as Box<dyn Renderable>),
                Err(e) => Err(format!("{e}"))
            }
        };

        Box::new(func) as FnRenderableParse
    }
}

impl<'a> FromJson for ColoredRect<'a> {
    fn from_json<E: serde::de::Error>(hashmap: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized {

        // Get the position, size, color and alignment from the JSON data
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

        // Create the struct
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

        // Get the position, size, color, corner rounding radius and alignment from the JSON data
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

        // Create the struct
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
        let err = serde::de::Error::custom;

        // Get the position, wrapping width, font size, color, font type, alignment and text array
        // from the JSON data
        let pos: String;
        match get_value_alternates(hashmap, vec!["pos", "position"])?.clone().try_into() {
            Ok(p) => pos = p,
            Err(_) => return Err(err("position needs to be a string"))
        }
        let width: String;
        match get_value_alternates(hashmap, vec!["width", "wrapping_width"])?.clone().try_into() {
            Ok(p) => width = p,
            Err(_) => return Err(err("wrapping width needs to be a string"))
        }
        let size: String;
        match get_value_alternates(hashmap, vec!["size", "height", "text_size", "text_height"])?.clone().try_into() {
            Ok(v) => size = v,
            Err(_) => return Err(err("text height needs to be a string"))
        }
        let col: String;
        match get_value_alternates(hashmap, vec!["col", "color"])?.clone().try_into() {
            Ok(v) => col = v,
            Err(_) => return Err(err("color needs to be a string"))
        }
        let font: String;
        match get_value_alternates(hashmap, vec!["font", "base_font"])?.clone().try_into() {
            Ok(v) => font = v,
            Err(_) => return Err(err("font radius needs to be a string"))
        }
        let alignment: String;
        match get_value_alternates(hashmap, vec!["align", "alignment"])?.clone().try_into() {
            Ok(v) => alignment = v,
            Err(_) => return Err(err("alignment needs to be a string"))
        }
        let text_alignment: String;
        match get_value_alternates(hashmap, vec!["text_align", "text_alignment"])?.clone().try_into() {
            Ok(v) => text_alignment = v,
            Err(_) => text_alignment = "LEFT".to_owned()
        }
        let placeholders: HashMap<String, TextPlaceholderExpr<'a>> =
        match get_value_alternates::<String, JSONValue, &'static str, deser_hjson::Error>(hashmap, vec!["placeholders"]) {
            Ok(placeholders_json) => {
                use crate::presentation::util::DEFAULT_CONTEXT;

                let context = &DEFAULT_CONTEXT;

                let placeholder_map: HashMap<String, JSONValue> = placeholders_json.clone().try_into().map_err(|_|err("placeholder list must be a dict"))?;
                let mut placeholder_hash_map = HashMap::with_capacity(placeholder_map.len());
                for (key, json) in placeholder_map {
                    let expr_string: String = json.try_into().map_err(|_|err("placeholders have to be strings"))?;
                    placeholder_hash_map.insert(key, TextPlaceholderExpr::parse(expr_string, context));
                }
                placeholder_hash_map
            },
            Err(_) => {
                HashMap::new()
            }
        };

        let mut texts: Vec<String> = Vec::new();
        match < JSONValue as TryInto<Vec<JSONValue>> >::try_into(get_value_alternates(&hashmap, vec!["text","texts","lines"])?.clone()) {
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

        // Create the struct
        Ok(
            Text::new(
                pos,
                texts,
                width,
                size,
                alignment,
                col,
                font,
                &*crate::app::FONTS.get().ok_or(serde::de::Error::custom("error getting font-list"))?,
                placeholders,
                text_alignment)
        )
    }
}

impl<'a> FromJson for Image<'a> {
    fn from_json<E: serde::de::Error>(hashmap: &HashMap<String, JSONValue>) -> Result<Self, E>
    where Self: Sized {

        // Get the position, size, file path and alignment from the JSON data
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

        // Create the struct
        Ok(
            Image::new(
                PathBuf::try_from(path).map_err(|_| serde::de::Error::custom("invalid file path specified"))?,
                pos,
                size,
                alignment)
        )
    }
}