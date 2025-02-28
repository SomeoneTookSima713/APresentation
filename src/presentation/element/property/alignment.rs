use super::{ PropertyCompatible, Value };

#[derive(Clone, Copy)]
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

impl PropertyCompatible for Alignment {
    fn from_value(val: Value) -> Option<Self>
    where Self: Sized {
        if let Value::EnumVariant(variant, v) = val {
            Some(match variant.to_lowercase().replace("_", "").as_str() {
                "topleft" => Self::TopLeft,
                "topcenter" => Self::TopCenter,
                "topright" => Self::TopRight,
                "midleft" => Self::MidLeft,
                "midcenter"|"centered" => Self::MidCenter,
                "midright" => Self::MidRight,
                "bottomleft" => Self::BottomLeft,
                "bottomcenter" => Self::BottomCenter,
                "bottomright" => Self::BottomRight,
                "custom" if let Some(v) = v.map(Box::into_inner) => {
                    let align = <(f64, f64) as PropertyCompatible>::from_value(v)?;
                    Self::Custom(align.0, align.1)
                },
                _ => None?
            })
        } else {
            None
        }
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