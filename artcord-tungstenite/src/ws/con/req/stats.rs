use std::collections::HashMap;

use artcord_state::{message::prod_client_msg::ClientPathType, misc::{throttle_connection::{IpBanReason}, throttle_threshold::{AllowCon, Threshold}}};
use chrono::{DateTime, TimeDelta, Utc};

// #[derive(Debug, Clone, PartialEq)]
// pub struct ReqStats {
//     pub paths: HashMap<ClientPathType, WsReqStat>,
// }

// impl ReqStats {
//     pub fn new() -> Self {
//         Self {
//             paths: HashMap::new()
//         }
//     }

//     pub async fn inc_path(
//         &mut self,
//         path: ClientPathType,
//         block_threshold: Threshold,
//         ban_threshold: &Threshold,
//         ban_duration: &TimeDelta,
//         banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
//         time: &DateTime<Utc>,
//     ) -> AllowCon {
//         let path = self
//             .paths
//             .entry(path)
//             .or_insert_with(|| WsReqStat::new(*time));

//         let result = path.throttle.allow(
//             &block_threshold,
//             ban_threshold,
//             IpBanReason::WsRouteBruteForceDetected,
//             ban_duration,
//             time,
//             banned_until,
//         );

//         path.total_count += 1;

//         match &result {
//             AllowCon::Allow => {
//                 path.total_allow_count += 1;
//             }
//             AllowCon::Unbanned => {
//                 path.total_allow_count += 1;
//             }
//             AllowCon::Blocked => {
//                 path.total_blocked_count += 1;
//             }
//             AllowCon::Banned(_) => {
//                 path.total_banned_count += 1;
//             }
//             AllowCon::AlreadyBanned => {
//                 path.total_banned_count += 1;
//             }
//         }

//         result
//     }
// }