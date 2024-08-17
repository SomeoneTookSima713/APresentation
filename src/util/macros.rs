pub mod resource {
    macro_rules! int_resource {
        () => {};
        ($resource:ty) => {
            (<$resource>::get_type().to_string(), Box::new(<$resource>::create_boxed) as Box<FnResource>)
        };
        ($resource:ty, $($variadic:ty),+) => {
            int_resource!($($variadic),+),
            (<$resource>::get_type(), Box::new(<$resource>::create_boxed))
        };
    }
    
    macro_rules! resources {
        ($($variadic:ty),*) => {
            Lazy::new(|| HashMap::<String, Box<FnResource>>::from([
                $(crate::util::macros::resource::int_resource!($variadic)),*
            ]))
        };
    }

    pub(crate) use int_resource;
    pub(crate) use resources;
}

pub mod renderable {
    macro_rules! renderables {
        ($($name:literal => $rend:ty),*) => {
            pub fn register_renderables(hm: &mut HashMap<String, fn(ParseableRenderable<'static>) -> anyhow::Result<Box<dyn RenderableObjectSafe>>>) {
                // crate::util::macros::renderable::int_renderable!($($name => $variadic),*);
                $(hm.insert($name.to_string(), <$rend>::from_parseable_boxed);)*
            }
        };
    }

    pub(crate) use renderables;
}