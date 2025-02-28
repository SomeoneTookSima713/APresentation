use hashbrown::HashMap;

use crate::presentation::element::property::{ Property, PropertyCompatible };

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Option(Option<Box<Value>>),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    /// The contained value is either a [`Value::List`] or a [`Value::Map`]
    EnumVariant(String, Option<Box<Value>>),
    RhaiCode(String)
}

#[derive(Clone, Copy)]
pub enum ValueStructure {
    Int,
    Float,
    String,
    Bool,
    Option(&'static ValueStructure),
    /// This represents a [`Value::List`] containing only one type of structure
    /// a specific amount of times.
    Array(&'static ValueStructure, usize),
    /// This is mainly used inside [`ValueStructure::Enum`]
    Tuple(&'static [ValueStructure]),
    /// This represents a [`Value::List`] with a dynamic amount of items, all
    /// matching the given structure.
    List(&'static ValueStructure),
    /// This represents a [`Value::Map`] with the structure of, well, a struct.
    /// It checks if some specific keys exist in the map and if the values at
    /// these keys have the structure specified for them.
    Struct(&'static [(&'static str, ValueStructure)]),
    /// This represents a [`Value::Map`] in which all values match the
    /// specified structure.
    Map(&'static ValueStructure),
    /// This represents a [`Value::EnumVariant`] and specifies all possible
    /// variants of the enum with an optional structure, if said variant should
    /// have associated data.
    /// 
    /// **Important:** The associated structure must either be a
    /// [`ValueStructure::Tuple`] specifying tuple-like associated data, or a
    /// [`ValueStructure::Struct`] specifying struct-like associated data.
    Enum(&'static [(&'static str, Option<ValueStructure>)]),
    /// This allows multiple alternative structures to match in one place.
    Or(&'static [ValueStructure]),
    /// This allows any type of value with any structure.
    Any
}

impl Value {
    pub fn validate_structure(&self, structure: ValueStructure) -> bool {
        // Don't you love obscenely long match statements that are the most verbose shit you've ever witnessed?
        // I certainly do!
        match (self, structure) {
            (Self::Int(_), ValueStructure::Int) |
            (Self::Float(_), ValueStructure::Float) |
            (Self::String(_), ValueStructure::String) |
            (Self::Bool(_), ValueStructure::Bool) => true,
            (Self::Option(None), ValueStructure::Option(_)) => true,
            (Self::Option(Some(v)), ValueStructure::Option(s)) => v.validate_structure(*s),
            (v, ValueStructure::Option(s)) => v.validate_structure(*s),
            (Self::List(vals), ValueStructure::Array(s, a)) => vals.len() == a && vals.iter().all(|v| v.validate_structure(*s)),
            (Self::List(vals), ValueStructure::Tuple(structures)) => vals.len() == structures.len() && vals.iter().zip(structures.iter()).all(|(v,s)| v.validate_structure(*s)),
            (Self::List(vals), ValueStructure::List(s)) => vals.iter().all(|v| v.validate_structure(*s)),
            (Self::Map(vals), ValueStructure::Struct(structures)) => structures.iter().all(|(idx, s)| vals.contains_key(*idx) && vals.get(*idx).unwrap().validate_structure(*s)),
            (Self::Map(vals), ValueStructure::Map(s)) => vals.values().all(|v| v.validate_structure(*s)),
            // I think this match arm really needs some explaining:
            // It checks if there is any enum variant in the given structure that matches the
            // value's variant and if the associated data (as long as it should exist) matches the
            // structure defined for it.
            (Self::EnumVariant(var, v), ValueStructure::Enum(structures)) => structures.iter().find(|(svar, s)| var.eq(*svar) && v.is_some() == s.is_some() && v.as_ref().and_then(|v| s.map(|s| v.validate_structure(s))).unwrap_or(true)).is_some(),
            (Self::RhaiCode(_), _) => true,
            (v, ValueStructure::Or(structures)) => structures.iter().any(|s| v.validate_structure(*s)),
            (_, ValueStructure::Any) => true,
            _ => false
        }
    }
}

#[derive(Clone)]
pub struct ParsedStructure(HashMap<String, Value>);

impl ParsedStructure {
    pub fn new(map: HashMap<String, Value>) -> Self { Self(map) }

    pub fn try_get_property<T: PropertyCompatible, S: std::hash::Hash + hashbrown::Equivalent<String> + ?Sized>(&self, idx: &S, engine: &rhai::Engine) -> Option<Property<T>> {
        if let Some(v) = self.0.get(idx) {
            Property::from_value(v.clone(), engine)
        } else {
            None
        }
    }

    pub fn inner(&self) -> &HashMap<String, Value> { &self.0 }
    pub fn inner_mut(&mut self) -> &mut HashMap<String, Value> { &mut self.0 }
    pub fn into_inner(self) -> HashMap<String, Value> { self.0 }
}