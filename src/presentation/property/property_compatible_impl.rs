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

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(Vec::from([for_tuples!( #( Tuple.move_into() ),* )])))))
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

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(self.into_iter().map(|v|v.move_into()).collect()))))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Float(self as f64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Float(self))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::UInt(self as u64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::UInt(self as u64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::UInt(self as u64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::UInt(self))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Int(self as i64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Int(self as i64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Int(self as i64))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Int(self))
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

    fn move_into(self) -> Property<'lua>
        where Self: Sized {
        Property::Constant(PropertyValue::Bool(self))
    }
}

impl<'lua> PropertyCompatible<'lua> for Rc<String> {
    const STRUCTURE: PropertyStructure = PropertyStructure::String(None);

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

    fn move_into(self) -> Property<'lua> {
        Property::Constant(PropertyValue::String(self))
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

    fn move_into(self) -> Property<'lua> {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(self.into_iter().map(|v|v.move_into()).collect()))))
    }
}

impl<'lua> PropertyCompatible<'lua> for crate::texture::TextureSamplerSelection {
    const STRUCTURE: PropertyStructure = PropertyStructure::String(Some("linear|Linear|nearest_neighbor|NearestNeighbor|nearest_neighbour|NearestNeighbour|pixel_perfect|PixelPerfect"));

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
    where Self: Sized {
        use crate::texture::TextureSamplerSelection;
        match value.get_recursively(args)? {
            EvaluatedPropertyValue::String(s) => {
                match s.replace("_", "").to_lowercase().as_str() {
                    "linear" => Ok(TextureSamplerSelection::Linear),
                    "nearestneighbor"|"nearestneighbour"|"pixelperfect" => Ok(TextureSamplerSelection::PixelPerfect),
                    _ => anyhow::bail!("Invalid TextureSamplerSelection! (Expected String)")
                }
            },
            _ => anyhow::bail!("Invalid TextureSamplerSelection! (Expected String)")
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        use crate::texture::TextureSamplerSelection;
        Property::Constant(PropertyValue::String(Rc::new(match self {
            TextureSamplerSelection::Linear => "linear",
            TextureSamplerSelection::PixelPerfect => "pixel_perfect"
        }.to_owned())))
    }

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        use crate::texture::TextureSamplerSelection;
        Property::Constant(PropertyValue::String(Rc::new(match self {
            TextureSamplerSelection::Linear => "linear",
            TextureSamplerSelection::PixelPerfect => "pixel_perfect"
        }.to_owned())))
    }
}

impl<'lua> PropertyCompatible<'lua> for crate::util::math::Rect {
    const STRUCTURE: PropertyStructure = PropertyStructure::Array(&[PropertyStructure::Number;4]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get_recursively(args)? {
            EvaluatedPropertyValue::List(l) => {
                let list = l.borrow();
                if let Some(&[ref x, ref y, ref w, ref h]) = list.get(0..4) {
                    let ex = match x {
                        EvaluatedPropertyValue::Float(f) => *f as f32,
                        EvaluatedPropertyValue::Int(i) => *i as f32,
                        EvaluatedPropertyValue::UInt(u) => *u as f32,
                        _ => anyhow::bail!("Item in list wasn't a number!")
                    };
                    let ey = match y {
                        EvaluatedPropertyValue::Float(f) => *f as f32,
                        EvaluatedPropertyValue::Int(i) => *i as f32,
                        EvaluatedPropertyValue::UInt(u) => *u as f32,
                        _ => anyhow::bail!("Item in list wasn't a number!")
                    };
                    let ew = match w {
                        EvaluatedPropertyValue::Float(f) => *f as f32,
                        EvaluatedPropertyValue::Int(i) => *i as f32,
                        EvaluatedPropertyValue::UInt(u) => *u as f32,
                        _ => anyhow::bail!("Item in list wasn't a number!")
                    };
                    let eh = match h {
                        EvaluatedPropertyValue::Float(f) => *f as f32,
                        EvaluatedPropertyValue::Int(i) => *i as f32,
                        EvaluatedPropertyValue::UInt(u) => *u as f32,
                        _ => anyhow::bail!("Item in list wasn't a number!")
                    };
                    Ok(Self::new_vals(ex, ey, ew, eh))
                } else {
                    anyhow::bail!("Not enough items in list")
                }
            },
            _ => anyhow::bail!("Invalid Property!")
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(vec![self.x(),self.y(),self.w(),self.h()].into_iter().map(|n|Property::Constant(PropertyValue::Float(n as f64))).collect()))))
    }

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        Property::Constant(PropertyValue::List(Rc::new(RefCell::new(vec![self.x(),self.y(),self.w(),self.h()].into_iter().map(|n|Property::Constant(PropertyValue::Float(n as f64))).collect()))))
    }
}