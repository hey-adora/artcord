

use chrono::{DateTime, TimeDelta, Utc};
use tracing::error;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
// pub struct TimeMachine {

// }

// impl TimeMachine {
//     pub fn time() {
//         Date
//     }
// }

#[derive(Clone, Debug)]
pub struct Clock {
    time_machine: mpsc::Sender<oneshot::Sender<DateTime<Utc>>>
}

impl Clock {
    pub async fn time(&self) -> Result<DateTime<Utc>, oneshot::error::RecvError> {
        let (time_tx, time_rx) = oneshot::channel::<DateTime<Utc>>();
        self.time_machine.send(time_tx);
        time_rx.await
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
