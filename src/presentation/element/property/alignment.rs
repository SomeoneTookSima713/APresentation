use super::{ Property, PropertyCompatible, Value };

#[derive(Clone, Copy, Debug)]
pub enum Alignment {
    TopLeft,
    TopCenter,
    TopRight,
    MidLeft,
    MidCenter,
    MidRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Custom(f64, f64)
}

pub enum UncomputedAlignment {
    TopLeft,
    TopCenter,
    TopRight,
    MidLeft,
    MidCenter,
    MidRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Custom(Property<f64>, Property<f64>)
}

impl PropertyCompatible for Alignment {
    type InnerRepresentation = UncomputedAlignment;

    fn from_value(val: Value, engine: &rhai::Engine) -> Option<Self::InnerRepresentation>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val {
            Some(match variant.to_lowercase().replace("_", "").as_str() {
                "topleft" => UncomputedAlignment::TopLeft,
                "topcenter" => UncomputedAlignment::TopCenter,
                "topright" => UncomputedAlignment::TopRight,
                "midleft" => UncomputedAlignment::MidLeft,
                "midcenter"|"centered" => UncomputedAlignment::MidCenter,
                "midright" => UncomputedAlignment::MidRight,
                "bottomleft" => UncomputedAlignment::BottomLeft,
                "bottomcenter" => UncomputedAlignment::BottomCenter,
                "bottomright" => UncomputedAlignment::BottomRight,
                "custom" if let Some(v) = v.map(Box::into_inner) => {
                    let align = <(f64, f64) as PropertyCompatible>::from_value(v, engine)?;
                    UncomputedAlignment::Custom(align.0, align.1)
                },
                _ => None?
            })
        } else {
            None
        }
    }

    fn to_self(base: &Self::InnerRepresentation, engine: &rhai::Engine, scope: &mut rhai::Scope<'static>) -> Option<Self>
    where Self: Sized {
        Some(match base {
            UncomputedAlignment::TopLeft => Self::TopLeft,
            UncomputedAlignment::TopCenter => Self::TopCenter,
            UncomputedAlignment::TopRight => Self::TopRight,
            UncomputedAlignment::MidLeft => Self::MidLeft,
            UncomputedAlignment::MidCenter => Self::MidCenter,
            UncomputedAlignment::MidRight => Self::MidRight,
            UncomputedAlignment::BottomLeft => Self::BottomLeft,
            UncomputedAlignment::BottomCenter => Self::BottomCenter,
            UncomputedAlignment::BottomRight => Self::BottomRight,
            UncomputedAlignment::Custom(x, y) => Self::Custom(x.evaluate(scope, engine)?, y.evaluate(scope, engine)?)
        })
    }

    fn build_custom_rhai_type() -> Option<(String, rhai::Module)>
    where Self: Sized + 'static {
        let mut module = rhai::Module::new();
        module.set_native_fn("TopLeft", || Ok(Self::TopLeft));
        module.set_native_fn("TopCenter", || Ok(Self::TopCenter));
        module.set_native_fn("TopRight", || Ok(Self::TopRight));
        module.set_native_fn("MidLeft", || Ok(Self::MidLeft));
        module.set_native_fn("MidCenter", || Ok(Self::MidCenter));
        module.set_native_fn("MidRight", || Ok(Self::MidRight));
        module.set_native_fn("BottomLeft", || Ok(Self::BottomLeft));
        module.set_native_fn("BottomCenter", || Ok(Self::BottomCenter));
        module.set_native_fn("BottomRight", || Ok(Self::BottomRight));
        module.set_native_fn("Custom", |x: f64, y: f64| Ok(Self::Custom(x, y)));

        Some(("Alignment".to_string(), module))
    }
}

impl Into<(f64, f64)> for Alignment {
    fn into(self) -> (f64, f64) {
        use Alignment::*;
        match self {
            TopLeft =>      (0.0, 0.0),
            TopCenter =>    (0.5, 0.0),
            TopRight =>     (1.0, 0.0),
            MidLeft =>      (0.0, 0.5),
            MidCenter =>    (0.5, 0.5),
            MidRight =>     (1.0, 0.5),
            BottomLeft =>   (0.0, 1.0),
            BottomCenter => (0.5, 1.0),
            BottomRight =>  (1.0, 1.0),
            Custom(x, y) => (  x,   y)
        }
    }
}

impl Alignment {
    pub fn into_fracts(&self) -> (f64, f64) {
        (*self).into()
    }
}