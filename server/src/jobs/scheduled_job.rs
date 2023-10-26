use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::database::shared_state::SharedState;

use super::job::Job;
use chrono::{DateTime, Duration, TimeZone, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum JobBuilderError {
    #[error("Non-positive duration frequency: {0}")]
    NonPositiveFrequency(Duration),

    #[error("Invalid end date: {{ start: {0}, end: {1} }}")]
    InvalidEndDate(DateTime<Utc>, DateTime<Utc>),

    #[error("Non positive runs: {0}")]
    NonPositiveRuns(u64),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

pub(crate) struct ScheduledJob {
    task: Box<dyn Job>, // can sub with job task
    start_dt: DateTime<Utc>,
    end_dt: Option<DateTime<Utc>>,
    next_run: DateTime<Utc>,
    interval: Option<Duration>,
    n_runs: Option<u64>,
    expired: bool
}

impl ScheduledJob {
    pub(crate) fn has_expired(&self) -> bool {
        self.expired
    }

    pub(crate) fn is_due(&self) -> bool {
        self.next_run >= Utc::now()
    }

    pub(crate) fn reschedule(&mut self) {
        self.n_runs = self.n_runs.map(|v| {
            if v == 1 {
                self.expired = true;
            }
            v - 1
        });
        
        self.next_run = if let Some(i) = self.interval {
            self.next_run + i
        } else {
            self.expired = true;
            self.next_run
        };

        self.expired = if !self.expired {
            self.end_dt.map_or_else(|| false, |dt| dt > Utc::now())
        } else {
            self.expired
        }
    }

    pub(crate) fn run(&mut self, state: Arc<SharedState>) -> Result<(), ()> {
        self.reschedule();
        self.task.run(state)
    }
}

impl PartialEq for ScheduledJob {
    fn eq(&self, other: &Self) -> bool {
        self.start_dt == other.start_dt
    }
}

impl Eq for ScheduledJob {}

impl PartialOrd for ScheduledJob {
    // todo: double check
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.start_dt > other.start_dt {
            return Some(Ordering::Less);
        }

        if self.start_dt < other.start_dt {
            return Some(Ordering::Greater);
        }

        let ref s_freq = self.interval;
        let ref o_freq = other.interval;

        if s_freq.is_some() && o_freq.is_some() {
            if s_freq < o_freq {
                return Some(Ordering::Greater);
            }

            if s_freq > o_freq {
                return Some(Ordering::Less);
            }
        }

        Some(Ordering::Equal)
    }
}

impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Debug for ScheduledJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScheduledJob {{ f: fn, start: {:?}, end: {:?}, freq: {:?}, n_runs: {:?}, next_run: {:?} }}",
            self.start_dt,
            self.end_dt,
            self.interval,
            self.n_runs,
            self.next_run,
        )
    }
}

pub(crate) struct JobBuilder {
    task: Box<dyn Job>,
    start_dt: Option<DateTime<Utc>>,
    end_dt: Option<DateTime<Utc>>,
    interval: Option<Duration>,
    n_runs: Option<u64>,
}

impl JobBuilder {
    pub(crate) fn new(f: Box<dyn Job>) -> Self {
        Self {
            task: f,
            start_dt: None,
            end_dt: None,
            interval: None,
            n_runs: None,
        }
    }

    pub(crate) fn with_start_dt<Tz>(mut self, start_dt: &DateTime<Tz>) -> Self
    where
        Tz: TimeZone,
    {
        self.start_dt = Some(start_dt.with_timezone(&Utc));
        self
    }

    pub(crate) fn with_end_dt<Tz>(mut self, end_dt: &DateTime<Tz>) -> Self
    where
        Tz: TimeZone,
    {
        self.end_dt = Some(end_dt.with_timezone(&Utc));
        self
    }

    pub(crate) fn with_interval(mut self, interval: &Duration) -> Self {
        self.interval = Some(interval.clone());
        self
    }

    pub(crate) fn with_n_runs(mut self, n: u64) -> Self {
        self.n_runs = Some(n);
        self
    }

    fn is_valid(&self) -> Result<(), JobBuilderError> {
        let start_dt = if self.start_dt.is_some() {
            self.start_dt
        } else {
            Some(Utc::now())
        };

        if self.end_dt.is_some() && start_dt >= self.end_dt {
            return Err(JobBuilderError::InvalidEndDate(
                start_dt.clone().unwrap(),
                self.end_dt.clone().unwrap(),
            ));
        }

        if self.n_runs.is_some_and(|v| v == 0) {
            return Err(JobBuilderError::NonPositiveRuns(0));
        }

        if self.interval.is_none() && self.n_runs.is_some_and(|n| n != 1) {
            return Err(JobBuilderError::InvalidConfiguration(
                "Interval is not declared and number of runs not equal to 1.".to_string(),
            ));
        }

        if self
            .interval
            .is_some_and(|freq| freq.is_zero() || freq.to_std().is_err())
        {
            return Err(JobBuilderError::NonPositiveFrequency(
                self.interval.clone().unwrap(),
            ));
        }

        Ok(())
    }

    // pub(crate) fn build_consume(mut self) -> Result<ScheduledJob, JobBuilderError> {
    //     self.is_valid()
    //         .map(|_|
    //             ScheduledJob {
    //                 task: self.task,
    //                 start_dt: self.start_dt.unwrap(),
    //                 end_dt: self.end_dt,
    //                 freq: self.freq,
    //                 n_runs: self.n_runs,
    //             }
    //         )
    // }

    pub(crate) fn build(self) -> Result<ScheduledJob, JobBuilderError> {
        let start = self.start_dt.unwrap();
        self.is_valid().map(|_| ScheduledJob {
            task: self.task,
            start_dt: start,
            end_dt: self.end_dt,
            interval: self.interval,
            n_runs: self.n_runs,
            next_run: start,
            expired: false
        })
    }
}
