use std::f64::consts::PI;
use std::rc::Rc;
use std::time::Duration;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::ServerMsg;
use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use leptos::html::canvas;
use leptos::html::Canvas;
use leptos::html::Div;
use leptos::html::ElementDescriptor;
use leptos::*;
// use leptos_chart::*;
use leptos_use::use_event_listener;
use leptos_use::use_mouse;
use leptos_use::use_resize_observer;
use tracing::debug;
use tracing::error;
use tracing::trace;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::app::global_state::GlobalState;
use crate::app::hooks::use_graph::use_graph;

use super::WebAdminStatCountType;
use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;

#[component]
pub fn Overview() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let page = global_state.pages.admin;
    let ws = global_state.ws;
    let (canvas_ref, canvas_data) = use_graph();
   // let can = Can::new();

      //  canvas_data.set(vec![0.0, 0.0, 10.0, 10.0, 20.0, 10.0]);
 //   let canvas_ref = can.canvas;
    //let container_ref = can.container;

    let ws_old_ws_stats = ws.channel().timeout(30).start();

    ws_old_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                ServerMsg::WsStatsTotalCount(stats) => {
                    page.set_old_stats_pagination(*stats);
                }
                ServerMsg::WsStatsPage(stats) => {
                    //page.set_old_stats_paged(stats.clone());

                    let mut new_data: Vec<f64> = Vec::with_capacity(stats.len() * 2);

                    let day_milis = 24 * 60 * 60 * 1000;
                    let time_duration = day_milis;

                    let Some(first_day) = stats.first().cloned() else {
                        return;
                    };
                    let Some(last_day) = stats.last().cloned() else {
                        return;
                    };


                    let mut prev_start_of_day: i64 = last_day.created_at.checked_sub(last_day.created_at % time_duration).unwrap_or(0);
                    
                    let mut data_item: f64 = 0_f64;

                    let date = DateTime::from_timestamp_millis(first_day.created_at);
                    let Some(date) = date else {
                        return;
                    };
                    let weekday = date.weekday();
                    let from_monday = weekday.num_days_from_monday();
                    let first_day_of_the_week = (first_day.created_at - (first_day.created_at % day_milis)) - (from_monday as i64 * day_milis);
                    let Some(first_day_of_the_week_date) = DateTime::from_timestamp_millis(first_day_of_the_week) else {
                        return;
                    };

                    let date = DateTime::from_timestamp_millis(last_day.created_at);
                    let Some(date) = date else {
                        return;
                    };
                    let weekday = date.weekday();
                    let to_sunday = weekday.num_days_from_sunday();
                    let last_day_of_the_week = (last_day.created_at - (last_day.created_at % day_milis))   - (to_sunday as i64 * day_milis);
                    let Some(last_day_of_the_week_date) = DateTime::from_timestamp_millis(last_day_of_the_week) else {
                        return;
                    };

                    let diff = last_day_of_the_week - first_day_of_the_week;
                    let steps = diff / day_milis * 7;


                    trace!("graph: time: {} {} {} {} {} {}", from_monday, to_sunday, first_day_of_the_week_date, last_day_of_the_week_date, diff, steps);


                    for stat in stats.iter().rev() {
                        let Some(created_at_start_of_the_day) = stat.created_at.checked_sub(stat.created_at % time_duration) else {
                            error!("graph: invalid date: {:#?}", stats);
                            continue;
                        };
                        if created_at_start_of_the_day > prev_start_of_day {
                            new_data.push(prev_start_of_day as f64);
                            new_data.push(data_item);
                            prev_start_of_day = created_at_start_of_the_day;
                            data_item = 0.0;
                            continue;
                        }

                        data_item += 1_f64;

                        // let created_at = DateTime::from_timestamp_millis(stat.created_at);
                        // let Some(created_at) = created_at else {
                        //     continue;
                        // };
                        //created_at.day()
                        //Duration::from_secs(MILI) 
                        //Utc::now().timestamp_millis()
                        // new_data.push(stat.created_at as f64);
                        // new_data.push(stat.d);
                    }

                    new_data.push(prev_start_of_day as f64);
                    new_data.push(data_item);

                    trace!("admin: overview data: {:#?}", &new_data);
                    canvas_data.set(new_data);
                    //stats.iter().map(|stat| );
                }
                ServerMsg::WsStatsWithPagination {
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

    let _ = ws_old_ws_stats.sender().send(ClientMsg::WsStatsPaged {
        page: 0,
        amount: 100000,
        from: Utc::now().timestamp_micros(),
    });

    // let canvas_size = RwSignal::new((0_u32, 0_u32));
    // let mouse_on_canvas = RwSignal::new(false);
    // let ws_live_ws_stats = ws.channel().timeout(30).single_fire().start();
    // // let live_ws_stats = page.live_connections;
    // //
    // // ws_live_ws_stats
    // //     .recv()
    // //     .start(move |server_msg, _| match server_msg {
    // //         WsRecvResult::Ok(server_msg) => match server_msg {
    // //             ServerMsg::LiveWsStats(msg) => match msg {
    // //                 LiveWsStatsRes::Started(stats) => {
    // //                     page.set_live_stats(stats.clone());
    // //                 }
    // //                 LiveWsStatsRes::UpdateAddedStat { con_key, stat } => {
    // //                     page.add_live_stat(con_key.clone(), stat.clone().into());
    // //                 }
    // //                 LiveWsStatsRes::UpdateInc { con_key, path } => {
    // //                     page.inc_live_stat(con_key, path);
    // //                 }
    // //                 LiveWsStatsRes::UpdateRemoveStat { con_key } => {
    // //                     page.remove_live_stat(con_key);
    // //                 }
    // //                 _ => {}
    // //             },
    // //             ServerMsg::WsStats(stats) => {}
    // //             _ => {}
    // //         },
    // //         WsRecvResult::TimeOut => {}
    // //     });
    // //
    // // ws_live_ws_stats
    // //     .sender()
    // //     .resend_on_reconnect()
    // //     .on_cleanup(ClientMsg::LiveWsStats(false))
    // //     .send(ClientMsg::LiveWsStats(true));
    // //
    // // let live_connection_count_view = move |count: WebAdminStatCountType| {
    // //     WsPath::iter()
    // //         .map(|path| {
    // //             let count = count.get(&path).cloned();
    // //             view! {
    // //                 <th>{move || count.map(|count| count.get()).unwrap_or(0u64)}</th>
    // //             }
    // //         })
    // //         .collect_view()
    // // };
    // //
    // // let live_connection_view = move || {
    // //     view! {
    // //         <For each=move || live_ws_stats.get().into_iter() key=|item| item.0.clone() let:item>
    // //             <tr>
    // //                 <td>{item.1.addr}</td>
    // //                 { live_connection_count_view(item.1.count) }
    // //             </tr>
    // //         </For>
    // //     }
    // // };
    //
    // let chart = Cartesian::new(
    //     Series::from(vec![0., 1.0, 2.]),
    //     Series::from(vec![3., 1.0, 5.]),
    // )
    // .set_view(820, 620, 3, 100, 100, 20);
    //
    // let canvas = create_node_ref::<Canvas>();
    // // let set_canvas_size
    // // let draw = move || {
    // // };
    // let mouse = use_mouse();
    //
    // create_effect(move |_| {
    //     let Some(canvas) = canvas.get() else {
    //         error!("error getting canvas context ");
    //         return;
    //     };
    //
    //     let ctx = canvas.get_context("2d");
    //     let ctx = match ctx {
    //         Ok(ctx) => ctx,
    //         Err(err) => {
    //             error!("error getting canvas context {:?}", err);
    //             return;
    //         }
    //     };
    //     let Some(ctx) = ctx else {
    //         error!("error getting canvas context ");
    //         return;
    //     };
    //
    //     let ctx = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>();
    //     let ctx = match ctx {
    //         Ok(ctx) => ctx,
    //         Err(err) => {
    //             error!("error getting canvas context {:?}", err);
    //             return;
    //         }
    //     };
    //     let (width, heigth) = canvas_size.get();
    //
    //     // ctx.move_to(0.0, 0.0);
    //     // ctx.line_to(200.0, 100.0);
    //     // ctx.stroke();
    //     let x = mouse.x.get();
    //     let y = mouse.y.get();
    //
    //     ctx.set_line_width(10.0);
    //     ctx.begin_path();
    //     ctx.move_to(5.0, 5.0);
    //     ctx.line_to(x, y);
    //     // ctx.arc(75.0, 75.0, 50.0, 0.0, PI * 2.0);
    //     let style = JsValue::from_str("red");
    //     ctx.set_stroke_style(&style);
    //     ctx.stroke();
    // });
    //
    //let container = create_node_ref::<Div>();
    // use_resize_observer(container, move |entries, observer| {
    //     let rect = entries[0].content_rect();
    //     // trace!("width: {}, height: {}", rect.width(), rect.height());
    //     // let Some(canvas) = canvas.get_untracked() else {
    //     //     error!("error getting canvas context ");
    //     //     return;
    //     // };
    //     let width = rect.width() as u32;
    //     let height = rect.height() as u32;
    //     // canvas.set_width(width);
    //     // canvas.set_height(height);
    //     canvas_size.set((width, height));
    //     // draw();
    // });
    //
    // let on_mouse_enter = move |event| {
    //     mouse_on_canvas.set(true);
    // };
    //
    // let on_mouse_leave = move |event| {
    //     mouse_on_canvas.set(false);
    // };

    // let color = Color::from("#925CB3");
    view! {
        <div class="grid grid-rows-[auto_1fr] overflow-y-hidden">
            <div>"Overview"</div>
            <div class="overflow-y-scroll grid grid-rows-[1fr_1fr]">
                <div  class=" ">
                    // <div class="w-[100rem] h-[100rem] box"></div>
                    <canvas _ref=canvas_ref class="w-full box max-w-full bg-dark-night aspect-video"/>
                </div>
                // <svg viewBox="0 0 820 620">
                //     <g class="" transform="translate(100, 480)">
                //         <text>"wowowwowo"</text>
                //         <circle class="hover:text-blue-600 z-1" cx="0" cy="-288" r="5" fill="currentColor"></circle>
                //         <path d="M 0,-288  340,-96  680,-480 " stroke-width="5" stroke="currentColor" fill="none"></path>
                //     </g>
                // </svg>
                // <LineChart chart=chart class="text-light-flower " stroke_width=5 circle_width=5 />
                // <table> ///////
                //     <tr class="sticky top-0 left-0 bg-light-flower ">
                //         <th>"ip"</th>
                //         <WsPathTableHeaderView/>
                //     </tr>
                //     {move || live_connection_view()}
                // </table>

            </div>
        </div>
    }
}
