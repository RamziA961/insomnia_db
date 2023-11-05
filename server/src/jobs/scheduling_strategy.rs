use std::{cmp::Ordering, fmt::Debug};

use chrono::{DateTime, Duration, TimeZone, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum SchedulingStrategyError {
    #[error("Non-positive duration frequency: {0}")]
    NonPositiveInterval(Duration, String),

    #[error("Invalid end date: {{ start: {0}, end: {1} }}. {2}")]
    InvalidEndDate(DateTime<Utc>, DateTime<Utc>, String),

    #[error("Invalid start date: {0}. {1}")]
    InvalidStartDate(DateTime<Utc>, String),

    #[error("Non-positive runs: {0}. {1}.")]
    InvalidNumberOfRuns(u64, String),
}

pub(crate) enum SchedulingStrategy<Tz>
where
    Tz: TimeZone,
{
    Once {
        start_at: DateTime<Tz>,
    },
    NTimes {
        n: u64,
        start_at: DateTime<Tz>,
        interval: Duration,
    },
    Between {
        start_at: DateTime<Tz>,
        end_at: DateTime<Tz>,
        interval: Duration,
    },
}

impl<Tz> SchedulingStrategy<Tz>
where
    Tz: TimeZone,
{
    pub(crate) fn validate(&self) -> Result<(), SchedulingStrategyError> {
        match self {
            Self::Once { start_at } => SchedulingStrategy::validate_start_date(start_at),
            Self::NTimes {
                n,
                start_at,
                interval,
            } => {
                if n <= &1 {
                    return Err(SchedulingStrategyError::InvalidNumberOfRuns(
                        n.clone(),
                        "Number of runs must be greater than 1.".to_string(),
                    ));
                }

                SchedulingStrategy::<Tz>::validate_invterval(interval)?;
                SchedulingStrategy::validate_start_date(start_at)
            }
            Self::Between {
                start_at,
                end_at,
                interval,
            } => {
                SchedulingStrategy::<Tz>::validate_invterval(interval)?;
                SchedulingStrategy::validate_start_date(start_at)?;
                SchedulingStrategy::validate_end_date(start_at, end_at)
            }
        }
    }

    fn validate_start_date(start: &DateTime<Tz>) -> Result<(), SchedulingStrategyError> {
        let utc = start.naive_utc().and_utc();
        let cutoff = Utc::now() - Duration::milliseconds(10);

        (cutoff.signed_duration_since(utc) >= Duration::zero())
            .then(|| ())
            .ok_or_else(|| {
                SchedulingStrategyError::InvalidStartDate(
                    utc,
                    format!("Given start date exceeds threshold for past datetimes. The cutoff datetime is {cutoff}.")
                )
            })
    }

    fn validate_end_date(
        start: &DateTime<Tz>,
        end: &DateTime<Tz>,
    ) -> Result<(), SchedulingStrategyError> {
        let start = start.naive_utc().and_utc();
        let end = end.naive_utc().and_utc();
        (end.signed_duration_since(start).num_milliseconds() > 0)
            .then(|| ())
            .ok_or_else(|| {
                SchedulingStrategyError::InvalidEndDate(
                    start.clone(),
                    end.clone(),
                    "End date is less than or equal to start date.".to_string(),
                )
            })
    }

    fn validate_invterval(interval: &Duration) -> Result<(), SchedulingStrategyError> {
        interval.is_zero().then(|| ()).ok_or_else(|| {
            SchedulingStrategyError::NonPositiveInterval(
                interval.clone(),
                "Interval must be greater than 0ms.".to_string(),
            )
        })
    }

    fn get_start_at(&self) -> &DateTime<Tz> {
        match self {
            SchedulingStrategy::Once { start_at } => start_at,
            SchedulingStrategy::NTimes { start_at, .. } => start_at,
            SchedulingStrategy::Between { start_at, .. } => start_at,
        }
    }

    pub(crate) fn to_utc(&self) -> SchedulingStrategy<Utc> {
        match self {
            SchedulingStrategy::Once { start_at } => SchedulingStrategy::Once {
                start_at: start_at.naive_utc().and_utc(),
            },
            SchedulingStrategy::NTimes {
                n,
                start_at,
                interval,
            } => SchedulingStrategy::NTimes {
                start_at: start_at.naive_utc().and_utc(),
                n: n.clone(),
                interval: interval.clone(),
            },
            SchedulingStrategy::Between {
                start_at,
                end_at,
                interval,
            } => SchedulingStrategy::Between {
                start_at: start_at.naive_utc().and_utc(),
                end_at: end_at.naive_utc().and_utc(),
                interval: interval.clone(),
            },
        }
    }
}

impl<Tz> PartialOrd for SchedulingStrategy<Tz>
where
    Tz: TimeZone,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let diff = self
            .get_start_at()
            .signed_duration_since(other.get_start_at());

        // times further into the future have a lower priority
        match diff.cmp(&Duration::zero()) {
            Ordering::Less => Some(Ordering::Greater),
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Greater => Some(Ordering::Less),
        }
    }
}

impl<Tz> Ord for SchedulingStrategy<Tz>
where
    Tz: TimeZone,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<Tz> PartialEq for SchedulingStrategy<Tz>
where
    Tz: TimeZone,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_start_at().eq(other.get_start_at())
    }
}

impl<Tz> Eq for SchedulingStrategy<Tz> where Tz: TimeZone {}

impl<Tz> Debug for SchedulingStrategy<Tz>
where
    Tz: TimeZone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SchedulingStrategy::Once { start_at } => format!("Once {{ start_at: {start_at:?} }}"),
            SchedulingStrategy::NTimes {
                n,
                start_at,
                interval,
            } => format!("NTimes {{ n: {n}, start_at: {start_at:?}, interval: {interval} }}"),
            SchedulingStrategy::Between {
                start_at,
                end_at,
                interval,
            } => format!(
                "Between {{ start_at: {start_at:?}, end_at: {end_at:?}, interval: {interval} }}"
            ),
        };

        write!(f, "{}", s)
    }
}
