use std::{num::TryFromIntError, str::FromStr};

use crate::util::time::{time_is_past, time_passed};
use chrono::{DateTime, Days, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, trace, warn};

use super::throttle_connection::IpBanReason;

impl ThrottleSimple {

    pub fn inc(&mut self) {
        self.tracker.inc_total();
        self.tracker.inc();
    }

    
}

impl ThrottleRanged {
    pub fn new(range: u64, started_at: DateTime<Utc>) -> Self {
        Self {
            range,
            tracker: ThresholdTracker::new(started_at),
            amount: 0,
        }
    }

    pub fn ban(
        &mut self,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
        ban_reason: IpBanReason,
        ban_until: DateTime<Utc>,
    ) {
        *banned_until = Some((ban_until, ban_reason));
    }

    pub fn unban_on_throttle(
        &mut self,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
    ) {
        *banned_until = None;
    }
    


    pub fn is_banned(
        &mut self,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
        time: &DateTime<Utc>,
    ) -> IsBanned {
        is_banned(banned_until, time)
    }

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
        trace!("throttle: ban status: {:?}", ban_status);

        match ban_status {
            IsBanned::Banned => {
                self.tracker.inc_total();
                return AllowCon::AlreadyBanned;
            }
            IsBanned::UnBanned => {
                self.tracker.reset_threshold(time);
            }
            IsBanned::NotBanned => {}
        }

        trace!(
            "throttle: range {} >= {} = {}",
            { self.amount },
            { self.range },
            self.amount >= self.range
        );
        if self.amount >= self.range {
            self.tracker.inc_total();

            let range_status = !self.tracker.allow(threshold, time);
            trace!("throttle: range allow: {}", range_status);

            if range_status {
                let ban_until = *time + ban_duration;
                self.ban(banned_until, ban_reason, ban_until);
                return AllowCon::Banned((ban_until, ban_reason));
            }

            self.tracker.inc();

            return if ban_status == IsBanned::UnBanned {
                AllowCon::UnbannedAndBlocked
            } else {
                AllowCon::Blocked
            };
        }

        self.amount += 1;
        if ban_status == IsBanned::UnBanned {
            AllowCon::UnbannedAndAllow
        } else {
            AllowCon::Allow
        }
    }
}

impl ThrottleDoubleLayer {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            block_tracker: ThresholdTracker::new(started_at),
            ban_tracker: ThresholdTracker::new(started_at),
        }
    }

    pub fn ban(
        &mut self,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
        ban_reason: IpBanReason,
        ban_until: DateTime<Utc>,
    ) {
        *banned_until = Some((ban_until, ban_reason));
    }

    pub fn is_banned(
        &mut self,
        banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
        time: &DateTime<Utc>,
    ) -> IsBanned {
        is_banned(banned_until, time)
    }

    
}


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


#[derive(Error, Debug)]
pub enum DbThrottleDoubleLayerFromError {
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
    use chrono::DateTime;
    use chrono::{TimeDelta, Utc};
    use tracing::Level;

    use crate::misc::{throttle_connection::IpBanReason, throttle_threshold::AllowCon};

    use super::{Threshold, ThrottleDoubleLayer, ThrottleRanged};


}
