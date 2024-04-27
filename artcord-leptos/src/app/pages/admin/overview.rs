use std::f64::consts::PI;
use std::rc::Rc;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_server_msg::ServerMsg;
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

use super::WebAdminStatCountType;
use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;

struct Can<T: ElementDescriptor + 'static> {
    canvas: NodeRef<Canvas>,
    container: NodeRef<T>,
    size: StoredValue<(f64, f64)>,
    mouse: StoredValue<Option<(f64, f64)>>,
    // mouse_inside: StoredValue<bool>,
    // size: StoredValue<(f64, f64)>,
}

impl<T: ElementDescriptor + Clone + 'static> Can<T> {
    pub fn new() -> Self {
        let canvas_ref = NodeRef::new();
        let container_ref = NodeRef::new();
        let canvas_size = StoredValue::new((0.0, 0.0));
        let canvas_mouse = StoredValue::new(None);
        // let mouse_inside = StoredValue::new(false);

        // create_effect(move |_| {
        //     let canvas_elm: Option<HtmlElement<Canvas>> = canvas_ref.get();
        //     let Some(canvas_elm) = canvas_elm else {
        //         error!("error getting canvas context ");
        //         return;
        //     };
        //
        //     let width = canvas_elm.width();
        //     let height: u32 = canvas_elm.height();
        //     canvas_elm.set_width(width);
        //     canvas_elm.set_height(height);
        //
        //     // size.set_value((width as f64, height as f64));
        //
        //     Self::draw(canvas_elm, canvas_size, canvas_mouse);
        // });

        use_resize_observer(canvas_ref, move |entries, observer| {
            let canvas_elm: Option<HtmlElement<Canvas>> = canvas_ref.get_untracked();
            let Some(canvas) = canvas_elm else {
                error!("error getting canvas context ");
                return;
            };

            let rect = entries[0].content_rect();
            let width = rect.width();
            let height = rect.height();

            canvas.set_width(width as u32);
            canvas.set_height(height as u32);
            canvas_size.set_value((width, height));
            Self::draw(canvas, canvas_size, canvas_mouse);
        });

        // let current_mouse = use_mouse();

        let _ = use_event_listener(canvas_ref, ev::mousemove, move |ev| {
            let Some(canvas) = canvas_ref.get_untracked() else {
                error!("error getting canvas context ");
                return;
            };

            let x = ev.offset_x();
            let y = ev.offset_y();
            // trace!("{} {}", x, y);
            canvas_mouse.set_value(Some((x as f64, y as f64)));

            Self::draw(canvas, canvas_size, canvas_mouse);
        });

        use_event_listener(canvas_ref, ev::mouseleave, move |ev| {
            let Some(canvas) = canvas_ref.get_untracked() else {
                error!("error getting canvas context ");
                return;
            };

            // trace!("mouse left!");
            canvas_mouse.set_value(None);
            Self::draw(canvas, canvas_size, canvas_mouse);
        });

        Self {
            canvas: canvas_ref,
            container: container_ref,
            size: canvas_size,
            mouse: canvas_mouse,
        }
    }
    fn get_ctx(canvas: &HtmlElement<Canvas>) -> Option<CanvasRenderingContext2d> {
        let ctx = canvas.get_context("2d");
        let ctx = match ctx {
            Ok(ctx) => ctx,
            Err(err) => {
                error!("error getting canvas context {:?}", err);
                return None;
            }
        };
        let Some(ctx) = ctx else {
            error!("error getting canvas context ");
            return None;
        };

        let ctx = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>();
        let ctx = match ctx {
            Ok(ctx) => ctx,
            Err(err) => {
                error!("error getting canvas context {:?}", err);
                return None;
            }
        };

        Some(ctx)
    }

    // fn set_canvas_size(canvas: NodeRef<Canvas>, width: f64, height: f64) {
    //     let Some(canvas) = canvas.get_untracked() else {
    //         error!("error getting canvas context ");
    //         return;
    //     };
    //
    //     canvas.set_width(width as u32);
    //     canvas.set_height(height as u32);
    // }

    pub fn draw(
        canvas: HtmlElement<Canvas>,
        size: StoredValue<(f64, f64)>,
        mouse: StoredValue<Option<(f64, f64)>>,
    ) {
        let Some(ctx) = Self::get_ctx(&canvas) else {
            return;
        };

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        let padding = 100.0;
        let line_height = 15.0;

        // let (width, heigth) = size.get_value();
        let mouse_pos = mouse.get_value();
        // debug!("{} {} {} {}", width, height, x, y);
        // debug!("{} {}", width, height);

        // ctx.move_to(0.0, 0.0);
        // ctx.line_to(200.0, 100.0);
        // ctx.stroke();
        // let x = mouse.x.get();
        // let y = mouse.y.get();

        ctx.clear_rect(0.0, 0.0, width, height);

        // let data: Vec<f64> = vec![
        //     5.0, 2.0, 20.0, 55.0, 20.0, 3.0, 200.0, 150.0, 2.0, 2.0, 69.0, 88.0,
        // ];
        let data: Vec<f64> = vec![0.0, 0.0, 10.0, 10.0, 20.0, 10.0];
        let data = data.chunks(2);

        // let data: Vec<f64> = vec![50.0, 0.0];
        let mut max_x = 0.0;
        let mut max_y = 0.0;
        let mut text_render_index: Option<usize> = None;
        let mut text_render_distance: Option<f64> = None;
        for (i, chunk) in data.clone().enumerate() {
            let x = chunk.get(0).cloned();
            let y = chunk.get(1).cloned().unwrap_or(0.0);
            let Some(x) = x else {
                break;
            };

            if x > max_x {
                max_x = x;
            }

            if y > max_y {
                max_y = y;
            }
        }

        let adjust_y = (height - padding) / max_y;
        let adjust_x = (width - padding) / max_x;

        for (i, chunk) in data.clone().enumerate() {
            let x = chunk.get(0).cloned();
            let y = chunk.get(1).cloned().unwrap_or(0.0);
            let Some(x) = x else {
                break;
            };

            let y = (height - (y * adjust_y)) - (padding / 2.0);
            let x = x * adjust_x + (padding / 2.0);

            if let Some((mouse_x, mouse_y)) = mouse_pos {
                // let a = text_render_distance
                let new_distance = ((x - mouse_x).powi(2) + (y - mouse_y).powi(2)).sqrt();
                if let Some(text_render_distance) = &mut text_render_distance {
                    if *text_render_distance > new_distance {
                        *text_render_distance = new_distance;
                        text_render_index = Some(i);
                    }
                } else {
                    text_render_distance = Some(new_distance);
                    text_render_index = Some(i);
                }
            }
        }
        // trace!(
        //     "graph: max_x: {}, adjust_x: {}, max_y: {}, adjust_y: {}, mouse_pos: {:?}, text_i: {:?}, text_dist: {:?}",
        //     max_x,
        //     adjust_x,
        //     max_y,
        //     adjust_y,
        //     mouse_pos,
        //     text_render_index,
        //     text_render_distance,
        // );

        let mut prev_point: Option<(f64, f64)> = None;
        for (i, chunk) in data.enumerate() {
            let org_x = chunk.get(0).cloned();
            let org_y = chunk.get(1).cloned().unwrap_or(0.0);
            let Some(org_x) = org_x else {
                break;
            };
            let y = (height - (org_y * adjust_y)) - (padding / 2.0);
            let x = (org_x * adjust_x) + (padding / 2.0);
            ctx.begin_path();
            let style = JsValue::from_str("red");
            ctx.set_fill_style(&style);
            let radius = 2.0;
            ctx.arc(x, y, radius, 0.0, PI * 2.0);
            ctx.fill();

            if let Some((prev_x, prev_y)) = prev_point {
                ctx.begin_path();
                ctx.move_to(prev_x, prev_y);
                ctx.line_to(x, y);
                ctx.set_line_width(2.0);
                let style = JsValue::from_str("red");
                ctx.set_stroke_style(&style);
                ctx.stroke();
                prev_point = Some((x, y));
            } else {
                prev_point = Some((x, y));
            }

            let Some(text_render_index) = text_render_index else {
                continue;
            };
            if text_render_index != i {
                continue;
            }
            ctx.set_font("0.7rem Arial");
            let text = format!("{:.2}\n{:.2}", org_x, org_y);
            let text = text.split("\n");
            let texts: Vec<&str> = text.collect();
            let len = texts.len().checked_sub(1).unwrap_or(1);
            for (i, text) in texts.iter().enumerate() {
                let Ok(text_w) = ctx
                    .measure_text(text)
                    .inspect_err(|err| error!("failed to measure text: {:?}", err))
                else {
                    return;
                };

                let text_x = x - (text_w.width() / 2.0);
                let text_y = (y - (radius * 2.0)) - line_height * (len - i) as f64;

                ctx.fill_text(text, text_x, text_y);
            }
            {
                let Some((mouse_x, mouse_y)) = mouse_pos else {
                    return;
                };
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(mouse_x, mouse_y);
                ctx.set_line_width(1.0);
                let style = JsValue::from_str("red");
                ctx.set_stroke_style(&style);
                ctx.stroke();
            }
        }

        // ctx.begin_path();
        // ctx.set_line_width(10.0);
        // ctx.move_to(0.0, 0.0);
        // ctx.line_to(33.0, 69.0);
        // // ctx.move_to(20.0, 20.0);
        // let style = JsValue::from_str("red");
        // ctx.set_stroke_style(&style);
        // ctx.stroke();
    }

    // pub fn draw_set_color(canvas: NodeRef<Canvas>) {
    //     let Some(ctx) = Self::get_ctx(canvas) else {
    //         return;
    //     };
    //
    //     let style = JsValue::from_str("red");
    //     ctx.set_stroke_style(&style);
    // }
}

#[component]
pub fn Overview() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let page = global_state.pages.admin;
    let ws = global_state.ws;
    let can = Can::<Div>::new();
    let canvas_ref = can.canvas;
    let container_ref = can.container;
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
    let container = create_node_ref::<Div>();
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
                <div  _ref=container_ref class=" bg-dark-night">
                    // <div class="w-[100rem] h-[100rem] box"></div>
                    <canvas _ref=canvas_ref class="w-full h-full box"/>
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
