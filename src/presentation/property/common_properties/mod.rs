use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub(self) use super::{ Property, PropertyCompatible, PropertyStructure, PropertyValue, EvaluatedPropertyValue };

pub mod color;
pub mod image;

pub use color::*;
pub use image::*;

#[derive(Debug)]
/// A Property-Compatible value that allows either a singular value or a list of values to be supplied.
/// 
/// 
pub enum ContextualList<'lua, T: PropertyCompatible<'lua> + std::fmt::Debug, const N: usize> {
    One(T, PhantomData<&'lua ()>),
    All([T; N])
}

impl<'lua, T: PropertyCompatible<'lua> + std::fmt::Debug, const N: usize> PropertyCompatible<'lua> for ContextualList<'lua, T, N> {
    const STRUCTURE: PropertyStructure = PropertyStructure::Or(&[
        T::STRUCTURE,
        PropertyStructure::Array(&[T::STRUCTURE; N])
    ]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
    where Self: Sized {
        if let Ok(v) = T::convert_from(value.clone(), args.clone()) {
            Ok(Self::One(v, PhantomData))
        } else if let PropertyValue::List(l) = value.get(args.clone())? {
            let list = l.borrow();
            if list.len() != N { anyhow::bail!("Invalid property! (List has invalid length)") }
            let mut resultlist = Vec::new();

            for p in list.iter().cloned() {
                match T::convert_from(p, args.clone()) {
                    Ok(v) => resultlist.push(v),
                    Err(e) => anyhow::bail!("Invalid property! (Error evaluating list of values: {e})")
                }
            }

            Ok(Self::All(resultlist.try_into().map_err(|_| anyhow::anyhow!("Couldn't convert Vec to Array! (Shouldn't happen)"))?))
        } else {
            anyhow::bail!("Invalid property!")
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        match self {
            Self::One(v, _) => v.convert_into(),
            Self::All(l) => Property::Constant(PropertyValue::List(Rc::new(RefCell::new(l.iter().map(|v|v.convert_into()).collect()))))
        }
    }
    
    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        match self {
            Self::One(v, _) => v.move_into	(),
            Self::All(l) => Property::Constant(PropertyValue::List(Rc::new(RefCell::new(l.into_iter().map(|v|v.move_into()).collect()))))
        }
    }
}

impl<'lua, T: PropertyCompatible<'lua> + std::fmt::Debug, const N: usize> ContextualList<'lua, T, N> {
    pub fn get_list(&self) -> [&T; N] {
        match self {
            Self::One(v, _) => [v;N],
            Self::All(l) => unsafe {
                use std::mem::MaybeUninit;
                
                let mut arr: [MaybeUninit<&T>; N] = MaybeUninit::uninit().assume_init();

                for (i,v) in l.iter().enumerate() {
                    arr[i] = MaybeUninit::new(v);
                }

                MaybeUninit::array_assume_init(arr)
            }
        }
    }
}

impl<'lua, T: PropertyCompatible<'lua> + std::fmt::Debug + Copy, const N: usize> ContextualList<'lua, T, N> {
    pub fn get_list_copied(&self) -> [T; N] {
        match self {
            Self::One(v, _) => [*v;N],
            Self::All(l) => *l
        }
    }
}

impl<'lua, T: PropertyCompatible<'lua> + std::fmt::Debug + Clone, const N: usize> Clone for ContextualList<'lua, T, N> {
    fn clone(&self) -> Self {
        match self {
            Self::One(v, _) => Self::One(v.clone(), PhantomData),
            Self::All(l) => Self::All(l.clone())
        }
    }
}