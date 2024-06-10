// mod aggregation;
// mod message;
// mod misc;
// mod model;
// mod shared_global;
// mod util;
// mod ws;

pub mod global {
    use artcord_leptos_web_sockets::WsPackage;
    use chrono::{DateTime, TimeDelta, Utc};
    use enum_index_derive::EnumIndex;
    use field_types::FieldName;
    use serde::{Deserialize, Serialize};
    use std::{
        collections::HashMap,
        fmt::{Display, Formatter},
        net::{IpAddr, SocketAddr},
        num::TryFromIntError,
        str::FromStr,
    };
    use strum::{AsRefStr, EnumCount, EnumString, IntoStaticStr, VariantNames};
    use thiserror::Error;
    use tracing::{error, info, trace, warn};

    pub type ClientPathType = usize;
    pub type TempConIdType = u128;
    pub type BanType = Option<(DateTime<Utc>, IpBanReason)>;
    pub type DbBanType = Option<(i64, String)>;

    pub const SEC_IN_MS: i64 = 1000;
    pub const MIN_IN_MS: i64 = 60 * SEC_IN_MS;
    pub const HOUR_IN_MS: i64 = 60 * MIN_IN_MS;
    pub const DAY_IN_MS: i64 = 24 * HOUR_IN_MS;

    pub const MIN_IN_SEC: i64 = 60;
    pub const HOUR_IN_SEC: i64 = 60 * MIN_IN_MS;
    pub const DAY_IN_SEC: i64 = 24 * HOUR_IN_MS;

    pub const MINIMUM_PASSWORD_LENGTH: usize = 10;

    pub trait TimeMiddleware {
        fn get_time(&self) -> impl std::future::Future<Output = DateTime<Utc>> + Send;
    }

    pub trait ClientThresholdMiddleware {
        fn get_threshold(&self, msg: &ClientMsg) -> Threshold;
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
    pub enum ServerMsg {
        WsLiveStatsIpCons(Vec<ConnectedWsIp>),

        WsLiveStatsConnected {
            ip: IpAddr,
            socket_addr: SocketAddr,
            con_id: TempConIdType,
            banned_until: BanType,
            req_stats: HashMap<ClientPathType, WsConReqStat>,
        },
        WsLiveStatsDisconnected {
            con_id: TempConIdType,
        },

        WsLiveStatsReqAllowed {
            con_id: TempConIdType,
            path: ClientPathType,
            total_amount: u64,
        },
        WsLiveStatsReqBlocked {
            con_id: TempConIdType,
            path: ClientPathType,
            total_amount: u64,
        },
        WsLiveStatsReqBanned {
            con_id: TempConIdType,
            path: ClientPathType,
            total_amount: u64,
        },

        WsLiveStatsIpBanned {
            ip: IpAddr,
            date: DateTime<Utc>,
            reason: IpBanReason,
        },
        WsLiveStatsIpUnbanned {
            ip: IpAddr,
        },

        WsLiveStatsConAllowed {
            ip: IpAddr,
            total_amount: u64,
        },
        WsLiveStatsConBlocked {
            ip: IpAddr,
            total_amount: u64,
        },
        WsLiveStatsConBanned {
            ip: IpAddr,
            total_amount: u64,
        },

        WsSavedStatsWithPagination {
            total_count: u64,
            latest: Option<i64>,
            stats: Vec<SavedWsCon>,
        },
        WsSavedStatsPage(Vec<SavedWsCon>),

        WsSavedStatsGraph(Vec<f64>),

        GalleryMain(Vec<AggImg>),
        GalleryUser(Option<Vec<AggImg>>),
        User(Option<DbUser>),
        LoginSuccess {
            user_id: String,
            token: String,
        },
        LoginErr(String),
        RegistrationSuccess,
        RegistrationErr(RegistrationInvalidMsg),
        LoggedOut,

        Error,
        None,
        Reset,
        NotImplemented,
        TooManyRequests,
    }

