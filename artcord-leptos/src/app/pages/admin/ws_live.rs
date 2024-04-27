use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::ServerMsg;
use leptos::*;

use crate::app::global_state::GlobalState;

use super::WebAdminStatCountType;
use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;

#[component]
pub fn WsLive() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    let ws_live_ws_stats = ws.channel().timeout(30).single_fire().start();
    let page = global_state.pages.admin;
    let live_ws_stats = page.live_connections;

    ws_live_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::WsLiveStatsStarted(stats) => {
                    page.set_live_stats(stats.clone());
                }
                ServerMsg::WsLiveStatsUpdateAddedStat { con_key, stat } => {
                    page.add_live_stat(con_key.clone(), stat.clone().into());
                }
                ServerMsg::WsLiveStatsUpdateInc { con_key, path } => {
                    page.inc_live_stat(con_key, path);
                }
                ServerMsg::WsLiveStatsUpdateRemoveStat { con_key } => {
                    page.remove_live_stat(con_key);
                }
                ServerMsg::WsStatsPage(stats) => {}
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    ws_live_ws_stats
        .sender()
        .resend_on_reconnect()
        .on_cleanup(ClientMsg::LiveWsStats(false))
        .send(ClientMsg::LiveWsStats(true));

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
    };

    view! {
        <div class="grid grid-rows-[auto_1fr] overflow-y-hidden">
            <div>"Live WebSocket Connections"</div>
            <div class="overflow-y-scroll ">
                <table>
                    <tr class="sticky top-0 left-0 bg-light-flower ">
                        <th>"ip"</th>
                        <WsPathTableHeaderView/>
                    </tr>
                    {move || live_connection_view()}
                </table>

            </div>
        </div>
    }
}
