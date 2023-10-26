use std::sync::Mutex;

use super::state::State;

#[derive(Debug)]
pub(crate) struct SharedState {
    pub(crate) state: Mutex<State>,
}
