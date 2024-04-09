use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
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
use ws_route::ws_admin_throttle::WsHandleAdminThrottleError;
use ws_route::ws_statistics::WsStatisticsError;
use ws_route::ws_user_gallery::WsHandleUserGalleryError;

use crate::user_task::UserTask;
use crate::ws_route::ws_admin_throttle::ws_hadnle_admin_throttle;
use crate::ws_route::ws_main_gallery::ws_handle_main_gallery;
use crate::ws_route::ws_statistics;
use crate::ws_route::ws_statistics::ws_statistics;
use crate::ws_route::ws_user::ws_handle_user;
use crate::ws_route::ws_user_gallery::ws_handle_user_gallery;

pub mod user_task;
pub mod ws_app;
pub mod ws_route;
pub mod ws_throttle;

const WS_LIMIT_MAX_CONNECTIONS: u64 = 10;
const WS_LIMIT_MAX_RED_FLAGS: u64 = 2;
const WS_EXPIRE_RED_FLAGS_DAYS: u64 = 30;
const WS_BAN_UNTIL_DAYS: u64 = 30;
//const WS_LIMIT_THROTTLE: u64 = 10;

// pub async fn create_websockets(db: Arc<DB>) {}
