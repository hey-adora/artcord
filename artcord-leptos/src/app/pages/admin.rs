use artcord_leptos_web_sockets::WsResourceResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::model::statistics::Statistic;
use leptos::*;
use leptos_router::use_params_map;
use leptos_use::use_interval_fn;
use tracing::error;
use tracing::trace;

use crate::app::components::navbar::Navbar;

use crate::app::global_state::GlobalState;

#[derive(Copy, Clone, Debug)]
pub struct AdminPageState {
    pub statistics: RwSignal<Vec<Statistic>>,
}

impl AdminPageState {
    pub fn new() -> Self {
        Self {
            statistics: RwSignal::new(Vec::new()),
        }
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    let _params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let ws = global_state.ws;
    let page = global_state.pages.admin;

    let ws_statistics = ws.create_singleton();

    // ws_statistics.send_or_skip(Vgc, on_receive)
    create_effect(move |_| {
        nav_tran.set(true);
    });

    create_effect(move |_| {
        use_interval_fn(
            move || {
                let result = ws_statistics.send_or_skip(
                    ClientMsg::Statistics,
                    move |server_msg: WsResourceResult<ServerMsg>| {
                        trace!("statistics: msg: {:?}", &server_msg);
                        match server_msg {
                            WsResourceResult::Ok(server_msg) => match server_msg {
                                ServerMsg::Statistics(stats) => {
                                    page.statistics.set(stats);
                                }
                                server_msg => {
                                    error!("statistics: wrong server response: {:?}", server_msg);
                                }
                            },
                            WsResourceResult::TimeOut => {
                                error!("statistics: timeout");
                            }
                        }
                    },
                );
                match result {
                    Ok(result) => {
                        trace!("statistics: send_result: {:?}", &result);
                    }
                    Err(err) => {
                        error!("statistics: {}", err);
                    }
                }
            },
            1000,
        );
    });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})>
            <Navbar/>
            <div class="flex gap-4 bg-white ">
                <div class="flex flex-col gap-4 bg-dark-night  px-6 py-4">
                    <div class="font-bold">"DASHBOARD"</div>
                    <div class="flex flex-col gap-2 ">
                        <div>"Activity"</div>
                        <div>"Banned IP's"</div>
                        <div>"Users"</div>
                    </div>
                </div>
                <div class="w-full text-black py-4 gap-4 flex  flex-col  ">
                    <div class="font-bold">"Activity"</div>
                    <div>"Activity"</div>
                    <table>
                        <tr class="sticky top-[4rem] left-0 bg-light-flower ">
                            <th>"one"</th>
                            <th>"two"</th>
                            <th>"three"</th>
                        </tr>

                        {
                            move || {


                                page.statistics.get().into_iter().map(|stat: Statistic| view! {
                                    <tr>
                                        <td>{stat.ip}</td>
                                        <td>"item"</td>
                                        <td>"item"</td>
                                    </tr>
                                }).collect_view()

                            }
                        }

                    </table>
                </div>
            </div>
        </main>
    }
}
