use super::{entry, shared_state::SharedState, state::State};
use crate::jobs::job_queue::{self, JobQueue};

use bytes::Bytes;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{Arc, Mutex},
};
use tokio::{
    sync::{broadcast, Notify},
    time::{Duration, Instant},
};
use tracing::instrument;

#[derive(Clone)]
pub(crate) struct Database {
    shared_state: Arc<SharedState>,
    job_queue: Arc<tokio::sync::Mutex<JobQueue>>,
}

impl Database {
    pub(crate) fn new() -> Self {
        let shared_state = Arc::new(SharedState {
            state: Mutex::new(State {
                data: BTreeMap::new(),
                pub_sub_map: HashMap::new(),
                expiration_set: BTreeSet::new(),
                active: true,
            }),
            expiration_task: Notify::new(),
            job_queue_task: Notify::new(),
        });

        let job_queue = Arc::new(tokio::sync::Mutex::new(JobQueue::new()));

        //background task for cleaning expired data
        tokio::spawn(purge_expired(shared_state.clone()));

        // background task for job queue
        // tokio::spawn(async move {
        // });
        tokio::spawn(job_queue::run_background_task(
            shared_state.clone(),
            job_queue.clone(),
        ));

        Self {
            shared_state,
            job_queue,
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared_state.state.lock().unwrap();
        state.data.get(key).map(|v| v.buf.clone())
    }

    pub(crate) fn set(&self, key: String, val: Bytes, expiration: Option<Duration>) {
        let mut state = self.shared_state.state.lock().unwrap();

        // should notify the task
        let mut should_notify = false;

        let expiration = expiration.map(|dur| {
            let time = Instant::now() + dur;

            should_notify = state.get_expired().map(|next| next > time).unwrap_or(true);

            time
        });

        let new_entry = entry::Entry::builder()
            .with_bytes(val)
            .with_expiration(expiration)
            .build_consume()
            .unwrap();

        // need to store, to ensure that if value is replaced the corresponding
        // expiration data in the expiration_set is cleared.
        let replaced = state.data.insert(key.clone(), new_entry);

        if let Some(val) = replaced {
            if let Some(expiration) = val.expiration {
                state.expiration_set.remove(&(expiration, key.clone()));
            }
        }

        if let Some(exp) = expiration {
            state.expiration_set.insert((exp, key.clone()));
        }

        drop(state);

        if should_notify {
            self.shared_state.expiration_task.notify_one();
        }
    }

    /// Request a reciever for a requested channel identified by its key.
    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        use std::collections::hash_map::Entry;

        let mut state = self.shared_state.state.lock().unwrap();

        match state.pub_sub_map.entry(key) {
            Entry::Occupied(o) => {
                // Broadcast channel exists already, so subscribe.
                o.get().subscribe()
            }
            Entry::Vacant(v) => {
                // Broadcast channel does not exist yet, so create one.
                //
                // Here a capacity of 1024 is chosen to prevent slow subscribers from causing
                // messages to be held indefinitely. This value may need fine-tuning.
                let (tx, rx) = broadcast::channel(1024);
                v.insert(tx);
                rx
            }
        }
    }

    /// Publish a message to a channel. Returns the number of subscribed listeners.
    pub(crate) fn publish(&self, key: &str, val: Bytes) -> usize {
        let state = self.shared_state.state.lock().unwrap();

        state
            .pub_sub_map
            .get(key)
            .map(|tx| tx.send(val).unwrap_or(0))
            .unwrap_or(0)
    }

    fn halt_background_tasks(&self) {
        todo!()
    }
}

#[instrument(name = "purge_expired")]
async fn purge_expired(shared: Arc<SharedState>) {
    while shared.has_shutdown() {
        if let Some(time) = shared.purge_expired() {
            tokio::select! {
                _ = tokio::time::sleep_until(time) => {},
                _ = shared.expiration_task.notified() => {}
            }
        } else {
            shared.expiration_task.notified().await;
        }
    }
}
