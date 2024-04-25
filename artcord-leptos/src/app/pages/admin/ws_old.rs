use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::model::ws_statistics::ReqCount;
use leptos::*;
use strum::VariantNames;

use crate::app::global_state::GlobalState;

use super::WebAdminStatCountType;
use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;

#[component]
pub fn WsOld() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    let ws_old_ws_stats = ws.channel().timeout(30).single_fire().start();
    let page = global_state.pages.admin;
    let old_ws_stats = page.old_connections;

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

    ws_old_ws_stats
        .sender()
        .resend_on_reconnect()
        .send(ClientMsg::WsStats);

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
                        </tr>
                }
            })
            .collect_view();
        view! {
            {
                list
            }
        }
    };

    view! {
        <div class="grid overflow-y-hidden grid-rows-[auto_1fr_auto] ">
            <div>"WebSocket Connection History"</div>
            <div class="overflow-y-scroll ">
                <table class="">
                    <tr class="sticky top-0 left-0 bg-light-flower ">
                        <th>"ip"</th>
                        <WsPathTableHeaderView/>
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
    }
}
