use crate::app::components::navbar::shrink_nav;
use crate::app::global_state::GlobalState;
use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_router::{use_location, use_params_map};
use leptos_use::{use_event_listener, use_window};
use web_sys::Event;

use crate::app::utils::{
    calc_fit_count, resize_imgs, LoadingNotFound, SelectedImg, ServerMsgImgResized, NEW_IMG_HEIGHT,
};
use crate::message::server_msg::{SERVER_MSG_PROFILE, SERVER_MSG_PROFILE_IMGS_NAME};
use crate::server::client_msg::ClientMsg;

//F: Fn(ServerMsgImgResized) -> IV + 'static, IV: IntoView
#[component]
pub fn ProfileGallery() -> impl IntoView {
    let params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let nav_tran = global_state.nav_tran;
    let global_gallery_imgs = global_state.page_profile.gallery_imgs;
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);
    let loaded_sig = global_state.page_profile.gallery_loaded;
    let connection_load_state_name = SERVER_MSG_PROFILE_IMGS_NAME;
    let location = use_location();

    let on_click = move |img: ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("/assets/gallery/pfp_{}.webp", img.user.id.clone()),
            author_id: img.user_id.clone(),
            width: img.width,
            height: img.height,
        }))
    };

    let on_fetch = move |from: i64, amount: u32| {
        let Some(new_user) = params.with(|p| p.get("id").cloned()) else {
            return;
        };

        let msg = ClientMsg::UserGalleryInit {
            amount,
            from,
            user_id: String::from(new_user),
        };
        log!("USER GALLERY FETCH: {:#?}", &msg);
        //global_state.socket_send(&msg);
    };

    create_effect(move |_| {
        let loaded = loaded_sig.with_untracked(|state| *state == LoadingNotFound::Loaded);
        if !loaded {
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
        // if !global_state.socket_state_is_ready(connection_load_state_name) {
        //     return;
        // }

        let Some(last) = global_gallery_imgs.with_untracked(|imgs| match imgs.last() {
            Some(l) => Some(l.created_at),
            None => None,
        }) else {
            return;
        };

        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let scroll_top = section.scroll_top();
        let client_height = section.client_height();
        let scroll_height = section.scroll_height();
        let client_width = section.client_width();

        shrink_nav(nav_tran, scroll_top as u32);

        let left = scroll_height - (client_height + scroll_top);

        if left < client_height {
            //global_state.socket_state_used(connection_load_state_name);
            on_fetch(
                last,
                calc_fit_count(client_width as u32, client_height as u32) * 2,
            );
        }
    };

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), resize, move |_| {
            // log!("TRYING TO RESIZE");
            global_gallery_imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    // log!("RESIZING!!!!!!!!!!!!");
                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                };
            });
        });
    });

    create_effect(move |_| {
        let Some(new_user) = params.with(|p| p.get("id").cloned()) else {
            return;
        };

        let user = global_state.page_profile.user.get();

        let same_user = if let Some(ref user) = user {
            new_user == user.id
        } else {
            false
        };
        log!("ONE {} {:?}", same_user, user);
        // if !global_state.socket_state_is_ready(SERVER_MSG_PROFILE) {
        //     return;
        // }

        if !same_user {
            let msg = ClientMsg::User {
                user_id: String::from(new_user),
            };
            //global_state.socket_send(&msg);
        }
    });

    create_effect(move |_| {
        let connected = global_state.socket_connected.get();
        log!("TWO CONNECTED {}", connected);
        if !connected {
            return;
        }

        let not_loaded = loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded);
        log!("TWO LOADED {}", not_loaded);
        if !not_loaded {
            return;
        }

        let user = global_state.page_profile.user.get();
        log!("TWO {:?}", user);
        let Some(user) = user else {
            return;
        };

        // if !global_state.socket_state_is_ready(SERVER_MSG_PROFILE_IMGS_NAME) {
        //     return;
        // }

        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let client_height = section.client_height();
        let client_width = section.client_width();

        //global_state.socket_state_used(connection_load_state_name);

        global_gallery_imgs.set(vec![]);

        on_fetch(
            Utc::now().timestamp_millis(),
            calc_fit_count(client_width as u32, client_height as u32) * 2,
        );

        loaded_sig.set(LoadingNotFound::Loading);
    });

    view! {
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
        <section id="profile_gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
            <Show when=move|| global_state.socket_connected.get() fallback=move || { "Connecting..." }>
                <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded || *state == LoadingNotFound::Loading) >
                  <div>"LOADING..."</div>
                </Show>
                <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotFound) >
                  <div>"No Images Found."</div>
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
    }
}
