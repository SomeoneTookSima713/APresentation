use crate::presentation::parser::data::Value;

pub mod alignment;

pub mod base;

#[derive(Clone)]
pub enum Property<T: PropertyCompatible> {
    Literal(T::InnerRepresentation),
    Script(rhai::AST)
}

impl<T: PropertyCompatible> Property<T> {
    pub fn evaluate(&self, scope: &mut rhai::Scope<'static>, engine: &rhai::Engine) -> Option<T> {
        match self {
            Self::Literal(v) => Some(T::to_self(v, engine, scope)?),
            Self::Script(ast) => engine.eval_ast_with_scope::<T>(scope, ast).ok()
        }
    }

    /// Tries to construct a [`Property`] from a [`Value`].
    /// 
    /// It fails if the conversion of a concrete value didn't return anything
    /// or if rhai code was supplied and a parsing error occured.
    #[tracing::instrument(skip(engine))]
    pub fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self> {
        match val {
            Value::RhaiCode(c) => Some(Self::Script(match engine.compile_expression(c) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Error compiling rhai code: {e}");
                    None?
                }
            })),
            v => Some(Self::Literal(T::from_value(v, engine)?))
        }
    }
}

pub trait PropertyCompatible: rhai::Variant + Clone {
    type InnerRepresentation;

    /// Tries to convert from the generic [`Value`] enum to the more specific
    /// implementor of this trait. Contrary to the fact that a [`rhai::Engine`]
    /// is supplied as an argument, this method doesn't normally need to handle
    /// [`Value::RhaiCode`], because that is already handled in
    /// [`Property::from_value()`]. Only if you need to convert additional
    /// `Value`s you have to use it.
    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized;

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
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
        ($prim:ty, $valty:tt $(, $($othervalty:tt),*)?) => {
            impl PropertyCompatible for $prim {
                type InnerRepresentation = Self;

                fn from_value(val: Value, _engine: &rhai::Engine) -> Option<Self>
                where Self: Sized {
                    // I love doing cursed macro shenanigans :)
                    match val {
                        Value::$valty(v) => Some(v as $prim),
                        $($( Value::$othervalty(v) => Some(v as $prim), )*)?
                        Value::Option(Some(boxv)) => match &*boxv {
                            Value::$valty(v) =>Some(*v as $prim),
                            $($( Value::$othervalty(v) => Some(*v as $prim), )*)?
                            _ => None
                        },
                        _ => None
                    }
                }

                fn to_self(base: &Self, _engine: &rhai::Engine, _scope: &mut rhai::Scope<'static>) -> Option<Self>
                where Self: Sized { Some(*base) }
            }
        };
    }

    impl_for_primitive!(u8, Int, Float);
    impl_for_primitive!(u16, Int, Float);
    impl_for_primitive!(u32, Int, Float);
    impl_for_primitive!(u64, Int, Float);
    impl_for_primitive!(u128, Int, Float);
    impl_for_primitive!(usize, Int, Float);
    impl_for_primitive!(i8, Int, Float);
    impl_for_primitive!(i16, Int, Float);
    impl_for_primitive!(i32, Int, Float);
    impl_for_primitive!(i64, Int, Float);
    impl_for_primitive!(i128, Int, Float);
    impl_for_primitive!(isize, Int, Float);
    impl_for_primitive!(f32, Int, Float);
    impl_for_primitive!(f64, Int, Float);
    impl_for_primitive!(bool, Bool);

    impl PropertyCompatible for String {
        type InnerRepresentation = Self;

        fn from_value(val: Value, _engine: &rhai::Engine) -> Option<Self>
        where Self: Sized {
            if let Value::String(v) = val {
                Some(v)
            } else if let Value::Option(Some(boxv)) = val && let Value::String(v) = Box::into_inner(boxv) {
                Some(v)
            } else {
                None
            }
        }

        fn to_self(base: &Self, _engine: &rhai::Engine, _scope: &mut rhai::Scope<'static>) -> Option<Self>
        where Self: Sized { Some(base.clone()) }
    }

    impl<T: PropertyCompatible> PropertyCompatible for Option<T> {
        type InnerRepresentation = Option<super::Property<T>>;

        fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
        where Self: Sized {
            if let Value::Option(v) = val {
                Some(match v {
                    Some(v) => Some(super::Property::from_value(*v, engine)?),
                    None => None
                })
            } else {
                Some(Some(super::Property::from_value(val, engine)?))
            }
        }

        fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
        where Self: Sized {
            base.as_ref().map(|v| v.evaluate(scope, engine))
        }
    }

    impl<T: PropertyCompatible> PropertyCompatible for Vec<T> {
        type InnerRepresentation = Vec<super::Property<T>>;

        fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
        where Self: Sized {
            if let Value::List(v) = val {
                let mut vec = Vec::with_capacity(v.len());
                let map = v.into_iter().map(|v| super::Property::from_value(v, engine));
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
                let map = v.into_iter().map(|v| super::Property::from_value(v, engine));
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

        fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
        where Self: Sized {
            base.iter().map(|p| p.evaluate(scope, engine)).try_collect()
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

        for_tuples!( type InnerRepresentation = ( #(super::Property<Tuple>),* ); );

        fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
        where Self: Sized {
            if let Value::List(v) = val {
                Some((for_tuples!(
                    #( super::Property::from_value(v.get(TUPLE_TO_IDX.Tuple)?.clone(), engine)? ),*
                )))
            } else if let Value::Option(Some(boxv)) = val && let Value::List(v) = Box::into_inner(boxv) {
                Some((for_tuples!(
                    #( super::Property::from_value(v.get(TUPLE_TO_IDX.Tuple)?.clone(), engine)? ),*
                )))
            } else {
                None
            }
        }

        fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
        where Self: Sized {
            Some((for_tuples!(
                #( base.Tuple.evaluate(scope, engine)? ),*
            )))
        }
    }

    impl<T: PropertyCompatible, const N: usize> PropertyCompatible for [T; N] {
        type InnerRepresentation = [super::Property<T>; N];

        fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
        where Self: Sized {
            if let Value::List(v) = val {
                Some(std::array::try_from_fn(|i| super::Property::from_value(v.get(i)?.clone(), engine))?)
            } else if let Value::Option(Some(boxv)) = val && let Value::List(v) = Box::into_inner(boxv) {
                Some(std::array::try_from_fn(|i| super::Property::from_value(v.get(i)?.clone(), engine))?)
            } else {
                None
            }
        }

        fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
        where Self: Sized {
            std::array::try_from_fn(|i| base[i].evaluate(scope, engine))
        }
    }
}