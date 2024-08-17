use super::*;

#[derive(Debug)]
pub struct Font(Rc<String>);

impl Resource for Font {
    fn load(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        todo!();

        Ok(())
    }

    fn create(value: &HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Self>
    where Self: Sized {
        let path = if let EvaluatedPropertyValue::String(s) = value.get("path").ok_or(anyhow::anyhow!("File path of Font unspecified! (property name: 'path')"))? {
            s.clone()
        } else {
            anyhow::bail!("'path' property needs to be of type String!")
        };
        Ok(Self(path))
    }

    fn get_name(&self) -> &str { self.0.as_str() }

    fn get_type() -> &'static str { "Font" }
}