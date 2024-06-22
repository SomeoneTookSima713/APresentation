use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::super::{ Property, PropertyCompatible, PropertyStructure, EvaluatedPropertyValue };

#[derive(Clone, Copy)]
pub struct Color(pub palette::Srgba<f64>);

impl<'lua> PropertyCompatible<'lua> for Color {
    const STRUCTURE: PropertyStructure = PropertyStructure::Or(&[
        PropertyStructure::String(Some(r"#\d{6}\d{2}?")), // Standard Hex code #RRGGBB(AA)
        PropertyStructure::Array(&[PropertyStructure::Number;3]), // RGB values in the range from 0 to 1
        PropertyStructure::Array(&[PropertyStructure::Number;4]), // RGBA values in the range from 0 to 1
        PropertyStructure::Dict(&[
            ("r", PropertyStructure::Number),
            ("g", PropertyStructure::Number),
            ("b", PropertyStructure::Number),
            // ("a", PropertyStructure::Number), // This one is optional and thus is only in a comment, otherwise it'd be required
        ]), // RGB(A) values in the range from 0 to 1
        PropertyStructure::Dict(&[
            ("h", PropertyStructure::Number),
            ("s", PropertyStructure::Number),
            ("l", PropertyStructure::Number),
            // ("a", PropertyStructure::Number), // This one is optional and thus is only in a comment, otherwise it'd be required
        ]), // HSL(A) values in the range from 0 to 1
        PropertyStructure::Dict(&[
            ("h", PropertyStructure::Number),
            ("s", PropertyStructure::Number),
            ("v", PropertyStructure::Number),
            // ("a", PropertyStructure::Number), // This one is optional and thus is only in a comment, otherwise it'd be required
        ]), // HSV(A) values in the range from 0 to 1
        PropertyStructure::Dict(&[
            ("l", PropertyStructure::Number),
            ("a", PropertyStructure::Number),
            ("b", PropertyStructure::Number),
            // ("a", PropertyStructure::Number), // This one is optional and thus is only in a comment, otherwise it'd be required
        ]), // Oklab(A) values in the range from 0 to 1
    ]);

