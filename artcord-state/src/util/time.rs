

use chrono::{DateTime, TimeDelta, Utc};
use tracing::error;
use thiserror::Error;
// pub struct TimeMachine {

// }

// impl TimeMachine {
//     pub fn time() {
//         Date
//     }
// }



// impl TestClock {
//     pub async fn time(&self) -> Result<DateTime<Utc>, ClockErr> {
//         let (time_tx, time_rx) = oneshot::channel::<DateTime<Utc>>();
//         self.time_machine.send(time_tx).await?;
//         Ok(time_rx.await?)
//     }
// }

#[derive(Clone, Debug)]
pub struct Clock;

impl Default for Clock {
    fn default() -> Self {
        Self
    }
}

impl Clock {
    pub fn new() -> Self {
        Self
    }
}

pub trait TimeMiddleware {
    fn get_time(&self) -> impl std::future::Future<Output = DateTime<Utc>> + Send;
}


impl TimeMiddleware for Clock {
    async fn get_time(&self) -> DateTime<Utc> {
        Utc::now()
    }
}



pub fn time_is_past(time: &DateTime<Utc>) -> bool {
    Utc::now() >= *time
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
