use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use web_sys::Event;

use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::ClientMsg;

const NEW_IMG_HEIGHT: u32 = 250;

fn resize_imgs(
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

fn render_gallery(max_width: u32, imgs: &mut [ServerMsgImgResized]) -> () {
    let loop_start = 0;
    let loop_end = imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    let new_height: u32 = NEW_IMG_HEIGHT;

    for index in loop_start..loop_end {
        let org_img = &imgs[index];
        let width: u32 = org_img.width;
        let height: u32 = org_img.height;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;

        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
            if index == loop_end - 1 {
                resize_imgs(max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            resize_imgs(max_width, new_row_start, new_row_end, imgs);

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_imgs(max_width, new_row_start, new_row_end, imgs);
            }
        }
    }
}

fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();

    create_effect(move |_| {
        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let _ = use_event_listener(use_window(), resize, move |_| {
            global_state.gallery_imgs.update(|_| {});
        });

        let client_height = section.client_height();
        let client_width = section.client_width();

        let msg = ClientMsg::GalleryInit {
            amount: calc_fit_count(client_width as u32, client_height as u32),
            from: DateTime::from_millis(Utc::now().timestamp_nanos_opt().unwrap()),
        };
        log!("{:#?}", &msg);
        global_state.socket_send(msg);
    });

    let section_scroll = move |_: Event| {
        if !global_state.socket_state_imgs_is_ready() {
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
                amount: calc_fit_count(client_width as u32, client_height as u32),
                from: last,
            };
            global_state.socket_send(msg);
            global_state.socket_state_imgs_used();
        }
    };

    view! {
        <section on:scroll=section_scroll on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg  overflow-x-hidden content-start flex flex-wrap overflow-y-scroll " style=move|| format!("max-height: calc(100vh - 80px); ")>
            { move || {
                  let mut imgs = global_state.gallery_imgs.get();
                  let section = gallery_section.get();
                  if let Some(section) = section {
                      let width = section.client_width() as u32;

                      render_gallery(width, &mut imgs);
                  };
                  imgs.into_iter().enumerate().map(|(_i, img)|{
                    view! {
                        <div
                            class="bg-center bg-contain bg-no-repeat flex-shrink-0 font-bold grid place-items-center  border hover:shadow-glowy hover:z-[101] transition-shadow duration-300 bg-mid-purple  border-low-purple"
                            style:height=move || format!("{}px", img.new_height)
                            style:width=move || format!("{}px", img.new_width)
                            style:background-image=move || format!("url('assets/gallery/org_{}.{}')", img.org_hash, img.format)
                        >
                            <div class="relative flex opacity-0 hover:opacity-100 transition duration-300 w-full h-full flex-col text-center justify-center gap-2  "  >
                                <div class="absolute bg-dark-purple bottom-0 left-0 translate-y-full w-full">{img.user.name}</div>
                                // <h3>{i}</h3>
                                // <h3>{img.width}x{img.height}</h3>
                                // <h3>{img.new_width}x{img.new_height}</h3>
                                // <h3>{img.new_width as f32 /img.new_height as f32}</h3>
                            </div>
                        </div>
                    } }).collect_view()

                }
            }
        </section>
    }
}
