use std::sync::Mutex;

use tokio::sync::Notify;

use super::state::State;

#[derive(Debug)]
pub(crate) struct SharedState {
    pub(crate) state: Mutex<State>,
    pub(crate) expiration_task: Notify,
}