    fn convert_from<A: mlua::IntoLuaMulti<'lua> + Clone>(value: Property<'lua>, args: A) -> anyhow::Result<Self>
        where Self: Sized {
        match value.get_recursively(args)? {
            EvaluatedPropertyValue::String(ref str) => {
                let mut components = [255_u8; 4];
                let hexstr = str.split_at(1).1;
                if hexstr.len()<6 {
                    anyhow::bail!("Color hex string too short! (Expected at least six, got {} characters!)", hexstr.len());
                } else if hexstr.len()%2 == 1 {
                    anyhow::bail!("Color hex string incomplete! (Got uneven amount of number characters, which shouldn't happen!)");
                }
                hex::decode_to_slice(hexstr, components.as_mut_slice())?;
                Ok(Color(palette::cast::from_array::<palette::Srgba<f64>>(components.map(|u|u as f64 / 255.0))))
            },
            EvaluatedPropertyValue::List(vecrc) => if let Ok(vec) = vecrc.try_borrow() {
                // Gets all color components (Alpha is optional, thus `map()`
                // instead of `and_then()`) and errors if R, G, and B
                // components aren't found.
                let components = vec.get(0)
                    .and_then(|r| vec.get(1).map(|g| [r,g]))
                    .and_then(|[r,g]| vec.get(2).map(|b| [r,g,b]))
                    .map(|[r,g,b]| [r.clone(),g.clone(),b.clone(), vec.get(3).cloned().unwrap_or(EvaluatedPropertyValue::Float(1.0))])
                    .ok_or(anyhow::anyhow!("Invalid Property! (too few components in color)"))?;
                // Converts the `EvaluatedPropertyValue`s into floats (or errors if they aren't numbers)
                let floats = components.into_iter().enumerate().map(|(i, v)| match v {
                    EvaluatedPropertyValue::Float(f) => Ok(f),
                    _ => Err(anyhow::anyhow!("Invalid Property! (color component #{} has invalid type)",i+1))
                }).try_collect::<Cow<[f64]>>()?;
                Ok(Color(palette::cast::from_component_slice::<palette::Srgba<f64>>(floats.as_ref())[0]))
            } else {
                anyhow::bail!("Internal error! (Vec of EvalueatedPropertyValue::List is mutably borrowed)");
            },
            EvaluatedPropertyValue::Dict(hmrc) => if let Ok(hm) = hmrc.try_borrow() {
                use palette::convert::FromColor;
                use palette::{ Srgba, Hsla, Hsva, Oklaba };

                let a = match hm.get("alpha").cloned().unwrap_or(EvaluatedPropertyValue::Float(1.0)) {
                    EvaluatedPropertyValue::Float(f) => f,
                    _ => anyhow::bail!("Invalid Property! (alpha component of color has invalid type)")
                };
                // println!("{hm:#?}");
                let mut hmconv = hm.iter().map(|(k,v)| (k.as_str(), v)).filter(|e|e.0 != "alpha").collect::<Vec<(&str, &EvaluatedPropertyValue)>>();
                hmconv.sort_unstable_by(|a, b| a.0.cmp(b.0));
                // println!("{hmconv:#?}");
                match hmconv.as_slice() {
                    &[
                        ("b", bref),
                        ("g", gref),
                        ("r", rref),
                        ..
                    ] => {
                        match (rref.clone(), gref.clone(), bref.clone()) {
                            (
                                EvaluatedPropertyValue::Float(r),
                                EvaluatedPropertyValue::Float(g),
                                EvaluatedPropertyValue::Float(b),
                            ) => {
                                Ok(Color(Srgba::new(r,g,b,a)))
                            },
                            _ => anyhow::bail!("Invalid Property! (color components have invalid types)")
                        }
                    },
                    &[
                        ("h", href),
                        ("l", lref),
                        ("s", sref),
                        ..
                    ] => {
                        match (href.clone(), sref.clone(), lref.clone()) {
                            (
                                EvaluatedPropertyValue::Float(h),
                                EvaluatedPropertyValue::Float(s),
                                EvaluatedPropertyValue::Float(l),
                            ) => {
                                Ok(Color(Srgba::from_color(Hsla::new(h, s, l, a))))
                            },
                            _ => anyhow::bail!("Invalid Property! (color components have invalid types)")
                        }
                    },
                    &[
                        ("h", href),
                        ("s", sref),
                        ("v", vref),
                        ..
                    ] => {
                        match (href.clone(), sref.clone(), vref.clone()) {
                            (
                                EvaluatedPropertyValue::Float(h),
                                EvaluatedPropertyValue::Float(s),
                                EvaluatedPropertyValue::Float(v),
                            ) => {
                                Ok(Color(Srgba::from_color(Hsva::new(h, s, v, a))))
                            },
                            _ => anyhow::bail!("Invalid Property! (color components have invalid types)")
                        }
                    },
                    &[
                        ("a", caref),
                        ("b", bref),
                        ("l", lref),
                        ..
                    ] => {
                        match (lref.clone(), caref.clone(), bref.clone()) {
                            (
                                EvaluatedPropertyValue::Float(l),
                                EvaluatedPropertyValue::Float(ca),
                                EvaluatedPropertyValue::Float(b),
                            ) => {
                                Ok(Color(Srgba::from_color(Oklaba::new(l, ca, b, a))))
                            },
                            _ => anyhow::bail!("Invalid Property! (color components have invalid types)")
                        }
                    },
                    _ => anyhow::bail!("Invalid Property! (unsupported/unrecognized list of color components)")
                }
            } else {
                anyhow::bail!("Internal error! (HashMap of EvalueatedPropertyValue::Dict is mutably borrowed)");
            },
            _ => anyhow::bail!("Invalid Property!")
        }
    }

    fn convert_into(&'lua self) -> Property<'lua> {
        Property::Constant(crate::presentation::property::PropertyValue::Dict(Rc::new(RefCell::new(HashMap::from([
            ("r", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.red))),
            ("g", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.green))),
            ("b", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.blue))),
            ("alpha", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.alpha))),
        ].map(|(s,p)|(s.to_string(), p)))))))
    }

    fn move_into(self) -> Property<'lua>
    where Self: Sized {
        Property::Constant(crate::presentation::property::PropertyValue::Dict(Rc::new(RefCell::new(HashMap::from([
            ("r", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.red))),
            ("g", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.green))),
            ("b", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.blue))),
            ("alpha", Property::Constant(crate::presentation::property::PropertyValue::Float(self.0.alpha))),
        ].map(|(s,p)|(s.to_string(), p)))))))
    }
}

impl Into<[f64; 4]> for Color {
    fn into(self) -> [f64; 4] {
        palette::cast::into_array(self.0)
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        palette::cast::into_array(self.0).map(|f| f as f32)
    }
}