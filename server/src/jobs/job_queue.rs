use std::{collections::binary_heap::BinaryHeap, sync::Arc};

use super::scheduled_job::ScheduledJob;
use crate::database::shared_state::SharedState;

#[derive(Debug)]
pub(crate) struct JobQueue {
    queue: BinaryHeap<ScheduledJob>,
}

impl JobQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn push(&mut self, job: ScheduledJob) {
        self.queue.push(job);
    }

    pub(crate) fn run_ready(&mut self, state: Arc<SharedState>) -> Result<(), ()> {
        let n_tasks_due = {
            let mut count = 0;
            for t in self.queue.iter() {
                if !t.is_due() {
                    break;
                }
                count += 1;
            }
            count
        };

        if n_tasks_due == 0 {
            return Ok(());
        }

        let mut failed = false;
        let mut layover: Vec<ScheduledJob> = Vec::with_capacity(n_tasks_due);

        for _ in 0..n_tasks_due {
            let mut t = self.queue.pop().unwrap();

            if failed {
                t.reschedule();
            } else {
                if t.run(state.clone()).is_err() {
                    failed = true;
                }
            }

            layover.push(t);
        }

        layover
            .into_iter()
            .filter(|t| !t.has_expired())
            .for_each(|t| self.queue.push(t));

        if failed {
            Err(())
        } else {
            Ok(())
        }
    }
}
