use std::collections::HashMap;
use std::time::Duration;

use leptos::html::U;
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
use artcord_state::global;

use crate::app::components::navbar::Navbar;

use crate::app::global_state::GlobalState;
use crate::app::hooks::use_ws_live_stats::LiveWsStats;
use crate::app::hooks::use_ws_live_throttle_cache::LiveThrottleCache;
use crate::app::utils::LoadingNotFound;
use crate::app::utils::PageUrl;

use self::ws_old::PAGE_AMOUNT;

pub mod overview;
pub mod ws_live;
pub mod ws_old;
pub mod throttle_cached;



#[component]
pub fn WsPathTableHeaderView() -> impl IntoView {
    global::ClientMsg::VARIANTS
        .iter()
        .map(|v| {
            view! {

                <th class="px-2">{*v}</th>
            }
        })
        .collect_view()
}



// impl From<&HashMap<WsPath, AdminStat>> for HashMap<String, WebAdminStat> {
//     fn from(value: &HashMap<WsPath, AdminStat>) -> Self {}
// }

pub enum AdminWsOldPageState {
    New,
    Fetch {
        page: u64,
        amount: u64,
        from: u64,
    },
    Refresh,
}

#[derive(Copy, Clone, Debug)]
pub struct AdminPageState {
    pub live_connections: LiveWsStats,
    pub live_throttle_cache: LiveThrottleCache,
    pub old_connections: RwSignal<Vec<global::DbWsCon>>,
    pub old_connections_pagination: RwSignal<Option<u64>>,
    pub old_connections_active_page: RwSignal<u64>,
    pub old_connections_loading: RwSignal<bool>,
    pub old_connections_loaded: RwSignal<Option<u64>>,
    pub old_connections_from: RwSignal<Option<i64>>,
    pub overview_state: RwSignal<LoadingNotFound>,
    pub overview_old_connections: RwSignal<Vec<global::DbWsCon>>,
    pub overview_selected_days: RwSignal<u64>,
    pub overview_selected_unique: RwSignal<bool>,
}

impl AdminPageState {
    pub fn new() -> Self {
        Self {
            live_connections: LiveWsStats::new(),
            live_throttle_cache: LiveThrottleCache::new(),
            old_connections: RwSignal::new(Vec::new()),
            old_connections_pagination: RwSignal::new(None),
            old_connections_active_page: RwSignal::new(0),
            old_connections_loading: RwSignal::new(false),
            old_connections_from: RwSignal::new(None),
            old_connections_loaded: RwSignal::new(None),
            overview_old_connections: RwSignal::new(Vec::new()),
            overview_selected_days: RwSignal::new(7),
            overview_selected_unique: RwSignal::new(true),
            overview_state: RwSignal::new(LoadingNotFound::NotLoaded),
        }
    }

    pub fn set_overview_old_stats(&self, stats: Vec<global::DbWsCon>) {
        self.overview_old_connections.set(stats);
    }

    pub fn set_old_stats_pagination(&self, pagination: u64) {
        self.old_connections_pagination.set(Some(pagination.div_ceil(PAGE_AMOUNT)));
    }

    pub fn set_old_stats_paged(&self, stats: Vec<global::DbWsCon>) {
        self.old_connections.set(stats);
        self.old_connections_loading.set(false);
        self.old_connections_loaded.set(Some(self.old_connections_active_page.get_untracked()));
    }

    pub fn set_old_stats_with_pagination(&self, total_count: u64, from: Option<i64>, stats: Vec<global::DbWsCon>) {
        // let mut web_stats: HashMap<String, WsStat> = HashMap::with_capacity(stats.len());
        // for (path, stat) in stats {
        //     web_stats.insert(path, stat.into());
        // }

        // if let Some(pagination) = pagination {
        //     self.old_connections_pagination.set(Some(pagination.div_ceil(PAGE_AMOUNT)));
        // }
      //  stats.fir
        // self.old_connections_from.set(Some(()))
        // if self.old_connections_from.with_untracked(|v|v.is_none()) {
        //     let from = stats.first().map(|v|v.created_at);
        // }
        self.old_connections_pagination.set(Some(total_count.div_ceil(PAGE_AMOUNT)));
        self.old_connections_from.set(from);
        self.old_connections.set(stats);
        self.old_connections_loading.set(false);
        self.old_connections_loaded.set(Some(self.old_connections_active_page.get_untracked()));
    }

   
}

#[component]
pub fn Admin() -> impl IntoView {
    let _params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let ws = global_state.ws;
    let page = global_state.pages.admin;
    let page_url = global_state.current_page_url;
    
    create_effect(move |_| {
        nav_tran.set(true);
    });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] h-[100dvh] top-0 transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})
            // style:max-height=move || format!("{}", if nav_tran.get() { "calc(100dvh - 4rem)" } else { "calc(100dvh" })
        >
            <Navbar/>
            <div class="grid grid-cols-[auto_1fr] gap-4 bg-dark-night/90 p-4 ">
                <div class="flex flex-col gap-4 bg-dark-night  px-6 ">
                    <div class="font-bold">"DASHBOARD"</div>
                    <div class="flex flex-col gap-2 ">
                        <div>"Activity"</div>
                        <div>"Banned IP's"</div>
                        <div>"Users"</div>
                    </div>
                </div>
                <div class="h-full overflow-y-hidden grid gap-4 grid-rows-[auto_1fr] "
                    style:max-height="calc(100dvh - 6rem)"
                    >
                        <div class="font-bold text-lg text-white gap-4 flex  ">
                            <a href=PageUrl::url_dash() class=move || format!(" rounded-2xl px-4 {}", if page_url.get() == PageUrl::AdminDash { "bg-mid-purple" } else { "border-white border-2" }) >"Overview"</a>
                            <a href=PageUrl::url_throttle_cached() class=move || format!(" rounded-2xl px-4 {}", if page_url.get() == PageUrl::AdminThrottleCached { "bg-mid-purple" } else { "border-white border-2" }) >"ThrottleCached"</a>
                            <a href=PageUrl::url_dash_wslive() class=move || format!(" rounded-2xl px-4 {}", if page_url.get() == PageUrl::AdminDashWsLive { "bg-mid-purple" } else { "border-white border-2" }) >"WsLive"</a>
                            <a href=PageUrl::url_dash_wsold() class=move || format!(" rounded-2xl px-4 {}", if page_url.get() == PageUrl::AdminDashWsOld { "bg-mid-purple" } else { "border-white border-2" }) >"WsOld"</a>
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
