use crate::presentation::parser::data::Value;

pub mod alignment;

#[derive(Clone)]
pub enum Property<T: PropertyCompatible> {
    Literal(T),
    Script(rhai::AST)
}

impl<T: PropertyCompatible> Property<T> {
    pub fn evaluate(&self, scope: &mut rhai::Scope<'static>, engine: &rhai::Engine) -> Option<T> {
        match self {
            Self::Literal(v) => Some(v.clone()),
            Self::Script(ast) => engine.eval_ast_with_scope::<T>(scope, ast).ok()
        }
    }

    /// Tries to construct a [`Property`] from a [`Value`].
    /// 
    /// It fails if the conversion of a concrete value didn't return anything
    /// or if rhai code was supplied and a parsing error occured.
    pub fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self> {
        match val {
            Value::RhaiCode(c) => Some(Self::Script(engine.compile_expression(c).ok()?)),
            v => Some(Self::Literal(T::from_value(v)?))
        }
    }
}

pub trait PropertyCompatible: rhai::Variant + Clone {
    /// Tries to convert from the generic [`Value`] enum to the more specific
    /// implementor of this trait. This method doesn't need to handle
    /// [`Value::RhaiCode`], because that is handled in
    /// [`PropertyCompatible::from_value_or_rhai()`].
    fn from_value(val: Value) -> Option<Self>
    where Self: Sized;

    // This method shouldn't be necessary, but I'm keeping the code here just in case.
    // 
    // /// Either tries to evaluate rhai code to the type implementing this trait,
    // /// or tries to construct it from a [`Value`].
    // /// 
    // /// It is recommended to use this method over
    // /// [`PropertyCompatible::from_value()`], as that method doesn't account
    // /// for the [`Value::RhaiCode`] variant and thus wouldn't return anything
    // /// when encountering rhai code.
    // fn from_value_or_rhai(val: Value, scope: &mut rhai::Scope<'static>, engine: &rhai::Engine) -> Option<Self>
    // where Self: Sized {
    //     match val {
    //         Value::RhaiCode(c) => engine.eval_expression_with_scope(scope, &c).ok(),
    //         v => Self::from_value(v)
    //     }
    // }
}

mod prop_comp_impls {
    use super::PropertyCompatible;
    use crate::presentation::parser::data::Value;

    macro_rules! impl_for_primitive {
        ($prim:ty, $valty:tt) => {
            impl PropertyCompatible for $prim {
                fn from_value(val: Value) -> Option<Self>
                where Self: Sized {
                    if let Value::$valty(v) = val {
                        Some(v as $prim)
                    } else if let Value::Option(Some(boxv)) = val && let Value::$valty(v) = &*boxv {
                        Some(*v as $prim)
                    } else {
                        None
                    }
                }
            }
        };
    }

    impl_for_primitive!(u8, Int);
    impl_for_primitive!(u16, Int);
    impl_for_primitive!(u32, Int);
    impl_for_primitive!(u64, Int);
    impl_for_primitive!(u128, Int);
    impl_for_primitive!(usize, Int);
    impl_for_primitive!(i8, Int);
    impl_for_primitive!(i16, Int);
    impl_for_primitive!(i32, Int);
    impl_for_primitive!(i64, Int);
    impl_for_primitive!(i128, Int);
    impl_for_primitive!(isize, Int);
    impl_for_primitive!(f32, Float);
    impl_for_primitive!(f64, Float);
    impl_for_primitive!(bool, Bool);

    impl PropertyCompatible for String {
        fn from_value(val: Value) -> Option<Self>
        where Self: Sized {
            if let Value::String(v) = val {
                Some(v)
            } else if let Value::Option(Some(boxv)) = val && let Value::String(v) = Box::into_inner(boxv) {
                Some(v)
            } else {
                None
            }
        }
    }

    impl<T: PropertyCompatible> PropertyCompatible for Option<T> {
        fn from_value(val: Value) -> Option<Self>
        where Self: Sized {
            if let Value::Option(v) = val {
                Some(v.and_then(|v| T::from_value(Box::into_inner(v))))
            } else {
                None
            }
        }
    }

    impl<T: PropertyCompatible> PropertyCompatible for Vec<T> {
        fn from_value(val: Value) -> Option<Self>
        where Self: Sized {
            if let Value::List(v) = val {
                let mut vec = Vec::with_capacity(v.len());
                let map = v.into_iter().map(|v| T::from_value(v));
                for val in map {
                    if let Some(value) = val {
                        vec.push(value);
                    } else {
                        // Early-return `None` because we can't convert every
                        // element in the array.
                        return None;
                    }
                }
                Some(vec)
            } else if let Value::Option(Some(boxv)) = val && let Value::List(v) = Box::into_inner(boxv) {
                let mut vec = Vec::with_capacity(v.len());
                let map = v.into_iter().map(|v| T::from_value(v));
                for val in map {
                    if let Some(value) = val {
                        vec.push(value);
                    } else {
                        // Early-return `None` because we can't convert every
                        // element in the array.
                        return None;
                    }
                }
                Some(vec)
            } else {
                None
            }
        }
    }

    /// *For internal use only!*
    /// 
    /// Helper for getting dynamic indices inside the
    /// [`impl_for_tuples`](impl_trait_for_tuples::impl_for_tuples) derive
    /// macro.
    const TUPLE_TO_IDX: (usize,usize,usize,usize,usize,usize,usize,usize,usize,usize,usize,usize,usize,usize,usize,usize)
        = (0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15);
    #[impl_trait_for_tuples::impl_for_tuples(1, 4)]
    impl PropertyCompatible for Tuple {
        for_tuples!( where #( Tuple: PropertyCompatible )* );

        fn from_value(val: Value) -> Option<Self>
        where Self: Sized {
            if let Value::List(v) = val {
                Some((for_tuples!(
                    #( Tuple::from_value(v.get(TUPLE_TO_IDX.Tuple)?.clone())? ),*
                )))
            } else if let Value::Option(Some(boxv)) = val && let Value::List(v) = Box::into_inner(boxv) {
                Some((for_tuples!(
                    #( Tuple::from_value(v.get(TUPLE_TO_IDX.Tuple)?.clone())? ),*
                )))
            } else {
                None
            }
        }
    }

    impl<T: PropertyCompatible, const N: usize> PropertyCompatible for [T; N] {
        fn from_value(val: Value) -> Option<Self>
        where Self: Sized {
            if let Value::List(v) = val {
                Some(std::array::try_from_fn(|i| T::from_value(v.get(i)?.clone()))?)
            } else if let Value::Option(Some(boxv)) = val && let Value::List(v) = Box::into_inner(boxv) {
                Some(std::array::try_from_fn(|i| T::from_value(v.get(i)?.clone()))?)
            } else {
                None
            }
        }
    }
}