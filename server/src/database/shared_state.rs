use std::sync::Mutex;

use tokio::{sync::Notify, time::Instant};
use tracing::info;

use super::state::State;

#[derive(Debug)]
pub(crate) struct SharedState {
    pub(crate) state: Mutex<State>,
    pub(crate) expiration_task: Notify,
    pub(crate) job_queue_task: Notify,
}

impl SharedState {
    pub(super) fn purge_expired(&self) -> Option<Instant> {
        let mut state = self.state.lock().unwrap();

        if !state.active {
            return None;
        }

        let now = Instant::now();
        let state = &mut *state;

        while let Some(&(time, ref key)) = state.expiration_set.iter().next() {
            if time > now {
                return Some(time);
            }

            info!(key = key.clone(), "purging key");
            state.data.remove(key);
            state.expiration_set.remove(&(time, key.clone()));
        }

        None
    }

    pub(crate) fn has_shutdown(&self) -> bool {
        !self.state.lock().unwrap().active
    }
}
