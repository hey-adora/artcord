use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::model::ws_statistics::WebAdminStatCountType;
use leptos::*;

use crate::app::global_state::GlobalState;
use crate::app::hooks::use_ws_live_stats::use_ws_live_stats;

use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;

#[component]
pub fn WsLive() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    
    let page = global_state.pages.admin;
    
    let live_stats = page.live_connections;
    use_ws_live_stats(ws, live_stats);

    

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
            <For each=move || live_stats.stats.get().into_iter() key=|item| item.0.clone() let:item>
                <tr>
                    <td>{item.1.addr}</td>
                    { live_connection_count_view(item.1.count) }
                </tr>
            </For>
        }
    };

    view! {
        <div class="grid grid-rows-[auto_1fr] overflow-y-hidden">
            <div>"Live WebSocket Connections"</div>
            <div class="overflow-y-scroll ">
                <table>
                    <tr class="sticky top-0 left-0 bg-mid-purple ">
                        <th>"ip"</th>
                        <WsPathTableHeaderView/>
                    </tr>
                    {move || live_connection_view()}
                </table>

            </div>
        </div>
    }
}
