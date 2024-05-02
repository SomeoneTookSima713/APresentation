use super::{ Renderable, ParseableRenderable, BaseProperties, extended_structure };
use crate::presentation::property::{ PropertyStructure, TypedProperty, PropertyCompatible };

pub struct Rectangle<'lua> {
    base_properties: BaseProperties<'lua>,
    size: TypedProperty<'lua, (f64, f64)>,
    corner_rounding: TypedProperty<'lua, f64>
}

impl<'lua> Renderable<'lua> for Rectangle<'lua> {
    const PROPERTY_STRUCTURE: &'static [(&'static str, PropertyStructure)] = &extended_structure::<BaseProperties, _>([
        ("size", <(f64, f64) as PropertyCompatible<'lua>>::STRUCTURE),
        ("corner_rounding", f64::STRUCTURE),
    ]);

    fn from_parseable(parseable: ParseableRenderable<'lua>) -> anyhow::Result<Self>
    where Self: Sized {
        let size = TypedProperty::new(parseable.get("size").ok_or(anyhow::anyhow!("Invalid Property!"))?.clone())?;
        let corner_rounding = TypedProperty::new(parseable.get("corner_rounding").ok_or(anyhow::anyhow!("Invalid Property!"))?.clone())?;
        Ok(Self {
            base_properties: BaseProperties::from_parseable(parseable)?,
            size,
            corner_rounding,
        })
    }

    fn to_parseable(&'lua self) -> anyhow::Result<ParseableRenderable<'lua>> {
        let mut properties = self.base_properties.to_parseable()?;
        properties.extend(&[
            ("size".to_string(), self.size.convert_into()),
            ("corner_rounding".to_string(), self.corner_rounding.convert_into()),
        ]).map_err(|_| anyhow::anyhow!("Impossible error: Couldn't immutably borrow unborrowed RefCell!"))?;
        Ok(properties)
    }

    fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        // \operatorname{abs}\left(\frac{x}{w}\right)^{p}+\operatorname{abs}\left(\frac{y}{h}\right)^{p}\le1
        todo!();
        Ok(())
    }
}