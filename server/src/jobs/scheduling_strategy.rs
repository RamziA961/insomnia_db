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
        at: DateTime<Tz>,
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
            Self::Once { at } => SchedulingStrategy::validate_start_date(at),
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
}
