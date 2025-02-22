use crate::app::components::navbar::shrink_nav;
use crate::app::components::navbar::Navbar;
use crate::app::global_state::GlobalState;
use crate::app::utils::img_resize::calc_fit_count;
use crate::app::utils::img_resize::resize_imgs;
use crate::app::utils::img_resize::NEW_IMG_HEIGHT;
use crate::app::utils::img_resized::ServerMsgImgResized;
use crate::app::utils::{LoadingNotFound, SelectedImg};
use artcord_state::global;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_router::{use_location, use_params_map};
use leptos_use::{use_event_listener, use_window};
use tracing::{debug, error, trace};
use web_sys::Event;

#[derive(Copy, Clone, Debug)]
pub struct PageUserGalleryState {
    // pub not_found: RwSignal<bool>,
    pub user: RwSignal<Option<global::DbUser>>,
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    pub gallery_loaded: RwSignal<LoadingNotFound>,
}

impl PageUserGalleryState {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            gallery_imgs: RwSignal::new(Vec::new()),
            gallery_loaded: RwSignal::new(LoadingNotFound::NotLoaded),
        }
    }
}

#[component]
pub fn UserGalleryPage() -> impl IntoView {
    let params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let nav_tran = global_state.nav_tran;
    let global_gallery_imgs = global_state.page_profile.gallery_imgs;
    let global_gallery_user = global_state.page_profile.user;
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);
    let loaded_sig = global_state.page_profile.gallery_loaded;
    let ws = global_state.ws;
    let location = use_location();

    // let ws_gallery = ws.builder().portal().build();
    // let ws_user = ws.builder().portal().build();

    let on_click = move |img: ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("/assets/gallery/pfp_{}.webp", img.user.author_id.clone()),
            author_id: img.user_id.clone(),
            width: img.width,
            height: img.height,
        }))
    };

    let on_fetch = move || {
        trace!("user_gallery: fetching started");

        // if loaded_sig.with_untracked(|v| *v == LoadingNotFound::Loading || *v == LoadingNotFound::NotFound) {
        //     trace!("user_gallery: returned: {:?}", loaded_sig.get_untracked());
        //     return;
        // }

        let Some(new_user) = params.with(|p| p.get("id").cloned()) else {
            trace!("user_gallery: returned: missing gallery section");
            return;
        };

        let current_user = global_gallery_user.get_untracked();

        let same_user = current_user
            .as_ref()
            .map(|user| user.author_id == new_user)
            .unwrap_or(false);

        trace!(
            "user_gallery: same_user: {}, new_user: {}, current_user: {}",
            same_user,
            &new_user,
            current_user
                .as_ref()
                .map(|u| u.author_id.clone())
                .unwrap_or("None".to_string())
        );

        let fetch = {
            move |user_id: String| {
                debug!("user_gallery: fetching imgs...");

                let Some(section) = gallery_section.get_untracked() else {
                    trace!("user_gallery: returned: missing gallery section");
                    return;
                };

                let last = global_gallery_imgs
                    .with_untracked(|imgs| imgs.last().map(|img| img.created_at))
                    .unwrap_or(Utc::now().timestamp_millis());
                let client_height = section.client_height();
                let client_width = section.client_width();

                let msg = global::ClientMsg::UserGalleryInit {
                    amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
                    from: last,
                    user_id,
                };

                // match ws_gallery.send_and_recv(msg, move |server_msg| match server_msg {
                //     WsRecvResult::Ok(server_msg) => match server_msg {
                //         ServerMsg::UserGallery(response) => match response {
                //             UserGalleryRes::Imgs(new_imgs) => {
                //                 let new_imgs = new_imgs
                //                     .iter()
                //                     .map(|img| ServerMsgImgResized::from(img.to_owned()))
                //                     .collect::<Vec<ServerMsgImgResized>>();
                //
                //                 global_gallery_imgs.update(|imgs| {
                //                     imgs.extend(new_imgs);
                //                     let gallery_section = gallery_section.get_untracked();
                //                     let Some(gallery_section) = gallery_section else {
                //                         return;
                //                     };
                //                     let width = gallery_section.client_width() as u32;
                //                     resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                //                 });
                //                 trace!("user_gallery: loadeding state set to: Loaded");
                //                 loaded_sig.set(LoadingNotFound::Loaded);
                //             }
                //             UserGalleryRes::UserNotFound => {
                //                 trace!("user_gallery: loadeding state set to: NotFound");
                //                 loaded_sig.set(LoadingNotFound::NotFound);
                //             }
                //         },
                //         ServerMsg::Error => {
                //             trace!("user_gallery: server error: loadeding state set to: Error");
                //             loaded_sig.set(LoadingNotFound::Error);
                //         }
                //         msg => {
                //             error!("user_gallery: received wrong msg: {:#?}", msg);
                //             loaded_sig.set(LoadingNotFound::Error);
                //         }
                //     },
                //     WsRecvResult::TimeOut => {
                //         trace!("user_gallery: timeout: loadeding state set to: Error");
                //         loaded_sig.set(LoadingNotFound::Error);
                //     }
                // }) {
                //     Ok(result) => {
                //         trace!("user_gallery: fetch_imgs returned: {:?}", result);
                //     }
                //     Err(err) => {
                //         error!("user_gallery: send error: {}", err);
                //         loaded_sig.set(LoadingNotFound::Error);
                //     }
                // };
            }
        };

        // Err(err) => {
        //     trace!("user_gallery: loadeding state set to: Error");
        //     loaded_sig.set(LoadingNotFound::Error);
        // }

        if same_user {
            fetch(new_user);
        } else {
            debug!("user_gallery: fetching user...");
            trace!("user_gallery: loadeding state set to: Loading");
            loaded_sig.set(LoadingNotFound::Loading);
            global_gallery_imgs.set(Vec::new());

            let msg = global::ClientMsg::User { user_id: new_user };
            // match ws_user.send_and_recv(msg, move |server_msg| match server_msg {
            //     WsRecvResult::Ok(server_msg) => match server_msg {
            //         ServerMsg::User(response) => match response {
            //             UserRes::User(user) => {
            //                 let user_id = user.author_id.clone();
            //                 global_gallery_user.set(Some(user));
            //                 trace!("user_gallery: user received '{}', fetching imgs", &user_id);
            //                 fetch(user_id);
            //             }
            //             UserRes::UserNotFound => {
            //                 trace!("user_gallery: loadeding state set to: NotFound");
            //                 loaded_sig.set(LoadingNotFound::NotFound);
            //             }
            //         },
            //         ServerMsg::Error => {
            //             trace!("user_gallery: server error:loadeding state set to: Error");
            //             loaded_sig.set(LoadingNotFound::Error);
            //         }
            //         msg => {
            //             error!("user_gallery: received wrong msg: {:#?}", msg);
            //             loaded_sig.set(LoadingNotFound::Error);
            //         }
            //     },
            //     WsRecvResult::TimeOut => {
            //         trace!("user_gallery: timeout: loadeding state set to: Error");
            //         loaded_sig.set(LoadingNotFound::Error);
            //     }
            // }) {
            //     Ok(result) => {
            //         trace!("user_gallery: fetch_user returned: {:?}", result);
            //     }
            //     Err(err) => {
            //         error!("user_gallery: send error: {}", err);
            //         loaded_sig.set(LoadingNotFound::Error);
            //     }
            // };
        }
    };

    create_effect(move |_| {
        nav_tran.set(true);
    });

    create_effect(move |_| {
        if !loaded_sig.with_untracked(|state| *state == LoadingNotFound::Loaded) {
            return;
        }

        let _ = location.pathname.get();
        let _ = location.hash.get();

        global_gallery_imgs.update(|imgs| {
            let section = gallery_section.get_untracked();
            if let Some(section) = section {
                let width = section.client_width() as u32;

                resize_imgs(NEW_IMG_HEIGHT, width, imgs);
            };
        });
    });

    let section_scroll = move |_: Event| {
        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let scroll_top = section.scroll_top();
        let client_height = section.client_height();
        let scroll_height = section.scroll_height();

        shrink_nav(nav_tran, scroll_top as u32);

        let left = scroll_height - (client_height + scroll_top);

        if left < client_height {
            on_fetch();
        }
    };

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), resize, move |_| {
            global_gallery_imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                };
            });
        });
    });

    create_effect(move |_| {
        on_fetch();
    });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})>
            <Navbar/>
            {
                move || {
                    match selected_img.get() {
                        Some(img) => Some(view! {
                            <div on:click=move |_| { selected_img.set(None); } class=" absolute grid grid-rows-[1fr] left-0 top-0 w-screen h-[100dvh] place-items-center bg-gradient-to-br from-mid-purple/50 to-dark-purple/50 z-[150] ">
                                <div on:click=move |e| { e.stop_propagation();  } >
                                    <div class="flex justify-between items-center rounded-t-lg bg-dark-purple pl-2">
                                           <div class="flex gap-2">
                                                <div>"By "</div>
                                                <img class="border border-low-purple rounded-full bg-mid-purple h-[25px] " src=img.author_pfp/>
                                                <a href=move||format!("/user/{}", img.author_id)>{img.author_name}</a>
                                           </div>
                                         <img on:click=move |_| { selected_img.set(None); } class="cursor-pointer border-2 border-low-purple rounded-full bg-mid-purple w-[30px] h-[30px] p-1 m-2" src="/assets/x.svg"/>
                                    </div>
                                    <img class="bg-mid-purple object-contain " alt="loading..." style=move|| format!("max-height: calc(100dvh - 70px); max-width: 100vw; height: min({1}px, calc(100vw * ( {1} / {0} ))); aspect-ratio: {0} / {1};", img.width, img.height)  src=img.org_url/>
                                </div>
                            </div> }),
                    None => None
                    }
                }
            }
            <section id="user_gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
                <Show when=move|| true fallback=move || { "Connecting..." }>
                    <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded || *state == LoadingNotFound::Loading) >
                    <div>"LOADING..."</div>
                    </Show>
                    <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotFound) >
                    <div>"No Images Found."</div>
                    </Show>
                    <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::Error) >
                    <div>"Error loading."</div>
                    </Show>
                </Show>

                <For each=move || global_gallery_imgs.get().into_iter().enumerate()  key=|state| state.1.id.clone() let:data > {
                        let img = data.1;
                        let i = data.0;
                        let height = img.new_height;
                        let width = img.new_width;
                        let top = img.top;
                        let left = img.left;
                        let bg_img = format!("url('{}')", &img.display_preview);

                            view! {
                                <div
                                    class="absolute bg-center bg-contain transition-all bg-no-repeat flex-shrink-0 font-bold grid place-items-center border hover:shadow-glowy hover:z-[99]  duration-300 bg-mid-purple  border-low-purple"
                                    style:height=move || format!("{}px", height.get())
                                    style:width=move || format!("{}px", width.get())
                                    style:top=move || format!("{}px", top.get())
                                    style:left=move || format!("{}px", left.get())
                                    style:background-image=bg_img
                                    on:click=  move |_| on_click(global_gallery_imgs.with_untracked(|imgs|imgs[i].clone()))
                                >
                                </div>
                            }
                        }

                </For>
            </section>
        </main>
    }
}
