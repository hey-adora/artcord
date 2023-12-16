use crate::app::components::gallery::{resize_imgs, Gallery};
use crate::app::components::navbar::{shrink_nav, Navbar};
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
use crate::server::{ClientMsg, ServerMsg, ServerMsgImg, SERVER_MSG_IMGS_NAME};

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
    let imgs = global_state.gallery_imgs;
    let temp_gallery_imgs = RwSignal::new(create_client_test_imgs());

    create_effect(move |_| {
        nav_tran.set(true);
    });

    let add_imgs = move |_: MouseEvent| {
        let mut new_imgs: Vec<ServerMsgImg> = Vec::new();
        for _ in 0..25 {
            new_imgs.push(ServerMsgImg::default());
        }
        global_state
            .socket_recv
            .set(ServerMsg::Imgs(new_imgs.clone()));
    };

    view! {

        // <button on:click=add_imgs>"add more"</button>
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran() {"pt-[4rem]"} else {"pt-[0rem]"})>
               <Navbar/>
               <Gallery global_gallery_imgs=imgs render_prop=||view! { "" }/>
            // <div class=move || format!("{}", if nav_tran() {"h-[4rem]"} else {"h-[3rem]"})>
            // </div>
        </main>
    }
}
