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
    states: HashMap<String, PresentationState>,
    assets: asset::AssetManager,
    curr_state_idx: usize,
    curr_state: ActivePresState
}

#[derive(Clone, Debug)]
pub struct PresentationState {
    pub background_color: [f64; 3],
    pub removed_elements: Vec<ElementID>,
    pub new_elements: HashMap<ElementID, RcElement>
}

pub struct ActivePresState {
    background_color: [f64; 3],
    elements: HashMap<ElementID, (RcElement, f64)>
}

impl ActivePresState {
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