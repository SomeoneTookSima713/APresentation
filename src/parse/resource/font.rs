use super::*;

#[derive(Debug)]
pub struct Font(Rc<String>);

fn load_fonts_recursively<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    for f in std::fs::read_dir(path.as_ref())? {
        let file = f?;
        let file_path = file.path();
        if file.metadata()?.is_dir() {
            load_fonts_recursively(file_path)?;
        } else {
            if let Err(e) = crate::presentation::resource_managers::FONT_MANAGER.load_font(&file_path) {
                log::warn!("Error loading font at path {file_path:?}: {e}");
            }
        }
    }
    Ok(())
}

impl Resource for Font {
    fn load(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        if std::fs::metadata(self.0.as_ref())?.is_dir() {
            load_fonts_recursively(self.0.as_ref())?;
        } else {
            crate::presentation::resource_managers::FONT_MANAGER.load_font(self.0.as_ref())?;
        }

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