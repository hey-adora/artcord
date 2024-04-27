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

pub const PAGE_AMOUNT: u64 = 100;

#[component]
pub fn WsOld() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    let ws_old_ws_stats = ws.channel().timeout(30).single_fire().start();
    let page = global_state.pages.admin;
    let old_ws_stats = page.old_connections;
    let pagination = page.old_connections_pagination;
    let active_page = page.old_connections_active_page;
    let loading = page.old_connections_loading;

    ws_old_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::WsStatsPage(stats) => {
                    page.set_old_stats(stats.clone(), None);
                }
                ServerMsg::WsStatsFirstPage {
                    total_count,
                    first_page,
                } => {
                    
                    page.set_old_stats(first_page.clone(), Some(*total_count));
                }
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    create_effect({
        let ws_old_ws_stats = ws_old_ws_stats.clone();
        move |_| {
            if old_ws_stats.with_untracked(move |stats| stats.len() > 0) {
                return;
            }
            ws_old_ws_stats
            .sender()
            .resend_on_reconnect()
            .send(ClientMsg::WsStatsFirstPage {
                amount: PAGE_AMOUNT,
            });
        }
    });
    

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
                    <th class="border border-mid-purple ">{count}</th>
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
                        <tr class="border border-mid-purple">
                            <td class="border border-mid-purple">{v.addr}</td>
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

    let pagination_on_click = move |page: u64| {
        ws_old_ws_stats
            .sender()
            .resend_on_reconnect()
            .send(ClientMsg::WsStatsPaged {
                page,
                amount: PAGE_AMOUNT,
            });
    };

    let pagination_view = move || {
        let pagination_on_click = pagination_on_click.clone();
        if let Some(count) = pagination.get() {
            (0..count).map( |i| {
                let pagination_on_click = pagination_on_click.clone();
                view!{<button 
                    on:click=move |_| {
                        if loading.get_untracked() {
                            return;
                        }
                        pagination_on_click(i);
                        active_page.set(i);
                        loading.set(true);
                    }
                    class=move || format!(" px-2 font-black border-2 {}", if active_page.get() == i { "bg-mid-purple border-transparent" } else { "border-mid-purple" } )>{i + 1}</button>
                }
            }).collect_view()
        } else {
            view! { <div>"Loading..."</div> }.into_view()
        }
    };

    view! {
        <div class="grid grid-rows-[auto_1fr_auto] overflow-y-hidden">
            <div>"WebSocket Connection History"</div>
            <div class="overflow-y-scroll overflow-x-auto">
                <Show when=move || !loading.get() fallback=move || view!{
                    <div class="h-full">"Loading..."</div>
                }>
                    <table class="border-spacing-5 border border-mid-purple ">
                        <tr class="sticky top-0 left-0 z-10 bg-mid-purple border border-mid-purple ">
                            <th>"ip"</th>
                            <WsPathTableHeaderView/>
                        </tr>
                        {move || old_connections_view()}
                    </table>
                </Show>
            </div>
            <div class="flex gap-1">
                { move || pagination_view() }
            </div>
        </div>
    }
}
