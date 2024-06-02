use std::time::Duration;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::model::ws_statistics::DbReqStatPath;
use leptos::*;
use leptos_router::use_navigate;
use leptos_router::use_params_map;
use leptos_router::use_query_map;
use strum::VariantNames;
use tracing::error;

use crate::app::global_state::GlobalState;
use crate::app::utils::PageUrl;

use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;
use tracing::debug;
use tracing::trace;

pub const PAGE_AMOUNT: u64 = 100;

#[component]
pub fn WsOld() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    let ws_old_ws_stats = ws.channel().timeout(30).start();
    let page = global_state.pages.admin;
    let old_ws_stats = page.old_connections;
    let pagination = page.old_connections_pagination;
    let active_page = page.old_connections_active_page;
    //let active_page_mem = create_memo(move |_| active_page.get());
    let loading = page.old_connections_loading;
    let loaded = page.old_connections_loaded;
    let from = page.old_connections_from;
    //let from_mem = create_memo(move |_| from.get());
    let query = use_query_map();
    let use_navigate = use_navigate();

    ws_old_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                // ServerMsg::WsStatsTotalCount(stats) => {
                //     page.set_old_stats_pagination(*stats);
                // }
                ServerMsg::WsSavedStatsPage(stats) => {
                    page.set_old_stats_paged(stats.clone());
                }
                ServerMsg::WsSavedStatsWithPagination {
                    total_count,
                    latest,
                    stats,
                } => {
                    page.set_old_stats_with_pagination(*total_count, latest.clone(), stats.clone());
                }
                // ServerMsg::WsStatsFirstPage {
                //     total_count,
                //     first_page,
                // } => {
                //     page.set_old_stats(first_page.clone(), Some(*total_count));
                // }
                _ => {}
            },
            WsRecvResult::TimeOut => {}
        });

    // create_effect({
    //     let ws_old_ws_stats = ws_old_ws_stats.clone();
    //     move |_| {

    //     }
    // });

    // let fetch_data = move || {
    //     //loading.set(true);

    // };

    // create_effect({
    //     let ws_old_ws_stats = ws_old_ws_stats.clone();
    //     move |_| {
    //         let pagination = pagination.get();
    //         let from = from.get_untracked();
    //         if pagination.is_none() {
    //             ws_old_ws_stats.sender().send(ClientMsg::WsStatsTotalCount { from });
    //         }
    //     }
    // });

    // let fetch_stats = move |page: Option<String>| {

    // };

    create_effect(move |_| {
        let from_q = query
            .with(move |query| query.get("f").map(|v| i64::from_str_radix(v, 10).ok()))
            .flatten();

        if let Some(from_q) = from_q {
            from.set(Some(from_q));
        }
    });

    create_effect(move |_| {
        let page = query
            .with(move |query| {
                query.get("p").map(|v| {
                    u64::from_str_radix(v, 10)
                        .inspect_err(|err| error!("error parsing page {}", err))
                        .ok()
                })
            })
            .flatten();

        if let Some(page) = page {
            active_page.set(page);
        }
    });

    create_effect(move |_| {
        let page = active_page.get();
        let from = from.get();
        
        let Some(from) = from else {
            trace!("dash: wsold: fetching with pagination...");
            loading.set(true);
            let _ = ws_old_ws_stats
                .sender()
                .send(ClientMsg::WsStatsWithPagination {
                    page,
                    amount: PAGE_AMOUNT,
                });
            return;
        };

        if pagination.get_untracked().is_none() {
            trace!("dash: wsold: fetching pages seperatly");
            let _ = ws_old_ws_stats
                .sender()
                .send(ClientMsg::WsStatsTotalCount { from: Some(from) });
        }

        if loaded.with_untracked(move |loaded_page| {
            loaded_page
                .map(|loaded_page| loaded_page == page)
                .unwrap_or(false)
        }) {
            trace!("dash: wsold: its already fetched for page: {page}");
            return;
        } else {
            trace!("dash: wsold: fetching without pagination... {:?} {}", loaded.get_untracked(), page);
        }
        
        loading.set(true);

        let _ = ws_old_ws_stats.sender().send(ClientMsg::WsStatsPaged {
            page,
            amount: PAGE_AMOUNT,
            from,
        });

        active_page.set(page);
    });

    // create_effect({
    //     let ws_old_ws_stats = ws_old_ws_stats.clone();
    //     move |_| {
    //         trace!("dash: ws_old: fetching pages...");
    //         let page_q = query.with(move |query| query.get("p").cloned());
    //         let from_q = query
    //             .with(move |query| query.get("f").map(|v| i64::from_str_radix(v, 10).ok()))
    //             .flatten();
    //         let Some(page_q) = page_q else {
    //             trace!("dash: ws_old: fetching first page");
    //             let already_fetched = old_ws_stats.with_untracked(move |stats| stats.len() > 0);
    //             if already_fetched {
    //                 trace!("dash: ws_old: first page already fetched");
    //                 return;
    //             }

    //             ws_old_ws_stats
    //                 .sender()
    //                 .send(ClientMsg::WsStatsWithPagination {
    //                     page: 0,
    //                     amount: PAGE_AMOUNT,
    //                 });
    //             return;
    //         };

    //         let page = match u64::from_str_radix(&page_q, 10) {
    //             Ok(page) => page,
    //             Err(err) => {
    //                 error!("error parsing page '{}': {}", page_q, err);
    //                 return;
    //             }
    //         };

    //         let Some(from_parsed) = from.get_untracked() else {
    //             if let Some(from_q) = from_q {
    //                 trace!(
    //                     "dash: ws_old: fetching page with pagination using url 'f' {} {}",
    //                     from_q,
    //                     page
    //                 );
    //                 from.set(Some(from_q));
    //                 ws_old_ws_stats
    //                     .sender()
    //                     .send(ClientMsg::WsStatsTotalCount { from: Some(from_q) });
    //                 ws_old_ws_stats.sender().send(ClientMsg::WsStatsPaged {
    //                     page,
    //                     amount: PAGE_AMOUNT,
    //                     from: from_q,
    //                 });
    //             } else {
    //                 trace!("dash: ws_old: fetching page with pagination {}", page);
    //                 ws_old_ws_stats
    //                     .sender()
    //                     .send(ClientMsg::WsStatsWithPagination {
    //                         page: page,
    //                         amount: PAGE_AMOUNT,
    //                     });
    //             }
    //             return;
    //         };

    //         let from = from_q
    //             .map(|new_from| {
    //                 if new_from == from_parsed {
    //                     from_parsed
    //                 } else {
    //                     trace!(
    //                         "dash: ws_old: from and 'f' did not match, updating {} == {}",
    //                         from_parsed,
    //                         new_from
    //                     );
    //                     from.set(Some(new_from));
    //                     new_from
    //                 }
    //             })
    //             .unwrap_or(from_parsed);

    //         trace!("dash: ws_old: fetching page {}", page);

    //         if pagination.get_untracked().is_none() {
    //             ws_old_ws_stats
    //                 .sender()
    //                 .send(ClientMsg::WsStatsTotalCount { from: Some(from) });
    //         }

    //         ws_old_ws_stats.sender().send(ClientMsg::WsStatsPaged {
    //             page,
    //             amount: PAGE_AMOUNT,
    //             from,
    //         });
    //     }
    // });

    let old_connections_count_view = move |count: Vec<DbReqStatPath>| {
        // let count_iter = count.into_iter();
        <ClientMsg as VariantNames>::VARIANTS
            .into_iter()
            .map(|path| {
                // let path_str = path.
                let count = count
                    .iter()
                    .find(|v| v.path == *path)
                    .map(|v| v.throttle.block_tracker.total_amount)
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
                            <td class="border border-mid-purple">{v.ip}</td>
                            <td class="border border-mid-purple">{v.addr}</td>
                            <td class="border border-mid-purple">{ format!("{:?}", Duration::from_millis((v.disconnected_at - v.connected_at) as u64)) }</td>
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

    // let on_refresh = move |_| {
    //     let page = query
    //         .with_untracked(move |query| query.get("p").map(|v| u64::from_str_radix(v, 10)))
    //         .unwrap_or(Ok(0))
    //         .inspect_err(|err| error!("error parsing page {}", err))
    //         .unwrap_or(0);
    //     trace!("dash: ws_old: fetching page {}", page);

    //     ws_old_ws_stats
    //         .sender()
    //         .send(ClientMsg::WsStatsWithPagination {
    //             page: page,
    //             amount: PAGE_AMOUNT,
    //         });
    // };

    // let pagination_on_click = move |page: u64| {

    // };

    let pagination_view = move || {
        //let pagination_on_click = pagination_on_click.clone();

        // let active_page = query
        //     .with(move |query| query.get("p").cloned())
        //     .map(|v| u64::from_str_radix(&v, 10).unwrap_or(0))
        //     .unwrap_or(0);
        let active_page = active_page.get();
        trace!("dash: ws_old: building pagination: active page {active_page}");
        if let (Some(count), Some(from)) = (pagination.get(), from.get()) {
            trace!("dash: ws_old: pagination received total count");
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

                (first, last)
            } else {
                (0, count)
            };

            trace!("dash: pagination: {} {}", first_page, last_page);

            let btn = {
                //let pagination_on_click = pagination_on_click.clone();
                move |i: u64| {
                    //let pagination_on_click = pagination_on_click.clone();
                    //let active_page = active_page.clone();

                    let loading = loading.clone();
                    let count = count.clone();
                    view! {  <a
                        href=PageUrl::url_dash_wsold_paged(i, from)
                    //     on:click=move |_| {
                    //         if loading.get_untracked() {
                    //             return;
                    //         }
                    //         //pagination_on_click(i);

                    // }
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
            trace!("dash: ws_old: pagination waiting to recv total count");
            view! { <div>"Loading..."</div> }
        }
    };

    let refresh_click = move |_| {
        let link = query
            .with(move |query| query.get("p").cloned())
            .map(|v| {
                u64::from_str_radix(&v, 10)
                    .map(|page| PageUrl::url_dash_wsold_refresh(page))
                    .ok()
            })
            .flatten()
            .unwrap_or(PageUrl::url_dash_wsold());

        use_navigate(&link, Default::default());
        from.set(None);
    };

    view! {
        <div class="grid grid-rows-[auto_1fr_auto] overflow-y-hidden gap-2">
            <div class="flex justify-between">
                <div>"WebSocket Connection History"</div>
                <button on:click=refresh_click class="border-2 px-2 border-low-purple">"Refresh"</button>
            </div>
            <div class="overflow-y-scroll overflow-x-auto">
                <Show when=move || !loading.get() fallback=move || view!{
                    <div class="h-full">"Loading..."</div>
                }>
                    <table class="border-spacing-5 border border-mid-purple ">
                        <tr class="sticky top-0 left-0 z-10 bg-mid-purple border border-mid-purple ">
                            <th>"ip"</th>
                            <th>"addr"</th>
                            <th>"duration"</th>
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
