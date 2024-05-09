use std::{collections::HashMap, net::IpAddr};

use artcord_leptos_web_sockets::{channel::WsRecvResult, runtime::WsRuntime};
use artcord_state::{message::{prod_client_msg::{ClientMsg, ClientMsgIndexType}, prod_server_msg::ServerMsg}, misc::throttle_connection::{LiveThrottleConnection, WebThrottleConnection, WebThrottleConnectionCount}, model::ws_statistics::{TempConIdType, WebWsStat, WsStatTemp}};
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
            if count <= 1 {
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

    pub fn on_start(&self, throttle_cache: HashMap<IpAddr, LiveThrottleConnection>) {
        self.ips.set(WebThrottleConnection::from_live(throttle_cache));
    }

    pub fn on_inc(&self, ip: &IpAddr, path_index: &ClientMsgIndexType) {
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
                ServerMsg::WsLiveThrottleCachedIncPath { ip, path } => {
                    live_throttle_cache.on_inc(ip, path);
                }
                ServerMsg::WsLiveThrottleCachedConnected { ip } => {
                    live_throttle_cache.on_con(ip);
                }
                ServerMsg::WsLiveThrottleCachedDisconnected { ip } => {
                    live_throttle_cache.on_disc(ip);
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