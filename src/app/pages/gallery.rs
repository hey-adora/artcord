use crate::app::utils::{GlobalState, ScrollDetect, ScrollSection};
use leptos::html::Section;
use leptos::*;

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
        // if section.get() != ScrollSection::Gallery {
        //     section.set(ScrollSection::Gallery);
        // }
    });

    view! {
        <section _ref=gallery_section style=move|| format!("min-height: calc(100vh - 100px)")>
            <h1>GALLERY</h1>
        </section>
    }
}
