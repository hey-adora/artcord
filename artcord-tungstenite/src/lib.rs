use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use chrono::DateTime;
use chrono::Month;
use chrono::Months;
use chrono::TimeDelta;
use chrono::Utc;
use futures::pin_mut;
use futures::FutureExt;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;

use cfg_if::cfg_if;
use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::sleep;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};

pub mod ws_app;



cfg_if! {
    if #[cfg(feature = "development")] {
        //const WS_LIMIT_MAX_CONNECTIONS: u64 = 5;
        const WS_CON_THRESHOLD_RANGE: u64 = 5;
        const WS_CON_THRESHOLD: Threshold = Threshold::new_const(10, TimeDelta::try_minutes(1));
        const WS_CON_THRESHOLD_BAN_DURATION: TimeDelta = match TimeDelta::try_minutes(1) { Some(delta) => delta, None => panic!("invalid delta") };
    } else {
        //const WS_MAX_FAILED_CON_ATTEMPTS: u64 = 1000;
        const WS_CON_THRESHOLD_RANGE: u64 = 1000;
        const WS_CON_THRESHOLD: Threshold = Threshold::new_const(1000, TimeDelta::try_minutes(1));
        const WS_CON_THRESHOLD_BAN_DURATION: TimeDelta = match TimeDelta::try_days(30) { Some(delta) => delta, None => panic!("invalid delta") };

    }
}

//const WS_MAX_FAILED_CON_ATTEMPTS_DELTA: TimeDelta = match TimeDelta::try_minutes(10) { Some(delta) => delta, None => panic!("invalid delta") };
//const WS_MAX_FAILED_CON_ATTEMPTS_RATE: u64 =  WS_MAX_FAILED_CON_ATTEMPTS / WS_MAX_FAILED_CON_ATTEMPTS_DELTA.num_minutes() as u64;
const WS_LIMIT_MAX_RED_FLAGS: u64 = 2;
const WS_EXPIRE_RED_FLAGS_DAYS: u64 = 30;
const WS_BAN_UNTIL_DAYS: u64 = 30;
const WS_TOKEN_SIZE: u64 = 256;
//const WS_LIMIT_THROTTLE: u64 = 10;
// pub async fn create_websockets(db: Arc<DB>) {}
