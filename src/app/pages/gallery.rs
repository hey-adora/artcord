use chrono::Utc;
use leptos::ev::{load, resize};
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::prelude::*;
use web_sys::Event;

use crate::app::utils::GlobalState;
use crate::server::{ClientMsg, ServerMsgImg};

const new_img_height: u32 = 250;

fn resize_imgs(
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    org_imgs: &[ServerMsgImg],
    resized_imgs: &mut [ServerMsgImg],
) {
    let mut total_ratio: f32 = 0f32;
    //log!("{}..{},{}", new_row_start, new_row_end +

    for i in new_row_start..(new_row_end + 1) {
        let org_img = &org_imgs[i];
        total_ratio += org_img.width as f32 / org_img.height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;

    for i in new_row_start..(new_row_end + 1) {
        let org_img = &org_imgs[i];
        let ratio = org_img.width as f32 / org_img.height as f32;
        let new_prev_img_w: f32 = optimal_height * ratio;
        let new_prev_img_h: f32 = optimal_height;
        resized_imgs[i].width = new_prev_img_w as u32;
        resized_imgs[i].height = new_prev_img_h as u32;

        log!(
            "-: {}, f: {}, w: {}, c: {}, d: {}, o: {}, l: {}..{}",
            i,
            0,
            max_width,
            1 + new_row_end - new_row_start,
            0,
            optimal_height,
            new_row_start,
            new_row_end
        );
    }
}

fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (new_img_height * new_img_height)
}

