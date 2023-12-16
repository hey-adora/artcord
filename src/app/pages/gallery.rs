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
    // let a = use_interval_fn(
    //     move || {
    //         log!("ya ya");
    //         // open();
    //     },
    //     1000,
    // );

    // create_effect(move |_| {
    //     global_state.socket_recv.with(|server_msg| {
    //         if let ServerMsg::Imgs(new_imgs) = server_msg {
    //             if !new_imgs.is_empty() {
    //                 let new_imgs = new_imgs
    //                     .iter()
    //                     .map(|img| ServerMsgImgResized::from(img.to_owned()))
    //                     .collect::<Vec<ServerMsgImgResized>>();
    //                 global_state.gallery_imgs.update(|imgs| {
    //                     imgs.extend(new_imgs);
    //                     // imgs.extend(temp_gallery_imgs.get());
    //                     // resize_imgs(250, 1091, imgs);
    //                     // let section = gallery_section.get_untracked();
    //                     // if let Some(section) = section {
    //                     //     let width = section.client_width() as u32;
    //                     //
    //                     //     resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //                     // };
    //                 });
    //                 // global_gallery_imgs.update(|imgs| {
    //                 //     imgs.extend(new_imgs);
    //                 //     let section = gallery_section.get_untracked();
    //                 //     if let Some(section) = section {
    //                 //         let width = section.client_width() as u32;
    //                 //
    //                 //         resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //                 //     };
    //                 // });
    //             }
    //             global_state.socket_state_reset(&server_msg.name());
    //         }
    //     });
    // });

    // let received_imgs: RwSignal<Vec<ServerMsgImgResized>> = RwSignal::new(Vec::new());
    // // let received_imgs: RwSignal<ServerMsg> = RwSignal::new(ServerMsg::None);
    //
    // create_effect(move |_| {
    //     // let mut a = create_client_test_imgs();
    //     received_imgs.with(move |new_imgs| {
    //         let a = vec![ServerMsgImgResized::default()];
    //         global_state
    //             .gallery_imgs
    //             .set([global_state.gallery_imgs.get(), a.clone()].concat());
    //         global_state.gallery_imgs.update(|imgs| {
    //             // yyy.set(create_client_test_imgs());
    //             // let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    //             // for _ in 0..25 {
    //             //     imgs.push(ServerMsgImgResized::default());
    //             //     // new_imgs.push(ServerMsgImgResized::default());
    //             // }
    //             imgs.extend([ServerMsgImgResized::default()]);
    //             // imgs.extend_from_slice(&new_imgs);
    //             // imgs.extend_from_slice(&new_imgs[..]);
    //         });
    //         // global_state.gallery_imgs.update(|imgs| {
    //         //     resize_imgs(250, 1091, imgs);
    //         // });
    //     });
    // });

    // create_effect(move |_| {
    //     received_imgs.with(|server_msg| {
    //         global_state.gallery_imgs.update(|imgs| {
    //             imgs.extend_from_slice(&create_client_test_imgs()[..]);
    //             resize_imgs(250, 1091, imgs);
    //         });
    //         global_state.socket_state_reset(&server_msg.name());
    //     });
    // });

    create_effect(move |_| {
        nav_tran.set(true);
    });

    let add_imgs = move |_: MouseEvent| {
        let mut new_imgs: Vec<ServerMsgImg> = Vec::new();
        for _ in 0..25 {
            new_imgs.push(ServerMsgImg::default());
        }
        // imgs.update(move |imgs| {
        //     imgs.extend(new_imgs);
        //     resize_imgs(250, 426, imgs);
        // });
        global_state
            .socket_recv
            .set(ServerMsg::Imgs(new_imgs.clone()));
        // received_imgs.set(ServerMsg::Imgs(new_imgs));
        // received_imgs.update(ServerMsg::Imgs(new_imgs));

        // let client_imgs = create_client_test_imgs();
        // received_imgs.set(client_imgs);

        // received_imgs.update(|imgs| {
        //     imgs.extend(client_imgs);
        // });

        // global_state.gallery_imgs.update(|imgs| {
        //     imgs.extend_from_slice(&new_imgs);
        //     resize_imgs(250, 1091, imgs);
        // });
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
