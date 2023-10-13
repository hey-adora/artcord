use leptos::ev::{load, resize};
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::prelude::*;

fn resize_imgs(
    max_width: i32,
    new_row_start: usize,
    new_row_end: usize,
    org_imgs: &[(i32, i32)],
    resized_imgs: &mut [(i32, i32)],
) {
    let mut total_ratio: f32 = 0f32;
    //log!("{}..{},{}", new_row_start, new_row_end +

    for i in new_row_start..(new_row_end + 1) {
        let (prev_img_w, prev_img_h) = &org_imgs[i];
        total_ratio += *prev_img_w as f32 / *prev_img_h as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;

    for i in new_row_start..(new_row_end + 1) {
        let (prev_img_w, prev_img_h) = &org_imgs[i];
        let ratio = *prev_img_w as f32 / *prev_img_h as f32;
        let new_prev_img_w: f32 = optimal_height * ratio;
        let new_prev_img_h: f32 = optimal_height;
        resized_imgs[i].0 = new_prev_img_w as i32;
        resized_imgs[i].1 = new_prev_img_h as i32;

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

fn render_gallery(max_width: i32, org_imgs: &[(i32, i32)], resized_imgs: &mut [(i32, i32)]) -> () {
    let loop_start = 0;
    let loop_end = org_imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = loop_end - 1;
    let mut current_row_filled_width: i32 = 0;
    let new_height: i32 = match max_width {
        _ => 250,
    };

    for index in loop_start..loop_end {
        let (w, h) = &org_imgs[index];
        let width: i32 = *w;
        let height: i32 = *h;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: i32 = height - new_height;
        let new_width: i32 = width - (height_diff as f32 * ratio) as i32;

        //&& (new_row_end - index != 0)
        if ((current_row_filled_width + new_width) <= max_width) {
            current_row_filled_width += new_width;
            new_row_end = index;
            log!(
                "+: {}, f: {}, w: {}, c: {}, d: {}, l: {}..{}",
                index,
                current_row_filled_width,
                max_width,
                1 + new_row_end - new_row_start,
                max_width - current_row_filled_width,
                new_row_start,
                new_row_end
            );
            if index == loop_end - 1 {
                log!("FIRST: END;");
                resize_imgs(
                    max_width,
                    new_row_start,
                    new_row_end,
                    org_imgs,
                    resized_imgs,
                );
            }
        } else {
            resize_imgs(
                max_width,
                new_row_start,
                new_row_end,
                org_imgs,
                resized_imgs,
            );

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            log!(
                "+: {}, f: {}, w: {}, c: {}, d: {}, l: {}..{}",
                index,
                current_row_filled_width,
                max_width,
                1 + new_row_end - new_row_start,
                max_width - current_row_filled_width,
                new_row_start,
                new_row_end
            );
            if index == loop_end - 1 {
                // log!("SECOND: END;");
                resize_imgs(
                    max_width,
                    new_row_start,
                    new_row_end,
                    org_imgs,
                    resized_imgs,
                );
            }
        }
    }
}

fn render_gallery3(max_width: i32, images: &Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let max_width = max_width - 48;
    let mut resized_images: Vec<(i32, i32)> = Vec::new();
    let mut new_row_start = 0;
    let mut new_row_end = 0;
    let mut current_row_filled_width: i32 = 0;
    let new_height: i32 = match max_width {
        _ => max_width,
    };
    for (index, (w, h)) in images.iter().enumerate() {
        let width: i32 = w.to_owned();
        let height: i32 = h.to_owned();
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: i32 = height - new_height;
        let new_width: i32 = width - (height_diff as f32 * ratio) as i32;
        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
        } else {
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
    let (org_images, set_org_images): (ReadSignal<Vec<(i32, i32)>>, WriteSignal<Vec<(i32, i32)>>) =
        create_signal::<Vec<(i32, i32)>>(
            (0..25)
                .map(|_| {
                    (
                        rand::thread_rng().gen_range(500..1000),
                        rand::thread_rng().gen_range(500..1000),
                    )
                })
                .collect::<Vec<(i32, i32)>>(),
        );

    let (gallery_images, set_gallery_images): (
        ReadSignal<Vec<(i32, i32)>>,
        WriteSignal<Vec<(i32, i32)>>,
    ) = create_signal::<Vec<(i32, i32)>>(org_images.get_untracked());

    let (gallery_width, set_gallery_width): (ReadSignal<i32>, WriteSignal<i32>) =
        create_signal::<i32>(0);

    let gallery_section = create_node_ref::<Section>();
    let resize_images = move || {
        let section = gallery_section.get_untracked();
        if let Some(section) = section {
            let width = section.parent_element().unwrap().client_width();
            set_gallery_width(width);

            set_gallery_images.update(move |imgs| {
                render_gallery(
                    gallery_width.get_untracked(),
                    &org_images.get_untracked(),
                    imgs,
                );
            });
            log!("WIDTH: {:?}", width);
            log!("ORG: {:?}", org_images.get_untracked());
            log!("NEW: {:?}", gallery_images.get_untracked());
        };
    };

    // create_effect(move |_| {
    //     for i in 0..=25 {
    //         log!("BOOM, {}", i);
    //     }
    // });

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), load, move |_| {
            // resize_images();
            // log!("LOADED");
        });
    });

    create_effect(move |_| {
        resize_images();

        let _ = use_event_listener(use_window(), resize, move |_| resize_images());
    });

    view! {
        <section on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg  overflow-x-hidden content-start flex flex-wrap  " style=move|| format!("min-height: calc(100vh - 100px); ")>
                       { move || {

                  gallery_images.get().into_iter().enumerate().map(|(i, (w, h))|{

                    view! {
                        <div
                            class="flex-shrink-0 font-bold grid place-items-center  border hover:shadow-glowy hover:z-10 transition-shadow duration-300 bg-mid-purple  border-low-purple"
                            style:height=move || format!("{}px", h)
                            style:width=move || format!("{}px", w)
                        >
                            <div class="flex flex-col text-center justify-center gap-2">
                                <h3>{i}</h3>
                                <h3>{org_images.with(|m|m[i].0)}x{org_images.with(|m|m[i].1)}</h3>
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
