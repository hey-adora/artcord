use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops::Div;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use chrono::DateTime;
use chrono::Days;
use chrono::Month;
use chrono::Months;
use chrono::TimeDelta;
use chrono::Utc;
use thiserror::Error;

use tracing::debug;
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};

use crate::global;
use crate::global::AllowCon;
use crate::global::IsBanned;

pub fn ws_ip_throttle(
    con_flicker_tracker: &mut global::ThresholdTracker,
    con_count_tracker: &mut global::ThresholdTracker,
    current_con_count: &mut u64,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
    ws_threshold: &global::DefaultThreshold,
    time: &DateTime<Utc>,
) -> AllowCon {
 
    let flicker_throttle_allow = simple_throttle(
        con_flicker_tracker,
        &ws_threshold.ws_con_flicker_threshold,
        &ws_threshold.ws_con_flicker_ban_duration,
        &ws_threshold.ws_con_flicker_ban_reason,
        time,
        banned_until,
    );

    trace!(
        "ws_ip_throttle: flicker throttle result: {:?}",
        flicker_throttle_allow
    );

    if matches!(
        flicker_throttle_allow,
        AllowCon::Banned(_) | AllowCon::AlreadyBanned | AllowCon::Blocked
    ) {
        return flicker_throttle_allow;
    }

    let ranged_throttle_allow = ranged_throttle(
        &ws_threshold.ws_max_con_threshold_range,
        current_con_count,
        con_count_tracker,
        &ws_threshold.ws_max_con_threshold,
        &ws_threshold.ws_max_con_ban_reason,
        &ws_threshold.ws_max_con_ban_duration,
        time,
        banned_until,
    );
    trace!("ws_ip_throttle: ranged throttle result: {:?}", ranged_throttle_allow);

    match ranged_throttle_allow {
        AllowCon::Allow => {
            con_flicker_tracker.amount += 1;
            debug!("flicker incremented to: {}", con_flicker_tracker.amount);
            if flicker_throttle_allow == AllowCon::UnbannedAndAllow {
                flicker_throttle_allow
            } else {
                ranged_throttle_allow
            }
        }
        AllowCon::Blocked => {
            if flicker_throttle_allow == AllowCon::UnbannedAndAllow {
                AllowCon::UnbannedAndBlocked
            } else {
                ranged_throttle_allow
            }
        }
        _ => ranged_throttle_allow,
    }
}

pub fn double_throttle(
    block_tracker: &mut global::ThresholdTracker,
    ban_tracker: &mut global::ThresholdTracker,
    block_threshold: &global::Threshold,
    ban_threshold: &global::Threshold,
    ban_reason: &global::IpBanReason,
    ban_duration: &TimeDelta,
    time: &DateTime<Utc>,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
) -> AllowCon {
    let ban_status = is_banned(banned_until, time);
    match ban_status {
        IsBanned::Banned => {
            return AllowCon::AlreadyBanned;
        }
        IsBanned::UnBanned => {
            ban_tracker.started_at = *time;
            ban_tracker.amount = 0;

            block_tracker.started_at = *time;
            block_tracker.amount = 0;
        }
        IsBanned::NotBanned => {}
    }

    if !threshold_allow(ban_tracker, ban_threshold, time) {
        let ban_until = *time + *ban_duration;
        *banned_until = Some((ban_until, ban_reason.clone()));
        return AllowCon::Banned((ban_until, ban_reason.clone()));
    }

    if !threshold_allow(block_tracker, block_threshold, time) {
        ban_tracker.amount += 1;
        return if ban_status == IsBanned::UnBanned {
            AllowCon::UnbannedAndBlocked
        } else {
            AllowCon::Blocked
        };
    } else {
        block_tracker.amount += 1;
    }

    if ban_status == IsBanned::UnBanned {
        AllowCon::UnbannedAndAllow
    } else {
        AllowCon::Allow
    }
}

