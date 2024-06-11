use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
};

use artcord_leptos_web_sockets::{channel::WsRecvResult, runtime::WsRuntime, WsRouteKey};
use chrono::{DateTime, Utc};
use leptos::{
    create_effect, on_cleanup, RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate,
    SignalUpdateUntracked, SignalWithUntracked,
};
use tracing::trace;
use tracing::{debug, warn};
use artcord_state::global;

pub type WebStatPathType = HashMap<global::ClientPathType, WebWsConReqStat>;

#[derive(Copy, Clone, Debug)]
pub struct LiveWsStats {
    pub stats: RwSignal<HashMap<global::TempConIdType, WebWsCon>>,
    pub ip_stats: RwSignal<HashMap<IpAddr, WebWsIpStat>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsCon {
    pub ip: IpAddr,
    pub addr: SocketAddr,
    pub paths: RwSignal<WebStatPathType>,
    pub banned_until: RwSignal<Option<(DateTime<Utc>, global::IpBanReason)>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsConReqStat {
    pub total_allowed: RwSignal<u64>,
    pub total_blocked: RwSignal<u64>,
    pub total_banned: RwSignal<u64>,
    pub total_already_banned: RwSignal<u64>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsIpStat {
    pub con_count: RwSignal<u64>,
    pub banned_until: RwSignal<Option<(DateTime<Utc>, global::IpBanReason)>>,
    //pub totals: RwSignal< HashMap< TempConIdType, u64 > >,
    pub total_allowed: RwSignal<u64>,
    pub total_blocked: RwSignal<u64>,
    pub total_banned: RwSignal<u64>,
    pub total_already_banned: RwSignal<u64>,
    pub paths: RwSignal<HashMap<global::ClientPathType, WebWsConReqStat>>,
}

// #[derive(Debug, PartialEq, Clone)]
// pub struct WebWsStatIpPath {
//     pub total_allowed: RwSignal<HashMap<TempConIdType, u64>>,
//     pub total_blocked: RwSignal<HashMap<TempConIdType, u64>>,
//     pub total_banned: RwSignal<HashMap<TempConIdType, u64>>,
//     pub total_already_banned: RwSignal<HashMap<TempConIdType, u64>>,
// }

impl WebWsIpStat {
    pub fn new(
        con_count: u64,
        total_allowed: u64,
        total_blocked: u64,
        total_banned: u64,
        total_already_banned: u64,
        banned_until: Option<(DateTime<Utc>, global::IpBanReason)>,
    ) -> Self {
        Self {
            con_count: RwSignal::new(con_count),
            banned_until: RwSignal::new(banned_until),
            total_allowed: RwSignal::new(total_allowed),
            total_blocked: RwSignal::new(total_blocked),
            total_banned: RwSignal::new(total_banned),
            total_already_banned: RwSignal::new(total_already_banned),
            paths: RwSignal::new(HashMap::new()),
        }
    }
}

// impl WebWsIpStat {
//     pub fn new(con_count: u64, con_id: Option<TempConIdType>, total_allowed: u64, total_blocked: u64, total_banned: u64, total_already_banned: u64 , banned_until: Option<(DateTime<Utc>, IpBanReason)> ) -> Self {
//         let make_hashmap = |total: u64| -> HashMap<TempConIdType, u64> {
//             con_id.map(|con_id| {
//                 let mut map: HashMap<TempConIdType, u64> = HashMap::new();
//                 map.insert(con_id, total);
//                 map
//              }).unwrap_or_default()
//         };

//         Self {
//             con_count: RwSignal::new(con_count),
//             banned_until: RwSignal::new(banned_until),
//             total_allowed: RwSignal::new(make_hashmap(total_allowed)),
//             total_blocked: RwSignal::new(make_hashmap(total_blocked) ),
//             total_banned: RwSignal::new(make_hashmap(total_banned) ),
//             total_already_banned: RwSignal::new(make_hashmap(total_already_banned) ),
//         }
//     }
// }

impl WebWsConReqStat {
    pub fn new(allowed: u64, blocked: u64, banned: u64, already_banned: u64) -> Self {
        Self {
            total_allowed: RwSignal::new(allowed),
            total_blocked: RwSignal::new(blocked),
            total_banned: RwSignal::new(banned),
            total_already_banned: RwSignal::new(already_banned),
        }
    }
}

impl WebWsCon {
    pub fn from_msg(ip: IpAddr, addr: SocketAddr, banned_until: Option<(DateTime<Utc>, global::IpBanReason)>, req_stats: HashMap<global::ClientPathType, global::WsConReqStat>) -> WebWsCon {
        let count_map =
        req_stats
                .iter()
                .fold(WebStatPathType::new(), |mut prev, (key, value)| {
                    prev.insert(
                        *key,
                        WebWsConReqStat::new(
                            value.total_allowed_count,
                            value.total_blocked_count,
                            value.total_banned_count,
                            value.total_already_banned_count,
                        ),
                    );
                    prev
                });
        WebWsCon {
            ip,
            addr,
            paths: RwSignal::new(count_map),
            banned_until: RwSignal::new(banned_until),
        }
    }
}

// impl From<ReqStat> for WebWsStat {
//     fn from(value: ReqStat) -> Self {

//     }
// }

impl Default for LiveWsStats {
    fn default() -> Self {
        Self {
            stats: RwSignal::new(HashMap::new()),
            ip_stats: RwSignal::new(HashMap::new()),
        }
    }
}

impl LiveWsStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_con_stat(&self,  ip: IpAddr, addr: SocketAddr, con_id: global::TempConIdType, banned_until: Option<(DateTime<Utc>, global::IpBanReason)>, new_paths: HashMap<global::ClientPathType, global::WsConReqStat>) {
        let new_con_id = &con_id;
        let new_ip = &ip;
        let new_paths = &new_paths;
        self.stats.update(|stats| {
            if let Some(stats) = stats.get(new_con_id) {
                for (new_path_index, new_path_count) in new_paths {
                    let updated = stats.paths.with_untracked(|stat_path_count| {
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
                        stats.paths.update(|stat_count| {
                            stat_count.insert(
                                *new_path_index,
                                WebWsConReqStat::new(
                                    new_path_count.total_allowed_count,
                                    new_path_count.total_blocked_count,
                                    new_path_count.total_banned_count,
                                    new_path_count.total_already_banned_count,
                                ),
                            );
                        });
                    }
                }
            } else {
                stats.insert(*new_con_id, WebWsCon::from_msg(ip, addr, banned_until, new_paths.clone()) );
            }
        });
        self.ip_stats.with_untracked(|stats| {
            if let Some(stat) = stats.get(new_ip) {
                let mut add_these = vec![];
                stat.paths.with_untracked(|current_paths| {
                    for (new_path, new_path_stat) in new_paths {
                        let Some(current_path) = current_paths.get(&new_path) else {
                            let stat = WebWsConReqStat::new(
                                new_path_stat.total_allowed_count,
                                new_path_stat.total_blocked_count,
                                new_path_stat.total_banned_count,
                                new_path_stat.total_already_banned_count,
                            );

                            add_these.push((new_path, stat));

                            continue;
                        };
                        current_path
                            .total_allowed
                            .update(|amount| *amount += new_path_stat.total_allowed_count);
                        current_path
                            .total_blocked
                            .update(|amount| *amount += new_path_stat.total_blocked_count);
                        current_path
                            .total_banned
                            .update(|amount| *amount += new_path_stat.total_banned_count);
                        current_path
                            .total_already_banned
                            .update(|amount| *amount += new_path_stat.total_already_banned_count);
                    }
                });
                if !add_these.is_empty() {
                    stat.paths.update(|current_paths| {
                        for (path, stat) in add_these {
                            current_paths.insert(*path, stat);
                        }
                    });
                }
            } else {
                warn!("ip not found");
            }
        });
    }

    pub fn ban_ip(&self, ip: IpAddr, date: DateTime<Utc>, reason: global::IpBanReason) {
        self.stats.with_untracked(|stats| {
            for (_, stat) in stats {
                if stat.ip == ip {
                    stat.banned_until.set(Some((date, reason.clone())));
                }
            }
        });
        self.ip_stats.with_untracked(|stats| {
            let stat = stats.get(&ip);
            let Some(stat) = stat else {
                debug!("ip_stat missing {}", ip);
                return;
            };
            stat.banned_until.set(Some((date, reason)));
        });
    }

    pub fn unban_ip(&self, ip: IpAddr) {
        self.stats.with_untracked(|stats| {
            for (_, stat) in stats {
                if stat.ip == ip {
                    stat.banned_until.set(None);
                }
            }
        });
        self.ip_stats.with_untracked(|stats| {
            let stat = stats.get(&ip);
            let Some(stat) = stat else {
                debug!("ip_stat missing {}", ip);
                return;
            };
            stat.banned_until.set(None);
        });
    }

    pub fn update_con_allowed_path(
        &self,
        con_id: global::TempConIdType,
        path: global::ClientPathType,
        amount: u64,
    ) {
        self.stats.with_untracked(|stats| {
            let Some(con) = stats.get(&con_id) else {
                debug!("ws live stats con not found: {}", con_id);
                return;
            };

            let updated = con.paths.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, con_id);
                    return false;
                };

                path.total_allowed.set(amount);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.paths.update(|paths| {
                        paths.insert(path, WebWsConReqStat::new(amount, 0, 0, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_con_blocked_path(
        &self,
        con_id: global::TempConIdType,
        path: global::ClientPathType,
        amount: u64,
    ) {
        self.stats.update_untracked(|stats| {
            let Some(con) = stats.get(&con_id) else {
                debug!("ws live stats con not found: {}", con_id);
                return;
            };

            let updated = con.paths.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, con_id);
                    return false;
                };

                path.total_blocked.set(amount);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.paths.update(|paths| {
                        paths.insert(path, WebWsConReqStat::new(0, amount, 0, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_con_banned_path(&self, con_id: global::TempConIdType, path: global::ClientPathType, amount: u64) {
        self.stats.update_untracked(|stats| {
            let Some(con) = stats.get(&con_id) else {
                debug!("ws live stats con not found: {}", con_id);
                return;
            };

            let updated = con.paths.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, con_id);
                    return false;
                };

                path.total_banned.set(amount);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.paths.update(|paths| {
                        paths.insert(path, WebWsConReqStat::new(0, 0, amount, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_ip_allowed_path(&self, ip: IpAddr, path: global::ClientPathType) {
        self.ip_stats.with_untracked(|stats| {
            let Some(con) = stats.get(&ip) else {
                debug!("ws live stats ip not found: {}", ip);
                return;
            };

            let updated = con.paths.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, ip);
                    return false;
                };

                path.total_allowed.update(|amount| *amount += 1);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.paths.update(|paths| {
                        paths.insert(path, WebWsConReqStat::new(1, 0, 0, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_ip_blocked_path(&self, ip: IpAddr, path: global::ClientPathType) {
        self.ip_stats.with_untracked(|stats| {
            let Some(con) = stats.get(&ip) else {
                debug!("ws live stats ip not found: {}", ip);
                return;
            };

            let updated = con.paths.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, ip);
                    return false;
                };

                path.total_blocked.update(|amount| *amount += 1);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.paths.update(|paths| {
                        paths.insert(path, WebWsConReqStat::new(0, 1, 0, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_ip_banned_path(&self, ip: IpAddr, path: global::ClientPathType) {
        self.ip_stats.with_untracked(|stats| {
            let Some(con) = stats.get(&ip) else {
                debug!("ws live stats ip not found: {}", ip);
                return;
            };

            let updated = con.paths.try_with_untracked(|paths| {
                let Some(path) = paths.get(&path) else {
                    debug!("ws live stats path '{}' not found for {}", path, ip);
                    return false;
                };

                path.total_banned.update(|amount| *amount += 1);
                true
            });
            if let Some(updated) = updated {
                if !updated {
                    con.paths.update(|paths| {
                        paths.insert(path, WebWsConReqStat::new(0, 0, 1, 0));
                    });
                }
            } else {
                debug!("should have returned bool???");
            }
        });
    }

    pub fn update_connections(&self, ip_stats: Vec<global::ConnectedWsIp>) {
        self.ip_stats.update(|stats| {
            for ip_stat in ip_stats {
                let Some(stat) = stats.get(&ip_stat.ip) else {
                    stats.insert(
                        ip_stat.ip,
                        WebWsIpStat::new(
                            0,
                            ip_stat.total_allow_amount,
                            ip_stat.total_block_amount,
                            ip_stat.total_banned_amount,
                            ip_stat.total_already_banned_amount,
                            ip_stat.banned_until,
                        ),
                    );
                    return;
                };
                if stat.total_allowed.get_untracked() == ip_stat.total_allow_amount {
                    stat.total_allowed.set(ip_stat.total_allow_amount);
                }
                if stat.total_blocked.get_untracked() == ip_stat.total_block_amount {
                    stat.total_blocked.set(ip_stat.total_block_amount);
                }
                if stat.total_banned.get_untracked() == ip_stat.total_banned_amount {
                    stat.total_banned.set(ip_stat.total_banned_amount);
                }
                if stat.total_already_banned.get_untracked() == ip_stat.total_block_amount {
                    stat.total_already_banned
                        .set(ip_stat.total_already_banned_amount);
                }
                if stat.banned_until.get_untracked() == ip_stat.banned_until {
                    stat.banned_until.set(ip_stat.banned_until);
                }
                // stat.paths.with_untracked(|paths| {
                //     for (path, stat) in paths {
                //         stat.total_allowed.set(0);
                //         stat.total_blocked.set(0);
                //         stat.total_banned.set(0);
                //         stat.total_already_banned.set(0);
                //     }
                // });
            }
        });
    }

    pub fn inc_ip_con_count(&self, ip: IpAddr) {
        let updated = self.ip_stats.try_with_untracked(|stats| {
            let Some(stat) = stats.get(&ip) else {
                return false;
            };
            stat.con_count.update(|count| *count += 1);
            true
        });
        let Some(updated) = updated else {
            warn!("expected Some() result");
            return;
        };
        if !updated {
            self.ip_stats.update(|stats| {
                stats.insert(ip, WebWsIpStat::new(1, 0, 0, 0, 0, None));
            });
        }
    }

    pub fn dec_ip_con_count(&self, ip: IpAddr) {
        let updated = self.ip_stats.try_with_untracked(|stats| {
            let Some(stat) = stats.get(&ip) else {
                return false;
            };
            stat.con_count.update(|count| {
                *count = count.checked_sub(1).unwrap_or_else(|| {
                    warn!("underflow detected in ws_live dec con");
                    0
                })
            });
            true
        });
        let Some(updated) = updated else {
            warn!("expected Some() result");
            return;
        };
        if !updated {
            self.ip_stats.update(|stats| {
                stats.insert(ip, WebWsIpStat::new(0, 0, 0, 0, 0, None));
            });
        }
    }

    pub fn inc_ip_path_count(&self, ip: IpAddr, path: global::ClientPathType) {
        let updated = self.ip_stats.try_with_untracked(|stats| {
            let Some(stat) = stats.get(&ip) else {
                return false;
            };
            stat.con_count.update(|count| *count += 1);
            true
        });
        let Some(updated) = updated else {
            warn!("expected Some() result");
            return;
        };
        if !updated {
            self.ip_stats.update(|stats| {
                stats.insert(ip, WebWsIpStat::new(1, 0, 0, 0, 0, None));
            });
        }
    }
    pub fn update_ip_allowed_count(&self, ip: IpAddr, total_amount: u64) {
        self.ip_stats.with_untracked(|stats| {
            let Some(stat) = stats.get(&ip) else {
                debug!("ip not found");
                return;
            };
            stat.total_allowed.set(total_amount);
        });
    }

    pub fn update_ip_blocked_count(&self, ip: IpAddr, total_amount: u64) {
        self.ip_stats.with_untracked(|stats| {
            let Some(stat) = stats.get(&ip) else {
                debug!("ip not found");
                return;
            };
            stat.total_blocked.set(total_amount);
        });
    }

    pub fn update_ip_banned_count(&self, ip: IpAddr, total_amount: u64) {
        self.ip_stats.with_untracked(|stats| {
            let Some(stat) = stats.get(&ip) else {
                debug!("ip not found");
                return;
            };
            stat.total_banned.set(total_amount);
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

    pub fn remove_live_stat(&self, con_key: &global::TempConIdType) {
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

pub fn use_ws_live_stats(ws: WsRuntime<global::ServerMsg, global::ClientMsg>, live_stats: LiveWsStats) {
    let ws_live_ws_stats = ws.channel().timeout(30).single_fire().start();

    ws_live_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                global::ServerMsg::WsLiveStatsIpCons(ip_stats) => {
                    live_stats.update_connections(ip_stats.clone());
                }
                global::ServerMsg::WsLiveStatsConnected { ip, socket_addr, con_id, banned_until, req_stats: req_stat } => {
                    live_stats.update_con_stat(*ip, *socket_addr, *con_id, banned_until.clone(), req_stat.clone());
                    live_stats.inc_ip_con_count(*ip);
                }
                global::ServerMsg::WsLiveStatsReqAllowed {
                    con_id,
                    path,
                    total_amount,
                } => {
                    let Some(ip) = live_stats
                        .stats
                        .with_untracked(|stats| stats.get(con_id).map(|stat| stat.ip))
                    else {
                        warn!("con_id not found for WsLiveStatsConReqAllowed: {}", con_id);
                        return;
                    };
                    live_stats.update_con_allowed_path(*con_id, *path, *total_amount);
                    live_stats.update_ip_allowed_path(ip, *path);
                }
                global::ServerMsg::WsLiveStatsReqBlocked {
                    con_id,
                    path,
                    total_amount,
                } => {
                    let Some(ip) = live_stats
                        .stats
                        .with_untracked(|stats| stats.get(con_id).map(|stat| stat.ip))
                    else {
                        warn!("con_id not found for WsLiveStatsConReqBlocked: {}", con_id);
                        return;
                    };
                    live_stats.update_con_blocked_path(*con_id, *path, *total_amount);
                    live_stats.update_ip_blocked_path(ip, *path);
                }
                global::ServerMsg::WsLiveStatsReqBanned {
                    con_id,
                    path,
                    total_amount,
                } => {
                    let Some(ip) = live_stats
                        .stats
                        .with_untracked(|stats| stats.get(con_id).map(|stat| stat.ip))
                    else {
                        warn!("con_id not found for WsLiveStatsConReqBanned: {}", con_id);
                        return;
                    };
                    live_stats.update_con_banned_path(*con_id, *path, *total_amount);
                    live_stats.update_ip_banned_path(ip, *path);
                }
                global::ServerMsg::WsLiveStatsDisconnected { con_id } => {
                    let Some(ip) = live_stats
                        .stats
                        .with_untracked(|stats| stats.get(con_id).map(|stat| stat.ip))
                    else {
                        warn!("con_id not found for WsLiveStatsDisconnected: {}", con_id);
                        return;
                    };
                    live_stats.remove_live_stat(con_id);
                    live_stats.dec_ip_con_count(ip);
                }
                global::ServerMsg::WsLiveStatsIpBanned { ip, date, reason } => {
                    live_stats.ban_ip(*ip, *date, reason.clone());
                }
                global::ServerMsg::WsLiveStatsIpUnbanned { ip } => {
                    live_stats.unban_ip(*ip);
                }
                global::ServerMsg::WsLiveStatsConAllowed { ip, total_amount } => {
                    live_stats.update_ip_allowed_count(*ip, *total_amount);
                }
                global::ServerMsg::WsLiveStatsConBlocked { ip, total_amount } => {
                    live_stats.update_ip_blocked_count(*ip, *total_amount);
                }
                global::ServerMsg::WsLiveStatsConBanned { ip, total_amount } => {
                    live_stats.update_ip_banned_count(*ip, *total_amount);
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
                global::ServerMsg::WsSavedStatsPage(stats) => {}
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    on_cleanup({
        let ws_live_ws_stats = ws_live_ws_stats.clone();
        move || {
            ws_live_ws_stats
                .sender()
                .send(global::ClientMsg::LiveWsStats(false));
        }
    });

    create_effect(move |_| {
        let _ = ws.connected.get();
        live_stats.ip_stats.update(|stats| stats.clear());
        live_stats.stats.update(|stats| stats.clear());
        ws_live_ws_stats.sender().send(global::ClientMsg::LiveWsStats(true));

        // ws_live_ws_stats
        //     .sender()
        //     .resend_on_reconnect()
        //     .on_cleanup(ClientMsg::LiveWsStats(false))
        //     .send(ClientMsg::LiveWsStats(true));
    });
}
