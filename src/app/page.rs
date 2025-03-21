pub mod home {
    use leptos::prelude::*;
    use reactive_stores::Store;
    use tracing::trace;
    use web_sys::{HtmlDivElement, HtmlElement};

    use crate::app::{
        components::gallery::{resize_imgs, Gallery, Img},
        GlobalState, GlobalStateStoreFields,
    };

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<Store<GlobalState>>();
        let imgs = global_state.imgs();

        Effect::new(move || {
            let new_imgs = Img::rand_vec(100);
            imgs.set(new_imgs);
        });

        let get_imgs = move || {
            let Some(gallery_elm): Option<HtmlElement> = main_ref.get() else {
                trace!("refresh target not found");
                return Vec::new();
            };
            trace!("refreshing...");
            let width = gallery_elm.client_width() as u32;

            let mut imgs = imgs.get();
            resize_imgs(200, width, &mut imgs);
            imgs.into_iter().enumerate().collect::<Vec<(usize, Img)>>()
        };

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <nav class="text-gray-200 pb-1">
                    <a href="/" class="font-black text-xl">
                        "ArtBounty"
                    </a>
                    <a href="/two">"two"</a>
                </nav>

                // <button on:click=switch_tab class="font-black text-xl text-white">
                // "refresh"
                // </button>
                // <button on:click=scroll_btn.clone() class="font-black text-xl text-white">
                // "scroll"
                // </button>

                // </div>
                <Gallery imgs=get_imgs />
            </main>
        }
    }
}
