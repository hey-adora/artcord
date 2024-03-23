use crate::app::components::navbar::{shrink_nav, Navbar};
use crate::app::global_state::GlobalState;
use crate::app::utils::img_resize::{calc_fit_count, resize_imgs, NEW_IMG_HEIGHT};
use crate::app::utils::img_resized::ServerMsgImgResized;
use crate::app::utils::{
    LoadingNotFound, SelectedImg, 
};
use artcord_leptos_web_sockets::{WsResult, WsRuntime};
use artcord_state::aggregation::server_msg_img::AggImg;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_server_msg::{MainGalleryResponse, ServerMsg, UserGalleryResponse};
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_router::use_location;
use leptos_use::{use_event_listener, use_window};
use tracing::{debug, error, trace};
use web_sys::Event;


#[derive(Copy, Clone, Debug)]
pub struct GalleryPageState {
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    pub gallery_loaded: RwSignal<LoadingNotFound>,
}

impl GalleryPageState {
    pub fn new() -> Self {
        Self {
            gallery_imgs: create_rw_signal(Vec::new()),
            gallery_loaded: create_rw_signal(LoadingNotFound::NotLoaded),
        }
    }
}

#[component]
pub fn MainGalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let imgs: RwSignal<Vec<ServerMsgImgResized>> = global_state.pages.gallery.gallery_imgs;
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);
    let gallery_section = create_node_ref::<Section>();
    let loaded_sig = global_state.pages.gallery.gallery_loaded;
    let location = use_location();

    let ws_gallery = global_state.ws.create_singleton();

    let on_fetch = move || {
        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let last = imgs.with_untracked(|imgs| imgs.last().map(|img|img.created_at)).unwrap_or(Utc::now().timestamp_millis());
        let client_height = section.client_height();
        let client_width = section.client_width();


        let msg = ClientMsg::GalleryInit {
            amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            from: last,
        };

        match ws_gallery.send_once(msg, move |server_msg| {
            match server_msg {
                WsResult::Ok(server_msg) => {
                    match server_msg {
                        ServerMsg::MainGallery(response) => {
                            match response {
                                MainGalleryResponse::Imgs(new_imgs) => {
                                    if new_imgs.is_empty() && loaded_sig.get_untracked() == LoadingNotFound::Loading {
                                        loaded_sig.set(LoadingNotFound::NotFound);
                                    } else {
                                        let new_imgs = new_imgs
                                            .iter()
                                            .map(|img| ServerMsgImgResized::from(img.to_owned()))
                                            .collect::<Vec<ServerMsgImgResized>>();
                            
                                        imgs.update(|imgs| {
                                            imgs.extend(new_imgs);
                                            let document = document();
                                            //let gallery_section = document.get_element_by_id("gallery_section");
                                            let gallery_section = gallery_section.get_untracked();
                                            let Some(gallery_section) = gallery_section else {
                                                return;
                                            };
                                            let width = gallery_section.client_width() as u32;
                                            resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                                        });
                            
                                        if loaded_sig.get_untracked() != LoadingNotFound::Loaded {
                                            loaded_sig.set(LoadingNotFound::Loaded);
                                        }
                                    }
                                }
                            }
                        }
                        ServerMsg::Error => {
                            error!("main_gallery: internal server error");
                            loaded_sig.set(LoadingNotFound::Error);
                        },
                        msg => {
                            error!("main_gallery: received wrong msg: {:#?}", msg);
                                loaded_sig.set(LoadingNotFound::Error);
                        }
                    }
                }
                WsResult::TimeOut => {
                    trace!("main_gallery: timeout: loadeding state set to: Error");
                    loaded_sig.set(LoadingNotFound::Error);
                }
            }
        }) {
            Ok(result) => {
                trace!("main_gallery: fetch_imgs returned: {:?}", result);
            }
            Err(err) => {
                error!("main_gallery: send error: {}", err);
                loaded_sig.set(LoadingNotFound::Error);
            }
        };
    };

    let select_click_img = move |img: ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("/assets/gallery/pfp_{}.webp", img.user.id.clone()),
            author_id: img.user_id.clone(),
            width: img.width,
            height: img.height,
        }))
    };

    let section_scroll = move |_: Event| {

        let Some(last) = imgs.with_untracked(|imgs| match imgs.last() {
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
            on_fetch();
        }
    };

    create_effect(move |_| {
        nav_tran.set(true);
    });

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), resize, move |_| {
            imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;
                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                };
            });
        });
    });

    create_effect(move |_| {
        if !loaded_sig.with_untracked(|state| *state == LoadingNotFound::Loaded) {
            return;
        }
        let _ = location.pathname.get();
        let _ = location.hash.get();

        imgs.update(|imgs| {
            let section = gallery_section.get_untracked();
            if let Some(section) = section {
                let width = section.client_width() as u32;

                resize_imgs(NEW_IMG_HEIGHT, width, imgs);
            };
        });
    });

    create_effect(move |_| {
        on_fetch();
    });

    view! {

        {
            move || {
                match selected_img.get() {
                    Some(img) => Some(view! {
                        <div on:click=move |_| { selected_img.set(None); } class=" absolute grid grid-rows-[1fr] left-0 top-0 w-screen h-[100dvh] place-items-center bg-gradient-to-br from-mid-purple/50 to-dark-purple/50 z-[150] ">
                            <div on:click=move |e| { e.stop_propagation();  }  >
                                <div class="flex justify-between items-center rounded-t-lg bg-dark-purple pl-2">
                                       <div class="flex gap-2">
                                            <div>"By "</div>
                                            <img class="border border-low-purple rounded-full bg-mid-purple h-[25px] " src=img.author_pfp/>
                                            <a href=move||format!("/user/{}", img.author_id)>{img.author_name}</a>
                                       </div>
                                     <img on:click=move |_| { selected_img.set(None); } class="cursor-pointer border-2 border-low-purple rounded-full bg-mid-purple w-[30px] h-[30px] p-1 m-2" src="/assets/x.svg"/>
                                </div>
                                <img class="bg-mid-purple object-contain " alt="loading..." style=move|| format!("max-height: calc(100dvh - 70px); max-width: 100vw; height: min({1}px, calc(100vw * ( {1} / {0} ))); aspect-ratio: {0} / {1};", img.width, img.height) src=img.org_url/>
                            </div>
                        </div> }),
                None => None
                }
            }
        }
        // <button on:click=add_imgs>"add more"</button>
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})>
               <Navbar/>
                // <div class="backdrop-blur text-low-purple w-full px-6 py-2 2xl:px-[6rem] desktop:px-[16rem]  flex   gap-2   duration-500  bg-gradient-to-r from-dark-night2/75 to-light-flower/10 supports-backdrop-blur:from-dark-night2/95 supports-backdrop-blur:to-light-flower/95">"WOW CAT"</div>
            //    <Gallery global_gallery_imgs=imgs on_click=select_click_img on_fetch=on_fetch loaded_sig=global_state.page_galley.gallery_loaded connection_load_state_name=SERVER_MSG_IMGS_NAME  />
            // <div class=move || format!("{}", if nav_tran() {"h-[4rem]"} else {"h-[3rem]"})>

            // </div>
            <section id="gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
            <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded || *state == LoadingNotFound::Loading)>
              <div>"LOADING..."</div>
            </Show>
            <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotFound) >
              <div>"No Images Found."</div>
            </Show>
            <For each=move || imgs.get().into_iter().enumerate()  key=|state| state.1.id.clone() let:data > {
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
                                on:click=  move |_| select_click_img(imgs.with_untracked(|imgs|imgs[i].clone()))
                            >
                            </div>
                        }
                    }

            </For>
        </section>
        </main>
    }
}