pub fn ranged_throttle(
    max: &u64,
    current: &mut u64,
    tracker: &mut global::ThresholdTracker,
    threshold: &global::Threshold,
    ban_reason: &global::IpBanReason,
    ban_duration: &TimeDelta,
    time: &DateTime<Utc>,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
) -> AllowCon {
    let ban_status = is_banned(banned_until, time);
    trace!("ranged throttle: ban status: {:?}", ban_status);

    match ban_status {
        IsBanned::Banned => {
            return AllowCon::AlreadyBanned;
        }
        IsBanned::UnBanned => {
            tracker.started_at = *time;
            tracker.amount = 0;
        }
        IsBanned::NotBanned => {}
    }

    trace!(
        "ranged throttle: {} >= {} = {}",
        current,
        max,
        *current >= *max
    );
    if *current >= *max {
        let allow = threshold_allow(tracker, threshold, time);
        //let range_status = !self.tracker.allow(threshold, time);
        trace!("ranged throttle: allow: {}", allow);

        if !allow {
            let ban_until = *time + *ban_duration;
            *banned_until = Some((ban_until, ban_reason.clone()));
            return AllowCon::Banned((ban_until, ban_reason.clone()));
        }

        tracker.amount += 1;

        return if ban_status == IsBanned::UnBanned {
            AllowCon::UnbannedAndBlocked
        } else {
            AllowCon::Blocked
        };
    }

    *current += 1;

    if ban_status == IsBanned::UnBanned {
        AllowCon::UnbannedAndAllow
    } else {
        AllowCon::Allow
    }
}

pub fn simple_throttle(
    tracker: &mut global::ThresholdTracker,
    threshold: &global::Threshold,
    ban_duration: &TimeDelta,
    ban_reason: &global::IpBanReason,
    time: &DateTime<Utc>,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
) -> AllowCon {
    match is_banned(banned_until, time) {
        IsBanned::Banned => {
            return AllowCon::AlreadyBanned;
        }
        IsBanned::UnBanned => {
            tracker.started_at = *time;
            tracker.amount = 0;
            return AllowCon::UnbannedAndAllow;
        }
        _ => {}
    }
    let allow = threshold_allow(tracker, threshold, time);
    if !allow {
        let ban = (*time + *ban_duration, ban_reason.clone());
        *banned_until = Some(ban.clone());
        return AllowCon::Banned(ban);
    }

    AllowCon::Allow
}

pub fn threshold_allow(
    tracker: &mut global::ThresholdTracker,
    threshold: &global::Threshold,
    time: &DateTime<Utc>,
) -> bool {
    let max_reatched = tracker.amount >= threshold.amount;
    let time_passed = (*time - tracker.started_at) >= threshold.delta;
    trace!("threshold_allow: max_reatched: {}({}/{}), time_passed: {}", max_reatched,tracker.amount, threshold.amount, time_passed);

    if time_passed {
        tracker.started_at = *time;
        tracker.amount = 0;
    }
    !max_reatched || time_passed
}

pub fn compare_pick_worst(a: AllowCon, b: AllowCon) -> AllowCon {
    let get_order = |v: &AllowCon| match v {
        AllowCon::AlreadyBanned => 5,
        AllowCon::Banned(_) => 4,
        AllowCon::UnbannedAndBlocked => 3,
        AllowCon::Blocked => 2,
        AllowCon::UnbannedAndAllow => 1,
        AllowCon::Allow => 0,
    };
    let a_level = get_order(&a);
    let b_level = get_order(&b);
    if a_level >= b_level {
        a
    } else {
        b
    }
}

pub fn is_banned(
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
    time: &DateTime<Utc>,
) -> IsBanned {
    let Some((date, reason)) = banned_until else {
        trace!("is_banned: entry doesnt exist");
        return IsBanned::NotBanned;
    };

    let un_banned = time >= date;

    trace!(
        "is_banned: {} >= {} = {}, reason: {:#?}",
        time,
        date,
        !un_banned,
        reason,
    );

    if un_banned {
        *banned_until = None;
        return IsBanned::UnBanned;
    }
    IsBanned::Banned
}

