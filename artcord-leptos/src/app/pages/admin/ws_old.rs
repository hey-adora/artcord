use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::model::ws_statistics::ReqCount;
use leptos::*;
use leptos_router::use_params_map;
use leptos_router::use_query_map;
use strum::VariantNames;
use tracing::error;

use crate::app::global_state::GlobalState;
use crate::app::utils::PageUrl;

use super::WebAdminStatCountType;
use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;
use tracing::trace;

pub const PAGE_AMOUNT: u64 = 10;

#[component]
pub fn WsOld() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    let ws_old_ws_stats = ws.channel().timeout(30).single_fire().start();
    let page = global_state.pages.admin;
    let old_ws_stats = page.old_connections;
    let pagination = page.old_connections_pagination;
    //let active_page = page.old_connections_active_page;
    let loading = page.old_connections_loading;
    let query = use_query_map();

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

    // create_effect({
    //     let ws_old_ws_stats = ws_old_ws_stats.clone();
    //     move |_| {

    //     }
    // });

    create_effect({
        let ws_old_ws_stats = ws_old_ws_stats.clone();
        move |_| {
            let page = query.with(move |query| query.get("p").cloned());
            let Some(page) = page else {
                if old_ws_stats.with_untracked(move |stats| stats.len() > 0) {
                    return;
                }
                ws_old_ws_stats
                    .sender()
                    .resend_on_reconnect()
                    .send(ClientMsg::WsStatsFirstPage {
                        amount: PAGE_AMOUNT,
                    });
                return;
            };

            let page = match u64::from_str_radix(&page, 10) {
                Ok(page) => page,
                Err(err) => {
                    error!("error parsing page '{}': {}", page, err);
                    return;
                }
            };

            ws_old_ws_stats
                .sender()
                .resend_on_reconnect()
                .send(ClientMsg::WsStatsPaged {
                    page,
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

    // let pagination_on_click = move |page: u64| {

    // };

    let pagination_view = move || {
        //let pagination_on_click = pagination_on_click.clone();
        let active_page = query
            .with(move |query| query.get("p").cloned())
            .map(|v| u64::from_str_radix(&v, 10).unwrap_or(0))
            .unwrap_or(0);
        if let Some(count) = pagination.get() {
            let PAEG_SIZE: u64 = 4;
            let (first_page, last_page) = if count > PAEG_SIZE + 1 {
                //let middle =  count / 3;

                //let first = active_page;
                let last = if active_page < 3 {
                    PAEG_SIZE + 2
                } else if active_page + PAEG_SIZE <= count - 1 {
                    active_page + PAEG_SIZE
                } else {
                    count
                };

                let first = if last == count {
                    count - 1 - PAEG_SIZE
                } else if active_page > 1 {
                    active_page - 1
                } else {
                    active_page
                };

                trace!("pagination: {} {}", first, last);

                (first, last)
            } else {
                (0, count)
            };

            let btn = {
                //let pagination_on_click = pagination_on_click.clone();
                move |i: u64| {
                    //let pagination_on_click = pagination_on_click.clone();
                    //let active_page = active_page.clone();

                    let loading = loading.clone();
                    let count = count.clone();
                    view! {  <a
                        href=PageUrl::url_dash_wsold_paged(i)
                        on:click=move |_| {
                            if loading.get_untracked() {
                                return;
                            }
                            //pagination_on_click(i);
                            loading.set(true);
                    }
                    class=move || format!(" px-2 font-black border-2 {}", if ((i == 0 || i == count - 1) && active_page != i) {"border-low-purple"} else { if active_page == i { "bg-mid-purple border-transparent" } else { "border-mid-purple" } },  )>{i + 1}</a> }
                }
            };

            let first_btn = {
                let btn = btn.clone();
                view! {
                    <Show  when=move || (count > PAEG_SIZE + 2 && first_page > 0) >
                        { btn(0) }
                        // <Show when=move || (first_page > 1)>{ view! { <span class="px-1">"..."</span> } }</Show>
                    </Show>
                }
            };

            let last_btn = {
                let btn = btn.clone();
                view! {
                    <Show  when=move || ( count > PAEG_SIZE + 2 && last_page < count )>
                    // <Show when=move || (last_page < count - 1)>{ view! { <span class="px-1">"..."</span> } }</Show>
                    { btn(count - 1) }
                    </Show>
                }
            };

            view! {
                <div>
                    { first_btn }
                    {
                        (first_page..last_page).map(btn).collect_view()
                    }
                    { last_btn }
                </div>
            }
        } else {
            view! { <div>"Loading..."</div> }
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
                        { move || old_connections_view() }
                    </table>
                </Show>
            </div>
            <div class="flex gap-1">
                { move || pagination_view() }
            </div>
        </div>
    }
}
