use std::{cmp::Ordering, sync::Arc};

use crate::database::shared_state::SharedState;

use super::{
    job::Job,
    scheduling_strategy::{SchedulingStrategy, SchedulingStrategyError},
};
use chrono::{DateTime, TimeZone, Utc};

pub(crate) struct ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    task: Box<dyn Job>,
    scheduling_strategy: SchedulingStrategy<Tz>,
    next_run: DateTime<Utc>,
    expired: bool,
}

impl<Tz> ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    pub fn try_new(
        task: Box<dyn Job>,
        scheduling_strategy: SchedulingStrategy<Tz>,
    ) -> Result<Self, SchedulingStrategyError> {
        SchedulingStrategy::validate(&scheduling_strategy)?;

        let next_run = (match scheduling_strategy {
            SchedulingStrategy::Once { start_at: at } => at,
            SchedulingStrategy::NTimes { start_at, .. } => start_at,
            SchedulingStrategy::Between { start_at, .. } => start_at,
        })
        .naive_utc()
        .and_utc();

        Ok(Self {
            task,
            scheduling_strategy,
            next_run,
            expired: false,
        })
    }

    pub(crate) fn has_expired(&self) -> bool {
        self.expired
    }

    pub(crate) fn is_due(&self) -> bool {
        let start = match self.scheduling_strategy {
            SchedulingStrategy::Once { start_at: at } => at,
            SchedulingStrategy::NTimes { start_at, .. } => start_at,
            SchedulingStrategy::Between { start_at, .. } => start_at,
        };

        start >= Utc::now()
    }

    pub(crate) fn run_job(&mut self, state: Arc<SharedState>) -> Result<(), ()> {
        match self.scheduling_strategy {
            SchedulingStrategy::Once { .. } => self.expired = true,
            SchedulingStrategy::NTimes {
                n,
                start_at,
                interval,
            } => {
                if n == 1 {
                    self.expired = true;
                }

                self.scheduling_strategy = SchedulingStrategy::NTimes {
                    n: n - 1,
                    start_at: start_at + interval,
                    interval,
                }
            }
            SchedulingStrategy::Between {
                start_at,
                end_at,
                interval,
            } => {
                if end_at - start_at <= interval {
                    self.expired = true;
                } else {
                    self.scheduling_strategy = SchedulingStrategy::Between {
                        start_at: start_at + interval,
                        end_at,
                        interval,
                    }
                }
            }
        }

        self.task.run(state)
    }
}

impl<Tz> Ord for ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.scheduling_strategy.cmp(&other.scheduling_strategy)
    }
}
