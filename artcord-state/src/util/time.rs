use chrono::{DateTime, TimeDelta, Utc};
use tracing::error;

pub fn time_is_past(time: &DateTime<Utc>) -> bool {
    Utc::now() > *time
}

pub fn time_passed(start: DateTime<Utc>, passed: TimeDelta) -> bool {
    let diff = Utc::now() - start;
    diff > passed
}

pub fn time_passed_days(start: DateTime<Utc>, days_passed: u64) -> bool {
    time_passed(
        start,
        TimeDelta::try_days(days_passed as i64).unwrap_or_else(|| {
            error!("throttle: failed try_weeks, something is wrong");
            TimeDelta::default()
        }),
    )
}
