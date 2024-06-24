use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{ Mutex, MutexGuard };
use std::rc::Rc;

use once_cell::sync::Lazy;

use mlua::Function;

mod property_compatible_impl;
pub mod property_structure;
pub mod alignment;
pub mod common_properties;

pub use property_structure::*;

pub use alignment::Alignment;

static RNG: Mutex<Lazy<rand::rngs::StdRng>> = Mutex::new(Lazy::new(|| {
    use rand::SeedableRng;
    rand::rngs::StdRng::from_entropy()
}));

#[derive(Clone)]
pub struct PropertyEnvironment(pub(self) HashMap<&'static str, PropEnvVal>);

impl std::ops::Deref for PropertyEnvironment {
    type Target = HashMap<&'static str, PropEnvVal>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PropertyEnvironment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub enum PropEnvVal {
    Number(f64),
    Function(mlua::Function<'static>),
    Table(HashMap<&'static str, PropEnvVal>)
}

impl From<f64> for PropEnvVal {
    fn from(value: f64) -> Self {
        PropEnvVal::Number(value)
    }
}
impl From<mlua::Function<'static>> for PropEnvVal {
    fn from(value: mlua::Function<'static>) -> Self {
        PropEnvVal::Function(value)
    }
}
impl mlua::IntoLua<'static> for PropEnvVal {
    fn into_lua(self, lua: &'static mlua::prelude::Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'static>> {
        match self {
            Self::Number(n) => n.into_lua(lua),
            Self::Function(f) => Ok(mlua::Value::Function(f)),
            Self::Table(t) => t.into_lua(lua)
        }
    }
}

#[derive(Clone)]
pub enum Property<'lua> {
    Constant(PropertyValue<'lua>),
    Eval(Function<'lua>)
}

#[derive(Clone)]
pub enum PropertyValue<'lua> {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(Rc<String>),
    List(Rc<RefCell<Vec<Property<'lua>>>>),
    Dict(Rc<RefCell<HashMap<String, Property<'lua>>>>),
}

#[allow(unused)]
#[derive(Clone, Debug)]
/// A Property which ensures that it has been fully evaluated. This is
/// neccessary because regular [`PropertyValue`]s can still contain evaluatable
/// [`Property`]s. That also means that a regular [`PropertyValue`] is a
/// non-recursively evaluated value, while this is a recursively evaluated
/// value.
pub enum EvaluatedPropertyValue {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(Rc<String>),
    List(Rc<RefCell<Vec<EvaluatedPropertyValue>>>),
    Dict(Rc<RefCell<HashMap<String, EvaluatedPropertyValue>>>),
}

impl EvaluatedPropertyValue {
    pub fn from_value<'lua, A: mlua::IntoLuaMulti<'lua> + Clone>(value: &PropertyValue<'lua>, args: A) -> anyhow::Result<Self> {
        Ok(match value {
            PropertyValue::Bool(b) => Self::Bool(*b),
            PropertyValue::Int(i) => Self::Int(*i),
            PropertyValue::UInt(u) => Self::UInt(*u),
            PropertyValue::Float(f) => Self::Float(*f),
            PropertyValue::String(s) => Self::String(s.clone()),
            PropertyValue::List(l) => Self::List(
                Rc::new(RefCell::new(
                    l.borrow().iter().map(|prop| {
                        prop.get(args.clone()).map(|v|EvaluatedPropertyValue::from_value(&v, args.clone()))?
                    }).try_collect()?))),
            PropertyValue::Dict(d) => Self::Dict(
                Rc::new(RefCell::new(
                    d.borrow().iter().map(|(k,v)| {
                        v.get(args.clone()).map(|v|EvaluatedPropertyValue::from_value(&v, args.clone()))?.map(|v| (k.clone(), v))
                    }).try_collect()?)))
        })
    }

    pub fn to_property<'lua>(&self) -> Property<'lua> {
        Property::Constant(match self {
            EvaluatedPropertyValue::Bool(b) => PropertyValue::Bool(*b),
            EvaluatedPropertyValue::Dict(d) => PropertyValue::Dict(Rc::new(RefCell::new(d.borrow().iter().map(|(k,v)| (k.clone(), v.to_property())).collect()))),
            EvaluatedPropertyValue::Float(f) => PropertyValue::Float(*f),
            EvaluatedPropertyValue::Int(i) => PropertyValue::Int(*i),
            EvaluatedPropertyValue::List(l) => PropertyValue::List(Rc::new(RefCell::new(l.borrow().iter().map(|v|v.to_property()).collect()))),
            EvaluatedPropertyValue::String(s) => PropertyValue::String(s.clone()),
            EvaluatedPropertyValue::UInt(u) => PropertyValue::UInt(*u)
        })
    }
}

impl<'a> From<mlua::Value<'a>> for Property<'a> {
    fn from(value: mlua::Value<'a>) -> Self {
        match value {
            mlua::Value::Boolean(b) => Self::Constant(PropertyValue::Bool(b)),
            mlua::Value::Error(e) => panic!("Error from lua function (possibly caused by Rust code): {e}"),
            mlua::Value::Function(f) => Self::Eval(f),
            mlua::Value::Integer(i) => Self::Constant(PropertyValue::Int(i)),
            mlua::Value::Number(f) => Self::Constant(PropertyValue::Float(f)),
            mlua::Value::String(s) => Self::Constant(PropertyValue::String(Rc::new(s.to_string_lossy().to_string()))),
            mlua::Value::Table(t) => {
                let mut array_hm: HashMap<u64, mlua::Value> = HashMap::new();
                let mut map_hm: HashMap<crate::util::HashableLuaValue, mlua::Value> = HashMap::new();

                let mut is_array = true;

                // Go through every key-value pair in the table and collect it
                // into two hashmaps: One for a potential list and one for a
                // potential dict.
                for res in t.pairs::<mlua::Value,mlua::Value>() {
                    match res {
                        Ok((k,v)) => {
                            // The most basic checks for if the table is an
                            // array or not: If it has non-positive integers or
                            // any non-integer keys, it can't be an array.
                            if is_array && let mlua::Value::Integer(i) = k {
                                if i>=0 {
                                    array_hm.insert(i as u64, v.clone());
                                } else {
                                    is_array = false;
                                }
                            } else {
                                is_array = false;
                            }
                            map_hm.insert(k.into(), v);
                        },
                        Err(_) => {
                            panic!("This error shouldn't happen! Conversion from lua value *to lua value* failed!");
                        }
                    }
                }

                // Last check to determine if the table is an array.

                let mut array_vec = array_hm.keys().map(|r|*r).collect::<Vec<_>>();
                array_vec.sort_unstable();
                let arr_len = array_vec.into_iter().reduce(|acc,e| {
                    // Because of the sort operation, we know that every
                    // element following any element in the array must be
                    // larger than said element, so this never results in
                    // underflows.
                    if e-acc > 1 {
                        is_array = false;
                    }
                    e
                }).unwrap_or(0);

                // If is_array is still true at this point, the table can be converted to an array.
                if is_array {
                    // I swear I'm good at coding
                    Self::Constant(PropertyValue::List(Rc::new(RefCell::new((1..=arr_len).into_iter().map(|i|array_hm.remove(&i).unwrap().into()).collect::<Vec<_>>()))))
                } else {
                    Self::Constant(PropertyValue::Dict(Rc::new(RefCell::new(map_hm.into_iter().map(|(k,v)| (k.to_string(),v.into())).collect()))))
                }
            },
            _ => panic!("Tried converting unsupported Lua type to Property!"),
        }
    }
}

impl<'lua> mlua::FromLuaMulti<'lua> for Property<'lua> {
    fn from_lua_multi(mut values: mlua::prelude::LuaMultiValue<'lua>, _lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Self> {
        match values.len() {
            0 => Err(mlua::Error::runtime("Can't convert nil to value!")),
            1 => {
                Ok(values.pop_front().unwrap().into())
            },
            _ => {
                Ok(Property::Constant(PropertyValue::List(Rc::new(RefCell::new(values.into_iter().map(|v|v.into()).collect())))))
            }
        }
    }
}

impl<'lua, T: PropertyCompatible<'lua>> From<&'lua T> for Property<'lua> {
    fn from(value: &'lua T) -> Self {
        value.convert_into()
    }
}

#[allow(unused)]
impl<'a> Property<'a> {
    /// Converts any convertible type into a [`Property`].
    pub fn from_constant<T: PropertyCompatible<'a> + 'a>(c: &'a T) -> Self {
        c.into()
    }

    /// Converts any lua code block (in string form) into a [`Property`], the
    /// code block's return value being the value of the property.
    pub fn from_lua_string<S: std::borrow::Borrow<String>>(lua: &'static mlua::Lua, string: S, env: PropertyEnvironment) -> anyhow::Result<Self> {
        const LUA_SNIPPET_APPEND: &str = "local t,w,h = ... ";

        log::debug!("Loading Lua-Snippet: {}", string.borrow());
        let func = lua.load(format!("{LUA_SNIPPET_APPEND}{}",string.borrow())).set_environment((*env).clone()).into_function()?;

        Ok(Self::Eval(func))
    }

    /// Gets the value of this [`Property`], evaluating a function with the
    /// supplied arguments if the stored value isn't a constant.
    /// 
    /// # Note
    /// This function only evaluates the topmost part of the value. This means
    /// that, for example, the returned value could be a list still containing
    /// a function that needs to be evaluated before being able to use it. For
    /// recursively evaluating a [`Property`], use the
    /// [`Property::get_recursively()`] function.
    pub fn get<A: mlua::IntoLuaMulti<'a> + Clone>(&self, args: A) -> anyhow::Result<PropertyValue<'a>> {
        match self {
            Self::Constant(c) => Ok(c.clone()),
            Self::Eval(func) => {
                func.call(args.clone()).map(|v: Property<'a>|v.get(args)).map_err(|e|anyhow::anyhow!(e))?
            }
        }
    }

    /// Recursively evaluates this [`Property`] and returns the evaluated
    /// values.
    /// 
    /// In contrast to [`Property::get()`], this function also evaluates any
    /// functions contained in all evaluated values. This means that the value
    /// returned by this function is always usable and doesn't need any further
    /// processing befure usage.
    pub fn get_recursively<A: mlua::IntoLuaMulti<'a> + Clone>(&self, args: A) -> anyhow::Result<EvaluatedPropertyValue> {
        Ok(EvaluatedPropertyValue::from_value(&self.get(args.clone()).map_err(|e|anyhow::anyhow!(e))?, args)?)
    }
}

pub fn get_environment(lua: &'static mlua::Lua) -> anyhow::Result<PropertyEnvironment> {
    let mut hm = HashMap::new();

    let mut math = HashMap::new();
    math.insert("abs", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.abs()))?));
    math.insert("acos", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.acos()))?));
    math.insert("asin", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.asin()))?));
    math.insert("atan", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.atan()))?));

    math.insert("ceil", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.ceil() as u64))?));
    math.insert("cos", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.cos()))?));
    math.insert("deg", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.to_degrees()))?));
    math.insert("rad", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.to_radians()))?));
    math.insert("exp", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.exp()))?));
    math.insert("floor", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.floor() as u64))?));
    math.insert("fmod", PropEnvVal::Function(lua.create_function(|_, (x, y): (f64, f64)| Ok(x % y))?));
    math.insert("huge", PropEnvVal::Number(f64::INFINITY));
    math.insert("log", PropEnvVal::Function(lua.create_function(|_, (x, base): (f64, Option<f64>)| Ok(x.log(base.unwrap_or(std::f64::consts::E))))?));
    math.insert("max", PropEnvVal::Function(lua.create_function(|_, vals: Vec<f64>| Ok(vals.into_iter().reduce(|acc,e| acc.max(e))))?));
    // TODO: I think those don't work precision-wise...
    math.insert("maxinteger", PropEnvVal::Number(i64::MAX as f64));
    math.insert("mininteger", PropEnvVal::Number(i64::MIN as f64));

    math.insert("min", PropEnvVal::Function(lua.create_function(|_, vals: Vec<f64>| Ok(vals.into_iter().reduce(|acc,e|acc.min(e))))?));
    math.insert("modf", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok((x.trunc(), x.fract())))?));
    math.insert("random", PropEnvVal::Function(lua.create_function(|_, (m, n): (Option<f64>, Option<f64>)| {
        use rand::Rng;
        let mut rng = RNG.lock().map_err(|_|mlua::Error::runtime("Random Number Generator errored!"))?;
        match (m,n) {
            (None, None) => Ok(rng.gen_range(0.0..1.0)),
            (Some(max), None) => Ok(rng.gen_range(1..=max as i32) as f64),
            (Some(min), Some(max)) => Ok(rng.gen_range(min as i32..max as i32) as f64),
            _ => Err(mlua::Error::runtime("Invalid argument combination!"))
        }
    })?));
    math.insert("randomseed", PropEnvVal::Function(lua.create_function(|_, x: f64| {
        use rand::SeedableRng;
        let mut rng = RNG.lock().map_err(|_|mlua::Error::runtime("Random Number Generator errored!"))?;
        **rng = rand::rngs::StdRng::seed_from_u64(x as u64);
        Ok(())
    })?));
    math.insert("sin", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.sin()))?));
    math.insert("sqrt", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.sqrt()))?));
    math.insert("tan", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x.tan()))?));
    math.insert("tointeger", PropEnvVal::Function(lua.create_function(|_, x: f64| Ok(x as i64))?));
    math.insert("type", PropEnvVal::Function(lua.create_function(|lua, x: mlua::Value| {
        use mlua::IntoLua;
        Ok(match x {
            mlua::Value::Integer(_) => "integer".into_lua(lua)?,
            mlua::Value::Number(_) => "float".into_lua(lua)?,
            _ => mlua::Value::Nil
        })
    })?));
    math.insert("ult", PropEnvVal::Function(lua.create_function(|_, (a, b): (f64,f64)| Ok((a as u64) < (b as u64)))?));

    // Lua Env
    hm.insert("math", PropEnvVal::Table(math));

    hm.insert("print", PropEnvVal::Function(lua.create_function(|_, args: mlua::Variadic<mlua::Value>| { log::info!("Lua printed: {args:?}"); Ok(()) })?));

    Ok(PropertyEnvironment(hm))
}

