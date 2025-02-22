pub mod parser;
pub mod element;

pub struct Presentation {
    states: Vec<PresentationState>,
    curr_state_idx: usize,
    curr_state: ActivePresState
}

#[derive(Clone)]
pub struct PresentationState {

}

pub struct ActivePresState {

}

impl ActivePresState {
    pub fn new(base: PresentationState) -> Self {
        todo!()
    }
}