    #[derive(
        Deserialize, Serialize, Debug, PartialEq, Clone, VariantNames, EnumIndex, EnumCount,
    )]
    pub enum ClientMsg {
        GalleryInit {
            amount: u32,

            from: i64,
        },

        UserGalleryInit {
            amount: u32,

            from: i64,

            user_id: String,
        },

        User {
            user_id: String,
        },

        Login {
            email: String,
            password: String,
        },
        Logout,
        Register {
            email: String,
            password: String,
        },
        WsStatsTotalCount {
            from: Option<i64>,
        },
        WsStatsWithPagination {
            page: u64,
            amount: u64,
        },
        WsStatsPaged {
            page: u64,
            amount: u64,
            from: i64,
        },
        WsStatsGraph {
            from: i64,
            to: i64,
            unique_ip: bool,
        },

        LiveWsStats(bool),
        LiveWsThrottleCache(bool),
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
    pub enum DebugServerMsg {
        Restart,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
    pub enum DebugClientMsg {
        BrowserReady,
        RuntimeReady,
    }

    #[derive(
        Deserialize,
        Serialize,
        Debug,
        Clone,
        Copy,
        PartialEq,
        IntoStaticStr,
        VariantNames,
        EnumString,
        // AsRefStr,
        strum::Display
    )]
    #[strum(serialize_all = "snake_case")]
    pub enum IpBanReason {
        WsTooManyReconnections,
        WsRouteBruteForceDetected,
        WsConFlickerDetected,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    pub enum ConStatus {
        Allow,
        Blocked(u64, u64),
        Banned((DateTime<Utc>, IpBanReason)),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub enum DbAccRole {
        Member,
        Moderator,
        Admin,
    }

    #[derive(Clone, Copy)]
    pub struct ProdThreshold;

    #[derive(Clone, Debug)]
    pub struct Clock;

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    pub struct RegistrationInvalidMsg {
        pub general_error: Option<String>,
        pub email_error: Option<String>,
        pub password_error: Option<String>,
    }

    #[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, FieldName)]
    pub struct AggImg {
        pub id: String,
        pub user: DbUser,
        pub user_id: String,
        pub org_url: String,
        pub org_hash: String,
        pub format: String,
        pub width: u32,
        pub height: u32,
        pub has_high: bool,
        pub has_medium: bool,
        pub has_low: bool,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
    pub struct DbAccSession {
        pub id: String,
        pub acc_id: String,
        pub ip: String,
        pub agent: String,
        pub token: String,
        pub last_used: i64,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
    pub struct DbAcc {
        pub id: String,
        pub email: String,
        pub password: String,
        pub verified_email: bool,
        pub email_verification_code: String,
        pub discord: Option<DbAccDiscord>,
        pub role: String,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct DbAccDiscord {
        pub user_id: String,
        pub token: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
    pub struct DbAllowedChannel {
        pub id: String,
        pub guild_id: String,
        pub name: String,
        pub channel_id: String,
        pub feature: String,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
    pub struct DbAllowedGuild {
        pub id: String,
        pub guild_id: String,
        pub name: String,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
    pub struct DbAllowedRole {
        pub id: String,
        pub role_id: String,
        pub guild_id: String,
        pub name: String,
        pub feature: String,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, FieldName)]
    pub struct DbAutoReaction {
        pub id: String,
        pub emoji_id: Option<String>,
        pub guild_id: String,
        pub unicode: Option<String>,
        pub name: Option<String>,
        pub animated: bool,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
    pub struct DbImg {
        pub id: String,
        pub msg_id: String,
        pub show: bool,
        pub guild_id: String,
        pub user_id: String,
        pub channel_id: String,
        pub org_url: String,
        pub org_hash: String,
        pub format: String,
        pub width: u32,
        pub height: u32,
        pub has_high: bool,
        pub has_medium: bool,
        pub has_low: bool,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
    pub struct DbMigration {
        pub name: String,
        pub version: u32,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
    pub struct DbUser {
        pub id: String,
        pub author_id: String,
        pub guild_id: String,
        pub name: String,
        pub pfp_hash: Option<String>,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, FieldName)]
    pub struct DbWsIp {
        pub id: String,
        pub ip: String,
        pub total_allow_amount: i64,
        pub total_block_amount: i64,
        pub total_banned_amount: i64,
        pub total_already_banned_amount: i64,
        pub con_count_tracker: DbThresholdTracker,
        pub con_flicker_tracker: DbThresholdTracker,
        pub banned_until: DbBanType,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, FieldName)]
    pub struct DbWsIpManager {
        pub id: String,
        pub ip: String,
        pub req_stats: Vec<DbWsConReqStat>,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
    pub struct DbWsCon {
        pub id: String,
        pub con_id: String,
        pub ip: String,
        pub addr: String,
        pub req_stats: Vec<DbWsConReqStat>,
        pub connected_at: i64,
        pub disconnected_at: i64,
        pub modified_at: i64,
        pub created_at: i64,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone, FieldName)]
    pub struct DbWsConReqStat {
        pub path: String,
        pub total_allowed_count: i64,
        pub total_blocked_count: i64,
        pub total_banned_count: i64,
        pub total_already_banned_count: i64,
        pub last_reset_at: i64,
        pub block_tracker: DbThresholdTracker,
        pub ban_tracker: DbThresholdTracker,
    }

    // #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    // pub struct DbThrottleDoubleLayer {
    //     pub block_tracker: DbThresholdTracker,
    //     pub ban_tracker: DbThresholdTracker,
    // }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, FieldName)]
    pub struct DbThresholdTracker {
        //pub total_amount: i64,
        pub amount: i64,
        pub started_at: i64,
    }

    // #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    // pub struct TempThrottleConnection {
    //     // pub ws_path_count: HashMap<ClientPathType, LiveThrottleConnectionCount>,
    //     pub con_throttle: ThrottleRanged,
    //     pub con_flicker_throttle: ThrottleSimple,
    //     //pub ip_stats: HashMap<ClientPathType, LiveThrottleConnectionCount>,
    //     pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    //     //pub cons_brodcast: broadcast::Sender<ConMsg>
    // }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, FieldName)]
    pub struct SavedWsIp {
        pub id: uuid::Uuid,
        pub ip: IpAddr,
        pub total_allow_amount: u64,
        pub total_block_amount: u64,
        pub total_banned_amount: u64,
        pub total_already_banned_amount: u64,
        pub con_count_tracker: ThresholdTracker,
        pub con_flicker_tracker: ThresholdTracker,
        pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
        pub modified_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, FieldName)]
    pub struct ConnectedWsIp {
        pub ip: IpAddr,
        pub total_allow_amount: u64,
        pub total_block_amount: u64,
        pub total_banned_amount: u64,
        pub total_already_banned_amount: u64,
        pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, FieldName)]
    pub struct SavedWsIpManager {
        pub id: uuid::Uuid,
        pub ip: IpAddr,
        pub req_stats: HashMap<ClientPathType, WsConReqStat>,
        pub modified_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
        //pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
    pub struct ConnectedWsCon {
        pub con_id: u128,
        pub ip: IpAddr,
        pub addr: SocketAddr,
        //pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
        pub req_stats: HashMap<ClientPathType, WsConReqStat>,
        pub connected_at: DateTime<Utc>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
    pub struct SavedWsCon {
        pub id: String,
        pub con_id: u128,
        pub ip: IpAddr,
        pub addr: SocketAddr,
        pub req_stats: HashMap<ClientPathType, WsConReqStat>,
        pub connected_at: DateTime<Utc>,
        pub disconnected_at: DateTime<Utc>,
        pub modified_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
    pub struct WsConReqStat {
        pub total_allowed_count: u64,
        pub total_blocked_count: u64,
        pub total_banned_count: u64,
        pub total_already_banned_count: u64,
        pub last_reset_at: DateTime<Utc>,
        pub block_tracker: ThresholdTracker,
        pub ban_tracker: ThresholdTracker,
    }

    // #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    // pub struct ThrottleSimple {
    //     pub tracker: ThresholdTracker,
    // }

    // #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    // pub struct ThrottleRanged {
    //     pub range: u64,
    //     pub amount: u64,
    //     pub tracker: ThresholdTracker,
    // }

    // #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    // pub struct ThrottleDoubleLayer {
    //     pub block_tracker: ThresholdTracker,
    //     pub ban_tracker: ThresholdTracker,
    // }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    pub struct ThresholdTracker {
        // pub total_amount: u64,
        pub amount: u64,
        pub started_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Threshold {
        pub amount: u64,
        pub delta: TimeDelta,
        pub rate_sec: u64,
    }

    // #[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
    // pub struct WsConReqStat {
    //     pub total_allowed_count: u64,
    //     pub total_blocked_count: u64,
    //     pub total_banned_count: u64,
    //     pub total_already_banned_count: u64,
    //     pub last_reset_at: DateTime<Utc>,
    //     pub throttle: ThrottleDoubleLayer,
    // }

    impl ClientThresholdMiddleware for ProdThreshold {
        fn get_threshold(&self, msg: &ClientMsg) -> Threshold {
            match msg {
                _ => Threshold::new_const(5, TimeDelta::try_seconds(10)),
            }
        }
    }

    impl TimeMiddleware for Clock {
        async fn get_time(&self) -> DateTime<Utc> {
            Utc::now()
        }
    }

    impl artcord_leptos_web_sockets::Receive for ServerMsg {
        fn recv_from_vec(bytes: &[u8]) -> Result<WsPackage<Self>, String>
        where
            Self: std::marker::Sized + Clone,
        {
            ServerMsg::from_bytes(bytes).map_err(|e| e.to_string())
        }
    }

    impl artcord_leptos_web_sockets::Send for ClientMsg {
        fn send_as_vec(package: &WsPackage<Self>) -> Result<Vec<u8>, String>
        where
            Self: Clone,
        {
            Self::as_vec(package).map_err(|e| e.to_string())
        }
    }

    impl artcord_leptos_web_sockets::Receive for DebugServerMsg {
        fn recv_from_vec(bytes: &[u8]) -> Result<WsPackage<Self>, String>
        where
            Self: std::marker::Sized + Clone,
        {
            DebugServerMsg::from_bytes(bytes).map_err(|e| e.to_string())
        }
    }

    impl artcord_leptos_web_sockets::Send for DebugClientMsg {
        fn send_as_vec(package: &WsPackage<Self>) -> Result<Vec<u8>, String>
        where
            Self: Clone,
        {
            Self::as_vec(package).map_err(|e| e.to_string())
        }
    }

    impl Display for DbAccRole {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    DbAccRole::Member => "member",
                    DbAccRole::Moderator => "moderator",
                    DbAccRole::Admin => "admin",
                }
            )
        }
    }

    impl Default for AggImg {
        fn default() -> Self {
            Self {
                user: DbUser {
                    id: uuid::Uuid::new_v4().to_string(),
                    author_id: String::from("id"),
                    guild_id: String::from("1159766826620817419"),
                    name: String::from("name"),
                    pfp_hash: Some(String::from("pfp_hash")),
                    modified_at: Utc::now().timestamp_millis(),
                    created_at: Utc::now().timestamp_millis(),
                },
                org_url: String::from("wow"),
                user_id: String::from("1159037321283375174"),
                id: String::from("1177244237021073450"),
                org_hash: String::from("2552bd2db66978a9b3675721e95d1cbd"),
                format: String::from("png"),
                width: 233,
                height: 161,
                has_high: false,
                has_medium: false,
                has_low: false,
                modified_at: Utc::now().timestamp_millis(),
                created_at: Utc::now().timestamp_millis(),
            }
        }
    }

    impl DbAllowedGuild {
        pub fn new(guild_id: String, name: String) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                guild_id,
                name,
                created_at: Utc::now().timestamp_millis(),
                modified_at: Utc::now().timestamp_millis(),
            }
        }
    }

    impl TryFrom<ThresholdTracker> for DbThresholdTracker {
        type Error = ThresholdTrackerToDbErr;
        fn try_from(value: ThresholdTracker) -> Result<Self, Self::Error> {
            Ok(Self {
                //total_amount: i64::try_from(value.total_amount)?,
                amount: i64::try_from(value.amount)?,
                started_at: value.started_at.timestamp_millis(),
            })
        }
    }

    impl TryFrom<DbThresholdTracker> for ThresholdTracker {
        type Error = ThresholdTrackerFromDbErr;
        fn try_from(value: DbThresholdTracker) -> Result<Self, Self::Error> {
            Ok(Self {
                //total_amount: u64::try_from(value.total_amount)?,
                amount: u64::try_from(value.amount)?,
                started_at: date_from_db(value.started_at)?,
            })
        }
    }

    // impl TryFrom<ThrottleDoubleLayer> for DbThrottleDoubleLayer {
    //     type Error = ThrottleDoubleLayerFromError;
    //     fn try_from(value: ThrottleDoubleLayer) -> Result<Self, Self::Error> {
    //         Ok(Self {
    //             ban_tracker: value.ban_tracker.try_into()?,
    //             block_tracker: value.block_tracker.try_into()?,
    //         })
    //     }
    // }

    // impl TryFrom<DbThrottleDoubleLayer> for ThrottleDoubleLayer {
    //     type Error = DbThrottleDoubleLayerFromError;
    //     fn try_from(value: DbThrottleDoubleLayer) -> Result<Self, Self::Error> {
    //         Ok(Self {
    //             ban_tracker: value.ban_tracker.try_into()?,
    //             block_tracker: value.block_tracker.try_into()?,
    //         })
    //     }
    // }

    impl TryFrom<DbWsIp> for SavedWsIp {
        type Error = WsIpFromDbErr;

        fn try_from(value: DbWsIp) -> Result<SavedWsIp, Self::Error> {
            //let id = uuid::Uuid::from_str(&value.id)?;

            Ok(Self {
                id: uuid::Uuid::from_str(&value.id)?,
                ip: IpAddr::from_str(&value.ip)?,
                total_allow_amount: value.total_allow_amount as u64,
                total_block_amount: value.total_block_amount as u64,
                total_banned_amount: value.total_banned_amount as u64,
                total_already_banned_amount: value.total_already_banned_amount as u64,
                banned_until: ban_from_db(value.banned_until)?,
                con_count_tracker: value.con_count_tracker.try_into()?,
                con_flicker_tracker: value.con_flicker_tracker.try_into()?,
                modified_at: date_from_db(value.modified_at)?,
                created_at: date_from_db(value.created_at)?,
            })
        }
    }

    impl TryFrom<DbWsCon> for ConnectedWsCon {
        type Error = WsConFromDbErr;

        fn try_from(value: DbWsCon) -> Result<ConnectedWsCon, Self::Error> {
            let req_stats = req_stats_from_db(value.req_stats)?;
            let con_id = uuid::Uuid::from_str(&value.con_id)?;

            Ok(Self {
                con_id: con_id.as_u128(),
                ip: IpAddr::from_str(&value.ip)?,
                addr: SocketAddr::from_str(&value.addr)?,
                req_stats,
                connected_at: date_from_db(value.connected_at)?,
            })
        }
    }

    impl TryFrom<DbWsCon> for SavedWsCon {
        type Error = WsConFromDbErr;

        fn try_from(value: DbWsCon) -> Result<SavedWsCon, Self::Error> {
            let req_stats = req_stats_from_db(value.req_stats)?;
            let con_id = uuid::Uuid::from_str(&value.con_id)?;

            Ok(Self {
                id: value.id,
                con_id: con_id.as_u128(),
                ip: IpAddr::from_str(&value.ip)?,
                addr: SocketAddr::from_str(&value.addr)?,
                req_stats,
                connected_at: date_from_db(value.connected_at)?,
                disconnected_at: date_from_db(value.disconnected_at)?,
                created_at: date_from_db(value.created_at)?,
                modified_at: date_from_db(value.modified_at)?,
            })
        }
    }

    impl TryFrom<(ClientPathType, WsConReqStat)> for DbWsConReqStat {
        type Error = WsConReqStatToDbErr;
        fn try_from((path, value): (ClientPathType, WsConReqStat)) -> Result<Self, Self::Error> {
            let path = ClientMsg::VARIANTS
                .get(path)
                .ok_or(WsConReqStatToDbErr::InvalidClientMsgEnumIndex(path))?;
            Ok(Self {
                path: path.to_string(),
                total_allowed_count: i64::try_from(value.total_allowed_count)?,
                total_blocked_count: i64::try_from(value.total_blocked_count)?,
                total_banned_count: i64::try_from(value.total_banned_count)?,
                total_already_banned_count: i64::try_from(value.total_already_banned_count)?,
                //total_unbanned_count: i64::try_from(value.total_unbanned_count)?,
                // total_count: i64::try_from(value.total_count)?,
                // count: i64::try_from(value.count)?,
                // total_count: i64::try_from(value.count)?
                last_reset_at: value.last_reset_at.timestamp_millis(),
                block_tracker: value.block_tracker.try_into()?,
                ban_tracker: value.ban_tracker.try_into()?,
            })
        }
    }

    //WsIpManagerFromDb
    impl TryFrom<DbWsIpManager> for SavedWsIpManager {
        type Error = WsIpManagerFromDb;
        fn try_from(value: DbWsIpManager) -> Result<Self, Self::Error> {
            let req_stats = req_stats_from_db(value.req_stats)?;

            Ok(Self {
                id: uuid::Uuid::from_str(&value.id)?,
                ip: IpAddr::from_str(&value.ip)?,
                req_stats,
                created_at: date_from_db(value.created_at)?,
                modified_at: date_from_db(value.modified_at)?,
            })
        }
    }

    impl TryFrom<DbWsConReqStat> for WsConReqStat {
        type Error = WsConReqStatFromDbErr;
        fn try_from(value: DbWsConReqStat) -> Result<Self, Self::Error> {
            Ok(Self {
                // total_count: u64::try_from(value.total_count)?,
                // count: u64::try_from(value.count)?,
                total_allowed_count: u64::try_from(value.total_allowed_count)?,
                total_blocked_count: u64::try_from(value.total_blocked_count)?,
                total_banned_count: u64::try_from(value.total_banned_count)?,
                total_already_banned_count: u64::try_from(value.total_already_banned_count)?,
                //total_unbanned_count: u64::try_from(value.total_unbanned_count)?,
                block_tracker: value.block_tracker.try_into()?,
                ban_tracker: value.ban_tracker.try_into()?,
                last_reset_at: date_from_db(value.last_reset_at)?,
            })
        }
    }

    impl RegistrationInvalidMsg {
        pub fn validate_registration(
            email: &str,
            password: &str,
        ) -> (bool, Option<String>, Option<String>) {
            let email_error = if email.len() < 1 {
                Some("Email field can't be empty.".to_string())
            } else {
                None
            };

            let password_error = if password.len() < MINIMUM_PASSWORD_LENGTH {
                Some("Password field can't be empty.".to_string())
            } else {
                None
            };

            let invalid = email_error.is_some() || password_error.is_some();

            (invalid, email_error, password_error)
        }

        pub fn new() -> Self {
            Self {
                general_error: None,
                email_error: None,
                password_error: None,
            }
        }

        pub fn general(mut self, error: String) -> Self {
            self.general_error = Some(error);

            self
        }
    }

    impl ServerMsg {
        pub fn from_bytes(bytes: &[u8]) -> Result<WsPackage<Self>, bincode::Error> {
            bincode::deserialize::<WsPackage<Self>>(bytes)
        }

        pub fn as_bytes(package: WsPackage<Self>) -> Result<Vec<u8>, bincode::Error> {
            bincode::serialize::<WsPackage<Self>>(&package)
        }
    }

    impl ClientMsg {
        pub fn as_vec(package: &WsPackage<Self>) -> Result<Vec<u8>, bincode::Error> {
            let a = bincode::serialize::<WsPackage<Self>>(package);
            //log!("SERIALIZE {:?} {:?}", self, a);
            a
        }

        pub fn from_bytes(bytes: &[u8]) -> Result<WsPackage<Self>, bincode::Error> {
            //log!("DESERIALIZE {:?}", bytes);
            let a = bincode::deserialize::<WsPackage<Self>>(bytes);
            a
        }
    }


    impl DebugServerMsg {
        pub fn from_bytes(bytes: &[u8]) -> Result<WsPackage<Self>, bincode::Error> {
            let result = bincode::deserialize::<WsPackage<Self>>(bytes);
            trace!(
                "debug server msg deserialized from {:?} to {:?}",
                bytes,
                &result
            );
            result
        }

        pub fn as_bytes(package: &WsPackage<Self>) -> Result<Vec<u8>, bincode::Error> {
            let result = bincode::serialize::<WsPackage<Self>>(package);
            trace!(
                "debug server msg serialized from {:?} {:?}",
                &package,
                &result
            );
            result
        }
    }

    impl DebugClientMsg {
        pub fn as_vec(package: &WsPackage<Self>) -> Result<Vec<u8>, bincode::Error> {
            let result: Result<Vec<u8>, Box<bincode::ErrorKind>> =
                bincode::serialize::<WsPackage<Self>>(package);
            trace!(
                "debug client msg serialized from {:?} {:?}",
                package,
                &result
            );
            result
        }

        pub fn from_bytes(bytes: &[u8]) -> Result<WsPackage<Self>, bincode::Error> {
            let result: Result<WsPackage<Self>, Box<bincode::ErrorKind>> =
                bincode::deserialize::<WsPackage<Self>>(bytes);
            trace!(
                "debug client msg deserialized from {:?} to {:?}",
                bytes,
                &result
            );
            result
        }
    }

    impl DbAutoReaction {
        pub fn new(
            guild_id: String,
            unicode: Option<String>,
            emoji_id: Option<String>,
            name: Option<String>,
            animated: bool,
        ) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                guild_id,
                unicode,
                emoji_id,
                name,
                animated,
                modified_at: Utc::now().timestamp_millis(),
                created_at: Utc::now().timestamp_millis(),
            }
        }
    }

    impl DbAllowedRole {
        pub fn new(role_id: String, guild_id: String, name: String, feature: String) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                role_id,
                guild_id,
                name,
                feature,
                created_at: Utc::now().timestamp_millis(),
                modified_at: Utc::now().timestamp_millis(),
            }
        }
    }

    impl DbAcc {
        pub fn new(
            email: &str,
            password: &str,
            email_verification_code: &str,
            time: &DateTime<Utc>,
        ) -> DbAcc {
            DbAcc {
                id: uuid::Uuid::new_v4().to_string(),
                email: email.to_string(),
                verified_email: false,
                email_verification_code: email_verification_code.to_string(),
                password: password.to_string(),
                role: DbAccRole::Member.to_string(),
                discord: None,
                modified_at: time.timestamp_millis(),
                created_at: time.timestamp_millis(),
            }
        }
    }

    impl DbAccSession {
        pub fn new(
            acc_id: String,
            ip: String,
            agent: String,
            token: String,
            time: &DateTime<Utc>,
        ) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                acc_id,
                ip,
                agent,
                token,
                last_used: time.timestamp_millis(),
                modified_at: time.timestamp_millis(),
                created_at: time.timestamp_millis(),
            }
        }
    }

    impl DbWsIp {
        pub fn try_new(
            ip: IpAddr,
            total_allow_amount: u64,
            total_block_amount: u64,
            total_banned_amount: u64,
            total_already_banned_amount: u64,
            con_count_tracker: ThresholdTracker,
            con_flicker_tracker: ThresholdTracker,
            banned_until: BanType,
            time: DateTime<Utc>,
        ) -> Result<Self, WsIpToDbErr> {
            Ok(
                Self {
                    id: uuid::Uuid::new_v4().to_string(),
                    ip: ip.to_string(),
                    total_allow_amount: total_allow_amount as i64,
                    total_block_amount: total_block_amount as i64,
                    total_banned_amount: total_banned_amount as i64,
                    total_already_banned_amount: total_already_banned_amount as i64,
                    con_count_tracker: con_count_tracker.try_into()?,
                    con_flicker_tracker: con_flicker_tracker.try_into()?,
                    banned_until: ban_to_db(banned_until),
                    modified_at: time.timestamp_millis(),
                    created_at: time.timestamp_millis()
                }
            )
        }
    }

    impl DbWsIpManager {
        pub fn try_new(
            ip: IpAddr,
            req_stats: HashMap<ClientPathType, WsConReqStat>,
            time: DateTime<Utc>,
        ) -> Result<Self, WsIpManagerToDb> {
            let req_stats: Vec<DbWsConReqStat> = req_stats_to_db(req_stats)?;

            Ok(Self {
                id: uuid::Uuid::new_v4().to_string(),
                ip: ip.to_string(),
                req_stats,
                created_at: time.timestamp_millis(),
                modified_at: time.timestamp_millis(),
            })
        }
    }


    impl DbWsCon {
        pub fn try_new(
            value: ConnectedWsCon,
            ip: IpAddr,
            addr: SocketAddr,
            con_id: String,
            connected_at: DateTime<Utc>,
            disconnected_at: DateTime<Utc>,
            time: DateTime<Utc>,
        ) -> Result<Self, WsConReqStatToDbErr> {
            let req_count: Vec<DbWsConReqStat> = req_stats_to_db(value.req_stats)?;

            Ok(Self {
                id: uuid::Uuid::new_v4().to_string(),
                con_id,
                ip: ip.to_string(),
                addr: addr.to_string(),
                req_stats: req_count,
                connected_at: connected_at.timestamp_millis(),
                disconnected_at: disconnected_at.timestamp_millis(),
                //throttle: value.throttle.try_into()?,
                modified_at: time.timestamp_millis(),
                created_at: time.timestamp_millis(),
            })
        }
    }

    impl WsConReqStat {
        pub fn new(time: DateTime<Utc>) -> Self {
            Self {
                last_reset_at: time,
                total_allowed_count: 0,
                total_blocked_count: 0,
                total_banned_count: 0,
                total_already_banned_count: 0,
                block_tracker: ThresholdTracker::new(time),
                ban_tracker: ThresholdTracker::new(time),
            }
        }
    }

    // impl ThrottleSimple {
    //     pub fn new(started_at: DateTime<Utc>) -> Self {
    //         Self {
    //             tracker: ThresholdTracker::new(started_at),
    //         }
    //     }
    // }

    // impl ThrottleRanged {
    //     pub fn new(range: u64, started_at: DateTime<Utc>) -> Self {
    //         Self {
    //             range,
    //             tracker: ThresholdTracker::new(started_at),
    //             amount: 0,
    //         }
    //     }
    // }

    // impl ThrottleDoubleLayer {
    //     pub fn new(started_at: DateTime<Utc>) -> Self {
    //         Self {
    //             block_tracker: ThresholdTracker::new(started_at),
    //             ban_tracker: ThresholdTracker::new(started_at),
    //         }
    //     }
    // }

    impl ThresholdTracker {
        pub fn new(started_at: DateTime<Utc>) -> ThresholdTracker {
            Self {
                //      total_amount: 0,
                amount: 0,
                started_at,
            }
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
    }

    impl ConnectedWsIp {
        pub fn new(ip: IpAddr) -> Self {
            Self {
                ip,
                total_allow_amount: 0,
                total_block_amount: 0,
                total_banned_amount: 0,
                total_already_banned_amount: 0,
                banned_until: None,
            }
        }
    }

    pub fn ban_from_db(banned_until: DbBanType) -> Result<BanType, BanFromDbErr> {
        match banned_until {
            Some((date, reason)) => {
                let date = date_from_db(date)?;
                let reason = IpBanReason::from_str(&reason).map_err(|err| BanFromDbErr::InvalidReason { reason, err })?;
                Ok(Some((date, reason)))
            }
            None => Ok(None),
        }
    }

    pub fn ban_to_db(banned_until: BanType) -> Option<(i64, String)> {
        banned_until.map(|(date, reason)| (date.timestamp_millis(), reason.to_string() ))
    }

    pub fn req_stats_to_db(
        current_req_stats: HashMap<ClientPathType, WsConReqStat>,
    ) -> Result<Vec<DbWsConReqStat>, WsConReqStatToDbErr> {
        current_req_stats
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<DbWsConReqStat>, WsConReqStatToDbErr>>()
    }

    pub fn req_stats_from_db(
        current_req_stats: Vec<DbWsConReqStat>,
    ) -> Result<HashMap<ClientPathType, WsConReqStat>, WsConReqStatFromDbErr> {
        let mut req_stats =
            HashMap::<ClientPathType, WsConReqStat>::with_capacity(current_req_stats.len());
        for req_count in current_req_stats {
            let client_msg_enum_index = ClientMsg::VARIANTS
                .iter()
                .position(|name| *name == req_count.path)
                .ok_or(WsConReqStatFromDbErr::InvalidClientMsgEnumName(
                    req_count.path.clone(),
                ))?;
            req_stats.insert(client_msg_enum_index, req_count.try_into()?);
        }

        Ok(req_stats)
    }

    pub fn date_from_db(date: i64) -> Result<DateTime<Utc>, InvalidDateErr> {
        Ok(
            DateTime::<Utc>::from_timestamp_millis(date)
                .ok_or(InvalidDateErr { date })?
        )
    }

    #[derive(Error, Debug)]
    pub enum BanFromDbErr {
        
        #[error("invalid ban reason: {reason:?}, err: {err:?}")]
        InvalidReason {
            reason: String,
            err: strum::ParseError
        },

        #[error("invalid ban date: {0}")]
        InvalidDate(#[from] InvalidDateErr),
    }

    #[derive(Error, Debug)]
    pub enum WsIpFromDbErr {
        #[error("banned_until conversion error: {0}")]
        BanError(#[from] BanFromDbErr),

        #[error("tracker error: {0}")]
        TrackerError(#[from] ThresholdTrackerFromDbErr),

        #[error("failed to parse string to socket_addr: {0}")]
        InvalidSocketAddr(#[from] std::net::AddrParseError),

        #[error("Invalid uuid: {0}")]
        InvalidUuid(#[from] uuid::Error),

        #[error("invalid ban date: {0}")]
        InvalidDate(#[from] InvalidDateErr),
        
    }

   

    #[derive(Error, Debug)]
    pub enum WsIpToDbErr {
        #[error("tracker error: {0}")]
        TrackerError(#[from] ThresholdTrackerToDbErr),
    }


    #[derive(Error, Debug)]
    pub enum WsIpManagerFromDb {
        #[error("failed to convert path from database: {0}")]
        WsConReqStatFromDbErr(#[from] WsConReqStatFromDbErr),

        #[error("failed to parse string to socket_addr: {0}")]
        InvalidSocketAddr(#[from] std::net::AddrParseError),

        #[error("Invalid date: {0}")]
        InvalidDate(#[from] InvalidDateErr),

        #[error("Invalid uuid: {0}")]
        InvalidUuid(#[from] uuid::Error),
    }

    #[derive(Error, Debug)]
    pub enum WsIpManagerToDb {
        #[error("Failed to convert req stats: {0}")]
        ReqStatErr(#[from] WsConReqStatToDbErr),
    }

    #[derive(Error, Debug)]
    pub enum WsConReqStatToDbErr {
        #[error("Failed to convert u64 to i64: {0}")]
        TryFromIntError(#[from] TryFromIntError),

        #[error("Invalid client msg enum index - out of bounds: {0}")]
        InvalidClientMsgEnumIndex(usize),

        #[error("error converting double_layer_throttle: {0}")]
        DoubleLayer(#[from] ThrottleDoubleLayerFromError),

        #[error("tracker error: {0}")]
        TrackerError(#[from] ThresholdTrackerToDbErr),
    }

    #[derive(Error, Debug)]
    pub enum WsConReqStatFromDbErr {
        #[error("Invalid client msg enum name - name not found: {0}")]
        InvalidClientMsgEnumName(String),

        #[error("Failed to convert i64 to u64: {0}")]
        TryFromIntError(#[from] TryFromIntError),

        #[error("error converting double_layer_throttle: {0}")]
        DoubleLayer(#[from] DbThrottleDoubleLayerFromError),

        #[error("invalid date: {0}")]
        InvalidDate(#[from] InvalidDateErr),

        #[error("tracker error: {0}")]
        TrackerError(#[from] ThresholdTrackerFromDbErr),
    }

    #[derive(Error, Debug)]
    pub enum WsConFromDbErr {
        #[error("failed to parse string to socket_addr: {0}")]
        InvalidSocketAddr(#[from] std::net::AddrParseError),

        #[error("failed to convert path from database: {0}")]
        WsConReqStatFromDbErr(#[from] WsConReqStatFromDbErr),

        #[error("failed to convert from database: {0}")]
        DbThrottleDoubleLayer(#[from] DbThrottleDoubleLayerFromError),

        #[error("Failed to convert i64 to u64: {0}")]
        TryFromIntError(#[from] TryFromIntError),

        #[error("Invalid date: {0}")]
        InvalidDate(#[from] InvalidDateErr),

        #[error("Invalid uuid: {0}")]
        InvalidUuid(#[from] uuid::Error),
    }

    #[derive(Error, Debug)]
    pub enum DbThrottleDoubleLayerFromError {
        #[error("db missing ban date")]
        MissingBannedDate,

        #[error("db missing ban reason")]
        MissingBannedReason,

        #[error("invalid date: {0}")]
        InvalidDate(#[from] InvalidDateErr),

        #[error("invalid date: {0}")]
        InvalidReason(#[from] strum::ParseError),

        #[error("tracker error: {0}")]
        TrackerError(#[from] ThresholdTrackerFromDbErr),
    }

    #[derive(Error, Debug)]
    pub enum ThrottleDoubleLayerFromError {
        #[error("tracker error: {0}")]
        TrackerError(#[from] ThresholdTrackerToDbErr),
    }

    #[derive(Error, Debug)]
    pub enum ThresholdTrackerToDbErr {
        #[error("Failed to convert int: {0}")]
        TryFromIntError(#[from] TryFromIntError),
    }

    #[derive(Error, Debug)]
    pub enum ThresholdTrackerFromDbErr {
        #[error("Failed to convert int: {0}")]
        TryFromIntError(#[from] TryFromIntError),

        #[error("invalid date: {0}")]
        InvalidDate(#[from] InvalidDateErr),
    }

    #[derive(Error, Debug)]
    pub struct InvalidDateErr {
        date: i64
    }

    impl Display for InvalidDateErr {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.date)
        }
    }
}