pub trait PropertyCompatible<'lua> {
    /// All compatible constellations of a lua type that this type can
    /// construct itself from.
    const STRUCTURE: PropertyStructure;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
    where Self: Sized;

    fn convert_into(&'lua self) -> Property<'lua>;

    fn move_into(self) -> Property<'lua>
    where Self: Sized;
}

#[derive(Clone)]
pub struct TypedProperty<'lua, T>
where T: PropertyCompatible<'lua> {
    prop: Property<'lua>,
    converted: RefCell<Option<T>>
}

impl<'lua, T> TypedProperty<'lua, T>
where T: PropertyCompatible<'lua> {
    /// Creates a new [`TypedProperty`] from a regular [`Property`].
    pub fn new(base: Property<'lua>) -> anyhow::Result<Self> {
        T::STRUCTURE.check_structure(&base)?.map_err(|e|anyhow::anyhow!("{e}"))?;

        Ok(Self { prop: base, converted: RefCell::new(None) })
    }

    /// Deletes the internal cache for the evaluated value.
    /// 
    /// If you want to deliberately reevaluate the value, use this function.
    /// 
    /// # Returns
    /// Returns a bool indicating if the deletion completed successfully. If
    /// the value contained in the cache is still borrowed by some part of the
    /// program, this will return an error.
    pub fn delete_cache(&self) -> bool {
        match self.converted.try_borrow_mut() {
            Ok(mut b) => { *b = None; true },
            Err(_) => false
        }
    }

    pub fn get<A: mlua::IntoLuaMulti<'lua> + Clone>(&self, args: A) -> anyhow::Result<std::cell::Ref<T>> {
        let Ok(mut borrow) = self.converted.try_borrow_mut() else {
            anyhow::bail!("Cache of TypedProperty couldn't be mutably borrowed!");
        };
        if borrow.is_none() {
            *borrow = Some(T::convert_from(self.prop.clone(), args)?);
        }
        drop(borrow);
        std::cell::Ref::filter_map(self.converted.borrow(), |r| r.as_ref()).map_err(|_|anyhow::anyhow!("[TypedProperty].converted was None even though it was set the literal line before!"))
    }
}

impl<'lua, T> PropertyCompatible<'lua> for TypedProperty<'lua, T>
where T: PropertyCompatible<'lua> {
    const STRUCTURE: PropertyStructure = T::STRUCTURE;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, _args: A) -> anyhow::Result<Self>
        where Self: Sized {
        Ok(Self::new(value)?)
    }

    fn convert_into(&self) -> Property<'lua> {
        self.prop.clone()
    }

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        self.prop
    }
}