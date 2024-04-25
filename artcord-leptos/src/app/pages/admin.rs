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
use leptos_router::Outlet;
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
use crate::app::utils::PageUrl;

pub mod overview;
pub mod ws_live;
pub mod ws_old;

pub type WebAdminStatCountType = HashMap<WsPath, RwSignal<u64>>;

#[component]
pub fn WsPathTableHeaderView() -> impl IntoView {
    WsPath::VARIANTS
        .iter()
        .map(|v| {
            view! {

                <th>{*v}</th>
            }
        })
        .collect_view()
}

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

    create_effect(move |_| {
        nav_tran.set(true);
    });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] h-[100dvh] overflow-y-hidden top-0 transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})
            // style:max-height=move || format!("{}", if nav_tran.get() { "calc(100dvh - 4rem)" } else { "calc(100dvh" })
        >
            <Navbar/>
            <div class="flex gap-4 bg-dark-night/90 p-4 ">
                <div class="flex flex-col gap-4 bg-dark-night  px-6 ">
                    <div class="font-bold">"DASHBOARD"</div>
                    <div class="flex flex-col gap-2 ">
                        <div>"Activity"</div>
                        <div>"Banned IP's"</div>
                        <div>"Users"</div>
                    </div>
                </div>
                <div class="h-full overflow-y-hidden grid gap-4 grid-rows-[auto_1fr] "
                    style:max-height="calc(100dvh - 4rem)"
                    >
                        <div class="font-bold text-lg text-white gap-4 flex  ">
                            <a href=PageUrl::url_dash() class="bg-mid-purple rounded-2xl px-4">"Overview"</a>
                            <a href=PageUrl::url_dash_wslive() class="bg-mid-purple rounded-2xl px-4">"WsLive"</a>
                            <a href=PageUrl::url_dash_wsold() class="border-white border-2 rounded-2xl px-4">"WsOld"</a>
                            // <a href="/" class="border-white border-2 rounded-2xl px-4">"Statistics"</a>
                        </div>
                        <Outlet/>
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
