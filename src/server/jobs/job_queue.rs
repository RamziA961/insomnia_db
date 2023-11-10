use std::{collections::binary_heap::BinaryHeap, sync::Arc};

use chrono::{TimeZone, Utc};
use tokio::sync::Mutex;
use tracing::{instrument, warn};

use super::scheduled_job::ScheduledJob;
use crate::server::database::shared_state::SharedState;

#[derive(Debug)]
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

    #[instrument(name = "run_pending_jobs")]
    pub(crate) fn run_pending_jobs(&mut self, state: Arc<SharedState>) -> Result<(), ()> {
        // More control over handling should be supported
        // Should subsequent due jobs run after a job fails?
        // Or should dependent subjobs be aggragated to allow for that behavior?

        // Prevent tasks with small intervals from blocking other tasks by
        // withholding them from the job queue until the end of the function
        let mut executed = vec![];

        while self.queue.peek().is_some_and(|v| v.is_due()) {
            let mut job = self.queue.pop().unwrap();

            let _ = job.run(state.clone()).map_err(|_| {
                warn!("Job execution failed.");
            });

            if !job.has_expired() {
                executed.push(job);
            }
        }

        executed.into_iter().for_each(|v| self.queue.push(v));

        Ok(())
    }

    pub(crate) fn peek(&self) -> Option<&ScheduledJob<Utc>> {
        self.queue.peek()
    }

    pub(crate) fn pop(&mut self) -> Option<ScheduledJob<Utc>> {
        self.queue.pop()
    }

    pub(crate) fn clear(&mut self) {
        self.queue.clear()
    }
}

pub(crate) async fn run_background_task(shared: Arc<SharedState>, queue: Arc<Mutex<JobQueue>>) {
    use tokio::time::Instant;

    while !shared.clone().has_shutdown() {
        let mut lock = queue.lock().await;
        let next = lock.peek();

        match next {
            Some(job) if job.is_due() => {
                lock.run_pending_jobs(shared.clone());
            }
            Some(job) => {
                let diff = job.due_at().clone() - Utc::now();
                let wake_at = Instant::now() + diff.to_std().unwrap();

                tokio::select! {
                    _ = tokio::time::sleep_until(wake_at) => {},
                    _ = shared.job_queue_task.notified() => {}
                };
            }
            None => {
                shared.job_queue_task.notified().await;
            }
        }
    }
}
