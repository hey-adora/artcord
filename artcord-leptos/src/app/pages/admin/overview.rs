use std::f64::consts::PI;
use std::rc::Rc;
use std::time::Duration;

use artcord_leptos_web_sockets::channel::WsRecvResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_server_msg::ServerMsg;
use chrono::DateTime;
use chrono::Datelike;
use chrono::Days;
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
use rand::Rng;
use strum::VariantArray;
use strum::VariantNames;
use tracing::debug;
use tracing::error;
use tracing::trace;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::app::global_state::GlobalState;
use crate::app::hooks::use_graph::use_graph;
use crate::app::utils::LoadingNotFound;

use super::WsPathTableHeaderView;
use strum::IntoEnumIterator;

#[component]
pub fn Overview() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let page = global_state.pages.admin;
    let ws = global_state.ws;
    let (canvas_ref, canvas_data) = use_graph();
    let selected_days = page.overview_selected_days;
    let selected_unique = page.overview_selected_unique;
    let selected_state = page.overview_state;

    let ws_old_ws_stats = ws.channel().timeout(30).start();

    ws_old_ws_stats
        .recv()
        .start(move |server_msg, _| match server_msg {
            WsRecvResult::Ok(server_msg) => match server_msg {
                // ServerMsg::WsStatsTotalCount(stats) => {
                //     page.set_old_stats_pagination(*stats);
                //     selected_state.set(LoadingNotFound::Loaded);
                // }
                ServerMsg::WsSavedStatsGraph(stats) => {
                    canvas_data.set(stats.clone());
                    selected_state.set(LoadingNotFound::Loaded);
                }
                _ => {
                    selected_state.set(LoadingNotFound::Error);
                }
            },
            WsRecvResult::TimeOut => {
                selected_state.set(LoadingNotFound::Error);
            }
        });

    let fetch = move |selected_days: u64, selected_unique: bool| {
        let _ = ws_old_ws_stats.sender().send(ClientMsg::WsStatsRange {
            from: Utc::now().timestamp_millis(),
            to: Utc::now().checked_sub_days(Days::new(selected_days)).map(|to| to.timestamp_millis()).unwrap_or_default(),
            unique_ip: selected_unique,
        });
    };

    create_effect({
        let fetch = fetch.clone();
        move |_| {
            let selected_days = selected_days.get();
            let selected_unique = selected_unique.get();
    
            fetch(selected_days, selected_unique);
        }
    });

    let on_add_data_test_click = move |days: u64| {
        canvas_data.update(move |data| {
            for _ in 0..days {
                let last_item = data.get(data.len() - 2);
                let Some(last_item) = last_item else {
                    return;
                };
                data.push(*last_item + (24 * 60 * 60 * 1000) as f64);
                data.push(rand::thread_rng().gen_range(0..1000) as f64);
            };
        });
    };

    let on_unique_click = move |unique: bool| {
        selected_state.set(LoadingNotFound::Loading);
        selected_unique.set(unique);
    };

    let on_days_click = move |days: u64| {
        selected_state.set(LoadingNotFound::Loading);
        selected_days.set(days);
    };

    let on_refresh_click = move |_| {
        selected_state.set(LoadingNotFound::Loading);
        let selected_days = selected_days.get_untracked();
        let selected_unique = selected_unique.get_untracked();

        fetch(selected_days, selected_unique);
    };

    let days_btn_view = move |days: u64| {
        view! {
            <button class=move || format!(" border-2  text-white px-2 font-black {}", if selected_days.get() == days {"bg-mid-purple border-transparent "} else {"border-low-purple"}) on:click={let on_add_data_click = on_days_click.clone(); move |_| on_add_data_click(days)}>{days} " days"</button>
        }
    };

    let unique_btn_view = move |text: &'static str, unique: bool| {
        view! {
            <button on:click=move |_| on_unique_click(unique) class=move || format!("border-2 text-white px-2 rounded-2xl font-bold {}", if selected_unique.get() == unique {"bg-mid-purple border-transparent"} else {"border-low-purple"})>{text}</button>
        }
    };

    let graph_view = move || {
        match selected_state.get() {
            LoadingNotFound::NotLoaded => {
                view! {
                    <div class="w-full aspect-video max-w-[80vh]">"starting...."</div>
                }
            }
            LoadingNotFound::Loading => {
                view! {
                    <div class="w-full aspect-video max-w-[80vh]">"downloading...."</div>
                }
            }
            LoadingNotFound::NotFound => {
                view! {
                    <div class="w-full aspect-video max-w-[80vh]">"no data found."</div>
                }
            }
            LoadingNotFound::Error => {
                view! {
                    <div class="w-full aspect-video max-w-[80vh]">"Server Error."</div>
                }
            }
            LoadingNotFound::Loaded => {
                view! {
                    <div class="w-full aspect-video max-w-[80vh]">"Loaded."</div>
                }
            }
        }
    };

    //<WsPath as VariantArray>::VARIANTS

    view! {
        <div class="grid grid-rows-[auto_1fr] overflow-y-hidden">
            <div>"Overview"</div>
                <div class="overflow-y-scroll grid grid-rows-[1fr_1fr]">
                    <div  class="  flex flex-col justify-center ">
                        <div class="bg-dark-night py-6 px-4  rounded-2xl max-w-[80vh] ">
                            <div class="px-6 flex gap-4 justify-between ">
                                <div class=" flex gap-4">
                                    { unique_btn_view("Unique IP", true) }
                                    { unique_btn_view("ALL", false) }
                                </div>
                                <div class=" flex gap-4">
                                    <button on:click=on_refresh_click class=" border-2 border-low-purple text-white px-2 rounded-2xl font-bold">"Refresh"</button>
                                </div>
                            </div>
                            <Show when=move|| selected_state.get() != LoadingNotFound::Loaded>
                                {move || graph_view()}
                            </Show>
                            <canvas _ref=canvas_ref class=move || format!("w-full  aspect-video max-w-[80vh] {}", if selected_state.get() == LoadingNotFound::Loaded { "flex" } else { "hidden" } )/>
                            <div class="px-6 flex gap-4 ">
                                { days_btn_view(7) }
                                { days_btn_view(30) }
                            </div>
                        </div>
                        <div>
                    </div>
                </div>
            </div>
        </div>
    }
}
