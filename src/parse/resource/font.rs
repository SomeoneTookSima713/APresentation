use super::*;

pub enum Font {
    VerySimple {
        filepath: Rc<String>,
        name: Rc<String>,
        index: u32,
    },
    Simple {
        font_regular: (Rc<String>, u32),
        font_bold: (Rc<String>, u32),
        font_italic: (Rc<String>, u32),
        font_bold_italic: (Rc<String>, u32),
        name: Rc<String>,
    },
    System {
        name: Rc<String>
    }
}

impl Resource for Font {
    fn load(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        todo!();

        Ok(())
    }

    fn create(value: &HashMap<String, EvaluatedPropertyValue>) -> anyhow::Result<Self>
    where Self: Sized {
        let name = if let EvaluatedPropertyValue::String(s) = value.get("name").ok_or(anyhow::anyhow!("Name of Font unspecified!"))? {
            s.clone()
        } else {
            anyhow::bail!("'name' property needs to be of type String!")
        };
        let filepaths = if let EvaluatedPropertyValue::List(l) = value.get("files").ok_or(anyhow::anyhow!("File path of Font unspecified! (Property name: 'files')"))? {
            if let [
                EvaluatedPropertyValue::String(ref s1),
                EvaluatedPropertyValue::String(ref s2),
                EvaluatedPropertyValue::String(ref s3),
                EvaluatedPropertyValue::String(ref s4)
            ] = l.borrow().get(0..4).ok_or(anyhow::anyhow!("'files' property needs to be a List of Strings!")) {
                (s1.clone(), s2.clone(), s3.clone(), s4.clone())
            } else {
                anyhow::bail!("'files' property needs to be a List of Strings!")
            }
        } else {
            anyhow::bail!("'files' property needs to be a List of Strings!")
        };
        let index = if let EvaluatedPropertyValue::UInt(i) = value.get("index").unwrap_or(&EvaluatedPropertyValue::UInt(0)) {
            *i as u32
        } else {
            anyhow::bail!("'index' property needs to be of type Number!")
        };

        Ok(Self { filepaths, name, index })
    }

    fn get_name(&self) -> &str { &self.name }

    fn get_type() -> &'static str { "Font" }
}