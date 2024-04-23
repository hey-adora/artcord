use std::collections::HashMap;
use std::time::Duration;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::AdminStatCountType;
use artcord_state::message::prod_server_msg::LiveWsStatsRes;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::message::prod_server_msg::WsStatTemp;
use artcord_state::model::ws_statistics;
use artcord_state::model::ws_statistics::ReqCount;
use artcord_state::model::ws_statistics::WsStat;
use leptos::*;
use leptos_router::use_params_map;
use leptos_use::use_interval_fn;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum::VariantArray;
use strum::VariantNames;
use tracing::debug;
use tracing::error;
use tracing::trace;
use tracing::warn;

use crate::app::components::navbar::Navbar;

use crate::app::global_state::GlobalState;

pub type WebAdminStatCountType = HashMap<WsPath, RwSignal<u64>>;

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsStat {
    pub addr: String,
    // pub is_connected: RwSignal<bool>,
    pub count: WebAdminStatCountType,
}

impl From<WsStatTemp> for WebWsStat {
    fn from(value: WsStatTemp) -> Self {
        let mut count_map: WebAdminStatCountType = HashMap::with_capacity(value.count.len());
        for path in WsPath::iter() {
            count_map.insert(
                path,
                RwSignal::new(value.count.get(&path).cloned().unwrap_or(0_u64)),
            );
        }
        // for (path, count) in value.count {
        //     count_map.insert(path, RwSignal::new(count));
        // }
        WebWsStat {
            addr: value.addr,
            // is_connected: RwSignal::new(true),
            count: count_map,
        }
    }
}

// impl From<&HashMap<WsPath, AdminStat>> for HashMap<String, WebAdminStat> {
//     fn from(value: &HashMap<WsPath, AdminStat>) -> Self {}
// }

#[derive(Copy, Clone, Debug)]
pub struct AdminPageState {
    pub live_connections: RwSignal<HashMap<String, WebWsStat>>,
    pub old_connections: RwSignal<Vec<WsStat>>,
}

impl AdminPageState {
    pub fn new() -> Self {
        Self {
            live_connections: RwSignal::new(HashMap::new()),
            old_connections: RwSignal::new(Vec::new()),
        }
    }

    pub fn set_old_stats(&self, stats: Vec<WsStat>) {
        // let mut web_stats: HashMap<String, WsStat> = HashMap::with_capacity(stats.len());
        // for (path, stat) in stats {
        //     web_stats.insert(path, stat.into());
        // }
        self.old_connections.set(stats);
    }

    pub fn set_live_stats(&self, stats: HashMap<String, WsStatTemp>) {
        let mut web_stats: HashMap<String, WebWsStat> = HashMap::with_capacity(stats.len());
        for (path, stat) in stats {
            web_stats.insert(path, stat.into());
        }
        self.live_connections.set(web_stats);
    }

    pub fn add_live_stat(&self, con_key: String, stat: WebWsStat) {
        self.live_connections.update(move |stats| {
            stats.insert(con_key.clone(), stat.clone().into());
        });
    }

