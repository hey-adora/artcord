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

fn render_gallery(width: i32, images: &Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let mut resized_images: Vec<(i32, i32)> = Vec::new();

    for image in images {
        resized_images.push(image.to_owned());
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

    let images: Vec<(i32, i32)> = (0..200)
        .map(|_| {
            (
                rand::thread_rng().gen_range(500..1000),
                rand::thread_rng().gen_range(500..1000),
            )
        })
        .collect();

    // create_effect(move |_| {
    //     let test = images.clone();
    //     let f = use_event_listener(use_window(), resize, move |event| {
    //         let a = test;
    //     });
    // });

    create_effect(move |_| {
        ScrollDetect::calc_section(section, ScrollSection::GalleryTop, &scroll_items);
    });

    create_effect(move |_| {});

    //use_resize_observer

    // create_render_effect(move |_| {
    //
    // });

    create_effect(move |_| {
        let imgs = images.clone();
        let f = use_event_listener(use_window(), resize, move |event| {
            let section = gallery_section.get_untracked();
            if let Some(section) = section {
                let width = section.offset_width();
                set_gallery_width(width);

                let imgs = imgs.clone();
                let img = render_gallery(gallery_width(), &imgs);
                set_gallery_images(img);
                log!("width: {}", width);
            };
        });

        let section = gallery_section.get_untracked();
        if let Some(section) = section {
            let width = section.offset_width();
            set_gallery_width(width);

            let imgs = images.clone();
            let img = render_gallery(gallery_width(), &imgs);
            set_gallery_images(img);
            log!("INITIAL: {}", width);
        };
        //log!("SOMETHING WENT WRONG");
        //log!("first load");
        //window().set_onresize(Some(&f));
        // window_event_listener();

        // if let Ok(x) = xxx {
        //     log!("x: {}", x);
        // }

        // let gallery_width = gallery_width() - 200;
        // let row_width = 0;
        //
        // let mut row_imgs: &[(i32, i32)] = &[(0, 0); 0];
        // let resized_images: Vec<(i32, i32)> = Vec::new();
        //
        // for img in images {
        //
        // }
        //
        // let render_gallery = move || {
        //
        //
        //     images.iter().map(|(h, w)|{
        //         let new_height = 250;
        //         let height = h.to_owned();
        //         let width = w.to_owned();
        //         let ratio = width / height;
        //         let height_diff = height - new_height;
        //         let new_width = width - ( height_diff * ratio );
        //
        //         if (row_width + new_width) > gallery_width {
        //
        //         }
        //
        //
        //
        //         view! {
        //             <div
        //                 class="flex-shrink-0 flex flex-col shadow-glowy  bg-mid-purple border-4 border-low-purple"
        //                 style:height=move || format!("{}px", new_height)
        //                 style:width=move || format!("{}px", new_width)
        //             >
        //                 <div class="flex justify-between gap-2">
        //                     <h3>hello</h3>
        //                     <div>2020</div>
        //                 </div>
        //             </div>
        //         } }).collect_view()
        // };gallery_width
    });

    view! {
        <section on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg  px-6 flex gap-2 flex-wrap  " style=move|| format!("min-height: calc(100vh - 100px)")>
            { move || {

                  gallery_images.get().into_iter().map(|(h, w)|{

                    view! {
                        <div
                            class="flex-shrink-0 flex flex-col shadow-glowy  bg-mid-purple border-4 border-low-purple"
                            style:height=move || format!("{}px", h)
                            style:width=move || format!("{}px", w)
                        >
                            <div class="flex justify-between gap-2">
                                <h3>hello</h3>
                                <div>2020</div>
                            </div>
                        </div>
                    } }).collect_view()

            }
        }
        </section>
    }
}
