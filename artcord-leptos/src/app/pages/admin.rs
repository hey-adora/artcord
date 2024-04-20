use std::collections::HashMap;
use std::time::Duration;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::AdminStat;
use artcord_state::message::prod_server_msg::AdminStatCountType;
use artcord_state::message::prod_server_msg::AdminStatsRes;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::model::statistics;
use artcord_state::model::statistics::Statistic;
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
pub struct WebAdminStat {
    pub addr: String,
    pub is_connected: RwSignal<bool>,
    pub count: WebAdminStatCountType,
}

impl From<AdminStat> for WebAdminStat {
    fn from(value: AdminStat) -> Self {
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
        WebAdminStat {
            addr: value.addr,
            is_connected: RwSignal::new(value.is_connected),
            count: count_map,
        }
    }
}

// impl From<&HashMap<WsPath, AdminStat>> for HashMap<String, WebAdminStat> {
//     fn from(value: &HashMap<WsPath, AdminStat>) -> Self {}
// }

#[derive(Copy, Clone, Debug)]
pub struct AdminPageState {
    pub statistics: RwSignal<HashMap<String, WebAdminStat>>,
}

impl AdminPageState {
    pub fn new() -> Self {
        Self {
            statistics: RwSignal::new(HashMap::new()),
        }
    }

    pub fn set_stats(&self, stats: HashMap<String, AdminStat>) {
        let mut web_stats: HashMap<String, WebAdminStat> = HashMap::with_capacity(stats.len());
        for (path, stat) in stats {
            web_stats.insert(path, stat.into());
        }
        self.statistics.set(web_stats);
    }

    pub fn inc_stat(&self, con_key: &str, path: &WsPath) {
        self.statistics.with_untracked(|stats| {
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
}

#[component]
pub fn Admin() -> impl IntoView {
    let _params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let ws = global_state.ws;
    let page = global_state.pages.admin;
    let statistics = page.statistics;

    // let ws_statistics = ws.builder().portal().stream().build();
    let ws_statistics = ws.channel().timeout(30).start();

    ws_statistics
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::AdminStats(msg) => match msg {
                    AdminStatsRes::Started(stats) => {
                        page.set_stats(stats.clone());
                    }
                    AdminStatsRes::UpdateAddedNew { con_key, stat } => {
                        statistics.update(move |stats| {
                            stats.insert(con_key.clone(), stat.clone().into());
                        });
                    }
                    AdminStatsRes::UpdateInc { con_key, path } => {
                        page.inc_stat(con_key, path);
                    }
                    _ => {}
                },
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    ws_statistics
        .sender()
        .resend_on_reconnect()
        .on_cleanup(ClientMsg::AdminThrottleListenerToggle(false))
        .send(ClientMsg::AdminThrottleListenerToggle(true));

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

    let table_body_paths_view = move |count: WebAdminStatCountType| {
        WsPath::iter()
            .map(|path| {
                let count = count.get(&path).cloned();
                view! {
                    <th>{move || count.map(|count| count.get()).unwrap_or(0u64)}</th>
                }
            })
            .collect_view()
    };

    let table_body_view = move || {
        view! {
            <For each=move || statistics.get().into_iter() key=|item| item.0.clone() let:item>
                <tr>
                    <td>{item.1.addr}</td>
                    { table_body_paths_view(item.1.count) }
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

    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})>
            <Navbar/>
            <div class="flex gap-4 bg-white ">
                <div class="flex flex-col gap-4 bg-dark-night  px-6 py-4">
                    <div class="font-bold">"DASHBOARD"</div>
                    <div class="flex flex-col gap-2 ">
                        <div>"Activity"</div>
                        <div>"Banned IP's"</div>
                        <div>"Users"</div>
                    </div>
                </div>
                <div class="w-full text-black py-4 gap-4 flex  flex-col  ">
                    <div class="font-bold">"Activity"</div>
                    <div>"Activity"</div>
                    <table>
                        <tr class="sticky top-[4rem] left-0 bg-light-flower ">
                            <th>"ip"</th>
                            {move || table_header_view()}
                            // <th>"one"</th>
                            // <th>"two"</th>
                            // <th>"three"</th>
                        </tr>

                        {move || table_body_view()}

                    </table>
                </div>
            </div>
        </main>
    }
}
