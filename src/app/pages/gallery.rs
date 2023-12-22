use crate::app::components::gallery::Gallery;
use crate::app::components::navbar::{shrink_nav, Navbar};
use crate::app::utils::{resize_imgs, SelectedImg};
use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_interval_fn, use_window};
use rand::Rng;
use web_sys::{Event, MouseEvent};

use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::{
    ClientMsg, ServerMsg, ServerMsgImg, SERVER_MSG_IMGS_NAME, SERVER_MSG_PROFILE_IMGS_NAME,
};

// fn calc(width: f64, sizes: &[(f64, f64)]) -> f64 {
//     let mut ratio: f64 = 0.0;
//     for size in sizes {
//         ratio += size.0 / size.1;
//     }
//     let height = width / ratio;
//
//     let mut reized_total_width: f64 = 0.0;
//     for size in sizes {
//         reized_total_width += f64::trunc((height * (size.0 / size.1)) * 1000.0) / 1000.0;
//     }
//
//     reized_total_width
// }

fn create_client_test_imgs() -> Vec<ServerMsgImgResized> {
    let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImgResized::default());
    }
    new_imgs
}

fn create_server_test_imgs() -> Vec<ServerMsgImg> {
    let mut new_imgs: Vec<ServerMsgImg> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImg::default());
    }
    new_imgs
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;
    let imgs = global_state.page_galley.gallery_imgs;
    // let temp_gallery_imgs = RwSignal::new(create_client_test_imgs());
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);

    create_effect(move |_| {
        nav_tran.set(true);
    });

    // let add_imgs = move |_: MouseEvent| {
    //     let mut new_imgs: Vec<ServerMsgImg> = Vec::new();
    //     for _ in 0..25 {
    //         new_imgs.push(ServerMsgImg::default());
    //     }
    //     global_state
    //         .socket_recv
    //         .set(ServerMsg::Imgs(new_imgs.clone()));
    // };

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

    let on_fetch = move |from: DateTime, amount: u32| {
        let msg = ClientMsg::GalleryInit { amount, from };
        global_state.socket_send(msg);
    };

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
        <main class=move||format!("grid grid-rows-[auto_1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran() {"pt-[4rem]"} else {"pt-[0rem]"})>
               <Navbar/>
                <div class="backdrop-blur text-low-purple w-full px-6 py-2 2xl:px-[6rem] desktop:px-[16rem]  flex   gap-2   duration-500  bg-gradient-to-r from-dark-night2/75 to-light-flower/10 supports-backdrop-blur:from-dark-night2/95 supports-backdrop-blur:to-light-flower/95">"WOW CAT"</div>
               <Gallery global_gallery_imgs=imgs on_click=select_click_img on_fetch=on_fetch loaded_sig=global_state.page_galley.gallery_loaded connection_load_state_name=SERVER_MSG_IMGS_NAME  />
            // <div class=move || format!("{}", if nav_tran() {"h-[4rem]"} else {"h-[3rem]"})>
            // </div>
        </main>
    }
}
