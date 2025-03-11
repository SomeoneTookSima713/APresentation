use crate::presentation::parser::data::Value;

pub mod alignment;

pub mod base;

#[derive(Clone)]
pub enum Property<T: PropertyCompatible + 'static> {
    Literal(T::InnerRepresentation),
    Script(rhai::AST)
}

impl<T: PropertyCompatible> std::fmt::Debug for Property<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(_) => write!(f, "Property<{}>::Literal", std::any::type_name::<T>()),
            Self::Script(_) => write!(f, "Property<{}>::Script", std::any::type_name::<T>()),
        }
    }
}

impl<T: PropertyCompatible> Property<T> {
    #[tracing::instrument(skip(scope, engine))]
    pub fn evaluate(&self, scope: &mut rhai::Scope<'static>, engine: &rhai::Engine) -> Option<T> {
        match self {
            Self::Literal(v) => Some(T::to_self(v, engine, scope)?),
            Self::Script(ast) => match engine.eval_ast_with_scope::<T>(scope, ast) {
                Ok(v) => Some(v),
                Err(e) => { tracing::error!("Couldn't evaluate property of type {}: {}", std::any::type_name::<T>(), e); None }
            }
        }
    }

    /// Tries to construct a [`Property`] from a [`Value`].
    /// 
    /// It fails if the conversion of a concrete value didn't return anything
    /// or if rhai code was supplied and a parsing error occured.
    #[tracing::instrument(skip(engine))]
    pub fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self> {
        tracing::debug!("{:?}", val);
        match val {
            Value::RhaiCode(c) => {
                Some(Self::Script(match engine.compile_expression(c) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Error compiling rhai code: {e}");
                        None?
                    }
                }))
            },
            v => Some(Self::Literal(T::from_value(v, engine)?))
        }
    }
}

/// Trait that allows a type to be used in the [`Property`] struct.
/// 
/// This makes it possible to easily parse complex data from a presentation
/// file with minimal code.
/// 
/// # Using in Rhai Code snippets
/// If you want to be able to construct or use this type in Rhai code snippets
/// inside the presentation, you have to annotate your implementation with the
/// [`proc_macros::register_impl`] macro, giving the fully qualified path to
/// your struct as an argument. Here's an example:
/// ```rust
/// #[derive(Clone)]
/// pub struct MyProperty {
///     foo: usize,
///     bar: String
/// }
/// 
/// struct MyUncomputedProperty {
///     foo: Property<usize>,
///     bar: Property<String>
/// }
/// 
/// // If your type isn't inside the `apresentation` crate,
/// // you need to replace `crate` with your crate name
/// #[proc_macros::register_impl(crate::elements::my_elem::MyProperty)]
/// impl PropertyCompatible for MyProperty {
///     type InnerRepresentation = MyUncomputedProperty;
/// 
///     fn from_value(...) -> Option<Self::InnerRepresentation> { ... }
/// 
///     fn to_self(...) -> Option<Self> { ... }
/// 
///     fn build_custom_rhai_type(...) { ... }
/// }
/// ```
pub trait PropertyCompatible: Clone {
    /// The inner representation of this property.
    /// 
    /// Due to the fact that nested values (e.g. in lists or maps) can be a
    /// code snippet themselves, there can potentially be infinite layers of
    /// uncomputed values that need to be computed at runtime. Due to this,
    /// an intermediate representation of the properties is needed, in which
    /// this information about nested uncomputed values is stored. This type is
    /// exactly that.
    /// 
    /// In practice, this means that this type should be an exact replica of
    /// the actual type implementing the trait, with the difference of each
    /// stored value's type being wrapped in a [`Property`]. This allows each
    /// nested value to have the possibility of being a dynamically evaluated
    /// value, which solves the problem stated above.
    /// 
    /// This type gets constructed in [`from_value()`](PropertyCompatible::from_value)
    /// and converted into the actual property type in
    /// [`to_self()`](PropertyCompatible::to_self).
    type InnerRepresentation;

    /// Tries to convert from the generic [`Value`] enum to the inner
    /// representation of this type.
    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized;

    /// Tries to convert the inner representation to the actual property type.
    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized;

    /// Function declaring how this type should look like when used in a Rhai
    /// code snippet.
    #[allow(unused_variables)]
    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static { None }
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

    impl<T: PropertyCompatible + 'static> PropertyCompatible for Option<T> {
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

    impl<T: PropertyCompatible + 'static> PropertyCompatible for Vec<T> {
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
        for_tuples!( where #( Tuple: PropertyCompatible + 'static )* );

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

    impl<T: PropertyCompatible + 'static, const N: usize> PropertyCompatible for [T; N] {
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