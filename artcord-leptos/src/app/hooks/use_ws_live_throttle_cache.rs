use std::{collections::HashMap, net::IpAddr};

use artcord_leptos_web_sockets::{channel::WsRecvResult, runtime::WsRuntime};
use artcord_state::{message::{prod_client_msg::{ClientMsg, ClientPathType}, prod_server_msg::ServerMsg}, misc::throttle_connection::{IpBanReason, TempThrottleConnection, WebThrottleConnection, WebThrottleConnectionCount}, model::ws_statistics::{TempConIdType, WebWsStat, WsStat}};
use chrono::{DateTime, Utc};
use leptos::{RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWithUntracked};
use tracing::warn;
use tracing::trace;
use tracing::debug;

#[derive(Copy, Clone, Debug)]
pub struct LiveThrottleCache {
    pub ips: RwSignal<HashMap<IpAddr, WebThrottleConnection>>
}

impl Default for LiveThrottleCache {
    fn default() -> Self {
        Self {
            ips: RwSignal::new(HashMap::new())
        }
    }
}

impl LiveThrottleCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_con(&self, ip: &IpAddr) {
        let updated = self.ips.with_untracked(|ips| {
            let Some(ip) = ips.get(ip) else {
                return false;
            };
            ip.ws_connection_count.update(|count| {
                *count += 1;
            });
            true
        });

        if !updated {
            self.ips.update(|ips| {
                ips.insert(*ip, WebThrottleConnection::new());
            });
        }
    }

    pub fn on_disc(&self, ip: &IpAddr) {
        let remove = self.ips.with_untracked(|ips| {
            let Some(ip) = ips.get(ip) else {
                return false;
            };
            let count = ip.ws_connection_count.get_untracked();
            if count < 1 {
                return true;
            }
            ip.ws_connection_count.update(|count| {
                *count -= 1;
            });
            false
        });

        if remove {
            self.ips.update(|ips| {
                ips.remove(ip);
            });
        }
    }

    pub fn on_start(&self, throttle_cache: HashMap<IpAddr, TempThrottleConnection>) {
        self.ips.update(|ips| {
            for (ip, new_con) in throttle_cache {
                if let Some(con) = ips.get(&ip) {
                    if con.ws_connection_count.get_untracked() != new_con.con_throttle.amount {
                        con.ws_connection_count.set(new_con.con_throttle.amount);
                    }
                    for (new_path_index, new_path_count) in new_con.ws_path_count {
                            let updated = con.ws_path_count.with_untracked(|con_path_count| {
                                let con_path_count = con_path_count.get(&new_path_index);
                                let Some(con_path_count) = con_path_count else {
                                    return false;
                                };
                                if con_path_count.total_count.get_untracked() != new_path_count.total_count {
                                    con_path_count.total_count.set(new_path_count.total_count);
                                }
                                if con_path_count.count.get_untracked() != new_path_count.count {
                                    con_path_count.count.set(new_path_count.count);
                                }
                                if con_path_count.last_reset_at.get_untracked() != new_path_count.last_reset_at {
                                    con_path_count.last_reset_at.set(new_path_count.last_reset_at);
                                }
                                true
                            });
                            if !updated {
                                con.ws_path_count.update(|con_path_count| {
                                    con_path_count.insert(new_path_index, new_path_count.into());
                                });
                            }
                    }
                    if con.ws_total_blocked_connection_attempts.get_untracked() != new_con.con_throttle.tracker.total_amount {
                        con.ws_total_blocked_connection_attempts.set(new_con.con_throttle.tracker.total_amount);
                    }
                    if con.ws_blocked_connection_attempts.get_untracked() != new_con.con_throttle.tracker.amount {
                        con.ws_blocked_connection_attempts.set(new_con.con_throttle.tracker.amount);
                    }
                    if con.ws_blocked_connection_attempts_last_reset_at.get_untracked() != new_con.con_throttle.tracker.started_at {
                        con.ws_blocked_connection_attempts_last_reset_at.set(new_con.con_throttle.tracker.started_at);
                    }
                    if con.ws_con_banned_until.get_untracked() != new_con.banned_until {
                        con.ws_con_banned_until.set(new_con.banned_until);
                    }
                    if con.ws_con_flicker_banned_until.get_untracked() != new_con.banned_until {
                        con.ws_con_flicker_banned_until.set(new_con.banned_until);
                    }
          
                    // for  in  {
                        
                    // }
                } else {
                    ips.insert(ip, new_con.into());
                }
            }
        });
        
        //self.ips.set(WebThrottleConnection::from_live(throttle_cache));
    }

    pub fn on_blocks(&self, ip: &IpAddr, total_blocks: u64, blocks: u64) {
        self.ips.with_untracked(|ips| {
            let con = ips.get(ip); 
            let Some(con) = con else {
                warn!("live throttle: ip not found!");
                return;
            };
            con.ws_total_blocked_connection_attempts.set(total_blocks);
            con.ws_blocked_connection_attempts.set(total_blocks);
        });
    }

    pub fn on_ban(&self, ip: &IpAddr, date: DateTime<Utc>, reason: IpBanReason) {
        self.ips.with_untracked(|ips| {
            let con = ips.get(ip); 
            let Some(con) = con else {
                warn!("live throttle: ip not found!");
                return;
            };
            con.ws_con_banned_until.set(Some((date, reason)));
        });
    }

    pub fn on_un_ban(&self, ip: &IpAddr) {
        self.ips.with_untracked(|ips| {
            let con = ips.get(ip); 
            let Some(con) = con else {
                warn!("live throttle: ip not found!");
                return;
            };
            con.ws_con_banned_until.set(None);
        });
    }

    pub fn on_inc(&self, ip: &IpAddr, path_index: &ClientPathType) {
        self.ips.with_untracked(|ips| {
            let con = ips.get(ip);
            let Some(con_paths) = con else {
                warn!("live throttle: ip not found {}", ip);
                return;
            };
            let updated = con_paths.ws_path_count.with_untracked(|paths| {
                let path = paths.get(path_index);
                let Some(path) = path else {
                    trace!("live throttle: path '{}' not found for {}", path_index, ip);
                    return false;
                };
                path.total_count.update(|total_count| {
                    *total_count += 1;
                });
                true
            });
            if !updated {
                con_paths.ws_path_count.update(|path_count| {
                    trace!("live throttle: path '{}' inserted for {}", path_index, ip);
                    path_count.insert(*path_index, WebThrottleConnectionCount::new());
                });
            }
        });
    }
}

