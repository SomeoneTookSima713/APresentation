use std::cell::RefCell;
use std::rc::Rc;

use super::*;

use impl_trait_for_tuples::impl_for_tuples;

// I have no idea how this works, but it does so don't touch it please.
#[impl_for_tuples(1,16)]
impl<'lua> PropertyCompatible<'lua> for Tuple {
    const STRUCTURE: PropertyStructure = PropertyStructure::Array(&[for_tuples!( #( Tuple::STRUCTURE ),* )]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
    where Self: Sized {
        match value.get(args.clone())? {
            PropertyValue::List(list) => {
                let borrow = list.borrow();
                let mut iter = borrow.iter().cloned();

                Ok(for_tuples!( (#(Tuple::convert_from(iter.next().ok_or(anyhow::anyhow!("Invalid property!"))?, args.clone())?),*) ))
            },
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(Vec::from([for_tuples!( #( Tuple.convert_into() ),* )])))))
    }
}

impl<'lua, T: PropertyCompatible<'lua>, const N: usize> PropertyCompatible<'lua> for [T;N] {
    
    const STRUCTURE: PropertyStructure = PropertyStructure::Array(&[T::STRUCTURE;N]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args.clone())? {
            PropertyValue::List(list) => list.borrow().iter().cloned().map(|p| T::convert_from(p, args.clone())).try_collect::<Vec<T>>()?.try_into().map_err(|_|anyhow::anyhow!("Conversion failed!")),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(self.iter().map(|v|v.convert_into()).collect()))))
    }
}

impl<'lua> PropertyCompatible<'lua> for f32 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => Ok(uint as f32),
            PropertyValue::Int(int) => Ok(int as f32),
            PropertyValue::Float(f) => Ok(f as f32),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Float(*self as f64))
    }
}

impl<'lua> PropertyCompatible<'lua> for f64 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => Ok(uint as f64),
            PropertyValue::Int(int) => Ok(int as f64),
            PropertyValue::Float(f) => Ok(f),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Float(*self))
    }
}

impl<'lua> PropertyCompatible<'lua> for u8 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as u8),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::UInt(*self as u64))
    }
}

impl<'lua> PropertyCompatible<'lua> for u16 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as u16),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::UInt(*self as u64))
    }
}

impl<'lua> PropertyCompatible<'lua> for u32 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as u32),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::UInt(*self as u64))
    }
}

impl<'lua> PropertyCompatible<'lua> for u64 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => Ok(uint),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as u64),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::UInt(*self))
    }
}

impl<'lua> PropertyCompatible<'lua> for i8 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as i8),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Int(*self as i64))
    }
}

impl<'lua> PropertyCompatible<'lua> for i16 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as i16),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Int(*self as i64))
    }
}

impl<'lua> PropertyCompatible<'lua> for i32 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => int.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Float(f) => Ok(f as i32),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Int(*self as i64))
    }
}

impl<'lua> PropertyCompatible<'lua> for i64 {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::UInt(uint) => uint.try_into().map_err(|_|anyhow::anyhow!("Number conversion error!")),
            PropertyValue::Int(int) => Ok(int),
            PropertyValue::Float(f) => Ok(f as i64),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Int(*self))
    }
}

impl<'lua> PropertyCompatible<'lua> for bool {
    const STRUCTURE: PropertyStructure = PropertyStructure::Bool;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::Bool(b) => Ok(b),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::Bool(*self))
    }
}

impl<'lua> PropertyCompatible<'lua> for Rc<String> {
    const STRUCTURE: PropertyStructure = PropertyStructure::String;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args)? {
            PropertyValue::String(s) => Ok(s.clone()),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&self) -> Property<'lua> {
        Property::Constant(PropertyValue::String(self.clone()))
    }
}

impl<'lua, T: PropertyCompatible<'lua>> PropertyCompatible<'lua> for Vec<T> {
    const STRUCTURE: PropertyStructure = PropertyStructure::List(&[T::STRUCTURE]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args.clone())? {
            PropertyValue::List(list) => Ok(list.borrow().iter().cloned().map(|p|T::convert_from(p, args.clone())).try_collect()?),
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }
    
    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(self.iter().map(|v|v.convert_into()).collect()))))
    }
}