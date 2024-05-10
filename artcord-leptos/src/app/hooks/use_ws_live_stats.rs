use std::collections::HashMap;

use artcord_leptos_web_sockets::{channel::WsRecvResult, runtime::WsRuntime};
use artcord_state::{message::{prod_client_msg::{ClientMsg, ClientMsgIndexType}, prod_server_msg::ServerMsg}, model::ws_statistics::{TempConIdType, WebWsStat, WsStatTemp}};
use leptos::{RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWithUntracked};
use tracing::warn;
use tracing::trace;

#[derive(Copy, Clone, Debug)]
pub struct LiveWsStats {
    pub stats: RwSignal<HashMap<TempConIdType, WebWsStat>>
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

    pub fn set_live_stats(&self, new_stats: HashMap<TempConIdType, WsStatTemp>) {
        //let mut web_stats: HashMap<TempConIdType, WebWsStat> = HashMap::with_capacity(stats.len());
     
        self.stats.update(|stats| {
            for (new_path, new_stat) in new_stats {
                let stat = stats.get(&new_path);
                let Some(stat) = stat else {
                    stats.insert(new_path, new_stat.into());
                    continue;
                };
                for (new_path_index, new_path_count) in new_stat.count {
                    let updated = stat.count.with_untracked(|stat_path_count| {
                        let Some(path_count) = stat_path_count.get(&new_path_index) else {
                            return false;
                        };

                        if path_count.get_untracked() != new_path_count.total_count {
                            path_count.set(new_path_count.total_count);
                        }

                        true
                    });
                    
                    if !updated {
                        stat.count.update(|stat_count| {
                            stat_count.insert(new_path_index, RwSignal::new(new_path_count.total_count));
                        });
                    }
                }
            }
        });
       
        //self.stats.set(web_stats);
    }

    pub fn add_live_stat(&self, con_key: TempConIdType, stat: WebWsStat) {
        self.stats.update(move |stats| {
            stats.insert(con_key.clone(), stat.clone().into());
        });
    }

    pub fn inc_live_stat(&self, con_key: &TempConIdType, path: ClientMsgIndexType) {
        self.stats.with_untracked(|stats| {
            let stat = stats.get(con_key);
            let Some(stat) = stat else {
                warn!("admin: con stat not found: {}", con_key);
                return;
            };
            let updated = stat.count.with_untracked(|count| {
                let count = count.get(&path);
                let Some(count) = count else {
                    return false;
                };
                count.update(|count| {
                    trace!("admin: con count stat incremented: {} {:?}", con_key, path);
                    *count += 1;
                });
                true
            });

            if !updated {
                stat.count.update(|count| {
                    trace!("admin: con count stat inserted: {} {:?}", con_key, path);
                    count.insert(path, RwSignal::new(0));
                });
            }
            
        });
    }

    pub fn remove_live_stat(&self, con_key: &TempConIdType) {
        self.stats.update(|stats| {
            let stat = stats.remove(con_key);
            if stat.is_some() {
                trace!("admin: live stat removed: {}", con_key);
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
                    live_stats.inc_live_stat(con_key, *path);
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