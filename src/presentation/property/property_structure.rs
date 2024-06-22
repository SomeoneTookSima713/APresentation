use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;

use regex::Regex;

use try_iterator::prelude::*;

use super::{ PropertyValue, Property };

/// Describes a structure of a lua type.
/// 
/// For example, if you had a table like this:
/// ```lua
/// {
///     string: "Hello, World!",
///     number: 32.84,
///     dynamically_sized_array: {1,3,7}
/// }
/// ```
/// The corresponding value of [`PropertyStructure`] would look like this:
/// ```
/// Dict(&[
///     ("string", String),
///     ("number", Number),
///     ("dynamically_sized_array", List(&[ Number ]))
/// ])
/// ```
/// 
/// Every part of the structure described with this type can also be replaced
/// with a function evaluating to the part of the structure it replaces. As a
/// consequence, the above described example could also have a lua-equivalent
/// looking like this:
/// ```lua
/// {
///     string: function() return "Hello, World!" end,
///     number: 32.84,
///     dynamically_sized_array: function() return {1, 3, 7} end
/// }
/// ```
/// 
/// This struct should only be constructed in static contexts. While it is
/// possible to construct it from a non-static context, any list- or dict-like
/// structures can only be represented using leaked or statically defined
/// slices.
/// 
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PropertyStructure {
    /// This represents any lua number, also integers.
    Number,
    /// This represents any lua string.
    /// 
    /// The [`Option`] contained in this variant is an optional RegExp the
    /// lua value has to match if supplied.
    String(Option<&'static str>),
    /// This represents any boolean.
    Bool,
    /// This represents any lua array (a table which's values' indices aren't
    /// explicitly defined) with a specific number of elements. The contained
    /// slice represents all values and their types in the lua array, in the
    /// order they should appear in.
    Array(&'static [PropertyStructure]),
    /// This represents any lua array (a table which's values' indices aren't
    /// explicitly defined) with a varying number of elements. The contained
    /// slice contains all valid lua types that are allowed to appear inside
    /// the array.
    List(&'static [PropertyStructure]),
    /// This represents any lua table. The contained slice should represent all
    /// key-value-pairs that need to be contained inside the corresponding lua
    /// table.
    Dict(&'static [(&'static str, PropertyStructure)]),
    /// This indicates that multiple different lua types are allowed at a
    /// certain point in the structure. The structures inside the contained
    /// slice represent all structures allowed.
    Or(&'static [PropertyStructure]),
}

static REGEXP_CACHE: Lazy<RwLock<HashMap<String, Regex>>> = Lazy::new(|| RwLock::new(HashMap::new()));

impl PropertyStructure {
    /// Checks if the structure of the supplied [`Property`] *could* be
    /// supported by this [`PropertyStructure`].
    /// 
    /// Emphasis on *could* here, because this function doesn't check the return
    /// value of functions, thus you could put a lua function returning a
    /// string in the place of a number and this function would still return
    /// `true`, so you should still do additional checks on top of using this
    /// function.
    pub fn check_structure<'lua>(&self, property: &Property<'lua>) -> anyhow::Result<bool> {
        if let &Property::Eval(_) = property { return Ok(true); }
        if let &Property::Constant(ref val) = property {
            self.check_structure_of_value(val)
        } else {
            Ok(false)
        }
    }

    fn check_structure_of_value<'lua>(&self, value: &PropertyValue<'lua>) -> anyhow::Result<bool> {
        match (self, value) {
            (
                PropertyStructure::Number,
                &PropertyValue::Int(_) | PropertyValue::UInt(_) | PropertyValue::Float(_)
            ) => Ok(true),
            (
                PropertyStructure::String(None),
                &PropertyValue::String(_)
            ) => Ok(true),
            (
                PropertyStructure::String(Some(regexp)),
                &PropertyValue::String(ref str)
            ) => {
                match REGEXP_CACHE.read() {
                    Ok(hm) => {
                        if let Some(exp) = hm.get(*regexp) {
                            let mut iter = exp.find_iter(str.as_str());
                            if iter.next().is_some() && iter.next().is_none() {
                                return Ok(true);
                            } else {
                                return Ok(false);
                            }
                        }
                    },
                    Err(_) => {
                        log::warn!("RegExp Cache couldn't be read!");
                    }
                }
                match REGEXP_CACHE.write() {
                    Ok(mut hm) => {
                        let exp = match Regex::new(regexp) {
                            Ok(exp) => exp,
                            Err(e) => {
                                log::error!("Couldn't parse RegExp: {e}");
                                anyhow::bail!("Couldn't parse RegExp: {e}");
                            }
                        };
                        let mut iter = exp.find_iter(str.as_str());
                        let exactly_one_match = iter.next().is_some() && iter.next().is_none();
                        drop(iter);
                        hm.insert(regexp.to_string(), exp);
                        return Ok(exactly_one_match);
                    },
                    Err(_) => {
                        log::warn!("RegExp Cache couldn't be written to!");
                    }
                }
                let exp = match Regex::new(regexp) {
                    Ok(exp) => exp,
                    Err(e) => {
                        log::error!("Couldn't parse RegExp: {e}");
                        anyhow::bail!("Couldn't parse RegExp: {e}");
                    }
                };
                let mut iter = exp.find_iter(str.as_str());
                return Ok(iter.next().is_some() && iter.next().is_none());
            },
            (
                PropertyStructure::Bool,
                &PropertyValue::Bool(_)
            ) => Ok(true),
            (
                PropertyStructure::List(slist),
                &PropertyValue::List(ref vlist)
            ) => Ok(vlist.borrow().iter().try_all(|p| slist.iter().try_any(|s| s.check_structure(p)))?),
            (
                PropertyStructure::Array(sarr),
                &PropertyValue::List(ref vlist)
            ) => Ok(sarr.iter().enumerate().try_all(|(i, s)| vlist.borrow().get(i).map(|p|s.check_structure(p)).unwrap_or(Ok(false)))?),
            (
                PropertyStructure::Dict(sdict),
                &PropertyValue::Dict(ref vdict)
            ) => Ok(sdict.iter().try_all(|(k,v)| vdict.borrow().get(*k).map(|p| v.check_structure(p)).unwrap_or(Ok(false)))?),
            (
                PropertyStructure::Or(spossib),
                val
            ) => {
                Ok(spossib.iter().try_any(|s| s.check_structure_of_value(val))?)
            },
            _ => Ok(false)
        }
    }
}