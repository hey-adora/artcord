use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::Rng;
use web_sys::Event;

use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::{ClientMsg, ServerMsg, SERVER_MSG_IMGS_NAME};

const NEW_IMG_HEIGHT: u32 = 250;

fn resize_img(
    top: &mut f64,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [ServerMsgImgResized],
) {
    let mut total_ratio: f64 = 0f64;

    for i in new_row_start..(new_row_end + 1) {
        let org_img = &imgs[i];
        total_ratio += org_img.width as f64 / org_img.height as f64;
    }
    let optimal_height: f64 = max_width as f64 / total_ratio;
    let mut left: f64 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        // let line = String::new();
        imgs[i].new_width = optimal_height * (imgs[i].width as f64 / imgs[i].height as f64);
        imgs[i].new_height = optimal_height;
        imgs[i].left = left;
        imgs[i].top = *top;
        left += imgs[i].new_width;
    }

    // let mut total: f64 = 0.0;
    // for i in new_row_start..(new_row_end + 1) {
    //     total += imgs[i].new_width;
    // }
    // log!("line: {}", total);

    *top += optimal_height;
}

fn resize_imgs(max_width: u32, imgs: &mut [ServerMsgImgResized]) -> () {
    let loop_start = 0;
    let loop_end = imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    let new_height: u32 = NEW_IMG_HEIGHT;
    let mut top: f64 = 0.0;

    let mut rand = rand::thread_rng();

    for index in loop_start..loop_end {
        let org_img = &mut imgs[index];
        let width: u32 = org_img.width;
        let height: u32 = org_img.height;
        let ratio: f64 = width as f64 / height as f64;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f64 * ratio) as u32;
        org_img.id = rand.gen::<u128>();

        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
            if index == loop_end - 1 {
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        }
    }
}

fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}

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

#[derive(Clone)]
struct SelectedImg {
    pub org_url: String,
    pub author_name: String,
    pub author_pfp: String,
    pub width: u32,
    pub height: u32,
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);

    // create_effect(move |_| {
    //     let sizes = [
    //         (3072.0, 4096.0),
    //         (3072.0, 4096.0),
    //         (1975.0, 2634.0),
    //         (3072.0, 4096.0),
    //         (3072.0, 4096.0),
    //     ];
    //     let total_width = calc(873.0, &sizes);
    //     log!("{}", total_width);
    // });

    create_effect(move |_| {
        global_state.socket_recv.with(|server_msg| {
            if let ServerMsg::Imgs(new_imgs) = server_msg {
                if new_imgs.len() > 0 {
                    global_state.gallery_imgs.update(|imgs| {
                        imgs.extend_from_slice(
                            &new_imgs
                                .iter()
                                .map(|img| ServerMsgImgResized::from(img.to_owned()))
                                .collect::<Vec<ServerMsgImgResized>>(),
                        );
                        let section = gallery_section.get_untracked();
                        if let Some(section) = section {
                            let width = section.client_width() as u32;

                            resize_imgs(width, imgs);
                        };
                    });
                }
                global_state.socket_state_reset(&server_msg.name());
            }
        });
    });

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), resize, move |_| {
            // log!("TRYING TO RESIZE");
            global_state.gallery_imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    // log!("RESIZING!!!!!!!!!!!!");
                    resize_imgs(width, imgs);
                };
            });
        });
    });

    create_effect(move |_| {
        let connected = global_state.socket_connected.get();
        let loaded = global_state.gallery_loaded.get();
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

        let msg = ClientMsg::GalleryInit {
            amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            from: DateTime::from_millis(Utc::now().timestamp_millis()),
        };

        global_state.socket_send(msg);
        global_state.gallery_loaded.set(true);
    });

    let section_scroll = move |_: Event| {
        if !global_state.socket_state_is_ready(SERVER_MSG_IMGS_NAME) {
            return;
        }

        let Some(last) = global_state
            .gallery_imgs
            .with_untracked(|imgs| match imgs.last() {
                Some(l) => Some(l.created_at),
                None => None,
            })
        else {
            return;
        };

        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let scroll_top = section.scroll_top();
        let client_height = section.client_height();
        let scroll_height = section.scroll_height();
        let client_width = section.client_width();

        let left = scroll_height - (client_height + scroll_top);

        if left < client_height {
            let msg = ClientMsg::GalleryInit {
                amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
                from: last,
            };
            global_state.socket_send(msg);
            global_state.socket_state_used(SERVER_MSG_IMGS_NAME);
        }
    };

    let select_click_img = move |img: &ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("assets/gallery/pfp_{}.webp", img.user.id.clone()),
            width: img.width,
            height: img.height,
        }))
    };

    view! {
        {
            move || {
                match selected_img.get() {
                    Some(img) => Some(view! {
                        <div on:click=move |_| { selected_img.set(None); } class=" absolute grid grid-rows-[1fr] left-0 top-0 w-screen h-screen place-items-center bg-gradient-to-br from-mid-purple/50 to-dark-purple/50 z-[150] ">
                            <div  >
                                <div class="flex justify-between items-center rounded-t-lg bg-dark-purple pl-2">
                                       <div class="flex gap-2">
                                            <div>"By "</div>
                                            <img class="border border-low-purple rounded-full bg-mid-purple h-[25px] " src=img.author_pfp/>
                                            <div>{img.author_name}</div>
                                       </div>
                                     <img class="cursor-pointer border-2 border-low-purple rounded-full bg-mid-purple w-[30px] h-[30px] p-1 m-2" src="assets/x.svg"/>
                                </div>
                                <img class="bg-mid-purple object-contain " alt="loading..." style=move|| format!("max-height: calc(100vh - 70px); max-width: 100vw; height: min({1}px, calc(100vw * ( {1} / {0} ))); aspect-ratio: {0} / {1};", img.width, img.height) on:click=move |e| { e.stop_propagation();  } src=img.org_url/>
                            </div>
                        </div> }),
                None => None
                }
            }
        }
        <section on:scroll=section_scroll _ref=gallery_section class="relative   overflow-x-hidden content-start overflow-y-scroll " style=move|| format!("max-height: calc(100vh - 80px); ")>
            <For each=global_state.gallery_imgs key=|state| state.id let:img >
                {
                    let height = format!("{}px", &img.new_height);
                    let with = format!("{}px", &img.new_width);
                    let bg_img = format!("url('{}')", &img.display_preview);
                    let top = format!("{}px", img.top);
                    let left = format!("{}px", img.left);

                    view! {
                        <div
                            class="absolute bg-center bg-contain transition-shadow bg-no-repeat flex-shrink-0 font-bold grid place-items-center border hover:shadow-glowy hover:z-[101]  duration-300 bg-mid-purple  border-low-purple"
                            style:height=height
                            style:width=with
                            style:background-image=bg_img
                            style:top=top
                            style:left=left
                            on:click= move |_| select_click_img(&img)
                        >
                            // <div class="relative flex opacity-0 hover:opacity-100 transition duration-300 w-full h-full flex-col text-center justify-center gap-2  "  >
                            //     <div class="absolute bg-dark-purple bottom-0 left-0 translate-y-full w-full">{&img.user.name}</div>
                            // </div>
                        </div>
                    }
                }
            </For>
        </section>
    }
}
