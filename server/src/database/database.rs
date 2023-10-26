use crate::jobs::job_queue::JobQueue;

use super::{shared_state::SharedState, state::State};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub(crate) struct Database {
    shared_state: Arc<SharedState>,
    job_queue: Arc<JobQueue>,
}

impl Database {
    pub(crate) fn new() -> Self {
        Self {
            shared_state: Arc::new(SharedState {
                state: Mutex::new(State {
                    data: BTreeMap::new(),
                    pub_sub_map: HashMap::new(),
                    expiration_set: BTreeSet::new(),
                    active: true,
                }),
            }),
            job_queue: Arc::new(JobQueue::new()),
        }
    }
}