fn render_gallery(
    max_width: u32,
    org_imgs: &[ServerMsgImg],
    resized_imgs: &mut [ServerMsgImg],
) -> Vec<usize> {
    // if org_imgs.len() < 1 || resized_imgs.len() < 1 {
    //     return vec![0];
    // }
    let mut row_img_count: Vec<usize> = Vec::new();
    let loop_start = 0;
    let loop_end = org_imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    let new_height: u32 = new_img_height;
    // let new_height: i32 = match max_width {
    //     _ => new_img_height,
    // };

    for index in loop_start..loop_end {
        let org_img = &org_imgs[index];
        let width: u32 = org_img.width;
        let height: u32 = org_img.height;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;

        //&& (new_row_end - index != 0)
        if ((current_row_filled_width + new_width) <= max_width) {
            current_row_filled_width += new_width;
            new_row_end = index;
            // log!(
            //     "+: {}, f: {}, w: {}, c: {}, d: {}, l: {}..{}",
            //     index,
            //     current_row_filled_width,
            //     max_width,
            //     1 + new_row_end - new_row_start,
            //     max_width - current_row_filled_width,
            //     new_row_start,
            //     new_row_end
            // );
            if index == loop_end - 1 {
                // log!("FIRST: END;");
                resize_imgs(
                    max_width,
                    new_row_start,
                    new_row_end,
                    org_imgs,
                    resized_imgs,
                );
                row_img_count.push((new_row_end + 1) - new_row_start);
            }
        } else {
            resize_imgs(
                max_width,
                new_row_start,
                new_row_end,
                org_imgs,
                resized_imgs,
            );
            row_img_count.push((new_row_end + 1) - new_row_start);

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            // log!(
            //     "+: {}, f: {}, w: {}, c: {}, d: {}, l: {}..{}",
            //     index,
            //     current_row_filled_width,
            //     max_width,
            //     1 + new_row_end - new_row_start,
            //     max_width - current_row_filled_width,
            //     new_row_start,
            //     new_row_end
            // );
            if index == loop_end - 1 {
                // log!("SECOND: END;");
                resize_imgs(
                    max_width,
                    new_row_start,
                    new_row_end,
                    org_imgs,
                    resized_imgs,
                );
                row_img_count.push((new_row_end + 1) - new_row_start);
            }
        }
    }

    row_img_count
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let (loaded, set_loaded): (ReadSignal<bool>, WriteSignal<bool>) = create_signal(false);
    let (row_img_count, set_row_img_count): (ReadSignal<Vec<usize>>, WriteSignal<Vec<usize>>) =
        create_signal(Vec::new());

    let (gallery_images, set_gallery_images): (
        ReadSignal<Vec<ServerMsgImg>>,
        WriteSignal<Vec<ServerMsgImg>>,
    ) = create_signal::<Vec<ServerMsgImg>>(Vec::new());

    let (gallery_width, set_gallery_width): (ReadSignal<u32>, WriteSignal<u32>) =
        create_signal::<u32>(0);

    let gallery_section = create_node_ref::<Section>();
    let resize_images = move || {
        let section = gallery_section.get_untracked();
        if let Some(section) = section {
            let width = section.client_width() as u32;
            set_gallery_width(width);

            set_gallery_images.update(move |imgs| {
                let org_imgs = &global_state.gallery_imgs.get_untracked();
                if imgs.len() < 1 || org_imgs.len() < 1 {
                    log!("ORG_IMGS: {}, RESIZED_IMGS: {}", org_imgs.len(), imgs.len());
                    return;
                }
                log!("INPUT {:?}", org_imgs);
                let row_img_count = render_gallery(gallery_width.get_untracked(), org_imgs, imgs);
                log!("OUTPUT {:?}", &imgs);

                set_row_img_count.set_untracked(row_img_count);
            });
        };
    };

    create_effect(move |_| {
        let msg = ClientMsg::GalleryInit {
            amount: 25,
            from: Utc::now().timestamp_nanos(),
        };
        log!("SENDING REQ: {:#?}", &msg);
        global_state.socket_send(msg);
    });

    // create_effect(move |_| {
    //     let _ = use_event_listener(use_window(), load, move |_| {
    //         //resize_images();
    //         // log!("LOADED");
    //     });
    // });

    create_effect(move |_| {
        log!("UPDATING EVENT");

        set_gallery_images.update(move |gallery_imgs| {
            let org_imgs = global_state.gallery_imgs.get();
            log!("Updating resized img vec with: {:?}", &org_imgs);
            *gallery_imgs = org_imgs;
        });

        resize_images();

        let _ = use_event_listener(use_window(), resize, move |_| resize_images());
    });

    let section_scroll = move |_: Event| {
        let section = gallery_section.get_untracked().unwrap();
        let scroll_top = section.scroll_top() as u32;

        let width = section.client_width() as u32;
        let height = section.client_height() as u32;
        let left = section.scroll_height() as u32 - (height + scroll_top);

        let fit_count = calc_fit_count(width, height) as usize;
        let row_fit_count = (height / new_img_height) as usize;
        let mut amount: usize = fit_count as usize;
        let row_img_count = row_img_count.get_untracked();
    };

    view! {
        <section on:scroll=section_scroll on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg  overflow-x-hidden content-start flex flex-wrap overflow-y-scroll " style=move|| format!("max-height: calc(100vh - 80px); ")>
            { move || {

                  gallery_images.get().into_iter().enumerate().map(|(i, img)|{

                    view! {
                        <div
                            class="bg-center bg-contain bg-no-repeat flex-shrink-0 font-bold grid place-items-center  border hover:shadow-glowy hover:z-10 transition-shadow duration-300 bg-mid-purple  border-low-purple"
                            style:height=move || format!("{}px", img.height)
                            style:width=move || format!("{}px", img.width)
                            style:background-image=move || format!("url('assets/gallery/org_{}.{}')", img.org_hash, img.format)
                        >
                            <div class="flex flex-col text-center justify-center gap-2  "  >
                                <h3>{i}</h3>
                                <h3>{global_state.gallery_imgs.with(|m|m[i].width)}x{global_state.gallery_imgs.with(|m|m[i].height)}</h3>
                                <h3>{img.width}x{img.height}</h3>
                                <h3>{img.width as f32 /img.height as f32}</h3>
                            </div>
                        </div>
                    } }).collect_view()

                }
            }
        </section>
    }
}
