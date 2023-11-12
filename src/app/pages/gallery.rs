use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use web_sys::Event;

use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::{ClientMsg, ServerMsg, SERVER_MSG_IMGS_NAME};

const NEW_IMG_HEIGHT: u32 = 250;

fn resize_img(
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [ServerMsgImgResized],
) {
    let mut total_ratio: f32 = 0f32;

    for i in new_row_start..(new_row_end + 1) {
        let org_img = &imgs[i];
        total_ratio += org_img.width as f32 / org_img.height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;

    for i in new_row_start..(new_row_end + 1) {
        let org_img = &imgs[i];
        let ratio = org_img.width as f32 / org_img.height as f32;
        let new_prev_img_w: f32 = optimal_height * ratio;
        let new_prev_img_h: f32 = optimal_height;
        imgs[i].new_width = new_prev_img_w as u32;
        imgs[i].new_height = new_prev_img_h as u32;
    }
}

fn resize_imgs(max_width: u32, imgs: &mut [ServerMsgImgResized]) -> () {
    let loop_start = 0;
    let loop_end = imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    let new_height: u32 = NEW_IMG_HEIGHT;

    for index in loop_start..loop_end {
        let org_img = &mut imgs[index];
        let width: u32 = org_img.width;
        let height: u32 = org_img.height;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;

        let url_picker = |img: &ServerMsgImgResized, skip: u8| -> String {
            match skip {
                s if s < 1 && img.has_low => {
                    format!("assets/gallery/low_{}.webp", img.org_hash)
                }
                s if s < 2 && img.has_medium => {
                    format!("assets/gallery/medium_{}.webp", img.org_hash)
                }
                s if s < 3 && img.has_high => {
                    format!("assets/gallery/high_{}.webp", img.org_hash)
                }
                _ => format!("assets/gallery/org_{}.{}", img.org_hash, &img.format),
            }
        };

        org_img.display_high = url_picker(org_img, 2);

        org_img.display_preview = match max_width {
            w if w < NEW_IMG_HEIGHT * 4 => url_picker(org_img, 1),
            w if w < NEW_IMG_HEIGHT * 3 => url_picker(org_img, 2),
            _ => url_picker(org_img, 0),
        };

        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
            if index == loop_end - 1 {
                resize_img(max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            resize_img(max_width, new_row_start, new_row_end, imgs);

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_img(max_width, new_row_start, new_row_end, imgs);
            }
        }
    }
}

fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}

#[derive(Clone)]
struct SelectedImg {
    pub display_url: String,
    pub org_url: String,
    pub author_name: String,
    pub author_pfp: String,
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);

    create_effect(move |_| {
        global_state.socket_recv.with(|server_msg| {
            if let ServerMsg::Imgs(new_imgs) = server_msg {
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
                global_state.socket_state_reset(&server_msg.name());
            }
        });
    });

    create_effect(move |_| {
        let Some(section) = gallery_section.get() else {
            return;
        };

        let _ = use_event_listener(use_window(), resize, move |_| {
            global_state.gallery_imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    resize_imgs(width, imgs);
                };
            });
        });

        let client_height = section.client_height();
        let client_width = section.client_width();

        let msg = ClientMsg::GalleryInit {
            amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            from: DateTime::from_millis(Utc::now().timestamp_nanos_opt().unwrap()),
        };

        global_state.socket_send(msg);
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
            display_url: img.display_high.clone(),
            org_url: format!("assets/gallery/org_{}.{}", img.org_hash, img.format),
            author_name: img.user.name.clone(),
            author_pfp: format!(
                "url('assets/gallery/pfp_{}.webp')",
                img.user.pfp_hash.clone().unwrap_or_default()
            ),
        }))
    };

    view! {
        {
            move || {
                match selected_img.get() {
                    Some(img) => Some(view! {
                        <div on:click=move |_| { selected_img.set(None); } class=" absolute grid grid-rows-[1fr] left-0 top-0 w-screen h-screen place-items-center bg-gradient-to-br from-mid-purple/50 to-dark-purple/50 z-[150] ">
                            <div  >
                                <div class="flex justify-end text-2xl"><img class="border-2 border-low-purple rounded-full bg-mid-purple w-[30px] h-[30px] p-1 m-2" src="assets/x.svg"/></div>
                                <img  style=move|| format!("max-height: calc(100vh - 70px); ") on:click=move |e| { e.stop_propagation();  } src=img.display_url/>
                                <div on:click=move |e| { e.stop_propagation();  } class="bg-dark-purple">"By "{img.author_name}</div>
                            </div>
                        </div> }),
                None => None
                }
            }
        }
        <section on:scroll=section_scroll on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg  overflow-x-hidden content-start flex flex-wrap overflow-y-scroll " style=move|| format!("max-height: calc(100vh - 80px); ")>
            <For each=global_state.gallery_imgs key=|state| (state.org_hash.clone(), state.new_width, state.new_height) let:img >
                {
                    let height = format!("{}px", &img.new_height);
                    let with = format!("{}px", &img.new_width);
                    let bg_img = format!("url('{}')", &img.display_preview);

                    view! {
                        <div
                            class="bg-center bg-contain bg-no-repeat flex-shrink-0 font-bold grid place-items-center  border hover:shadow-glowy hover:z-[101] transition-shadow duration-300 bg-mid-purple  border-low-purple"
                            style:height=height
                            style:width=with
                            style:background-image=bg_img
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