// #[derive(Error, Debug)]
// pub enum WsThrottleErr {
//     #[error("MainGallery error: {0}")]
//     Serialization(#[from] bincode::Error),
// }

#[cfg(test)]
mod throttle_tests {
    use crate::global;
    use crate::global::throttle::{double_throttle, ranged_throttle, ws_ip_throttle};
    use chrono::{DateTime, TimeDelta, Utc};
    use std::net::{IpAddr, Ipv4Addr};
    use std::str::FromStr;
    use tracing::{debug, trace};
    use super::{threshold_allow, AllowCon};

    #[test]
    fn ws_throttle_test() {
        init_logger();

        //let mut throttle = WsIpTracker::new();
        let mut time = Utc::now();
        let ws_threshold = global::DefaultThreshold {
            ws_max_con_threshold: global::Threshold::new_const(10, TimeDelta::try_minutes(1)),
            ws_max_con_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_max_con_threshold_range: 5,
            ws_max_con_ban_reason: global::IpBanReason::WsTooManyReconnections,
            ws_con_flicker_threshold: global::Threshold::new_const(20, TimeDelta::try_minutes(1)),
            ws_con_flicker_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_con_flicker_ban_reason: global::IpBanReason::WsConFlickerDetected,
            ws_req_ban_threshold: global::Threshold::new_const(1, TimeDelta::try_minutes(1)),
            ws_req_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
        };

        let ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 69));
        let max_con_count: u64 = 5;
        let mut current_con_count: u64 = 0;
        let mut flicker_tracker = global::ThresholdTracker::new(time);
        let mut con_tracker = global::ThresholdTracker::new(time);
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;
        //let ranged_throttle = global::ThrottleRanged::new(range, started_at)
        // let task_tracker = TaskTracker::new();
        // let cancellation_token = CancellationToken::new();
        // let time_middleware = global::Clock::new();

        for _ in 0..5 {
            let con_1 = ws_ip_throttle(
                &mut flicker_tracker,
                &mut con_tracker,
                &mut current_con_count,
                &mut banned_until,
                &ws_threshold,
                &time,
            );
            time += TimeDelta::try_minutes(1).unwrap();
            assert_eq!(con_1, AllowCon::Allow);
        }
        //time += TimeDelta::try_minutes(10).unwrap();
        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::Blocked);

       

        current_con_count -= 1;
        //throttle.dec_con(&ip, &time);

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::Allow);

       // time += TimeDelta::try_minutes(1).unwrap();

        for _ in 0..19 {
            current_con_count -= 1;
            let con_1 = ws_ip_throttle(
                &mut flicker_tracker,
                &mut con_tracker,
                &mut current_con_count,
                &mut banned_until,
                &ws_threshold,
                &time,
            );
            assert_eq!(con_1, AllowCon::Allow);
        }

        current_con_count -= 1;

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(
            con_1,
            AllowCon::Banned((
                time + TimeDelta::try_minutes(1).unwrap(),
                global::IpBanReason::WsConFlickerDetected
            ))
        );

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::AlreadyBanned);

        time += TimeDelta::try_minutes(1).unwrap();

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::UnbannedAndAllow);

        for _ in 0..10 {
            let con_1 = ws_ip_throttle(
                &mut flicker_tracker,
                &mut con_tracker,
                &mut current_con_count,
                &mut banned_until,
                &ws_threshold,
                &time,
            );
            assert_eq!(con_1, AllowCon::Blocked);
        }

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(
            con_1,
            AllowCon::Banned((
                time + TimeDelta::try_minutes(1).unwrap(),
                global::IpBanReason::WsTooManyReconnections
            ))
        );

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::AlreadyBanned);

        time += TimeDelta::try_minutes(1).unwrap();

        //debug!("ONE: {:#?}", throttle);
        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::UnbannedAndBlocked);
        //debug!("TWO: {:#?}", throttle);

        current_con_count -= 1;
        //debug!("THREE: {:#?}", throttle);

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::Allow);

        for _ in 0..5 {
            current_con_count -= 1;
        }

        // let ip_exists = throttle.ips.get(&ip).is_some();
        // assert!(!ip_exists);

        //trace!("throttle: {:#?}", throttle);
    }

    #[test]
    fn throttle_ranged_test() {
        init_logger();

        let time = Utc::now();
        let ban_reason = global::IpBanReason::WsTooManyReconnections;
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        //let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;

        let max = 10;
        let mut current = 0;
        let mut tracker = global::ThresholdTracker::new(time);
        let threshold = global::Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;
        //let mut throttle = global::ThrottleRanged::new(10, started_at);

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!((result, current, tracker.amount,), (AllowCon::Allow, 1, 0));

        for _ in 0..8 {
            let result = ranged_throttle(
                &max,
                &mut current,
                &mut tracker,
                &threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!((result, current, tracker.amount,), (AllowCon::Allow, 10, 0));

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::Blocked, 10, 1)
        );

        for _ in 0..9 {
            let result = ranged_throttle(
                &max,
                &mut current,
                &mut tracker,
                &threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (
                AllowCon::Banned((
                    time.checked_add_signed(ban_duration).unwrap(),
                    global::IpBanReason::WsTooManyReconnections
                )),
                10,
                10
            )
        );

        let time = time.checked_add_signed(ban_duration).unwrap();
        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::UnbannedAndBlocked, 10, 1,)
        );

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::Blocked, 10, 2)
        );

        current -= 1;

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!((result, current, tracker.amount,), (AllowCon::Allow, 10, 2));

        for _ in 0..8 {
            let result = ranged_throttle(
                &max,
                &mut current,
                &mut tracker,
                &threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (
                AllowCon::Banned((
                    time.checked_add_signed(ban_duration).unwrap(),
                    global::IpBanReason::WsTooManyReconnections
                )),
                10,
                10
            )
        );

        let time = time.checked_add_signed(ban_duration).unwrap();
        current -= 1;

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::UnbannedAndAllow, 10, 0)
        );
    }

    #[test]
    fn throttle_double_layer_test() {
        init_logger();

        let time = Utc::now();
        let ban_reason = global::IpBanReason::WsTooManyReconnections;
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;
        
        let mut block_tracker = global::ThresholdTracker::new(time);
        let mut ban_tracker = global::ThresholdTracker::new(time);
        let block_threshold = global::Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let ban_threshold = global::Threshold::new(10, TimeDelta::try_seconds(10).unwrap());

        

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(result, AllowCon::Allow);

        for _ in 0..15 {
            let result = double_throttle(
                &mut block_tracker,
                &mut ban_tracker,
                &block_threshold,
                &ban_threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::Blocked, 10, 7)
        );

        for _ in 0..3 {
            let result = double_throttle(
                &mut block_tracker,
                &mut ban_tracker,
                &block_threshold,
                &ban_threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );
        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (
                AllowCon::Banned((
                    time.checked_add_signed(ban_duration).unwrap(),
                    global::IpBanReason::WsTooManyReconnections
                )),
                10,
                10
            )
        );

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::AlreadyBanned,  10, 10)
        );

        let time = time.checked_add_signed(ban_duration).unwrap();
        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::UnbannedAndAllow, 1, 0)
        );

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::Allow, 2, 0)
        );
    }

    #[test]
    fn threshold_tracker() {
        init_logger();

        let mut time = Utc::now();
        let max = 5;
        let delta = TimeDelta::try_seconds(5).unwrap();
        let mut tracker = global::ThresholdTracker::new(time);
        let threshold = global::Threshold::new(max, delta);
        
        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);

        tracker.amount = max - 1;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);

        tracker.amount = max;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(!allow);

        tracker.amount = max - 1;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);

        tracker.amount = max;
        time += delta;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);
    }

    fn init_logger() {
        let _ = tracing_subscriber::fmt()
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            )
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                    .unwrap_or(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap()),
            )
            .try_init();
    }
}
