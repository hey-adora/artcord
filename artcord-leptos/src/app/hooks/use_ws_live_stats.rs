use std::collections::HashMap;

use artcord_leptos_web_sockets::{channel::WsRecvResult, runtime::WsRuntime};
use artcord_state::message::{prod_client_msg::{ClientMsg, WsPath}, prod_server_msg::{ServerMsg, WsStatTemp}};
use leptos::{RwSignal, SignalSet, SignalUpdate, SignalWithUntracked};
use tracing::warn;

use crate::app::pages::admin::WebWsStat;

#[derive(Copy, Clone, Debug)]
pub struct LiveWsStats {
    pub stats: RwSignal<HashMap<String, WebWsStat>>
}

impl Default for LiveWsStats {
    fn default() -> Self {
        Self {
            stats: RwSignal::new(HashMap::new())
        }
    }
}

impl LiveWsStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_live_stats(&self, stats: HashMap<String, WsStatTemp>) {
        let mut web_stats: HashMap<String, WebWsStat> = HashMap::with_capacity(stats.len());
        for (path, stat) in stats {
            web_stats.insert(path, stat.into());
        }
        self.stats.set(web_stats);
    }

    pub fn add_live_stat(&self, con_key: String, stat: WebWsStat) {
        self.stats.update(move |stats| {
            stats.insert(con_key.clone(), stat.clone().into());
        });
    }

    pub fn inc_live_stat(&self, con_key: &str, path: &WsPath) {
        self.stats.with_untracked(|stats| {
            let stat = stats.get(con_key);
            let Some(stat) = stat else {
                warn!("admin: con stat not found: {}", con_key);
                return;
            };
            let count = stat.count.get(path);
            let Some(count) = count else {
                warn!("admin: con count stat not found: {} {:?}", con_key, path);
                return;
            };
            count.update(|count| {
                *count += 1;
            });
        });
    }

    pub fn remove_live_stat(&self, con_key: &str) {
        self.stats.update(|stats| {
            let stat = stats.remove(con_key);
            if stat.is_some() {
                warn!("admin: live stat removed: {}", con_key);
            } else {
                warn!("admin: stat for removal not found: {}", con_key);
            }
        });
    }
}

pub fn use_ws_live_stats(ws: WsRuntime<ServerMsg, ClientMsg>, live_stats: LiveWsStats) {
    let ws_live_ws_stats = ws.channel().timeout(30).single_fire().start();

    ws_live_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::WsLiveStatsStarted(stats) => {
                    live_stats.set_live_stats(stats.clone());
                }
                ServerMsg::WsLiveStatsUpdateAddedStat { con_key, stat } => {
                    live_stats.add_live_stat(con_key.clone(), stat.clone().into());
                }
                ServerMsg::WsLiveStatsUpdateInc { con_key, path } => {
                    live_stats.inc_live_stat(con_key, path);
                }
                ServerMsg::WsLiveStatsUpdateRemoveStat { con_key } => {
                    live_stats.remove_live_stat(con_key);
                }
                ServerMsg::WsStatsPage(stats) => {}
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    ws_live_ws_stats
        .sender()
        .resend_on_reconnect()
        .on_cleanup(ClientMsg::LiveWsStats(false))
        .send(ClientMsg::LiveWsStats(true));
}