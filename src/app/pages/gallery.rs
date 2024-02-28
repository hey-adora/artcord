use crate::app::components::gallery::{Gallery};
use crate::app::components::navbar::{shrink_nav, Navbar};
use crate::app::global_state::GlobalState;
use crate::app::utils::{
    calc_fit_count, resize_imgs, LoadingNotFound, SelectedImg, NEW_IMG_HEIGHT,
};
use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_router::use_location;
use leptos_use::{use_event_listener, use_window};
use web_sys::Event;

use crate::app::utils::ServerMsgImgResized;
use crate::message::server_msg::{ServerMsg, SERVER_MSG_IMGS_NAME};
use crate::message::server_msg_img::AggImg;
use crate::server::client_msg::ClientMsg;

fn create_client_test_imgs() -> Vec<ServerMsgImgResized> {
    let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImgResized::default());
    }
    new_imgs
}

fn create_server_test_imgs() -> Vec<AggImg> {
    let mut new_imgs: Vec<AggImg> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(AggImg::default());
    }
    new_imgs
}

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
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let imgs = global_state.pages.gallery.gallery_imgs;
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);
    let gallery_section = create_node_ref::<Section>();
    let loaded_sig = global_state.pages.gallery.gallery_loaded;
    let location = use_location();
    //let sender = global_state.pages.gallery.img_sender;

    let sender = global_state.create_sender();

    create_effect(move |_| {
        nav_tran.set(true);

        let _ = use_event_listener(use_window(), resize, move |_| {
            // log!("TRYING TO RESIZE");
            imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    // log!("RESIZING!!!!!!!!!!!!");
                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                };
            });
        });
    });

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

    let on_fetch = move |new_imgs: Vec<AggImg>| {
        let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
        if new_imgs.is_empty() && loaded_sig.get_untracked() == LoadingNotFound::Loading {
            loaded_sig.set(LoadingNotFound::NotFound);
        } else {
            let new_imgs = new_imgs
                .iter()
                .map(|img| ServerMsgImgResized::from(img.to_owned()))
                .collect::<Vec<ServerMsgImgResized>>();

            // if !global_state.page_galley.gallery_loaded.get_untracked() {
            //     global_state.page_galley.gallery_loaded.set(true);
            // }

            imgs.update(|imgs| {
                imgs.extend(new_imgs);
                let document = document();
                let gallery_section = document.get_element_by_id("gallery_section");
                let Some(gallery_section) = gallery_section else {
                    return ;
                };
                let width = gallery_section.client_width() as u32;
                resize_imgs(NEW_IMG_HEIGHT, width, imgs);
            });

            loaded_sig.set(LoadingNotFound::Loaded);
        
        }
    };

    //let sender = Fender::<(),()>::new();

    // let send = create_sender(move |msg| -> Result<(), ()> {
    //     if let ServerMsg::Imgs(new_imgs) = msg {
    //         log!("oh wow ok??? {:?}", new_imgs);

    //         if new_imgs.is_empty() && global_state.page_galley.gallery_loaded.get_untracked() == LoadingNotFound::Loading {
    //             global_state.page_galley.gallery_loaded.set(LoadingNotFound::NotFound);
    //         } else {
    //             let new_imgs = new_imgs
    //                 .iter()
    //                 .map(|img| ServerMsgImgResized::from(img.to_owned()))
    //                 .collect::<Vec<ServerMsgImgResized>>();

    //             // if !global_state.page_galley.gallery_loaded.get_untracked() {
    //             //     global_state.page_galley.gallery_loaded.set(true);
    //             // }

    //             global_state.page_galley.gallery_imgs.update(|imgs| {
    //                 imgs.extend(new_imgs);
    //                 let document = document();
    //                 let gallery_section = document.get_element_by_id("gallery_section");
    //                 let Some(gallery_section) = gallery_section else {
    //                     return ;
    //                 };
    //                 let width = gallery_section.client_width() as u32;
    //                 resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //             });

    //             global_state
    //                 .page_galley
    //                 .gallery_loaded
    //                 .set(LoadingNotFound::Loaded);
            
    //         }
            
    //     }
        
    //     Ok(())
    // });

    // let on_fetch = move |from: i64, amount: u32| {
    //     let msg = ClientMsg::GalleryInit { amount, from };
    //     log!("hmmmmmmmmm");
    //     send(&msg);
    //     //global_state.socket_send(&msg);
    // };

    let section_scroll = move |_: Event| {
        if sender.is_loading() {
            log!("NOT READY YET");
            return;
        }

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
            let msg = ClientMsg::GalleryInit {
                amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
                from: last,
            };
            sender.send(&msg, move |server_msg| {
                if let ServerMsg::Imgs(imgs) = server_msg {
                    on_fetch(imgs);
                }
            });
            //global_state.socket_state_used(connection_load_state_name);
            // on_fetch(
            //     last,
            //     calc_fit_count(client_width as u32, client_height as u32) * 2,
            // );
            // let msg = ClientMsg::GalleryInit {
            //     amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            //     from: last,
            // };
            // global_state.socket_send(msg);
        }
    };

    create_effect(move |_| {
        let loaded = loaded_sig.with_untracked(|state| *state == LoadingNotFound::Loaded);
        if !loaded {
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

    // let lll = watch(move || global_state.socket_connected.get(), move |num, prev_num, aaa| {
    //     aaa.stop();
    // }, false);

    create_effect(move |_| {
        let connected = global_state.socket_connected.get();
        let not_loaded = loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded);
        if !not_loaded || !connected {
            return;
        }

        let Some(section) = gallery_section.get() else {
            return;
        };

        let client_height = section.client_height();
        let client_width = section.client_width();

        let msg = ClientMsg::GalleryInit {
            amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            from: Utc::now().timestamp_millis(),
        };

        loaded_sig.set_untracked(LoadingNotFound::Loading);

        sender.send(&msg, move |server_msg| {
            // on receive
            if let ServerMsg::Imgs(imgs) = server_msg {
                on_fetch(imgs);
            }
        });
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
