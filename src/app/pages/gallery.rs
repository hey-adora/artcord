use crate::app::utils::{GlobalState, ScrollDetect, ScrollSection};
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use rand::prelude::*;

#[component]
pub fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;

    let gallery_section = create_node_ref::<Section>();
    let scroll_items = [ScrollDetect::new(
        ScrollSection::Gallery,
        gallery_section,
        0,
        "/gallery",
    )];

    create_effect(move |_| {
        ScrollDetect::calc_section(section, ScrollSection::GalleryTop, &scroll_items);
    });

    let images: Vec<(i32, i32)> = (0..200)
        .map(|_| {
            (
                rand::thread_rng().gen_range(500..1000),
                rand::thread_rng().gen_range(500..1000),
            )
        })
        .collect();

    view! {
        <section _ref=gallery_section class="line-bg  px-6 flex gap-2 flex-wrap  " style=move|| format!("min-height: calc(100vh - 100px)")>
            { move || {
                images.iter().map(|(h, w)|{
                    let new_height = 250;
                    let height = h.to_owned();
                    let width = w.to_owned();
                    let ratio = width / height;
                    let height_diff = height - new_height;
                    let new_width = width - ( height_diff * ratio );

                    log!("{}", height);
                    view! {
                    <div
                        class="flex-shrink-0 flex flex-col shadow-glowy  bg-mid-purple border-4 border-low-purple"
                        style:height=move || format!("{}px", new_height)
                        style:width=move || format!("{}px", new_width)
                    >
                        <div class="flex justify-between gap-2">
                            <h3>hello</h3>
                            <div>2020</div>
                        </div>
                    </div>
                } }).collect_view()
            } }
        </section>
    }
}
