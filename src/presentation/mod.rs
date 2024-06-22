pub mod renderable;
pub mod property;
pub mod resource_managers;
pub mod state;

use std::any::TypeId;
use std::collections::HashMap;
use std::path::Path;
use std::sync::MutexGuard;
use std::time::Instant;

use renderable::{ RenderableRenderingManagerObjectSafe, RenderableObjectSafe };
use crate::parse;

pub struct Presentation {
    slides: Vec<Slide>,
    current_slide: usize,
    curr_slide_beginning: Instant,
    last_slide_beginning: Instant,
}

pub struct Slide {
    renderables: Vec<RenderableWrapper>
}

struct RenderableWrapper {
    renderable: Box<dyn RenderableObjectSafe>
}

impl Presentation {
    pub fn new() -> Self {
        let mut slides = Vec::new();
        slides.push(Slide::default());

        Self {
            slides,
            current_slide: 0,
            curr_slide_beginning: Instant::now(),
            last_slide_beginning: Instant::now()
        }
    }

    pub fn load_file<P: AsRef<Path>>(path: P, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<Self> {
        use std::io::prelude::*;
        use std::fs;

        let str = fs::read_to_string(path.as_ref())?;
        let ext = path.as_ref().extension().ok_or(anyhow::anyhow!("Couldn't determine the presentation's file extension!"))?.to_string_lossy();
        if let Some(parser) = parse::PARSER_IMPLEMENTATIONS.get(ext.as_ref()) {
            let parseable = parser.parse(str)?;
            for resource in parseable.resources {
                resource.load(device, queue)?;
            }
            Ok(Presentation {
                slides: parseable.slides.into_iter().map(|p| Slide::from_parseable(p)).chain([Ok(Slide::default())]).try_collect()?,
                current_slide: 0,
                curr_slide_beginning: Instant::now(),
                last_slide_beginning: Instant::now()
            })
        } else {
            anyhow::bail!("Couldn't find a suitable parsing implementation for given file! Parsing implementations are available for the following file types: {}",
                parse::PARSER_IMPLEMENTATIONS.keys().cloned().reduce(|acc, e| format!("{acc}, {e}")).unwrap_or("None lol".to_owned()))
        }
    }

    pub fn next_slide(&mut self) {
        self.current_slide = (self.current_slide+1) % self.slides.len();
        self.last_slide_beginning = self.curr_slide_beginning;
        self.curr_slide_beginning = Instant::now();
    }

    pub fn previous_slide(&mut self) {
        if self.current_slide==0 {
            self.current_slide = self.slides.len()-1;
        } else {
            self.current_slide -= 1;
        }

        self.last_slide_beginning = self.curr_slide_beginning;
        self.curr_slide_beginning = Instant::now();
    }

    pub fn update(&mut self, lua: &MutexGuard<'static, mlua::Lua>, dt: f64) -> anyhow::Result<()> {
        if let Some(slide) = self.slides.get_mut(self.current_slide) {
            slide.update(lua, dt)
        } else {
            log::warn!("No slide exists at current index of {}!", self.current_slide);
            anyhow::bail!("No slide exists at current index of {}!", self.current_slide)
        }
    }

    pub fn render(
        &mut self,
        rendering_managers: &mut HashMap<TypeId, Box<dyn RenderableRenderingManagerObjectSafe>>,
        args: mlua::Variadic<mlua::Value<'static>>
    ) -> anyhow::Result<()> {
        if let Some(slide) = self.slides.get_mut(self.current_slide) {
            slide.render(rendering_managers, args)
        } else {
            log::warn!("No slide exists at current index of {}!", self.current_slide);
            anyhow::bail!("No slide exists at current index of {}!", self.current_slide)
        }
    }
}

impl Slide {
    pub fn new() -> Self {
        Self { renderables: Vec::new() }
    }

    pub fn from_parseable(parseable: parse::ParseableSlide) -> anyhow::Result<Self> {
        Ok(Self {
            renderables: parseable.renderables.into_iter().enumerate().map(|(i, p)| {
                use property::{ Property, PropertyValue };
                
                let rend_type_prop = p.get("type").ok_or(anyhow::anyhow!("'type' property of renderable #{} not specified!", i+1))?;
                let rend_type = if let Property::Constant(PropertyValue::String(ref s)) = &*rend_type_prop {
                    s.clone()
                } else {
                    anyhow::bail!("'type' property of renderable #{} isn't of type String!", i+1)
                };
                drop(rend_type_prop);

                let renderable = (renderable::RENDERABLES.get(rend_type.as_str()).ok_or(anyhow::anyhow!("No renderable of type '{rend_type}' exists!"))?)(p)?;

                Ok(RenderableWrapper::new_boxed(renderable))
            }).try_collect()?
        })
    }

    pub fn update(&mut self, lua: &MutexGuard<'static, mlua::Lua>, dt: f64) -> anyhow::Result<()> {
        for renderable in self.renderables.iter() {
            renderable.update(lua, dt)?;
        }

        Ok(())
    }

    pub fn render(
        &mut self,
        rendering_managers: &mut HashMap<TypeId, Box<dyn RenderableRenderingManagerObjectSafe>>,
        args: mlua::Variadic<mlua::Value<'static>>
    ) -> anyhow::Result<()> {
        for renderable in self.renderables.iter() {
            renderable.render(rendering_managers, args.clone())?;
        }
        Ok(())
    }
}

impl Default for Slide {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderableWrapper {
    pub fn new<T: RenderableObjectSafe>(renderable: T) -> Self {
        Self {
            renderable: Box::new(renderable)
        }
    }

    pub fn new_boxed(renderable: Box<dyn RenderableObjectSafe>) -> Self {
        Self { renderable }
    }

    pub fn update(&self, lua: &MutexGuard<'static, mlua::Lua>, dt: f64) -> anyhow::Result<()> {
        self.renderable.begin_new_frame()
    }

    pub fn render(
        &self,
        rendering_managers: &mut HashMap<TypeId, Box<dyn RenderableRenderingManagerObjectSafe>>,
        args: mlua::Variadic<mlua::Value<'static>>
    ) -> anyhow::Result<()> {
        if let Some(rendering_manager) = rendering_managers.get_mut(&self.renderable.get_type()) {
            self.renderable.render(Box::as_mut(rendering_manager), args)
        } else {
            log::warn!("No rendering manager for renderable!");
            anyhow::bail!("No rendering manager for renderable!")
        }
    }
}