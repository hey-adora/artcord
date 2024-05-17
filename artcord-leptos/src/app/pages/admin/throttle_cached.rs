use std::collections::HashMap;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::{ClientMsg, ClientPathType};
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::misc::throttle_connection::{
    LiveThrottleConnectionCount, WebThrottleConnectionCount,
};
use artcord_state::model::ws_statistics::WebStatPathType;
use leptos::*;

use crate::app::hooks::use_ws_live_stats::use_ws_live_stats;
use crate::app::{
    global_state::GlobalState, hooks::use_ws_live_throttle_cache::use_ws_live_throttle_cache,
};

use super::WsPathTableHeaderView;
use strum::{EnumCount, IntoEnumIterator};

#[component]
pub fn ThrottleCached() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;

    let page = global_state.pages.admin;

    let live_throttle_cache = page.live_throttle_cache;
    use_ws_live_throttle_cache(ws, live_throttle_cache);
    //use_ws_live_stats(ws, live_stats);

    let live_connection_count_view =
        move |count: HashMap<ClientPathType, WebThrottleConnectionCount>| {
            (0..ClientMsg::COUNT)
                .map(|path| {
                    let count_view = move |item: Option<WebThrottleConnectionCount>| {
                        item
                            .map(|item_count| format!("{}", item_count.total_count.get()))
                            .unwrap_or(String::from("0"))
                    };
                    let item = count.get(&path).cloned();
              
                    view! {
                        <th>{ move || count_view(item.clone()) }</th>
                    }.into_view()
                })
                .collect_view()
        };

    // let live_connection_view = move || {
    //     view! {
            
    //     }
    // };

    view! {
        <div class="grid grid-rows-[auto_1fr] overflow-y-hidden">
            <div>"Throttle Cached"</div>
            <div class="overflow-y-scroll ">
                <table class="text-center">
                    <tr class="sticky top-0 left-0 bg-mid-purple  ">
                        <th>"ip"</th>
                        <th>"ConCount"</th>
                        <th>"WsTotalBlockedCons"</th>
                        <th>"WsBlockedCons"</th>
                        <th>"WsBlockedConsResetAt"</th>
                        <th>"WsConBannedUntil"</th>
                        <th>"WsConFlickerCount"</th>
                        <th>"WsConFlickerBannedUntil"</th>
                        <WsPathTableHeaderView/>
                    </tr>
                    <For each=move || live_throttle_cache.ips.get().into_iter() key=|item| item.0.clone() let:item>
                        <tr>
                            <td>{item.0.to_string()}</td>
                            <td>{move || item.1.ws_connection_count.get()}</td>
                            <td>{move || item.1.ws_total_blocked_connection_attempts.get()}</td>
                            <td>{move || item.1.ws_blocked_connection_attempts.get()}</td>
                            <td>{move || format!("{:?}", item.1.ws_blocked_connection_attempts_last_reset_at.get()) }</td>
                            <td>{move || item.1.ws_con_banned_until.get().map(|date| format!("{:?}", date)).unwrap_or("None".to_string())}</td>
                            <td>{move || item.1.ws_con_flicker_count.get()}</td>
                            <td>{move || item.1.ws_con_flicker_banned_until.get().map(|date| format!("{:?}", date)).unwrap_or("None".to_string())}</td>
                            { move || live_connection_count_view(item.1.ws_path_count.get()) }
                        </tr>
                    </For>
                </table>

            </div>
        </div>
    }
}
