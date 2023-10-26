use super::entry::Entry;
use bytes::Bytes;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use tokio::{sync::broadcast, time::Instant};

#[derive(Debug)]
pub(crate) struct State {
    pub(super) data: BTreeMap<String, Entry>,
    pub(super) pub_sub_map: HashMap<String, broadcast::Sender<Bytes>>,
    pub(super) expiration_set: BTreeSet<(Instant, String)>,
    pub(crate) active: bool,
}

impl State {
    pub(super) fn get_expired(&self) -> Option<Instant> {
        self.expiration_set.iter().next().map(|k| k.0)
    }
}
