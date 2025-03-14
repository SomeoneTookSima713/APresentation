use std::any::TypeId;
use std::rc::Rc;
use std::time::Instant;

use hashbrown::HashMap;
use rhai::packages::Package;

pub mod parser;
pub mod element;
pub mod asset;

type RcElement = Rc<dyn element::ElemObjS>;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ElementID {
    Custom(String),
    Generated(u64)
}

impl std::hash::Hash for ElementID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ElementID::Custom(s) => state.write_str(s.as_str()),
            ElementID::Generated(id) => state.write_u64(*id),
        }
    }
}

pub struct Presentation {
    states: Vec<PresentationState>,
    assets: asset::AssetManager,
    elements: element::RegisteredElements,
    element_renderers: HashMap<TypeId, Box<dyn element::ElemRendObjS>>,
    rhai_engine: rhai::Engine,
    curr_state_idx: usize,
    curr_state: ActivePresState,
    last_time: Instant
}

impl std::fmt::Debug for Presentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct Presentation<'a> {
            states: &'a Vec<PresentationState>,
            assets: &'a asset::AssetManager,
            elements: &'a element::RegisteredElements,
            element_renderers: &'a HashMap<TypeId, Box<dyn element::ElemRendObjS>>,
            curr_state_idx: usize,
            curr_state: &'a ActivePresState
        }

        let Self {
            states,
            assets,
            elements,
            element_renderers,
            curr_state_idx,
            curr_state,
            ..
        } = self;

        std::fmt::Debug::fmt(&Presentation {
            states,
            assets,
            elements,
            element_renderers,
            curr_state_idx: *curr_state_idx,
            curr_state
        }, f)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PresentationCreationError {
    #[error("error parsing the given file: {0}")]
    ParserError(#[from] parser::ParserError),
    #[error("couldn't open the given file")]
    FileOpenError,
    #[error("no slides in the presentation")]
    NoSlides
}

impl Presentation {
    #[tracing::instrument(skip(parser_collection))]
    pub fn new(
        filename: &str,
        registered_elements: element::RegisteredElements,
        mut asset_manager: asset::AssetManager,
        parser_collection: parser::ParserCollection,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_config: &wgpu::SurfaceConfiguration
    ) -> Result<Self, PresentationCreationError> {
        let mut rhai_engine = rhai::Engine::new_raw();
        rhai_engine.set_fail_on_invalid_map_property(true)
            .set_max_call_levels(64)
            // .set_optimization_level(rhai::OptimizationLevel::Full)
            .set_strict_variables(false);

        rhai::packages::BasicMathPackage::new().register_into_engine_as(&mut rhai_engine, "math");

        // TODO: Find a more modular way of doing this
        proc_macros::for_types!(
            element::property::alignment::Alignment,
            crate::elements::rect::RectSource,
            crate::elements::rect::CornerRounding,
            crate::elements::rect::SamplerType,
            => {
                if let Some((modname, module)) = <Implementor as element::property::PropertyCompatible>::build_custom_rhai_type() {
                    rhai_engine.register_static_module(modname.as_str(), std::rc::Rc::new(module));
                }
            }
        );

        let mut file = std::fs::File::open(filename).map_err(|_| PresentationCreationError::FileOpenError)?;
        let states = parser_collection.parse(
            &mut file,
            filename,
            &registered_elements,
            &rhai_engine,
            &mut asset_manager,
            asset::AssetLoadingParams {
                gpu_device: device.clone(),
                gpu_queue: queue.clone()
            }
        )?;

        let mut curr_state = ActivePresState::new();
        curr_state.apply_new_state(states.get(0).ok_or(PresentationCreationError::NoSlides)?.clone());

        let mut element_renderers = HashMap::with_capacity(registered_elements.element_renderer.len());
        for (k, rend) in registered_elements.element_renderer.iter() {
            element_renderers.insert(*k, rend.init(device.clone(), queue.clone(), &surface_config));
        }

        Ok(Self {
            states,
            assets: asset_manager,
            elements: registered_elements,
            element_renderers,
            rhai_engine,
            curr_state_idx: 0,
            curr_state,
            last_time: Instant::now(),
        })
    }

    pub fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        for (type_id, renderer_impl) in self.elements.element_renderer.iter() {
            let renderer_obj = self.element_renderers.get_mut(type_id).expect("Unreachable");
            renderer_impl.reconfigure(&mut **renderer_obj, surface_config);
        }
    }

    pub fn render(
        &mut self,
        output_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        surface_config: &wgpu::SurfaceConfiguration
    ) -> anyhow::Result<()> {
        let dt = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();

        let bgcol = self.curr_state.background_color;

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: bgcol[0], g: bgcol[1], b: bgcol[2], a: 1.0 }),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None
        });

        let mut base_scope = rhai::Scope::new();
        base_scope
            .push_constant("w", surface_config.width as f64)
            .push_constant("h", surface_config.height as f64);

        for (id, (elem, t)) in self.curr_state.elements.iter_mut() {
            if let Some(renderer_impl) = self.elements.element_renderer.get(&elem.underlying_type_id())
            && let Some(renderer_obj) = self.element_renderers.get_mut(&elem.underlying_type_id()) {
                let mut scope = base_scope.clone_visible();
                scope.push_constant("t", *t);

                renderer_impl.submit_to_render(&mut **renderer_obj, &**elem, &self.rhai_engine, scope, &self.assets, &mut render_pass)?;

                *t += dt;
            } else {
                anyhow::bail!("No suitable renderer for element with ID \"{:?}\" found!", id);
            }
        }

        for (type_id, renderer_impl) in self.elements.element_renderer.iter() {
            let renderer_obj = self.element_renderers.get_mut(type_id).expect("Unreachable");
            renderer_impl.finish_render(&mut **renderer_obj, &mut render_pass);
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PresentationState {
    pub background_color: [f64; 3],
    pub removed_elements: Vec<ElementID>,
    pub new_elements: HashMap<ElementID, RcElement>
}

#[derive(Debug)]
pub struct ActivePresState {
    background_color: [f64; 3],
    elements: HashMap<ElementID, (RcElement, f64)>
}

impl ActivePresState {
    pub fn new() -> Self {
        Self { background_color: [0.0;3], elements: HashMap::new() }
    }

    pub fn apply_new_state(&mut self, state: PresentationState) {
        self.background_color = state.background_color;
        for id in state.removed_elements {
            self.elements.remove(&id);
        }
        for (id, elem) in state.new_elements {
            self.elements.insert(id, (elem, 0.0));
        }
    }
}