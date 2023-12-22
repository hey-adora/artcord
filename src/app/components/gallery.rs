use crate::app::components::navbar::{shrink_nav, Navbar};
use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::{ElementDescriptor, Section};
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::Rng;
use web_sys::Event;

use crate::app::utils::{
    calc_fit_count, resize_imgs, GlobalState, SelectedImg, ServerMsgImgResized, NEW_IMG_HEIGHT,
};
use crate::server::{ClientMsg, ServerMsg, SERVER_MSG_IMGS_NAME};

//F: Fn(ServerMsgImgResized) -> IV + 'static, IV: IntoView
#[component]
pub fn Gallery<
    OnClick: Fn(ServerMsgImgResized) + Copy + 'static,
    OnFetch: Fn(DateTime, u32) + Copy + 'static,
>(
    global_gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    on_click: OnClick,
    on_fetch: OnFetch,
    loaded_sig: RwSignal<bool>,
    connection_load_state_name: &'static str,
) -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let nav_tran = global_state.nav_tran;

    // create_effect(move |_| {
    //     global_gallery_imgs.update_untracked(move |imgs| {
    //         log!("once on load");
    //         let section = gallery_section.get_untracked();
    //         if let Some(section) = section {
    //             let width = section.client_width() as u32;
    //
    //             resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //         };
    //     });
    // });
    let section_scroll = move |_: Event| {
        if !global_state.socket_state_is_ready(connection_load_state_name) {
            return;
        }

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
            global_state.socket_state_used(connection_load_state_name);
            on_fetch(
                last,
                calc_fit_count(client_width as u32, client_height as u32) * 2,
            );
            // let msg = ClientMsg::GalleryInit {
            //     amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            //     from: last,
            // };
            // global_state.socket_send(msg);
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
        let connected = global_state.socket_connected.get();
        let loaded = loaded_sig.get();
        if loaded || !connected {
            // log!("ITS NOT READY LOADED");
            return;
        }

        let Some(section) = gallery_section.get() else {
            return;
        };
        // log!("THIS SHOULDN'T HAPPEN {} {}", loaded, connected);

        let client_height = section.client_height();
        let client_width = section.client_width();

        global_state.socket_state_used(connection_load_state_name);

        on_fetch(
            DateTime::from_millis(Utc::now().timestamp_millis()),
            calc_fit_count(client_width as u32, client_height as u32) * 2,
        );

        // let msg = ClientMsg::GalleryInit {
        //     amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
        //     from: DateTime::from_millis(Utc::now().timestamp_millis()),
        // };
        loaded_sig.set(true);

        //global_state.socket_send(msg);
    });

    view! {
        <section id="gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
            <Show when=move||!loaded_sig.get()>
              <div>"LOADING..."</div>
            </Show>
            <Show when=move||loaded_sig.get()>
              <div>"No Images Found."</div>
            </Show>
            <For each=move || global_gallery_imgs.get().into_iter().enumerate()  key=|state| state.1._id let:data > {
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