    pub fn inc_live_stat(&self, con_key: &str, path: &WsPath) {
        self.live_connections.with_untracked(|stats| {
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
        self.live_connections.update(|stats| {
            let stat = stats.remove(con_key);
            if stat.is_some() {
                warn!("admin: live stat removed: {}", con_key);
            } else {
                warn!("admin: stat for removal not found: {}", con_key);
            }
        });
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    let _params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let ws = global_state.ws;
    let page = global_state.pages.admin;
    let live_ws_stats = page.live_connections;
    let old_ws_stats = page.old_connections;

    // let ws_statistics = ws.builder().portal().stream().build();
    let ws_live_ws_stats = ws.channel().timeout(30).single_fire().start();
    let ws_old_ws_stats = ws.channel().timeout(30).single_fire().start();

    ws_live_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::LiveWsStats(msg) => match msg {
                    LiveWsStatsRes::Started(stats) => {
                        page.set_live_stats(stats.clone());
                    }
                    LiveWsStatsRes::UpdateAddedStat { con_key, stat } => {
                        page.add_live_stat(con_key.clone(), stat.clone().into());
                    }
                    LiveWsStatsRes::UpdateInc { con_key, path } => {
                        page.inc_live_stat(con_key, path);
                    }
                    LiveWsStatsRes::UpdateRemoveStat { con_key } => {
                        page.remove_live_stat(con_key);
                    }
                    _ => {}
                },
                ServerMsg::WsStats(stats) => {}
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    ws_old_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::WsStats(stats) => {
                    page.set_old_stats(stats.clone());
                }
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    ws_live_ws_stats
        .sender()
        .resend_on_reconnect()
        .on_cleanup(ClientMsg::LiveWsStats(false))
        .send(ClientMsg::LiveWsStats(true));

    ws_old_ws_stats
        .sender()
        .resend_on_reconnect()
        .send(ClientMsg::WsStats);

    // ws_statistics.send_or_skip(Vgc, on_receive)
    create_effect(move |_| {
        nav_tran.set(true);
    });

    // spawn_local(async {
    //     debug!("ONE");
    //     async_std::task::sleep(Duration::from_secs(5)).await;
    //     debug!("TWO");
    // });

    // create_effect(move |_| {
    //     trace!("admin: sending to open admin throttle sender");
    //     ws_statistics.send_and_recv(ClientMsg::AdminThrottleListenerToggle(true), move |res| {
    //         match res {
    //             WsRecvResult::Ok(server_msg) => match server_msg {
    //                 ServerMsg::AdminStats(msg) => match msg {
    //                     AdminStatsRes::Started(stats) => {
    //                         statistics.set(stats);
    //                     }
    //                     AdminStatsRes::UpdateAddedNew { con_key, stat } => {
    //                         statistics.update(move |stats| {
    //                             stats.insert(con_key, stat);
    //                         });
    //                     }
    //                     _ => {}
    //                 },
    //                 _ => {}
    //             },
    //             WsRecvResult::TimeOut => {}
    //         }
    //         // trace!("admin: received: {:?}", res);
    //     });
    // });

    // on_cleanup(move || {
    //     trace!("admin: sending to close admin throttle sender");
    //     ws_statistics.send_and_recv(ClientMsg::AdminThrottleListenerToggle(false), |res| {
    //         trace!("admin: received: {:?}", res);
    //     });
    // });

    // create_effect(move |_| {
    //     use_interval_fn(
    //         move || {
    //             let result = ws_statistics.send_or_skip(
    //                 ClientMsg::Statistics,
    //                 move |server_msg: WsResourceResult<ServerMsg>| {
    //                     trace!("statistics: msg: {:?}", &server_msg);
    //                     match server_msg {
    //                         WsResourceResult::Ok(server_msg) => match server_msg {
    //                             ServerMsg::Statistics(stats) => {
    //                                 page.statistics.set(stats);
    //                             }
    //                             server_msg => {
    //                                 error!("statistics: wrong server response: {:?}", server_msg);
    //                             }
    //                         },
    //                         WsResourceResult::TimeOut => {
    //                             error!("statistics: timeout");
    //                         }
    //                     }
    //                 },
    //             );
    //             match result {
    //                 Ok(result) => {
    //                     trace!("statistics: send_result: {:?}", &result);
    //                 }
    //                 Err(err) => {
    //                     error!("statistics: {}", err);
    //                 }
    //             }
    //         },
    //         1000,
    //     );
    // });

    let table_header_view = move || {
        WsPath::VARIANTS
            .iter()
            .map(|v| {
                view! {

                    <th>{*v}</th>
                }
            })
            .collect_view()
        //
    };

    // let table_body_get_path = |stat: &AdminStat, v: &WsPath| -> String {
    //     // let path = WsPath::from_str(*v);
    //     // let Ok(path) = path else {
    //     //     return "0".to_string();
    //     // };
    //     stat.count.get(&path).cloned().unwrap_or(0_u64).to_string()
    // };

    let live_connection_count_view = move |count: WebAdminStatCountType| {
        WsPath::iter()
            .map(|path| {
                let count = count.get(&path).cloned();
                view! {
                    <th>{move || count.map(|count| count.get()).unwrap_or(0u64)}</th>
                }
            })
            .collect_view()
    };

    let live_connection_view = move || {
        view! {
            <For each=move || live_ws_stats.get().into_iter() key=|item| item.0.clone() let:item>
                <tr>
                    <td>{item.1.addr}</td>
                    { live_connection_count_view(item.1.count) }
                </tr>
            </For>
        }
        // statistics
        //     .get()
        //     .into_iter()
        //     .map(|(key, stat)| {
        //     })
        //     .collect_view()
    };

    let old_connections_count_view = move |count: Vec<ReqCount>| {
        // let count_iter = count.into_iter();
        <WsPath as VariantNames>::VARIANTS
            .into_iter()
            .map(|path| {
                // let path_str = path.
                let count = count
                    .iter()
                    .find(|v| v.path == *path)
                    .map(|v| v.count)
                    .unwrap_or(0_i64);
                view! {
                    <th>{count}</th>
                }
            })
            .collect_view()
    };

    let old_connections_view = move || {
        let list = old_ws_stats
            .get()
            .into_iter()
            .map(|v| {
                view! {
                        <tr>
                            <td>{v.addr}</td>
                            { old_connections_count_view(v.req_count) }
                            // { table_body_paths_view(item.1.count) }
                        </tr>
                }
            })
            .collect_view();
        view! {
            {
                list
            }
            // <For each=move || live_ws_stats.get().into_iter() key=|item| item.0.clone() let:item>
            //     <tr>
            //         <td>{item.1.addr}</td>
            //         { table_body_paths_view(item.1.count) }
            //     </tr>
            // </For>
        }
        // statistics
        //     .get()
        //     .into_iter()
        //     .map(|(key, stat)| {
        //     })
        //     .collect_view()
    };

    view! {
        <main class=move||format!("grid grid-rows-[1fr] h-[100dvh] overflow-y-hidden top-0 transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})
            // style:max-height=move || format!("{}", if nav_tran.get() { "calc(100dvh - 4rem)" } else { "calc(100dvh" })
        >
            <Navbar/>
            <div class="flex gap-4 bg-white  ">
                <div class="flex flex-col gap-4 bg-dark-night  px-6 py-4">
                    <div class="font-bold">"DASHBOARD"</div>
                    <div class="flex flex-col gap-2 ">
                        <div>"Activity"</div>
                        <div>"Banned IP's"</div>
                        <div>"Users"</div>
                    </div>
                </div>
                <div class="h-full overflow-y-hidden grid grid-rows-[auto_minmax(0,1fr)_1fr] text-black"
                    style:max-height="calc(100dvh - 4rem)"
                    >
                        <div class="font-bold">"Statistics"</div>
                        <div class="grid overflow-y-hidden grid-rows-[auto_1fr]">
                            <div>"Live WebSocket Connections"</div>
                            <div class="overflow-y-scroll ">
                                <table>
                                    <tr class="sticky top-0 left-0 bg-light-flower ">
                                        <th>"ip"</th>
                                        {move || table_header_view()}
                                        // <th>"one"</th>
                                        // <th>"two"</th>
                                        // <th>"three"</th>
                                    </tr>
                                    {move || live_connection_view()}
                                </table>

                            </div>
                        </div>
                        <div class="grid overflow-y-hidden grid-rows-[auto_1fr_auto] ">
                            <div>"WebSocket Connection History"</div>
                            <div class="overflow-y-scroll ">
                                <table class="">
                                    <tr class="sticky top-0 left-0 bg-light-flower ">
                                        <th>"ip"</th>
                                        {move || table_header_view()}
                                        // <th>"one"</th>
                                        // <th>"two"</th>
                                        // <th>"three"</th>
                                    </tr>
                                    {move || old_connections_view()}
                                </table>
                            </div>
                            <div class="flex gap-4">
                                <div>"1"</div>
                                <div>"2"</div>
                                <div>"3"</div>
                            </div>
                        </div>
                        // <div>"wowowowwowowowo"</div>
                    </div>

                // <div class="w-full   text-black py-4 gap-4 grid grid-rows-[1fr] "
                // style:max-height="calc(100dvh - 4rem)"
                //
                //     >
                // </div>
            </div>
        </main>
    }
}
