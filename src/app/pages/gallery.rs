use crate::app::utils::{GlobalState, ScrollDetect, ScrollSection};
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_router::use_resolved_path;
use leptos_use::{use_document, use_event_listener, use_window};
use std::rc::Rc;
//use leptos_use::use_resize_observer;
use rand::prelude::*;

fn render_gallery(max_width: i32, images: &Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let max_width = max_width - 48;
    let mut resized_images: Vec<(i32, i32)> = Vec::new();

    let mut new_row_start = 0;
    let mut new_row_end = 0;
    let mut current_row_filled_width: i32 = 0;

    for (index, (w, h)) in images.iter().enumerate() {
        let new_height: i32 = 250;
        let width: i32 = w.to_owned();
        let height: i32 = h.to_owned();
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: i32 = height - new_height;
        let new_width: i32 = width - (height_diff as f32 * ratio) as i32;

        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
        } else {
            let filled_with_diff = max_width - current_row_filled_width;
            let img_count: usize = (new_row_end - new_row_start) + 1;
            //log!("{}", img_count);

            if img_count != 0 {
                let add_width: f32 = filled_with_diff as f32 / img_count as f32;
                log!(
                    "i: {}, f: {}, w: {}, c: {}, d: {}",
                    index,
                    current_row_filled_width,
                    max_width,
                    img_count,
                    add_width
                );

                let mut total_ratio: f32 = 0f32;
                for i in new_row_start..(new_row_end + 1) {
                    let (prev_img_w, prev_img_h) = resized_images[i];
                    total_ratio += prev_img_w as f32 / prev_img_h as f32;
                }
                let optimal_height: f32 = max_width as f32 / total_ratio;

                for i in new_row_start..(new_row_end + 1) {
                    let (prev_img_w, prev_img_h) = resized_images[i];
                    let ratio = prev_img_w as f32 / prev_img_h as f32;
                    let new_prev_img_w: f32 = optimal_height * ratio;
                    let new_prev_img_h: f32 = optimal_height;

                    resized_images[i].0 = new_prev_img_w as i32;
                    resized_images[i].1 = new_prev_img_h as i32;
                }
            } else {
                log!(
                    "i: {}, f: {}, w: {}, c: {}, d: {}",
                    index,
                    current_row_filled_width,
                    max_width,
                    img_count,
                    0
                );
            }

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
        }

        resized_images.push((new_width, new_height));
    }

    resized_images
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let (gallery_images, set_gallery_images): (
        ReadSignal<Vec<(i32, i32)>>,
        WriteSignal<Vec<(i32, i32)>>,
    ) = create_signal::<Vec<(i32, i32)>>(Vec::new());

    let (gallery_width, set_gallery_width): (ReadSignal<i32>, WriteSignal<i32>) =
        create_signal::<i32>(1000);
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;

    let gallery_section = create_node_ref::<Section>();
    let scroll_items = [ScrollDetect::new(
        ScrollSection::Gallery,
        gallery_section,
        0,
        "/gallery",
    )];

    let images: Vec<(i32, i32)> = (0..25)
        .map(|_| {
            (
                rand::thread_rng().gen_range(500..1000),
                rand::thread_rng().gen_range(500..1000),
            )
        })
        .collect();

    create_effect(move |_| {
        ScrollDetect::calc_section(section, ScrollSection::GalleryTop, &scroll_items);
    });

    create_effect(move |_| {});

    create_effect(move |_| {
        let imgs = images.clone();
        let f = use_event_listener(use_window(), resize, move |event| {
            let section = gallery_section.get_untracked();
            if let Some(section) = section {
                let width = section.offset_width();
                set_gallery_width(width);

                let imgs = imgs.clone();
                let img = render_gallery(gallery_width.get_untracked(), &imgs);
                set_gallery_images(img);
            };
        });

        let section = gallery_section.get_untracked();
        if let Some(section) = section {
            let width = section.offset_width();
            set_gallery_width(width);

            let imgs = images.clone();
            let img = render_gallery(gallery_width.get_untracked(), &imgs);
            set_gallery_images(img);
        };
    });

    view! {
        <section on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg  px-6 flex flex-wrap  " style=move|| format!("min-height: calc(100vh - 100px)")>
            { move || {

                  gallery_images.get().into_iter().map(|(w, h)|{

                    view! {
                        <div
                            class="flex-shrink-0 font-bold grid place-items-center shadow-glowy  bg-mid-purple border-4 border-low-purple"
                            style:height=move || format!("{}px", h)
                            style:width=move || format!("{}px", w)
                        >
                            <div class="flex flex-col text-center justify-center gap-2">
                                <h3>{w}x{h}</h3>
                                <h3>{w as f32 /h as f32}</h3>
                            </div>
                        </div>
                    } }).collect_view()

            }
        }
        </section>
    }
}
