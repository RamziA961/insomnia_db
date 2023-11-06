use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::database::shared_state::SharedState;

use super::{
    job::Job,
    scheduling_strategy::{SchedulingStrategy, SchedulingStrategyError},
};
use chrono::{DateTime, TimeZone, Utc};
use tracing::error;

pub(crate) struct ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    task: Box<dyn Job + Send + Sync>,
    scheduling_strategy: SchedulingStrategy<Tz>,
    next_run: DateTime<Utc>,
    expired: bool,
}

impl<Tz> ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    pub fn try_new(
        task: Box<dyn Job + Send + Sync>,
        scheduling_strategy: SchedulingStrategy<Tz>,
    ) -> Result<Self, SchedulingStrategyError> {
        SchedulingStrategy::validate(&scheduling_strategy).map_err(|e| {
            error!(error = %e, "Invalid scheduling strategy.");
            e
        })?;

        let next_run = scheduling_strategy.get_start_at()
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
        self.scheduling_strategy.to_utc().get_start_at() >= &Utc::now()
    }

    pub(crate) fn run(&mut self, state: Arc<SharedState>) -> Result<(), ()> {
        match &self.scheduling_strategy {
            SchedulingStrategy::Once { .. } => self.expired = true,
            SchedulingStrategy::NTimes {
                n,
                start_at,
                interval,
            } => {
                if n == &1 {
                    self.expired = true;
                }

                self.scheduling_strategy = SchedulingStrategy::NTimes {
                    n: n - 1,
                    start_at: start_at.clone() + *interval,
                    interval: *interval,
                }
            }
            SchedulingStrategy::Between {
                start_at,
                end_at,
                interval,
            } => {
                if end_at.clone() - start_at.clone() <= *interval {
                    self.expired = true;
                } else {
                    self.scheduling_strategy = SchedulingStrategy::Between {
                        start_at: start_at.clone() + *interval,
                        end_at: end_at.clone(),
                        interval: *interval,
                    }
                }
            }
            SchedulingStrategy::Indefinite { start_at, interval } => {
               self.scheduling_strategy = SchedulingStrategy::Indefinite {
                    start_at: start_at.clone() + *interval,
                    interval: *interval
               } 
            }
        }

        self.task.run(state)
    }

    /// Consumes a `ScheduledJob` with a generic [`SchedulingStrategy`] and
    /// returns one fixed to the [`Utc`] timezone.
    ///
    /// [Utc]: chrono::Utc
    /// [ScedulingStrategy]: super::scheduling_strategy::ScedulingStrategy
    pub(crate) fn to_utc(self) -> ScheduledJob<Utc> {
        ScheduledJob {
            task: self.task,
            scheduling_strategy: self.scheduling_strategy.to_utc(),
            next_run: self.next_run.clone(),
            expired: self.expired,
        }
    }
}

impl<Tz> PartialOrd for ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.scheduling_strategy
            .partial_cmp(&other.scheduling_strategy)
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

impl<Tz> PartialEq for ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    fn eq(&self, other: &Self) -> bool {
        self.scheduling_strategy.eq(&other.scheduling_strategy)
    }
}

impl<Tz> Eq for ScheduledJob<Tz> where Tz: TimeZone {}

impl<Tz> Debug for ScheduledJob<Tz>
where
    Tz: TimeZone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScheduledJob {{ next_run: {:?}, strategy: {:?}, {} }}",
            self.next_run, self.scheduling_strategy, self.expired
        )
    }
}
