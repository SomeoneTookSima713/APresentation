use std::rc::Rc;

use hashbrown::HashMap;

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
    rhai_engine: rhai::Engine,
    curr_state_idx: usize,
    curr_state: ActivePresState
}

impl std::fmt::Debug for Presentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct Presentation<'a> {
            states: &'a Vec<PresentationState>,
            assets: &'a asset::AssetManager,
            elements: &'a element::RegisteredElements,
            curr_state_idx: usize,
            curr_state: &'a ActivePresState
        }

        let Self {
            states,
            assets,
            elements,
            curr_state_idx,
            curr_state,
            ..
        } = self;

        std::fmt::Debug::fmt(&Presentation {
            states,
            assets,
            elements,
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
    pub fn new(
        filename: &str,
        registered_elements: element::RegisteredElements,
        mut asset_manager: asset::AssetManager,
        parser_collection: parser::ParserCollection,
        device: wgpu::Device,
        queue: wgpu::Queue
    ) -> Result<Self, PresentationCreationError> {
        let mut rhai_engine = rhai::Engine::new();
        rhai_engine.set_fail_on_invalid_map_property(true)
            .set_max_call_levels(64)
            // .set_optimization_level(rhai::OptimizationLevel::Full)
            .set_strict_variables(false);

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

        Ok(Self {
            states,
            assets: asset_manager,
            elements: registered_elements,
            rhai_engine,
            curr_state_idx: 0,
            curr_state
        })
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