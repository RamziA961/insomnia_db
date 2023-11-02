use std::{collections::binary_heap::BinaryHeap, sync::Arc};

use chrono::{TimeZone, Utc};
use tracing::warn;

use super::scheduled_job::ScheduledJob;
use crate::database::shared_state::SharedState;

pub(crate) struct JobQueue {
    queue: BinaryHeap<ScheduledJob<Utc>>,
}

impl JobQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn push<Tz>(&mut self, job: ScheduledJob<Tz>)
    where
        Tz: TimeZone,
    {
        self.queue.push(job.to_utc());
    }

    pub(crate) fn run_pending_jobs(&mut self, state: Arc<SharedState>) -> Result<(), ()> {
        // More control over handling should be supported
        // Should subsequent due jobs run after a job fails?
        // Or should dependent subjobs be aggragated to allow for that behavior?

        // Prevent tasks with small intervals from blocking other tasks by
        // withholding them from the job queue until the end of the function
        let mut executed = vec![];

        while self.queue.peek().is_some_and(|v| v.is_due()) {
            let mut job = self.queue.pop().unwrap();

            job.run(state).map_err(|e| warn!("Job execution failed."));

            if !job.has_expired() {
                executed.push(job);
            }
        }

        executed.into_iter().for_each(|v| self.queue.push(v));

        Ok(())
    }

    pub(crate) fn peek(&mut self) -> Option<&ScheduledJob<Utc>> {
        self.queue.peek()
    }

    pub(crate) fn pop(&mut self) -> Option<ScheduledJob<Utc>> {
        self.queue.pop()
    }

    pub(crate) fn clear(&mut self) {
        self.queue.clear()
    }
}
