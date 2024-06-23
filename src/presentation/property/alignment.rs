use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{ PropertyCompatible, PropertyStructure, Property, PropertyValue };

/// Describes the alignment of something along one axis.
#[derive(Clone, Copy, Debug)]
pub enum Align1D {
    Start,
    Middle,
    End,
    Custom(f64)
}

impl Into<f64> for Align1D {
    fn into(self) -> f64 {
        match self {
            Self::Start => 0.0,
            Self::Middle => 0.5,
            Self::End => 1.0,
            Self::Custom(c) => c
        }
    }
}

impl From<f64> for Align1D {
    fn from(value: f64) -> Self {
        match (value*2.0) as u32 {
            0 => Self::Start,
            1 => Self::Middle,
            2 => Self::End,
            _ => Self::Custom(value)
        }
    }
}

/// Describes an alignment of something along two axes.
#[derive(Clone, Copy, Debug)]
pub struct Align2D {
    pub x: Align1D,
    pub y: Align1D
}

impl<T: Into<Align1D>> From<(T, T)> for Align2D {
    fn from(value: (T, T)) -> Self {
        Self { x: value.0.into(), y: value.1.into() }
    }
}

impl<T: Into<Align1D>> From<[T; 2]> for Align2D {
    fn from(value: [T; 2]) -> Self {
        let mut iter = value.into_iter();
        Self { x: iter.next().unwrap().into(), y: iter.next().unwrap().into() }
    }
}

impl Into<(f64, f64)> for Align2D {
    fn into(self) -> (f64, f64) {
        (self.x.into(), self.y.into())
    }
}

impl Into<[f64;2]> for Align2D {
    fn into(self) -> [f64;2] {
        [self.x.into(), self.y.into()]
    }
}

#[allow(unused)]
impl Align2D {
    pub const TOP_LEFT: Align2D      = Align2D { x: Align1D::Start,  y: Align1D::Start  };
    pub const TOP_CENTER: Align2D    = Align2D { x: Align1D::Middle, y: Align1D::Start  };
    pub const TOP_RIGHT: Align2D     = Align2D { x: Align1D::End,    y: Align1D::Start  };
    pub const MID_LEFT: Align2D      = Align2D { x: Align1D::Start,  y: Align1D::Middle };
    pub const MID_CENTER: Align2D    = Align2D { x: Align1D::Middle, y: Align1D::Middle };
    pub const MID_RIGHT: Align2D     = Align2D { x: Align1D::End,    y: Align1D::Middle };
    pub const BOTTOM_LEFT: Align2D   = Align2D { x: Align1D::Start,  y: Align1D::End    };
    pub const BOTTOM_CENTER: Align2D = Align2D { x: Align1D::Middle, y: Align1D::End    };
    pub const BOTTOM_RIGHT: Align2D  = Align2D { x: Align1D::End,    y: Align1D::End    };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x: x.into(), y: y.into() }
    }
}

impl<'lua> PropertyCompatible<'lua> for Align1D {
    const STRUCTURE: PropertyStructure = PropertyStructure::Number;

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        Ok(Self::from(f64::convert_from(value, args)?))
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(PropertyValue::Float((*self).into()))
    }

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        Property::Constant(PropertyValue::Float(self.into()))
    }
}

impl<'lua> PropertyCompatible<'lua> for Align2D {
    const STRUCTURE: PropertyStructure = PropertyStructure::Or(&[
        PropertyStructure::String(Some("(TOP|MID|BOTTOM)_(LEFT|CENTER|RIGHT)")),
        PropertyStructure::Array(&[PropertyStructure::Number;2]),
        PropertyStructure::Dict(&[("x", PropertyStructure::Number), ("y", PropertyStructure::Number)])
    ]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get(args.clone())? {
            PropertyValue::List(list) => {
                Ok(Self::from((
                    Align1D::convert_from(list.borrow().get(0).ok_or(anyhow::anyhow!("Invalid Property!"))?.clone(), args.clone())?,
                    Align1D::convert_from(list.borrow().get(1).ok_or(anyhow::anyhow!("Invalid Property!"))?.clone(), args.clone())?
                )))
            },
            PropertyValue::Dict(dict) => {
                Ok(Self::from((
                    Align1D::convert_from(dict.borrow().get("x").ok_or(anyhow::anyhow!("Invalid Property!"))?.clone(), args.clone())?,
                    Align1D::convert_from(dict.borrow().get("y").ok_or(anyhow::anyhow!("Invalid Property!"))?.clone(), args.clone())?
                )))
            },
            PropertyValue::String(s) => {
                Ok(match s.as_str() {
                    "TOP_LEFT"   => Align2D::TOP_LEFT,
                    "TOP_CENTER" => Align2D::TOP_CENTER,
                    "TOP_RIGHT"  => Align2D::TOP_RIGHT,

                    "MID_LEFT"   => Align2D::MID_LEFT,
                    "MID_CENTER" => Align2D::MID_CENTER,
                    "MID_RIGHT"  => Align2D::MID_RIGHT,

                    "BOTTOM_LEFT"   => Align2D::BOTTOM_LEFT,
                    "BOTTOM_CENTER" => Align2D::BOTTOM_CENTER,
                    "BOTTOM_RIGHT"  => Align2D::BOTTOM_RIGHT,

                    _ => unreachable!()
                })
            }
            _ => Err(anyhow::anyhow!("Invalid property!"))
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(PropertyValue::Dict(Rc::new(RefCell::new(HashMap::from([
            ("x".to_string(), self.x.convert_into()),
            ("y".to_string(), self.y.convert_into())
        ])))))
    }

    fn move_into(self) -> Property<'lua> {
        Property::Constant(PropertyValue::Dict(Rc::new(RefCell::new(HashMap::from([
            ("x".to_string(), self.x.move_into()),
            ("y".to_string(), self.y.move_into())
        ])))))
    }
}

pub type Alignment = Align2D;