pub fn use_ws_live_throttle_cache(ws: WsRuntime<ServerMsg, ClientMsg>, live_throttle_cache: LiveThrottleCache) {
    let ws_live_ws_stats = ws.channel().timeout(30).single_fire().start();

    ws_live_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::WsLiveThrottleCachedEntryAdded(stats) => {
                    live_throttle_cache.on_start(stats.clone());
                }
                ServerMsg::WsLiveThrottleCachedEntryUpdated(stats) => {
                    live_throttle_cache.on_start(stats.clone());
                }
                ServerMsg::WsLiveThrottleCachedIncPath { ip, path } => {
                    live_throttle_cache.on_inc(ip, path);
                }
                ServerMsg::WsLiveThrottleCachedConnected { ip } => {
                    live_throttle_cache.on_con(ip);
                }
                ServerMsg::WsLiveThrottleCachedDisconnected { ip } => {
                    live_throttle_cache.on_disc(ip);
                }
                ServerMsg::WsLiveThrottleCachedBlocks { ip, total_blocks, blocks } => {
                    live_throttle_cache.on_blocks(ip, *total_blocks, *blocks);
                }
                ServerMsg::WsLiveThrottleCachedBanned { ip, date, reason } => {
                    live_throttle_cache.on_ban(ip, *date, reason.clone());
                }
                ServerMsg::WsLiveThrottleCachedUnban { ip } => {
                    live_throttle_cache.on_un_ban(ip);
                }
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    ws_live_ws_stats
        .sender()
        .resend_on_reconnect()
        .on_cleanup(ClientMsg::LiveWsThrottleCache(false))
        .send(ClientMsg::LiveWsThrottleCache(true));
}