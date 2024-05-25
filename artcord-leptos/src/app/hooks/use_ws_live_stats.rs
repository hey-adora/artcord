use std::collections::HashMap;

use artcord_leptos_web_sockets::{channel::WsRecvResult, runtime::WsRuntime, WsRouteKey};
use artcord_state::{
    message::{
        prod_client_msg::{ClientMsg, ClientPathType},
        prod_server_msg::ServerMsg,
    },
    model::ws_statistics::{TempConIdType, WsStat},
};
use leptos::{
    RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalUpdateUntracked,
    SignalWithUntracked,
};
use tracing::trace;
use tracing::{debug, warn};

pub type WebStatPathType = HashMap<ClientPathType, WebWsStatPath>;

#[derive(Copy, Clone, Debug)]
pub struct LiveWsStats {
    pub stats: RwSignal<HashMap<TempConIdType, WebWsStat>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsStat {
    pub addr: String,
    pub count: RwSignal<WebStatPathType>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsStatPath {
    pub total_allowed: RwSignal<u64>,
    pub total_blocked: RwSignal<u64>,
    pub total_banned: RwSignal<u64>,
}

impl WebWsStatPath {
    pub fn new(allowed: u64, blocked: u64, banned: u64) -> Self {
        Self {
            total_allowed: RwSignal::new(allowed),
            total_blocked: RwSignal::new(blocked),
            total_banned: RwSignal::new(banned),
        }
    }
}

impl From<WsStat> for WebWsStat {
    fn from(value: WsStat) -> Self {
        let count_map =
            value
                .count
                .iter()
                .fold(WebStatPathType::new(), |mut prev, (key, value)| {
                    prev.insert(
                        *key,
                        WebWsStatPath::new(
                            value.total_allowed_count,
                            value.total_blocked_count,
                            value.total_banned_count,
                        ),
                    );
                    prev
                });
        WebWsStat {
            addr: value.addr.to_string(),
            count: RwSignal::new(count_map),
        }
    }
}

impl Default for LiveWsStats {
    fn default() -> Self {
        Self {
            stats: RwSignal::new(HashMap::new()),
        }
    }
}

impl LiveWsStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_con_stat(&self, con_stat: WsStat) {
        self.stats.update(|stats| {
            if let Some(stats) = stats.get(&con_stat.con_id) {
                for (new_path_index, new_path_count) in con_stat.count {
                    let updated = stats.count.with_untracked(|stat_path_count| {
                        let Some(path_count) = stat_path_count.get(&new_path_index) else {
                            return false;
                        };

                        if path_count.total_allowed.get_untracked()
                            != new_path_count.total_allowed_count
                        {
                            path_count
                                .total_allowed
                                .set(new_path_count.total_allowed_count);
                        }

                        if path_count.total_blocked.get_untracked()
                            != new_path_count.total_blocked_count
                        {
                            path_count
                                .total_blocked
                                .set(new_path_count.total_blocked_count);
                        }

                        if path_count.total_banned.get_untracked()
                            != new_path_count.total_banned_count
                        {
                            path_count
                                .total_banned
                                .set(new_path_count.total_banned_count);
                        }

                        true
                    });

                    if !updated {
                        stats.count.update(|stat_count| {
                            stat_count.insert(
                                new_path_index,
                                WebWsStatPath::new(
                                    new_path_count.total_allowed_count,
                                    new_path_count.total_blocked_count,
                                    new_path_count.total_banned_count,
                                ),
                            );
                        });
                    }
                }
            } else {
                stats.insert(con_stat.con_id, con_stat.into());
            }
        });
    }

    pub fn update_con_allowed_path(
        &self,
        con_id: TempConIdType,
        path: ClientPathType,
        amount: u64,
    ) {
        self.stats.with_untracked(|stats| {
            let Some(con) = stats.get(&con_id) else {
                debug!("ws live stats con not found: {}", con_id);
                return;
            };

            let updated = con.count.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, con_id);
                    return false;
                };

                path.total_allowed.set(amount);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.count.update(|paths| {
                        paths.insert(path, WebWsStatPath::new(amount, 0, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_con_blocked_path(
        &self,
        con_id: TempConIdType,
        path: ClientPathType,
        amount: u64,
    ) {
        self.stats.update_untracked(|stats| {
            let Some(con) = stats.get(&con_id) else {
                debug!("ws live stats con not found: {}", con_id);
                return;
            };

            let updated = con.count.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, con_id);
                    return false;
                };

                path.total_blocked.set(amount);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.count.update(|paths| {
                        paths.insert(path, WebWsStatPath::new(0, amount, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_con_banned_path(
        &self,
        con_id: TempConIdType,
        path: ClientPathType,
        amount: u64,
    ) {
        self.stats.update_untracked(|stats| {
            let Some(con) = stats.get(&con_id) else {
                debug!("ws live stats con not found: {}", con_id);
                return;
            };

            let updated = con.count.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, con_id);
                    return false;
                };

                path.total_banned.set(amount);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.count.update(|paths| {
                        paths.insert(path, WebWsStatPath::new(0, 0, amount));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }



    // pub fn set_live_stats(&self, new_stats: Vec<WsStat>) {
    //     //let mut web_stats: HashMap<TempConIdType, WebWsStat> = HashMap::with_capacity(stats.len());

    //     self.stats.update(|stats| {
    //         for new_stat in new_stats {
    //             let new_path = new_stat.con_id;
    //             let stat = stats.get(&new_path);
    //             let Some(stat) = stat else {
    //                 stats.insert(new_path, new_stat.into());
    //                 continue;
    //             };

    //         }
    //     });

    //     //self.stats.set(web_stats);
    // }

    // pub fn add_live_stat(&self, con_key: TempConIdType, stat: WebWsStat) {
    //     self.stats.update(move |stats| {
    //         stats.insert(con_key.clone(), stat.clone().into());
    //     });
    // }

    // pub fn inc_live_stat(&self, con_key: &TempConIdType, path: ClientPathType) {
    //     self.stats.with_untracked(|stats| {
    //         let stat = stats.get(con_key);
    //         let Some(stat) = stat else {
    //             warn!("admin: con stat not found: {}", con_key);
    //             return;
    //         };
    //         let updated = stat.count.with_untracked(|count| {
    //             let count = count.get(&path);
    //             let Some(count) = count else {
    //                 return false;
    //             };
    //             count.update(|count| {
    //                 trace!("admin: con count stat incremented: {} {:?}", con_key, path);
    //                 *count += 1;
    //             });
    //             true
    //         });

    //         if !updated {
    //             stat.count.update(|count| {
    //                 trace!("admin: con count stat inserted: {} {:?}", con_key, path);
    //                 count.insert(path, RwSignal::new(0));
    //             });
    //         }
    //     });
    // }

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
                ServerMsg::WsLiveStatsIpConnections(ip_stats) => {}
                ServerMsg::WsLiveStatsConnected(con_stat) => {
                    live_stats.update_con_stat(con_stat.clone());
                }
                ServerMsg::WsLiveStatsConReqAllowed {
                    con_id,
                    path,
                    total_amount,
                } => {
                    live_stats.update_con_allowed_path(*con_id, *path, *total_amount);
                }
                ServerMsg::WsLiveStatsConReqBlocked {
                    con_id,
                    path,
                    total_amount,
                } => {
                    live_stats.update_con_blocked_path(*con_id, *path, *total_amount);
                }
                ServerMsg::WsLiveStatsConReqBanned {
                    con_id,
                    path,
                    total_amount,
                } => {
                    live_stats.update_con_banned_path(*con_id, *path, *total_amount);
                }
                ServerMsg::WsLiveStatsDisconnected { con_id } => {
                    live_stats.remove_live_stat(con_id);
                }

                // ServerMsg::WsLiveStatsStarted(stats) => {
                //     live_stats.set_live_stats(stats.clone());
                // }
                // ServerMsg::WsLiveStatsUpdateAddedStat { con_key, stat } => {
                //     live_stats.add_live_stat(con_key.clone(), stat.clone().into());
                // }
                // ServerMsg::WsLiveStatsUpdateInc { con_key, path } => {
                //     live_stats.inc_live_stat(con_key, *path);
                // }
                // ServerMsg::WsLiveStatsUpdateRemoveStat { con_key } => {
                //     live_stats.remove_live_stat(con_key);
                // }
                ServerMsg::WsSavedStatsPage(stats) => {}
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
