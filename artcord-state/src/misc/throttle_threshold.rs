use std::{num::TryFromIntError, str::FromStr};

use crate::util::time::{time_is_past, time_passed};
use chrono::{DateTime, Days, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, trace, warn};

use super::throttle_connection::IpBanReason;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct DbThrottleDoubleLayer {
    //pub banned_until: Option<i64>,
    //pub banned_reason: Option<String>,
    pub block_tracker: DbThresholdTracker,
    pub ban_tracker: DbThresholdTracker,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct DbThresholdTracker {
    pub total_amount: i64,
    pub amount: i64,
    pub started_at: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ThrottleSimple {
    //pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    pub tracker: ThresholdTracker,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ThrottleRanged {
    pub range: u64,
    pub amount: u64,
    //pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    pub tracker: ThresholdTracker,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ThrottleDoubleLayer {
    //pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    pub block_tracker: ThresholdTracker,
    pub ban_tracker: ThresholdTracker,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ThresholdTracker {
    pub total_amount: u64,
    pub amount: u64,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Threshold {
    pub amount: u64,
    pub delta: TimeDelta,
    pub rate_sec: u64,
    // pub ban_reason: IpBanReason,
    // pub ban_duration: TimeDelta,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum AllowCon {
    Allow,
    Blocked,
    AlreadyBanned,
    Banned((DateTime<Utc>, IpBanReason)),
    Unbanned,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum IsBanned {
    Banned,
    NotBanned,
    UnBanned,
}

impl TryFrom<ThresholdTracker> for DbThresholdTracker {
    type Error = ThresholdTrackerFromError;
    fn try_from(value: ThresholdTracker) -> Result<Self, Self::Error> {
        Ok(Self {
            total_amount: i64::try_from(value.total_amount)?,
            amount: i64::try_from(value.amount)?,
            started_at: value.started_at.timestamp_millis(),
        })
    }
}

impl TryFrom<DbThresholdTracker> for ThresholdTracker {
    type Error = DbThresholdTrackerFromError;
    fn try_from(value: DbThresholdTracker) -> Result<Self, Self::Error> {
        Ok(Self {
            total_amount: u64::try_from(value.total_amount)?,
            amount: u64::try_from(value.amount)?,
            started_at: DateTime::<Utc>::from_timestamp_millis(value.started_at)
                .ok_or(DbThresholdTrackerFromError::InvalidDate(value.started_at))?,
        })
    }
}

impl TryFrom<ThrottleDoubleLayer> for DbThrottleDoubleLayer {
    type Error = ThrottleDoubleLayerFromError;
    fn try_from(value: ThrottleDoubleLayer) -> Result<Self, Self::Error> {
        // let (banned_until, banned_reason) = value
        //     .banned_until
        //     .map(|(a, b)| {
        //         let b: &'static str = b.into();
        //         (Some(a.timestamp_millis()), Some(b.to_string()))
        //     })
        //     .unwrap_or((None, None));
        Ok(Self {
       //     banned_until,
        //    banned_reason,
            ban_tracker: value.ban_tracker.try_into()?,
            block_tracker: value.block_tracker.try_into()?,
        })
    }
}

impl TryFrom<DbThrottleDoubleLayer> for ThrottleDoubleLayer {
    type Error = DbThrottleDoubleLayerFromError;
    fn try_from(value: DbThrottleDoubleLayer) -> Result<Self, Self::Error> {
        // if value.banned_reason.is_some() && value.banned_until.is_none() {
        //     return Err(DbThrottleDoubleLayerFromError::MissingBannedDate);
        // }

        // if value.banned_reason.is_none() && value.banned_until.is_some() {
        //     return Err(DbThrottleDoubleLayerFromError::MissingBannedReason);
        // }

        // let banned_until = if let (Some(a), Some(b)) = (value.banned_until, value.banned_reason) {
        //     Some((
        //         DateTime::<Utc>::from_timestamp_millis(a)
        //             .ok_or(DbThrottleDoubleLayerFromError::InvalidDate(a))?,
        //         IpBanReason::from_str(&b)?,
        //     ))
        // } else {
        //     None
        // };

        Ok(Self {
            //banned_until,
            ban_tracker: value.ban_tracker.try_into()?,
            block_tracker: value.block_tracker.try_into()?,
        })
    }
}

// impl TryFrom<ThrottleDoubleLayer> for DbThrottleDoubleLayer {
//     type Error = FromDbThrottleError;
//     fn try_from(value: DbThresholdTracker) -> Result<Self, Self::Error> {
//         Ok(Self {
//             total_amount: value.total_amount,
//             amount: value.amount,
//             started_at: DateTime::<Utc>::from_timestamp_millis(value.started_at)
//                 .ok_or(FromDbThrottleError::InvalidDate(value.started_at))?,
//         })
//     }
// }

impl ThrottleSimple {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            //banned_until: None,
            tracker: ThresholdTracker::new(started_at),
        }
    }

    pub fn is_banned(&mut self, banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>, time: &DateTime<Utc>) -> IsBanned {
        is_banned(banned_until, time)
    }

    pub fn inc(&mut self) {
        self.tracker.inc_total();
        self.tracker.inc();
    }

    pub fn allow(&mut self, treshold: &Threshold, ban_duration: &TimeDelta, ban_reason: &IpBanReason, time: &DateTime<Utc>, banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>) -> AllowCon {
        match self.is_banned(banned_until, time) {
            IsBanned::Banned => {
                return AllowCon::AlreadyBanned;
            }
            IsBanned::UnBanned => {
                self.tracker.reset_threshold(time);
                return AllowCon::Unbanned;
            }
            _ => {}
        }
        let allow = self.tracker.allow(treshold, time);
        if !allow {
            let ban = (*time + *ban_duration, *ban_reason);
            *banned_until = Some(ban);
            return AllowCon::Banned(ban);

        }

        AllowCon::Allow
    }
}

impl ThrottleRanged {
    pub fn new(range: u64, started_at: DateTime<Utc>) -> Self {
        Self {
            range,
            tracker: ThresholdTracker::new(started_at),
            //banned_until: None,
            amount: 0,
        }
    }

    pub fn ban(&mut self, banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>, ban_reason: IpBanReason, ban_until: DateTime<Utc>) {
        *banned_until = Some((ban_until, ban_reason));
    }

    pub fn is_banned(&mut self, banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>, time: &DateTime<Utc>) -> IsBanned {
        is_banned(banned_until, time)
    }

    // pub fn inc(&mut self) {
    //     if self.range > self.amount {
    //         self.amount += 1;
    //     }
    // }

    pub fn dec(&mut self) {
        self.amount = self.amount.saturating_sub(1);
    }

    pub fn inc(
        &mut self,
        threshold: &Threshold,
        ban_reason: IpBanReason,
        ban_duration: TimeDelta,
        time: &DateTime<Utc>,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
    ) -> AllowCon {
        let ban_status = self.is_banned(banned_until, time);
        match ban_status {
            IsBanned::Banned => {
                self.tracker.inc_total();
                return AllowCon::AlreadyBanned;
            }
            IsBanned::UnBanned => {
                self.tracker.reset_threshold(time);
                return AllowCon::Unbanned;
            }
            IsBanned::NotBanned => {}
        }

        if self.amount >= self.range {
            self.tracker.inc_total();

            if !self.tracker.allow(threshold, time) {
                let ban_until = *time + ban_duration;
                self.ban(banned_until, ban_reason, ban_until);
                return AllowCon::Banned((ban_until, ban_reason));
                // return self.banned_until
                //     .map(|date| AllowCon::Banned((date, ban_reason)))
                //     .unwrap_or(AllowCon::AlreadyBanned);
            }

            self.tracker.inc();

            return AllowCon::Blocked;
        }

        self.amount += 1;
        AllowCon::Allow
    }
}

impl ThrottleDoubleLayer {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
          //  banned_until: None,
            block_tracker: ThresholdTracker::new(started_at),
            ban_tracker: ThresholdTracker::new(started_at),
        }
    }

    pub fn ban(&mut self, banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>, ban_reason: IpBanReason, ban_until: DateTime<Utc>) {
        *banned_until = Some((ban_until, ban_reason));
    }

    pub fn is_banned(&mut self, banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>, time: &DateTime<Utc>) -> IsBanned {
        is_banned(banned_until, time)
    }

    pub fn allow(
        &mut self,
        block_threshold: &Threshold,
        ban_threshold: &Threshold,
        ban_reason: IpBanReason,
        ban_duration: &TimeDelta,
        time: &DateTime<Utc>,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
    ) -> AllowCon {
        let ban_status = self.is_banned(banned_until, time);
        match ban_status {
            IsBanned::Banned => {
                self.ban_tracker.inc_total();
                return AllowCon::AlreadyBanned;
            }
            IsBanned::UnBanned => {
                self.ban_tracker.reset_threshold(time);
                self.block_tracker.reset_threshold(time);
                return AllowCon::Unbanned;
            }
            IsBanned::NotBanned => {}
        }

        if !self.ban_tracker.allow(ban_threshold, time) {
            self.ban_tracker.inc_total();
            let ban_until = *time + *ban_duration;
            self.ban(banned_until, ban_reason, ban_until);
            return AllowCon::Banned((ban_until, ban_reason));
            // return self
            //     .ban(ban_reason, ban_duration, time)
            //     .map(|date| AllowCon::Banned((date, ban_reason)))
            //     .unwrap_or(AllowCon::AlreadyBanned);
        }

        if !self.block_tracker.allow(block_threshold, time) {
            self.ban_tracker.inc_total();
            self.ban_tracker.inc();
            return AllowCon::Blocked;
        } else {
            self.block_tracker.inc_total();
            self.block_tracker.inc();
        }

        AllowCon::Allow
    }
}

pub fn is_banned(
    banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
    time: &DateTime<Utc>,
) -> IsBanned {
    let Some((date, _)) = banned_until else {
        trace!("throttle: ban check: entry doesnt exist");
        return IsBanned::NotBanned;
    };

    let un_banned = time >= date;

    trace!(
        "throttle: is banned: {}, state: {:#?}",
        !un_banned,
        banned_until
    );

    if un_banned {
        *banned_until = None;
        return IsBanned::UnBanned;
    }
    IsBanned::Banned
}

// pub fn ban(
//     banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
//     ban_reason: IpBanReason,
//     ban_duration: TimeDelta,
//     time: DateTime<Utc>,
// ) -> Option<DateTime<Utc>> {
//     trace!("throttle - banned: {:?}", ban_reason);
//     let Some(date) = time.checked_add_signed(ban_duration) else {
//         error!(
//             "throtte: failed to ban, failed to add {:?} to time",
//             ban_duration
//         );
//         return None;
//     };
//     *banned_until = Some((date.clone(), ban_reason));
//     Some(date)
// }

impl ThresholdTracker {
    pub fn new(started_at: DateTime<Utc>) -> ThresholdTracker {
        Self {
            total_amount: 0,
            amount: 0,
            started_at,
        }
    }

    pub fn inc_total(&mut self) {
        trace!(
            "throttle - inc from: {} {} to {} {}",
            self.total_amount,
            self.amount,
            self.total_amount + 1,
            self.amount
        );
        self.total_amount += 1;
    }

    pub fn inc(&mut self) {
        trace!(
            "throttle - inc from: {} {} to {} {}",
            self.total_amount,
            self.amount,
            self.total_amount,
            self.amount + 1
        );
        self.amount += 1;
    }

    pub fn threshold_reatched(&self, threshold: &Threshold) -> bool {
        self.amount >= threshold.amount
    }

    pub fn delta_passed(&self, threshold: &Threshold, time: &DateTime<Utc>) -> bool {
        (*time - self.started_at) >= threshold.delta
    }

    pub fn reset_threshold(&mut self, time: &DateTime<Utc>) {
        self.started_at = *time;
        self.amount = 0;
    }

    pub fn allow(&mut self, threshold: &Threshold, time: &DateTime<Utc>) -> bool {
        if self.threshold_reatched(threshold) {
            if self.delta_passed(threshold, time) {
                self.reset_threshold(time);
            } else {
                return false;
            }
        } else {
            if self.delta_passed(threshold, time) {
                self.reset_threshold(time);
            }
        }
        true
    }
}

impl Threshold {
    pub const fn new(amount: u64, delta: TimeDelta) -> Self {
        Self {
            amount,
            delta,
            rate_sec: amount / delta.num_seconds() as u64,
        }
    }

    pub const fn new_const(amount: u64, delta: Option<TimeDelta>) -> Self {
        let delta = match delta {
            Some(delta) => delta,
            None => panic!("failed to create delta"),
        };

        Self {
            amount,
            delta,
            rate_sec: amount / delta.num_seconds() as u64,
        }
    }

    // pub fn threshold
}

#[derive(Error, Debug)]
pub enum DbThrottleDoubleLayerFromError {
    // trum::ParseError
    #[error("db missing ban date")]
    MissingBannedDate,

    #[error("db missing ban reason")]
    MissingBannedReason,

    #[error("invalid date: {0}")]
    InvalidDate(i64),

    #[error("invalid date: {0}")]
    InvalidReason(#[from] strum::ParseError),

    #[error("tracker error: {0}")]
    TrackerError(#[from] DbThresholdTrackerFromError),
}

#[derive(Error, Debug)]
pub enum ThrottleDoubleLayerFromError {
    #[error("tracker error: {0}")]
    TrackerError(#[from] ThresholdTrackerFromError),
}

#[derive(Error, Debug)]
pub enum ThresholdTrackerFromError {
    #[error("Failed to convert int: {0}")]
    TryFromIntError(#[from] TryFromIntError),
}

#[derive(Error, Debug)]
pub enum DbThresholdTrackerFromError {
    #[error("Failed to convert int: {0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("invalid date: {0}")]
    InvalidDate(i64),
}

#[cfg(test)]
mod throttle_tests {
    use chrono::{TimeDelta, Utc};
    use tracing::Level;
    use chrono::DateTime;

    use crate::misc::{throttle_connection::IpBanReason, throttle_threshold::AllowCon};

    use super::{Threshold, ThrottleDoubleLayer, ThrottleRanged};

    #[test]
    fn throttle_ranged_test() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .try_init();

        let started_at = Utc::now();
        let now = Utc::now();
        let ban_reason = IpBanReason::WsTooManyReconnections;
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        let threshold = Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let mut banned_until: Option<(DateTime<Utc>, IpBanReason)> = None;
        
        let mut throttle = ThrottleRanged::new(10, started_at);

        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (AllowCon::Allow, 1, 0, 0)
        );

        for _ in 0..8 {
            let _ = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);
        }

        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (AllowCon::Allow, 10, 0, 0)
        );

        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (AllowCon::Blocked, 10, 1, 1)
        );

        for _ in 0..9 {
            let _ = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);
        }

        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (
                AllowCon::Banned((
                    now.checked_add_signed(ban_duration).unwrap(),
                    IpBanReason::WsTooManyReconnections
                )),
                10,
                11,
                10
            )
        );

        let now = now.checked_add_signed(ban_duration).unwrap();
        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (AllowCon::Unbanned, 10, 11, 0,)
        );

        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (AllowCon::Blocked, 10, 12, 1)
        );

        throttle.dec();

        let result = throttle.inc(&threshold, ban_reason, ban_duration, &now, &mut banned_until);

        assert_eq!(
            (
                result,
                throttle.amount,
                throttle.tracker.total_amount,
                throttle.tracker.amount,
            ),
            (AllowCon::Allow, 10, 12, 1)
        );
    }

    #[test]
    fn throttle_double_layer_test() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .try_init();

        let started_at = Utc::now();
        let now = Utc::now();
        let ban_reason = IpBanReason::WsTooManyReconnections;
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        let block_threshold = Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let ban_threshold = Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let mut banned_until: Option<(DateTime<Utc>, IpBanReason)> = None;

        let mut throttle = ThrottleDoubleLayer::new(started_at);
        let result = throttle.allow(
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &now,
            &mut banned_until,
        );

        assert_eq!(result, AllowCon::Allow);

        for _ in 0..15 {
            let _ = throttle.allow(
                &block_threshold,
                &ban_threshold,
                ban_reason,
                &ban_duration,
                &now,
                &mut banned_until,
            );
        }

        let result = throttle.allow(
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &now,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                throttle.block_tracker.total_amount,
                throttle.block_tracker.amount,
                throttle.ban_tracker.total_amount,
                throttle.ban_tracker.amount
            ),
            (AllowCon::Blocked, 10, 10, 7, 7)
        );

        for _ in 0..3 {
            let _ = throttle.allow(
                &block_threshold,
                &ban_threshold,
                ban_reason,
                &ban_duration,
                &now,
                &mut banned_until,
            );
        }

        let result = throttle.allow(
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &now,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                throttle.block_tracker.total_amount,
                throttle.block_tracker.amount,
                throttle.ban_tracker.total_amount,
                throttle.ban_tracker.amount
            ),
            (
                AllowCon::Banned((
                    now.checked_add_signed(ban_duration).unwrap(),
                    IpBanReason::WsTooManyReconnections
                )),
                10,
                10,
                11,
                10
            )
        );

        let result = throttle.allow(
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &now,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                throttle.block_tracker.total_amount,
                throttle.block_tracker.amount,
                throttle.ban_tracker.total_amount,
                throttle.ban_tracker.amount
            ),
            (AllowCon::AlreadyBanned, 10, 10, 12, 10)
        );

        let now = now.checked_add_signed(ban_duration).unwrap();
        let result = throttle.allow(
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &now,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                throttle.block_tracker.total_amount,
                throttle.block_tracker.amount,
                throttle.ban_tracker.total_amount,
                throttle.ban_tracker.amount
            ),
            (AllowCon::Unbanned, 10, 0, 12, 0)
        );

        let result = throttle.allow(
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &now,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                throttle.block_tracker.total_amount,
                throttle.block_tracker.amount,
                throttle.ban_tracker.total_amount,
                throttle.ban_tracker.amount
            ),
            (AllowCon::Allow, 11, 1, 12, 0)
        );
    }
}
